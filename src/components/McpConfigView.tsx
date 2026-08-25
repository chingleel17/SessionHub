import { useEffect, useMemo, useState } from "react";
import { useI18n } from "../i18n/I18nProvider";
import type { McpProviderConfig, McpScope, McpServerEntry } from "../types";
import { compareProviders } from "../utils/providerOrder";
import { ConfirmDialog } from "./ConfirmDialog";
import { AddIcon, AutoDetectIcon, CopyToIcon, DeleteIcon, EditNotesIcon, ExternalLinkIcon, EyeIcon, EyeOffIcon, FolderIcon, RefreshIcon } from "./Icons";
import { CollapsibleSection } from "./CollapsibleSection";
import { Button } from "./ui/Button";
import { IconButton } from "./ui/IconButton";
import { Select } from "./ui/Select";

type McpServerFormType = "http" | "npx" | "binary" | "custom";

/** 單一 scope（專案或全域）的 MCP 資料與 handlers。 */
export type McpScopeGroup = {
  scope: McpScope;
  label: string;
  providers: McpProviderConfig[];
  isLoading: boolean;
  onRefresh: () => Promise<void>;
  onUpsert: (
    provider: string,
    name: string,
    originalName: string | null | undefined,
    configJson: string,
  ) => Promise<unknown>;
  onDelete: (provider: string, name: string) => Promise<unknown>;
  onSetEnabled: (provider: string, name: string, enabled: boolean) => Promise<unknown>;
  onTestConnection: (url: string, headers: Record<string, string>) => Promise<McpConnectionTestResult>;
  codexTrusted?: boolean;
};

type Props = {
  groups: McpScopeGroup[];
  onOpenExternal: (path: string) => void;
  onRevealPath: (path: string) => void;
};

/** HTTP header 單列：value 常帶有 token（如 Authorization），介面上以密碼欄位遮蔽並可個別顯示/隱藏；
 *  注意這僅避免螢幕外洩，設定檔寫入磁碟仍是明文，MCP 協定本身即是如此。 */
type HeaderRow = {
  key: string;
  value: string;
  revealed: boolean;
};

function emptyHeaderRow(): HeaderRow {
  return { key: "", value: "", revealed: false };
}

type EditorState = {
  originalName: string | null;
  name: string;
  formType: McpServerFormType;
  url: string;
  headerRows: HeaderRow[];
  packageName: string;
  extraArgsText: string;
  commandPath: string;
  argsText: string;
  envText: string;
  customJson: string;
  error: string | null;
};

export type McpConnectionTestResult =
  | { kind: "ok" }
  | { kind: "unauthorized" }
  | { kind: "unexpectedResponse"; status: number }
  | { kind: "connectionFailed"; message: string };

function getActiveProviderStorageKey(): string {
  return "mcp:tab:shared";
}

function groupStorageKey(scope: McpScope): string {
  return scope.kind === "global" ? "global" : scope.projectCwd.toLowerCase();
}

function emptyEditor(): EditorState {
  return {
    originalName: null,
    name: "",
    formType: "http",
    url: "",
    headerRows: [],
    packageName: "",
    extraArgsText: "",
    commandPath: "",
    argsText: "",
    envText: "",
    customJson: "{\n}",
    error: null,
  };
}

/** headerRows 轉為組裝設定用的 Record；key/value 皆空白的列忽略。重複 key 後者覆蓋前者。 */
function headerRowsToRecord(rows: HeaderRow[]): Record<string, string> {
  const result: Record<string, string> = {};
  for (const row of rows) {
    const key = row.key.trim();
    if (!key) continue;
    result[key] = row.value;
  }
  return result;
}

function headersToRows(headers: Record<string, unknown> | undefined): HeaderRow[] {
  if (!headers) return [];
  return Object.entries(headers).map(([key, value]) => ({
    key,
    value: typeof value === "string" ? value : JSON.stringify(value),
    revealed: false,
  }));
}

/** 解析每行一組的 key-value 文字（分隔符 `=` 或 `:`），空白行忽略；格式錯誤回傳 null。 */
function parseKeyValueLines(text: string, separator: "=" | ":"): Record<string, string> | null {
  const result: Record<string, string> = {};
  for (const rawLine of text.split("\n")) {
    const line = rawLine.trim();
    if (!line) continue;
    const index = line.indexOf(separator);
    if (index <= 0) return null;
    const key = line.slice(0, index).trim();
    const value = line.slice(index + 1).trim();
    if (!key) return null;
    result[key] = value;
  }
  return result;
}

function keyValueToLines(record: Record<string, unknown> | undefined, separator: "=" | ":"): string {
  if (!record) return "";
  return Object.entries(record)
    .map(([key, value]) => `${key}${separator}${typeof value === "string" ? value : JSON.stringify(value)}`)
    .join("\n");
}

function splitArgsText(text: string): string[] {
  return text
    .split("\n")
    .map((line) => line.trim())
    .filter(Boolean);
}

function pathBasename(value: string): string {
  const normalized = value.replace(/\\/g, "/");
  const segments = normalized.split("/").filter(Boolean);
  return segments.length > 0 ? segments[segments.length - 1] : value;
}

/** 摘要優先序：description > url > 指令 basename + 參數（D13）。 */
function summarizeConfig(configJson: string): string {
  try {
    const parsed = JSON.parse(configJson) as Record<string, unknown>;
    if (typeof parsed.description === "string" && parsed.description.trim()) {
      return parsed.description.trim();
    }
    if (typeof parsed.url === "string" && parsed.url.trim()) return parsed.url.trim();
    const command = parsed.command;
    const commandParts = Array.isArray(command)
      ? command.map(String)
      : typeof command === "string"
        ? [command]
        : [];
    if (commandParts.length === 0) return "-";
    const args = Array.isArray(parsed.args) ? parsed.args.map(String) : [];
    const allParts = [...commandParts, ...args];
    return [pathBasename(allParts[0]), ...allParts.slice(1)].join(" ");
  } catch {
    return "-";
  }
}

type ParsedConfigFields = Pick<
  EditorState,
  "formType" | "url" | "headerRows" | "packageName" | "extraArgsText" | "commandPath" | "argsText" | "envText"
> & {
  /** 若輸入是整段設定檔（外層包著 mcpServers/mcp_servers/mcp 區段），解出的 server 名稱。 */
  detectedName?: string;
  /** formType 為 custom 時，區分三種情況供 UI 顯示對應訊息：
   *  multipleServers＝偵測到多個 server（需自行留下一個）；
   *  unsupportedFields＝辨識出合法的 url/command 結構，但含有結構化表單無法承載的欄位
   *  （如 tools），為避免儲存時靜默丟失該欄位，刻意不轉換，整段保留為自訂 JSON；
   *  未設定＝完全無法辨識的格式錯誤。 */
  parseErrorKind?: "multipleServers" | "unsupportedFields";
  /** parseErrorKind 為 unsupportedFields 時，列出造成無法轉換的欄位名稱。 */
  unsupportedFieldNames?: string[];
};

const CUSTOM_PARSE_FIELDS: ParsedConfigFields = {
  formType: "custom",
  url: "",
  headerRows: [],
  packageName: "",
  extraArgsText: "",
  commandPath: "",
  argsText: "",
  envText: "",
};

/** 各 provider 設定檔內，MCP server 清單所在的區段鍵（對應 src-tauri/src/mcp_config.rs 的 provider_spec）。
 *  使用者常直接貼上官方文件範例（整份設定檔或單一 server 區段），自動解析時需先解開這層外殼。 */
const MCP_SECTION_KEYS = new Set(["mcpServers", "mcp_servers", "mcp"]);

type UnwrapResult =
  | { kind: "unwrapped"; value: Record<string, unknown>; detectedName: string }
  | { kind: "multipleServers" }
  | { kind: "notWrapped"; value: Record<string, unknown> };

/** 若物件含有已知的 MCP 區段鍵（不論是否還有其他同層鍵，如 $schema、其他設定），
 *  解開外殼取出內層的「單一 server 設定」，並回傳其名稱以便自動帶入名稱欄位。
 *  使用者常直接貼上整份官方文件範例設定檔，而非單一 server 區段。
 *  區段內若有多個 server，回傳 multipleServers 讓呼叫端給出對應訊息，而非誤判為格式不合法。 */
function unwrapKnownSection(parsed: Record<string, unknown>): UnwrapResult {
  const sectionKey = Object.keys(parsed).find((key) => MCP_SECTION_KEYS.has(key));
  if (!sectionKey) return { kind: "notWrapped", value: parsed };
  const section = parsed[sectionKey];
  if (typeof section !== "object" || section === null || Array.isArray(section)) return { kind: "notWrapped", value: parsed };
  const sectionMap = section as Record<string, unknown>;
  const serverNames = Object.keys(sectionMap);
  if (serverNames.length === 0) return { kind: "notWrapped", value: parsed };
  if (serverNames.length > 1) return { kind: "multipleServers" };
  const serverConfig = sectionMap[serverNames[0]];
  if (typeof serverConfig !== "object" || serverConfig === null || Array.isArray(serverConfig)) return { kind: "notWrapped", value: parsed };
  return { kind: "unwrapped", value: serverConfig as Record<string, unknown>, detectedName: serverNames[0] };
}

/** 將原生 MCP server JSON 解析為結構化表單欄位（D12）；無法對應者回 custom。
 *  供「反解析既有設定」與「自訂 JSON 自動解析」按鈕共用。 */
function parseConfigJson(configJson: string): ParsedConfigFields {
  let raw: Record<string, unknown>;
  try {
    const value = JSON.parse(configJson);
    if (typeof value !== "object" || value === null || Array.isArray(value)) return CUSTOM_PARSE_FIELDS;
    raw = value as Record<string, unknown>;
  } catch {
    return CUSTOM_PARSE_FIELDS;
  }

  const unwrapped = unwrapKnownSection(raw);
  if (unwrapped.kind === "multipleServers") return { ...CUSTOM_PARSE_FIELDS, parseErrorKind: "multipleServers" };
  const { value: parsed, detectedName } = unwrapped.kind === "unwrapped"
    ? { value: unwrapped.value, detectedName: unwrapped.detectedName as string | undefined }
    : { value: unwrapped.value, detectedName: undefined as string | undefined };

  const knownKeys = new Set(["type", "url", "headers", "http_headers", "command", "args", "env", "environment", "enabled", "description"]);
  const unknownKeys = Object.keys(parsed).filter((key) => !knownKeys.has(key));
  if (unknownKeys.length > 0) {
    // 有 url 或 command 代表這看起來就是一份合法的 MCP server 設定，只是帶有結構化表單
    // 無法承載的欄位（如 opencode 的 tools）。轉成結構化表單會在儲存時靜默丟失這些欄位，
    // 因此刻意不轉換，讓使用者知道原因並保留原樣為自訂 JSON。
    const looksLikeServerConfig = typeof parsed.url === "string" || parsed.command !== undefined;
    return {
      ...CUSTOM_PARSE_FIELDS,
      parseErrorKind: looksLikeServerConfig ? "unsupportedFields" : undefined,
      unsupportedFieldNames: looksLikeServerConfig ? unknownKeys : undefined,
    };
  }

  if (typeof parsed.url === "string") {
    const headers = (parsed.headers ?? parsed.http_headers) as Record<string, unknown> | undefined;
    return {
      ...CUSTOM_PARSE_FIELDS,
      formType: "http",
      url: parsed.url,
      headerRows: headersToRows(headers && typeof headers === "object" ? headers as Record<string, unknown> : undefined),
      detectedName,
    };
  }

  const command = parsed.command;
  const commandParts = Array.isArray(command)
    ? command.map(String)
    : typeof command === "string"
      ? [command]
      : null;
  if (!commandParts || commandParts.length === 0) return CUSTOM_PARSE_FIELDS;

  // opencode 的 command 為陣列（含參數）；claude/codex/copilot 為字串 + args 陣列。
  const args = Array.isArray(parsed.args) ? parsed.args.map(String) : commandParts.slice(1);
  const executable = commandParts[0];
  const env = (parsed.env ?? parsed.environment) as Record<string, unknown> | undefined;
  const envText = keyValueToLines(env && typeof env === "object" ? env as Record<string, unknown> : undefined, "=");

  if (executable === "npx") {
    const withoutYes = args[0] === "-y" ? args.slice(1) : args;
    if (withoutYes.length === 0) return CUSTOM_PARSE_FIELDS;
    return {
      ...CUSTOM_PARSE_FIELDS,
      formType: "npx",
      packageName: withoutYes[0],
      extraArgsText: withoutYes.slice(1).join("\n"),
      envText,
      detectedName,
    };
  }

  return {
    ...CUSTOM_PARSE_FIELDS,
    formType: "binary",
    commandPath: executable,
    argsText: args.join("\n"),
    envText,
    detectedName,
  };
}

/** 反解析既有設定值到結構化表單（D12）。 */
function editorFromEntry(entry: McpServerEntry): EditorState {
  const {
    detectedName: _detectedName,
    parseErrorKind: _parseErrorKind,
    unsupportedFieldNames: _unsupportedFieldNames,
    ...parsed
  } = parseConfigJson(entry.configJson);
  return {
    ...emptyEditor(),
    originalName: entry.name,
    name: entry.name,
    customJson: entry.configJson,
    ...parsed,
  };
}

/** 依 provider 將表單欄位組裝為該平台原生 schema（D12）。回傳 JSON 字串或錯誤鍵。 */
function assembleConfig(
  editor: EditorState,
  provider: string,
): { configJson: string } | { errorKey: "invalidJson" | "urlRequired" | "packageRequired" | "commandRequired" | "invalidKeyValue" } {
  if (editor.formType === "custom") {
    try {
      const parsed = JSON.parse(editor.customJson);
      if (typeof parsed !== "object" || parsed === null || Array.isArray(parsed)) return { errorKey: "invalidJson" };
      return { configJson: editor.customJson };
    } catch {
      return { errorKey: "invalidJson" };
    }
  }

  if (editor.formType === "http") {
    const url = editor.url.trim();
    if (!url) return { errorKey: "urlRequired" };
    const hasBlankKeyWithValue = editor.headerRows.some((row) => !row.key.trim() && row.value.trim());
    if (hasBlankKeyWithValue) return { errorKey: "invalidKeyValue" };
    const headers = headerRowsToRecord(editor.headerRows);
    const hasHeaders = Object.keys(headers).length > 0;
    let config: Record<string, unknown>;
    if (provider === "codex") {
      config = { url, ...(hasHeaders ? { http_headers: headers } : {}) };
    } else if (provider === "opencode") {
      config = { type: "remote", url, ...(hasHeaders ? { headers } : {}) };
    } else {
      config = { type: "http", url, ...(hasHeaders ? { headers } : {}) };
    }
    return { configJson: JSON.stringify(config, null, 2) };
  }

  const env = parseKeyValueLines(editor.envText, "=");
  if (env === null) return { errorKey: "invalidKeyValue" };
  const hasEnv = Object.keys(env).length > 0;

  let commandParts: string[];
  if (editor.formType === "npx") {
    const packageName = editor.packageName.trim();
    if (!packageName) return { errorKey: "packageRequired" };
    commandParts = ["npx", "-y", packageName, ...splitArgsText(editor.extraArgsText)];
  } else {
    const commandPath = editor.commandPath.trim();
    if (!commandPath) return { errorKey: "commandRequired" };
    commandParts = [commandPath, ...splitArgsText(editor.argsText)];
  }

  let config: Record<string, unknown>;
  if (provider === "opencode") {
    config = { type: "local", command: commandParts, ...(hasEnv ? { environment: env } : {}) };
  } else {
    const [command, ...args] = commandParts;
    config = { command, ...(args.length > 0 ? { args } : {}), ...(hasEnv ? { env } : {}) };
  }
  return { configJson: JSON.stringify(config, null, 2) };
}

export function McpConfigView({ groups, onOpenExternal, onRevealPath }: Props) {
  const { t } = useI18n();
  const providerIds = useMemo(
    () => groups
      .flatMap((group) => group.providers.map((provider) => provider.providerId))
      .filter((id, index, all) => all.indexOf(id) === index)
      .sort(compareProviders),
    [groups],
  );
  const [activeProvider, setActiveProvider] = useState<string>(() => {
    const stored = window.localStorage.getItem(getActiveProviderStorageKey());
    return stored ?? "";
  });

  useEffect(() => {
    window.localStorage.setItem(getActiveProviderStorageKey(), activeProvider);
  }, [activeProvider]);

  useEffect(() => {
    if (!providerIds.includes(activeProvider)) setActiveProvider(providerIds[0] ?? "");
  }, [activeProvider, providerIds]);

  return (
    <div className="mcp-config-content">
      <div className="sub-tab-bar agents-top-tabs">
        {providerIds.map((providerId) => (
          <button
            key={providerId}
            type="button"
            className={`sub-tab-item ${activeProvider === providerId ? "sub-tab-item--active" : ""}`}
            onClick={() => setActiveProvider(providerId)}
          >
            {t(`mcp.provider.${providerId}` as never)}
          </button>
        ))}
      </div>

      {groups.length > 1 ? (
        <div className="agents-scope-groups">
          {groups.map((group) => (
            <McpGroupCollapsible key={groupStorageKey(group.scope)} group={group} activeProvider={activeProvider} onOpenExternal={onOpenExternal} onRevealPath={onRevealPath} />
          ))}
        </div>
      ) : (
        <McpProviderPanel group={groups[0]} activeProvider={activeProvider} onOpenExternal={onOpenExternal} onRevealPath={onRevealPath} inlineHeader />
      )}
    </div>
  );
}

/** 操作按鈕群（外開 / 資料夾 / 重整 / 新增），供收折標題列或單一群組內嵌列共用。 */
function McpHeaderActions({
  configPath,
  onOpenExternal,
  onRevealPath,
  onRefresh,
  onAdd,
}: {
  configPath: string | undefined;
  onOpenExternal: (path: string) => void;
  onRevealPath: (path: string) => void;
  onRefresh: () => void;
  onAdd: () => void;
}) {
  const { t } = useI18n();
  return (
    <div className="settings-actions agents-toolbar-actions">
      <IconButton
        label={t("agents.action.openExternal")}
        className="agents-icon-button"
        disabled={!configPath}
        onClick={() => configPath && onOpenExternal(configPath)}
      >
        <ExternalLinkIcon size={15} />
      </IconButton>
      <IconButton
        label={t("agents.action.reveal")}
        className="agents-icon-button"
        disabled={!configPath}
        onClick={() => configPath && onRevealPath(configPath)}
      >
        <FolderIcon size={15} />
      </IconButton>
      <IconButton label={t("app.actions.refresh")} className="agents-icon-button" onClick={onRefresh}>
        <RefreshIcon size={15} />
      </IconButton>
      <Button variant="primary" onClick={onAdd}>
        {t("mcp.action.add")}
      </Button>
    </div>
  );
}

function McpGroupCollapsible({
  group,
  activeProvider,
  onOpenExternal,
  onRevealPath,
}: {
  group: McpScopeGroup;
  activeProvider: string;
  onOpenExternal: (path: string) => void;
  onRevealPath: (path: string) => void;
}) {
  const currentConfig = group.providers.find((p) => p.providerId === activeProvider);
  const count = currentConfig?.servers.length ?? 0;
  const key = `agents:groupExpanded:mcp:${groupStorageKey(group.scope)}:${group.scope.kind}`;
  const [expanded, setExpanded] = useState(() => {
    const stored = window.localStorage.getItem(key);
    if (stored === "true") return true;
    if (stored === "false") return false;
    return group.scope.kind === "project";
  });
  // 「新增」由標題列按鈕觸發，開啟訊號下傳給 panel。
  const [addSignal, setAddSignal] = useState(0);

  const toggle = () => {
    setExpanded((current) => {
      const next = !current;
      window.localStorage.setItem(key, String(next));
      return next;
    });
  };

  return (
    <CollapsibleSection
      title={`${group.label} (${count})`}
      expanded={expanded}
      onToggle={toggle}
      titleMeta={currentConfig?.configPath ?? undefined}
      actions={
        <McpHeaderActions
          configPath={currentConfig?.configPath}
          onOpenExternal={onOpenExternal}
          onRevealPath={onRevealPath}
          onRefresh={() => void group.onRefresh()}
          onAdd={() => {
            if (!expanded) toggle();
            setAddSignal((n) => n + 1);
          }}
        />
      }
    >
      <McpProviderPanel group={group} activeProvider={activeProvider} onOpenExternal={onOpenExternal} onRevealPath={onRevealPath} addSignal={addSignal} />
    </CollapsibleSection>
  );
}

function McpProviderPanel({
  group,
  activeProvider,
  onOpenExternal,
  onRevealPath,
  inlineHeader = false,
  addSignal = 0,
}: {
  group: McpScopeGroup;
  activeProvider: string;
  onOpenExternal: (path: string) => void;
  onRevealPath: (path: string) => void;
  /** 單一群組（無收折標題列）情境：在內容頂部自帶一列操作按鈕。 */
  inlineHeader?: boolean;
  /** 收折標題列「新增」按鈕的觸發訊號（遞增即開啟編輯器）。 */
  addSignal?: number;
}) {
  const { t } = useI18n();
  const [editor, setEditor] = useState<EditorState | null>(null);
  const [deleteTarget, setDeleteTarget] = useState<McpServerEntry | null>(null);
  const [busy, setBusy] = useState(false);
  const [testResult, setTestResult] = useState<McpConnectionTestResult | null>(null);
  const [testing, setTesting] = useState(false);
  const [copySource, setCopySource] = useState<McpServerEntry | null>(null);
  const [copyTargetProvider, setCopyTargetProvider] = useState<string>("");
  const [copyName, setCopyName] = useState("");
  const [copyError, setCopyError] = useState<string | null>(null);

  // 收折標題列的「新增」按鈕透過 addSignal 遞增觸發開啟編輯器。
  useEffect(() => {
    if (addSignal > 0) setEditor(emptyEditor());
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [addSignal]);

  // 每次開啟編輯器（新增或編輯）都應清除上一輪殘留的測試結果。
  const editorIdentityKey = editor === null ? null : (editor.originalName ?? "__new__");
  useEffect(() => {
    setTestResult(null);
  }, [editorIdentityKey]);

  useEffect(() => {
    setEditor(null);
    setDeleteTarget(null);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [activeProvider, group.scope.kind === "project" ? group.scope.projectCwd : "global"]);

  const currentConfig = useMemo(
    () => group.providers.find((p) => p.providerId === activeProvider),
    [group.providers, activeProvider],
  );

  // 可複製的目標 provider：排除來源自身，僅列出使用者已啟用的 provider。
  const copyTargetOptions = useMemo(
    () => group.providers.filter((p) => p.providerId !== activeProvider && p.enabled),
    [group.providers, activeProvider],
  );

  useEffect(() => {
    if (!copySource) return;
    setCopyTargetProvider(copyTargetOptions[0]?.providerId ?? "");
    setCopyName(copySource.name);
    setCopyError(null);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [copySource]);

  const showCodexTrustBanner = group.scope.kind === "project" && activeProvider === "codex" && !group.codexTrusted;

  const errorKeyToMessage = (errorKey: string): string => {
    switch (errorKey) {
      case "urlRequired":
        return t("mcp.editor.errorUrlRequired");
      case "packageRequired":
        return t("mcp.editor.errorPackageRequired");
      case "commandRequired":
        return t("mcp.editor.errorCommandRequired");
      case "invalidKeyValue":
        return t("mcp.editor.errorInvalidKeyValue");
      default:
        return t("mcp.editor.errorInvalidJson");
    }
  };

  const handleSaveEditor = async () => {
    if (!editor) return;
    const trimmedName = editor.name.trim();
    if (!trimmedName) {
      setEditor({ ...editor, error: t("mcp.editor.errorNameEmpty") });
      return;
    }
    const duplicate = (currentConfig?.servers ?? []).some(
      (server) => server.name === trimmedName && server.name !== editor.originalName,
    );
    if (duplicate) {
      setEditor({ ...editor, error: t("mcp.editor.errorDuplicateName") });
      return;
    }
    const assembled = assembleConfig(editor, activeProvider);
    if ("errorKey" in assembled) {
      setEditor({ ...editor, error: errorKeyToMessage(assembled.errorKey) });
      return;
    }

    setBusy(true);
    try {
      await group.onUpsert(activeProvider, trimmedName, editor.originalName, assembled.configJson);
      setEditor(null);
    } catch (error) {
      setEditor({ ...editor, error: error instanceof Error ? error.message : String(error) });
    } finally {
      setBusy(false);
    }
  };

  const handleToggleEnabled = async (entry: McpServerEntry) => {
    setBusy(true);
    try {
      await group.onSetEnabled(activeProvider, entry.name, !entry.enabled);
    } finally {
      setBusy(false);
    }
  };

  const handleConfirmDelete = async () => {
    if (!deleteTarget) return;
    setBusy(true);
    try {
      await group.onDelete(activeProvider, deleteTarget.name);
    } finally {
      setBusy(false);
      setDeleteTarget(null);
    }
  };

  const updateEditor = (patch: Partial<EditorState>) => {
    setEditor((current) => (current ? { ...current, ...patch, error: null } : current));
    setTestResult(null);
  };

  const updateHeaderRow = (index: number, patch: Partial<HeaderRow>) => {
    updateEditor({
      headerRows: (editor?.headerRows ?? []).map((row, i) => (i === index ? { ...row, ...patch } : row)),
    });
  };

  /** 純粹切換該列 header 的顯示/隱藏；不算資料變更，不應清除錯誤訊息或既有測試連線結果。 */
  const toggleHeaderRowRevealed = (index: number) => {
    setEditor((current) =>
      current
        ? { ...current, headerRows: current.headerRows.map((row, i) => (i === index ? { ...row, revealed: !row.revealed } : row)) }
        : current,
    );
  };

  const addHeaderRow = () => {
    updateEditor({ headerRows: [...(editor?.headerRows ?? []), emptyHeaderRow()] });
  };

  const removeHeaderRow = (index: number) => {
    updateEditor({ headerRows: (editor?.headerRows ?? []).filter((_, i) => i !== index) });
  };

  /** 將自訂 JSON 欄位貼上的原生設定自動解析回結構化表單；無法辨識時提示錯誤並停留於自訂模式。
   *  支援貼上整份設定檔（外層包 mcpServers/mcp_servers/mcp）：會自動解開外殼，
   *  並在名稱欄位尚為空白時，帶入解出的 server 名稱。 */
  const handleAutoDetect = () => {
    if (!editor) return;
    const { detectedName, parseErrorKind, unsupportedFieldNames, ...parsed } = parseConfigJson(editor.customJson);
    if (parsed.formType === "custom") {
      const error =
        parseErrorKind === "multipleServers"
          ? t("mcp.editor.autoDetectMultipleServers")
          : parseErrorKind === "unsupportedFields"
            ? t("mcp.editor.autoDetectUnsupportedFields", { fields: (unsupportedFieldNames ?? []).join(", ") })
            : t("mcp.editor.autoDetectFailed");
      setEditor({ ...editor, error });
      return;
    }
    setEditor({
      ...editor,
      ...parsed,
      name: editor.name.trim() ? editor.name : (detectedName ?? editor.name),
      error: null,
    });
    setTestResult(null);
  };

  const handleConfirmCopy = async () => {
    if (!copySource || !copyTargetProvider) return;
    const trimmedName = copyName.trim();
    if (!trimmedName) {
      setCopyError(t("mcp.editor.errorNameEmpty"));
      return;
    }
    const targetConfig = group.providers.find((p) => p.providerId === copyTargetProvider);
    const duplicate = (targetConfig?.servers ?? []).some((server) => server.name === trimmedName);
    if (duplicate) {
      setCopyError(t("mcp.editor.errorDuplicateName"));
      return;
    }
    const sourceEditor = editorFromEntry(copySource);
    if (sourceEditor.formType === "custom") {
      setCopyError(t("mcp.copy.errorUnsupportedFormat"));
      return;
    }
    const assembled = assembleConfig(sourceEditor, copyTargetProvider);
    if ("errorKey" in assembled) {
      setCopyError(errorKeyToMessage(assembled.errorKey));
      return;
    }

    setBusy(true);
    try {
      await group.onUpsert(copyTargetProvider, trimmedName, null, assembled.configJson);
      setCopySource(null);
    } catch (error) {
      setCopyError(error instanceof Error ? error.message : String(error));
    } finally {
      setBusy(false);
    }
  };

  const handleTestConnection = async () => {
    if (!editor || editor.formType !== "http") return;
    const url = editor.url.trim();
    if (!url) {
      setEditor({ ...editor, error: t("mcp.editor.errorUrlRequired") });
      return;
    }
    const hasBlankKeyWithValue = editor.headerRows.some((row) => !row.key.trim() && row.value.trim());
    if (hasBlankKeyWithValue) {
      setEditor({ ...editor, error: t("mcp.editor.errorInvalidKeyValue") });
      return;
    }
    const headers = headerRowsToRecord(editor.headerRows);

    setTesting(true);
    setTestResult(null);
    try {
      const result = await group.onTestConnection(url, headers);
      setTestResult(result);
    } catch (error) {
      setTestResult({ kind: "connectionFailed", message: error instanceof Error ? error.message : String(error) });
    } finally {
      setTesting(false);
    }
  };

  return (
    <div className="mcp-provider-panel">
      {inlineHeader ? (
        <div className="mcp-inline-header">
          {currentConfig?.configPath ? (
            <span className="mcp-inline-header-path path-text path-text--truncate">{currentConfig.configPath}</span>
          ) : null}
          <McpHeaderActions
            configPath={currentConfig?.configPath}
            onOpenExternal={onOpenExternal}
            onRevealPath={onRevealPath}
            onRefresh={() => void group.onRefresh()}
            onAdd={() => setEditor(emptyEditor())}
          />
        </div>
      ) : null}

      {showCodexTrustBanner ? (
        <div className="mcp-codex-trust-banner">
          <strong>{t("mcp.codexTrust.untrusted.title")}</strong>
          <span>{t("mcp.codexTrust.untrusted.description")}</span>
        </div>
      ) : null}

      {currentConfig?.error ? (
        <div className="mcp-provider-error">{currentConfig.error}</div>
      ) : null}
      {currentConfig?.diagnostics.map((diagnostic) => (
        <div key={`${diagnostic.providerId}:${diagnostic.kind}:${diagnostic.scope}`} className="mcp-provider-notice">
          {diagnostic.message}
        </div>
      ))}

      {group.isLoading ? <div className="explorer-content-loading">{t("plansSpecs.loading")}</div> : null}

      {!group.isLoading && (currentConfig?.servers.length ?? 0) === 0 ? (
        <div className="explorer-content-empty">{t("mcp.empty")}</div>
      ) : null}

      {!group.isLoading && (currentConfig?.servers.length ?? 0) > 0 ? (
        <div className="agents-matrix-card mcp-server-table-card">
          <table className="agents-matrix-table mcp-server-table">
            <thead>
              <tr>
                <th>{t("mcp.table.name")}</th>
                <th>{t("mcp.table.status")}</th>
                <th>{t("mcp.table.summary")}</th>
                <th>{t("mcp.table.actions")}</th>
              </tr>
            </thead>
            <tbody>
              {currentConfig?.servers.map((entry) => {
                const summary = summarizeConfig(entry.configJson);
                return (
                  <tr
                    key={entry.name}
                    className={entry.editable && !busy ? "mcp-row-clickable" : undefined}
                    onClick={entry.editable && !busy ? () => setEditor(editorFromEntry(entry)) : undefined}
                  >
                    <td>{entry.name}</td>
                    <td>
                      <div className="settings-actions agents-toolbar-actions">
                        <span className={`agents-status-pill agents-status-pill--${entry.enabled ? "done" : "neutral"}`}>
                          {entry.enabled ? t("mcp.status.enabled") : t("mcp.status.disabled")}
                        </span>
                        {entry.effective === true ? (
                          <span
                            className="agents-status-pill agents-status-pill--done"
                            title={[entry.source, entry.scope].filter(Boolean).join(" · ") || undefined}
                          >
                            {t("mcp.status.effective")}
                          </span>
                        ) : null}
                        {entry.scope ? (
                          <span className="agents-status-pill agents-status-pill--neutral">
                            {t(`mcp.scope.${entry.scope}` as never)}
                          </span>
                        ) : null}
                        {!entry.editable ? (
                          <span className="agents-status-pill agents-status-pill--neutral">
                            {t("mcp.status.readOnly")}
                          </span>
                        ) : null}
                      </div>
                    </td>
                    <td>
                      <span className="mcp-server-summary" title={summary}>{summary}</span>
                    </td>
                    <td onClick={(event) => event.stopPropagation()}>
                      <div className="mcp-row-actions">
                        <button
                          type="button"
                          className={`ghost-button mcp-toggle-button mcp-toggle-button--${entry.enabled ? "disable" : "enable"}`}
                          disabled={busy || !entry.editable}
                          onClick={() => void handleToggleEnabled(entry)}
                        >
                          {entry.enabled ? t("mcp.action.disable") : t("mcp.action.enable")}
                        </button>
                        <IconButton
                          label={t("mcp.action.edit")}
                          className="agents-icon-button"
                          disabled={busy || !entry.editable}
                          onClick={() => setEditor(editorFromEntry(entry))}
                        >
                          <EditNotesIcon size={15} />
                        </IconButton>
                        <IconButton
                          label={t("mcp.action.copyTo")}
                          className="agents-icon-button"
                          disabled={busy || copyTargetOptions.length === 0}
                          onClick={() => setCopySource(entry)}
                        >
                          <CopyToIcon size={15} />
                        </IconButton>
                        <IconButton
                          label={t("mcp.action.delete")}
                          className="agents-icon-button"
                          danger
                          disabled={busy || !entry.editable}
                          onClick={() => setDeleteTarget(entry)}
                        >
                          <DeleteIcon size={15} />
                        </IconButton>
                      </div>
                    </td>
                  </tr>
                );
              })}
            </tbody>
          </table>
        </div>
      ) : null}

      {editor ? (
        <div className="dialog-backdrop">
          <article className="dialog-card dialog-card--solid mcp-editor-dialog">
            <h3>{editor.originalName ? t("mcp.editor.editTitle") : t("mcp.editor.addTitle")}</h3>
            <div className="mcp-editor-fields">
              <label className="field-group">
                <span>{t("mcp.editor.nameLabel")}</span>
                <input
                  type="text"
                  value={editor.name}
                  onChange={(event) => updateEditor({ name: event.currentTarget.value })}
                />
              </label>
              <label className="field-group">
                <span>{t("mcp.editor.typeLabel")}</span>
                <Select
                  value={editor.formType}
                  onChange={(event) => updateEditor({ formType: event.currentTarget.value as McpServerFormType })}
                >
                  <option value="http">{t("mcp.editor.type.http")}</option>
                  <option value="npx">{t("mcp.editor.type.npx")}</option>
                  <option value="binary">{t("mcp.editor.type.binary")}</option>
                  <option value="custom">{t("mcp.editor.type.custom")}</option>
                </Select>
              </label>

              {editor.formType === "http" ? (
                <>
                  <label className="field-group">
                    <span>{t("mcp.editor.urlLabel")}</span>
                    <input
                      type="text"
                      value={editor.url}
                      placeholder="https://example.com/mcp"
                      onChange={(event) => updateEditor({ url: event.currentTarget.value })}
                    />
                  </label>
                  <div className="field-group">
                    <span className="mcp-editor-config-label-row">
                      {t("mcp.editor.headersLabel")}
                      <IconButton label={t("mcp.editor.addHeader")} className="ui-icon-button" onClick={addHeaderRow}>
                        <AddIcon size={15} />
                      </IconButton>
                    </span>
                    {editor.headerRows.length === 0 ? (
                      <span className="mcp-editor-headers-empty">{t("mcp.editor.headersEmpty")}</span>
                    ) : (
                      <div className="mcp-editor-header-rows">
                        {editor.headerRows.map((row, index) => (
                          // eslint-disable-next-line react/no-array-index-key
                          <div key={index} className="mcp-editor-header-row">
                            <input
                              type="text"
                              className="mcp-editor-header-key"
                              value={row.key}
                              placeholder={t("mcp.editor.headerKeyPlaceholder")}
                              onChange={(event) => updateHeaderRow(index, { key: event.currentTarget.value })}
                            />
                            <input
                              type={row.revealed ? "text" : "password"}
                              className="mcp-editor-header-value"
                              value={row.value}
                              placeholder={t("mcp.editor.headerValuePlaceholder")}
                              onChange={(event) => updateHeaderRow(index, { value: event.currentTarget.value })}
                            />
                            <IconButton
                              label={row.revealed ? t("mcp.editor.hideHeaderValue") : t("mcp.editor.showHeaderValue")}
                              className="ui-icon-button"
                              onClick={() => toggleHeaderRowRevealed(index)}
                            >
                              {row.revealed ? <EyeOffIcon size={15} /> : <EyeIcon size={15} />}
                            </IconButton>
                            <IconButton
                              label={t("mcp.editor.removeHeader")}
                              className="ui-icon-button"
                              danger
                              onClick={() => removeHeaderRow(index)}
                            >
                              <DeleteIcon size={15} />
                            </IconButton>
                          </div>
                        ))}
                      </div>
                    )}
                  </div>
                </>
              ) : null}

              {editor.formType === "npx" ? (
                <>
                  <label className="field-group">
                    <span>{t("mcp.editor.packageLabel")}</span>
                    <input
                      type="text"
                      value={editor.packageName}
                      placeholder="@modelcontextprotocol/server-filesystem"
                      onChange={(event) => updateEditor({ packageName: event.currentTarget.value })}
                    />
                  </label>
                  <label className="field-group">
                    <span>{t("mcp.editor.extraArgsLabel")}</span>
                    <textarea
                      className="mcp-editor-kv-textarea"
                      value={editor.extraArgsText}
                      placeholder={t("mcp.editor.argsPlaceholder")}
                      onChange={(event) => updateEditor({ extraArgsText: event.currentTarget.value })}
                    />
                  </label>
                </>
              ) : null}

              {editor.formType === "binary" ? (
                <>
                  <label className="field-group">
                    <span>{t("mcp.editor.commandLabel")}</span>
                    <input
                      type="text"
                      value={editor.commandPath}
                      placeholder="C:\\tools\\mcp-server.exe"
                      onChange={(event) => updateEditor({ commandPath: event.currentTarget.value })}
                    />
                  </label>
                  <label className="field-group">
                    <span>{t("mcp.editor.argsLabel")}</span>
                    <textarea
                      className="mcp-editor-kv-textarea"
                      value={editor.argsText}
                      placeholder={t("mcp.editor.argsPlaceholder")}
                      onChange={(event) => updateEditor({ argsText: event.currentTarget.value })}
                    />
                  </label>
                </>
              ) : null}

              {editor.formType === "npx" || editor.formType === "binary" ? (
                <label className="field-group">
                  <span>{t("mcp.editor.envLabel")}</span>
                  <textarea
                    className="mcp-editor-kv-textarea"
                    value={editor.envText}
                    placeholder={t("mcp.editor.envPlaceholder")}
                    onChange={(event) => updateEditor({ envText: event.currentTarget.value })}
                  />
                </label>
              ) : null}

              {editor.formType === "custom" ? (
                <div className="field-group">
                  <span className="mcp-editor-config-label-row">
                    {t("mcp.editor.configLabel")}
                    <IconButton
                      label={t("mcp.editor.autoDetect")}
                      className="ui-icon-button"
                      onClick={handleAutoDetect}
                    >
                      <AutoDetectIcon size={15} />
                    </IconButton>
                  </span>
                  <textarea
                    className="plan-textarea mcp-editor-textarea"
                    value={editor.customJson}
                    onChange={(event) => updateEditor({ customJson: event.currentTarget.value })}
                  />
                </div>
              ) : null}
            </div>
            {editor.error ? <div className="mcp-editor-error">{editor.error}</div> : null}
            <div className="dialog-actions mcp-editor-actions">
              {editor.formType === "http" ? (
                <div className="mcp-editor-test-group">
                  <button type="button" className="ghost-button" disabled={testing} onClick={() => void handleTestConnection()}>
                    {testing ? t("mcp.editor.testing") : t("mcp.editor.testConnection")}
                  </button>
                  {testResult ? (
                    <span className={`mcp-editor-test-result mcp-editor-test-result--${testResult.kind === "ok" ? "ok" : "error"}`}>
                      {testResult.kind === "ok" ? t("mcp.editor.testResult.ok") : null}
                      {testResult.kind === "unauthorized" ? t("mcp.editor.testResult.unauthorized") : null}
                      {testResult.kind === "unexpectedResponse" ? t("mcp.editor.testResult.unexpectedResponse", { status: testResult.status }) : null}
                      {testResult.kind === "connectionFailed" ? t("mcp.editor.testResult.connectionFailed", { message: testResult.message }) : null}
                    </span>
                  ) : null}
                </div>
              ) : <span />}
              <div className="mcp-editor-confirm-group">
                <button type="button" className="ghost-button" onClick={() => setEditor(null)} disabled={busy}>
                  {t("dialog.cancel")}
                </button>
                <button type="button" className="primary-button" onClick={() => void handleSaveEditor()} disabled={busy}>
                  {t("mcp.action.save")}
                </button>
              </div>
            </div>
          </article>
        </div>
      ) : null}

      {deleteTarget ? (
        <ConfirmDialog
          dialog={{
            title: t("mcp.delete.title"),
            message: t("mcp.delete.message", { name: deleteTarget.name }),
            actionLabel: t("mcp.action.delete"),
            tone: "danger",
            onConfirm: () => void handleConfirmDelete(),
          }}
          onCancel={() => setDeleteTarget(null)}
        />
      ) : null}

      {copySource ? (
        <div className="dialog-backdrop">
          <article className="dialog-card dialog-card--solid mcp-copy-dialog">
            <h3>{t("mcp.copy.title", { name: copySource.name })}</h3>
            <div className="mcp-editor-fields">
              <label className="field-group">
                <span>{t("mcp.copy.targetLabel")}</span>
                <Select
                  value={copyTargetProvider}
                  onChange={(event) => {
                    setCopyTargetProvider(event.currentTarget.value);
                    setCopyError(null);
                  }}
                >
                  {copyTargetOptions.map((provider) => (
                    <option key={provider.providerId} value={provider.providerId}>
                      {t(`mcp.provider.${provider.providerId}` as never)}
                    </option>
                  ))}
                </Select>
              </label>
              <label className="field-group">
                <span>{t("mcp.editor.nameLabel")}</span>
                <input
                  type="text"
                  value={copyName}
                  onChange={(event) => {
                    setCopyName(event.currentTarget.value);
                    setCopyError(null);
                  }}
                />
              </label>
            </div>
            {copyError ? <div className="mcp-editor-error">{copyError}</div> : null}
            <div className="dialog-actions">
              <button type="button" className="ghost-button" onClick={() => setCopySource(null)} disabled={busy}>
                {t("dialog.cancel")}
              </button>
              <button type="button" className="primary-button" onClick={() => void handleConfirmCopy()} disabled={busy}>
                {t("mcp.action.copyTo")}
              </button>
            </div>
          </article>
        </div>
      ) : null}
    </div>
  );
}
