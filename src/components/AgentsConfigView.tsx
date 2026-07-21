import DOMPurify from "dompurify";
import { marked } from "marked";
import { useCallback, useEffect, useMemo, useState } from "react";
import { useI18n } from "../i18n/I18nProvider";
import type { MessageKey } from "../locales/zh-TW";
import { prepareMarkdownForPreview } from "../utils/splitFrontmatter";
import type {
  AgentsMdEntry,
  AgentsMdScanResult,
  AgentsRootLinkStatus,
  AgentsScope,
  CommandsScanResult,
  McpProviderConfig,
  ProjectAgentsPrefs,
  SkillEntry,
  SkillsScanResult,
  SyncItem,
  SyncMode,
  SyncReport,
  SyncRequest,
  SyncStatus,
  TargetInfo,
  TargetStatus,
  TreeNode,
} from "../types";
import { buildAgentsMdTree } from "../utils/buildTree";
import { CollapsibleSection } from "./CollapsibleSection";
import { ContentViewer } from "./ContentViewer";
import { ExplorerTree } from "./ExplorerTree";
import { McpConfigView } from "./McpConfigView";
import {
  ChevronLeftIcon,
  EditNotesIcon,
  ExternalLinkIcon,
  EyeIcon,
  FolderIcon,
  RefreshIcon,
  SaveIcon,
  SearchIcon,
  SyncIcon,
} from "./Icons";
import { IconButton } from "./ui/IconButton";

type AgentsTab = "agents-md" | "skills" | "commands" | "mcp";

/** 單一 scope（專案或全域）的 Agents 相關資料與 handlers 集合。 */
export type AgentsScopeDataBundle = {
  scope: AgentsScope;
  agentsMdData?: AgentsMdScanResult;
  skillsData?: SkillsScanResult;
  commandsData?: CommandsScanResult;
  prefs: ProjectAgentsPrefs;
  isAgentsMdLoading: boolean;
  isSkillsLoading: boolean;
  isCommandsLoading: boolean;
  isPrefsLoading: boolean;
  onRefreshAgentsMd: () => Promise<void>;
  onRefreshSkills: () => Promise<void>;
  onRefreshCommands: () => Promise<void>;
  onPreviewSync: (request: SyncRequest) => Promise<SyncReport>;
  onApplySync: (request: SyncRequest) => Promise<SyncReport>;
  onUpdatePrefs: (prefs: ProjectAgentsPrefs) => Promise<void>;
  agentsRootLinkStatus?: AgentsRootLinkStatus | null;
  onCreateAgentsRootLink?: () => Promise<void>;
  mcpProviders: McpProviderConfig[];
  mcpLoading: boolean;
  onRefreshMcp: () => Promise<void>;
  onUpsertMcpServer: (
    provider: string,
    name: string,
    originalName: string | null | undefined,
    configJson: string,
  ) => Promise<unknown>;
  onDeleteMcpServer: (provider: string, name: string) => Promise<unknown>;
  onSetMcpServerEnabled: (provider: string, name: string, enabled: boolean) => Promise<unknown>;
  codexTrusted?: boolean;
};

type Props = AgentsScopeDataBundle & {
  onReadFile: (filePath: string) => Promise<string>;
  onWriteFile: (filePath: string, content: string) => Promise<void>;
  onOpenExternal: (path: string) => void;
  onRevealPath: (path: string) => void;
  /** 專案上下文中傳入全域資料，頁籤內容會多顯示一個「全域」收折群組。 */
  globalData?: AgentsScopeDataBundle;
};

type MatrixEntry = SkillEntry | CommandsScanResult["commands"][number];

const AGENTS_TARGET_ID = "agents";

const DEFAULT_PREFS: ProjectAgentsPrefs = {
  conflictChoice: null,
  ignoredPaths: [],
  enabledTargets: [],
};

function joinPath(base: string, tail: string): string {
  const separator = base.includes("\\") ? "\\" : "/";
  return `${base.replace(/[\\/]+$/, "")}${separator}${tail.replace(/^[\\/]+/, "")}`;
}

// GitHub Copilot 的 .github/prompts/ 慣例要求副檔名為 .prompt.md，其餘 target 一律 .md。
function commandFileName(name: string, targetId: string): string {
  return targetId === "copilot" ? `${name}.prompt.md` : `${name}.md`;
}

function getScopeStorageKey(scope: AgentsScope, key: string): string {
  return scope.kind === "global" ? `agents:${key}:global` : `agents:${key}:${scope.projectCwd.toLowerCase()}`;
}

function scopeGroupKey(scope: AgentsScope): "project" | "global" {
  return scope.kind === "global" ? "global" : "project";
}

function summarizeReport(report: SyncReport) {
  return report.actions.reduce(
    (summary, action) => {
      summary[action.action] = (summary[action.action] ?? 0) + 1;
      return summary;
    },
    {} as Record<string, number>,
  );
}

function getStatusTone(status: SyncStatus): string {
  switch (status) {
    case "in-sync":
    case "linked":
    case "canonical":
      return "done";
    case "target-missing":
    case "source-missing":
    case "not-in-source":
      return "not_started";
    case "differs":
    case "link-broken":
      return "in_progress";
    default:
      return "neutral";
  }
}

function buildPreviewHtml(content: string): string {
  return DOMPurify.sanitize(marked.parse(prepareMarkdownForPreview(content || ""), { async: false }));
}

function matchesSearch(entry: MatrixEntry, query: string): boolean {
  if (!query) return true;
  const haystack = `${entry.name} ${entry.description ?? ""}`.toLowerCase();
  return haystack.includes(query);
}

// 設計稿固定四平台順序（Agents.dc.html）。
const CHIP_PLATFORMS = ["claude", "codex", "opencode", "copilot"] as const;

type ChipState = "loaded" | "needsSync" | "notInstalled";

/** 依 target 的同步狀態與是否啟用，對應為三種晶片視覺狀態（design D1）。 */
function chipStateFromStatus(status: SyncStatus | undefined, enabled: boolean): ChipState {
  if (!enabled) return "notInstalled";
  switch (status) {
    case "in-sync":
    case "linked":
    case "canonical":
      return "loaded";
    case "differs":
    case "target-missing":
    case "link-broken":
    case "error":
      return "needsSync";
    default:
      // source-missing / not-in-source / undefined
      return "notInstalled";
  }
}

export function AgentsConfigView(props: Props) {
  const { t } = useI18n();
  const { onReadFile, onWriteFile, onOpenExternal, onRevealPath, globalData, ...projectOrGlobalData } = props;
  const primary = projectOrGlobalData as AgentsScopeDataBundle;
  const scopeStoragePrefix = getScopeStorageKey(primary.scope, "");

  const [activeTab, setActiveTab] = useState<AgentsTab>(() => {
    const stored = window.localStorage.getItem(getScopeStorageKey(primary.scope, "tab"));
    return stored === "skills" || stored === "commands" || stored === "mcp" ? stored : "agents-md";
  });
  const [selectedNode, setSelectedNode] = useState<TreeNode | null>(null);
  const [content, setContent] = useState<string | null>(null);
  const [contentLoading, setContentLoading] = useState(false);
  const [contentError, setContentError] = useState<string | null>(null);
  const [isEditing, setIsEditing] = useState(false);
  const [draft, setDraft] = useState("");
  const [searchQuery, setSearchQuery] = useState("");
  const [syncModalTab, setSyncModalTab] = useState<AgentsTab | null>(null);

  const groups: AgentsScopeDataBundle[] = globalData ? [primary, globalData] : [primary];

  useEffect(() => {
    window.localStorage.setItem(getScopeStorageKey(primary.scope, "tab"), activeTab);
  }, [activeTab, scopeStoragePrefix]);

  useEffect(() => {
    const stored = window.localStorage.getItem(getScopeStorageKey(primary.scope, "tab"));
    setActiveTab(stored === "skills" || stored === "commands" || stored === "mcp" ? stored : "agents-md");
    setSelectedNode(null);
    setContent(null);
    setContentError(null);
    setSearchQuery("");
    setSyncModalTab(null);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [scopeStoragePrefix]);

  useEffect(() => {
    setSelectedNode(null);
    setContent(null);
    setContentError(null);
    setSearchQuery("");
  }, [activeTab]);

  const loadContent = useCallback(async (filePath: string) => {
    setContentLoading(true);
    setContentError(null);
    try {
      const nextContent = await onReadFile(filePath);
      setContent(nextContent);
      setDraft(nextContent);
    } catch (error) {
      setContentError(error instanceof Error ? error.message : String(error));
    } finally {
      setContentLoading(false);
    }
  }, [onReadFile]);

  const previewNode = selectedNode?.id.startsWith("agents-preview:") ? selectedNode : null;

  useEffect(() => {
    if (!selectedNode?.filePath || (activeTab !== "agents-md" && !previewNode)) return;
    void loadContent(selectedNode.filePath);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [selectedNode]);

  const closeDetail = () => {
    setSelectedNode(null);
    setContent(null);
    setContentError(null);
  };

  const previewHtml = useMemo(() => buildPreviewHtml(draft), [draft]);

  const toScopeRelativePath = (scope: AgentsScope, path: string): string => {
    if (scope.kind !== "project") return path;
    const normalized = path.replace(/\//g, "\\");
    const normalizedRoot = scope.projectCwd.replace(/\//g, "\\").replace(/[\\]+$/, "");
    if (normalized.toLowerCase().startsWith(`${normalizedRoot.toLowerCase()}\\`)) {
      return normalized.slice(normalizedRoot.length + 1);
    }
    return path;
  };

  // ─── AGENTS.md 頁籤：單一群組的樹狀檢視 ─────────────────────────────────

  const renderAgentsMdGroup = (data: AgentsScopeDataBundle) => {
    const treeNodes = data.agentsMdData ? buildAgentsMdTree(data.agentsMdData, t) : [];
    const isActiveGroup = groupIsSelectedNodeOwner(data, selectedNode, treeNodes);
    const groupSelectedNode = isActiveGroup ? selectedNode : null;

    const selectedMdEntry: AgentsMdEntry | null = groupSelectedNode?.filePath && data.agentsMdData
      ? data.agentsMdData.entries.find(
        (entry) => entry.source.path === groupSelectedNode.filePath || entry.target.path === groupSelectedNode.filePath,
      ) ?? null
      : null;

    const mdActions = (
      <div className="settings-actions agents-toolbar-actions">
        <IconButton label={t("app.actions.refresh")} className="agents-icon-button" onClick={() => void data.onRefreshAgentsMd()}>
          <RefreshIcon size={15} />
        </IconButton>
        <IconButton label={t("agents.action.openExternal")} className="agents-icon-button" disabled={!groupSelectedNode?.filePath} onClick={() => groupSelectedNode?.filePath && onOpenExternal(groupSelectedNode.filePath)}>
          <ExternalLinkIcon size={15} />
        </IconButton>
        <IconButton label={t("agents.action.reveal")} className="agents-icon-button" disabled={!groupSelectedNode?.filePath} onClick={() => groupSelectedNode?.filePath && onRevealPath(groupSelectedNode.filePath)}>
          <FolderIcon size={15} />
        </IconButton>
        <IconButton label={t("agents.action.syncThisDir")} className="agents-icon-button" disabled={!selectedMdEntry} onClick={() => selectedMdEntry && void handlePreview(data, buildAgentsMdSyncRequest(selectedMdEntry, true))}>
          <SyncIcon size={15} />
        </IconButton>
      </div>
    );

    const body = (
      <div className="agents-md-layout">
        {treeNodes.length === 0 && !data.isAgentsMdLoading ? (
          <div className="explorer-content-empty">{t("agents.empty.agentsMd")}</div>
        ) : (
          <div className="explorer-layout" style={{ ["--explorer-width" as string]: "320px" }}>
            <div className="explorer-panel">
              <div className="explorer-panel-header">
                <div className="explorer-panel-heading">
                  <strong className="explorer-panel-title">{t("agents.tree.title")}</strong>
                </div>
              </div>
              <ExplorerTree
                nodes={treeNodes}
                selectedId={groupSelectedNode?.id ?? null}
                onSelect={(node) => setSelectedNode(node)}
              />
            </div>
            <div className="explorer-content agents-content-pane">
              <div className="agents-content-actions">
                <button type="button" className="ghost-button agents-inline-button" disabled={!groupSelectedNode?.filePath} onClick={() => setIsEditing((current) => !current)}>
                  {isEditing ? <EyeIcon size={15} /> : <EditNotesIcon size={15} />}
                  {isEditing ? t("agents.action.preview") : t("agents.action.edit")}
                </button>
                {isEditing ? (
                  <button
                    type="button"
                    className="primary-button agents-inline-button"
                    disabled={!groupSelectedNode?.filePath}
                    onClick={async () => {
                      if (!groupSelectedNode?.filePath) return;
                      await onWriteFile(groupSelectedNode.filePath, draft);
                      setContent(draft);
                      setIsEditing(false);
                    }}
                  >
                    <SaveIcon size={15} />
                    {t("agents.action.save")}
                  </button>
                ) : null}
              </div>

              {isEditing ? (
                <div className="plan-editor-layout">
                  <label className="field-group">
                    <span>{t("plan.editor")}</span>
                    <textarea className="plan-textarea" value={draft} onChange={(event) => setDraft(event.currentTarget.value)} />
                  </label>
                  <div className="plan-preview">
                    <span className="session-meta-label">{t("plan.preview")}</span>
                    <div className="plan-preview-markdown" dangerouslySetInnerHTML={{ __html: previewHtml }} />
                  </div>
                </div>
              ) : (
                <ContentViewer
                  content={groupSelectedNode ? content : null}
                  filePath={groupSelectedNode?.filePath ?? null}
                  filePathType="absolute"
                  isLoading={contentLoading || data.isAgentsMdLoading}
                  error={contentError}
                  isTaskSaving={false}
                  onToggleTask={async () => {}}
                />
              )}
            </div>
          </div>
        )}
      </div>
    );

    return { actions: mdActions, body };
  };

  function treeContainsNodeId(nodes: TreeNode[], id: string): boolean {
    return nodes.some((node) => node.id === id || (node.children ? treeContainsNodeId(node.children, id) : false));
  }

  function groupIsSelectedNodeOwner(data: AgentsScopeDataBundle, node: TreeNode | null, treeNodes: TreeNode[]): boolean {
    if (!node) return false;
    return treeContainsNodeId(treeNodes, node.id) || !data.agentsMdData;
  }

  const buildAgentsMdSyncRequest = (entry: AgentsMdEntry, dryRun: boolean): SyncRequest => ({
    items: [{ source: entry.source.path, target: entry.target.path }],
    dryRun,
    force: false,
    mode: "copy",
  });

  const handlePreview = async (data: AgentsScopeDataBundle, request: SyncRequest) => {
    await data.onPreviewSync(request);
  };

  // ─── Skills / Commands 頁籤：VS Code 式清單 + 同步 modal ─────────────────

  const resolveTargetInfos = (tab: "skills" | "commands", data: AgentsScopeDataBundle): TargetInfo[] => {
    if (tab === "skills") {
      const backendTargets = data.skillsData?.targets ?? [];
      const hasBackendAgentsTarget = backendTargets.some((target) => target.targetId === AGENTS_TARGET_ID);
      if (!hasBackendAgentsTarget && data.skillsData) {
        const sourceRootExists = data.skillsData.skills.some((skill) =>
          (skill.targets ?? []).every((status) => status.status !== "source-missing"),
        );
        return [
          { targetId: AGENTS_TARGET_ID, root: data.skillsData.sourceRoot, rootExists: sourceRootExists },
          ...backendTargets,
        ];
      }
      return backendTargets;
    }
    return data.commandsData?.targets ?? [];
  };

  const resolveMatrixEntries = (tab: "skills" | "commands", data: AgentsScopeDataBundle): MatrixEntry[] => {
    if (tab === "skills") return data.skillsData?.skills ?? [];
    return data.commandsData?.commands ?? [];
  };

  const resolveTargetStatuses = (tab: "skills" | "commands", data: AgentsScopeDataBundle, entry: MatrixEntry): TargetStatus[] => {
    const backendStatuses = entry.targets ?? [];
    if (tab !== "skills" || !data.skillsData) return backendStatuses;
    const hasBackendAgentsTarget = (data.skillsData.targets ?? []).some((target) => target.targetId === AGENTS_TARGET_ID);
    if (hasBackendAgentsTarget) return backendStatuses;
    const isNotInSource = backendStatuses.some((status) => status.status === "source-missing");
    const virtualStatus: TargetStatus = {
      targetId: AGENTS_TARGET_ID,
      targetRoot: data.skillsData.sourceRoot,
      status: isNotInSource ? "not-in-source" : "canonical",
      targetNewer: false,
    };
    return [virtualStatus, ...backendStatuses];
  };

  const renderListGroup = (tab: "skills" | "commands", data: AgentsScopeDataBundle) => {
    const isLoading = tab === "skills" ? data.isSkillsLoading : data.isCommandsLoading;
    const onRefresh = tab === "skills" ? data.onRefreshSkills : data.onRefreshCommands;
    const allEntries = resolveMatrixEntries(tab, data);
    const query = searchQuery.trim().toLowerCase();
    const entries = allEntries.filter((entry) => matchesSearch(entry, query));

    const openPreview = (entry: MatrixEntry) => {
      const previewPath = tab === "skills" ? (entry as SkillEntry).skillMdPath : (entry as CommandsScanResult["commands"][number]).sourcePath;
      setSelectedNode({
        id: `agents-preview:${previewPath}`,
        label: entry.name,
        filePath: previewPath,
        filePathType: "absolute",
      });
    };

    const legend = (
      <span className="agents-sync-legend">
        {(["loaded", "needsSync", "notInstalled"] as const).map((state) => (
          <span key={state} className="agents-sync-legend-item">
            <span className={`agents-target-chip-dot agents-target-chip-dot--${state}`} />
            {t(`agents.legend.${state}` as never)}
          </span>
        ))}
      </span>
    );

    const actionButtons = (
      <div className="settings-actions agents-toolbar-actions">
        <button type="button" className="ghost-button" onClick={() => setSyncModalTab(tab)}>
          {t("agents.action.sync")}
        </button>
        <button type="button" className="ghost-button agents-icon-button" onClick={() => void onRefresh()} title={t("app.actions.refresh")}>
          <RefreshIcon size={15} />
        </button>
      </div>
    );

    return (
      <div className="agents-list-group">
        <div className="agents-skills-compat-note agents-skills-compat-note--withActions">
          <span>{t(tab === "skills" ? "agents.skills.compatNote" : "agents.commands.compatNote")}</span>
          {legend}
          {actionButtons}
        </div>

        {tab === "skills" && data.agentsRootLinkStatus && data.agentsRootLinkStatus.status !== "linked" ? (
          <div className={`agents-root-link-banner agents-root-link-banner--${data.agentsRootLinkStatus.status}`}>
            {data.agentsRootLinkStatus.status === "partial" ? (
              <>
                <div>
                  <strong>{t("agents.rootLink.partial.title")}</strong>
                  <span>{t("agents.rootLink.partial.description")}</span>
                  <span>{data.agentsRootLinkStatus.unmatchedItems.join(", ")}</span>
                </div>
              </>
            ) : data.agentsRootLinkStatus.status === "unlinked-physical" ? (
              <>
                <div>
                  <strong>{t("agents.rootLink.unlinkedPhysical.title")}</strong>
                  <span>{t("agents.rootLink.unlinkedPhysical.description")}</span>
                </div>
              </>
            ) : (
              <>
                <strong>{t("agents.rootLink.notLinked.title")}</strong>
                {data.agentsRootLinkStatus.status === "missing" ? (
                  <button type="button" className="ghost-button" onClick={() => void data.onCreateAgentsRootLink?.()}>
                    {t("agents.rootLink.notLinked.action")}
                  </button>
                ) : null}
              </>
            )}
          </div>
        ) : null}

        {isLoading ? <div className="explorer-content-loading">{t("plansSpecs.loading")}</div> : null}
        {!isLoading && allEntries.length === 0 ? <div className="explorer-content-empty">{t(`agents.empty.${tab}` as never)}</div> : null}
        {!isLoading && allEntries.length > 0 && entries.length === 0 ? <div className="explorer-content-empty">{t("agents.list.noSearchResults")}</div> : null}

        {!isLoading && entries.length > 0 ? (
          <ul className="agents-vscode-list">
            {entries.map((entry) => {
              const statuses = resolveTargetStatuses(tab, data, entry);
              const enabledTargets = data.prefs.enabledTargets ?? [];
              const chips = CHIP_PLATFORMS.map((platform) => {
                const status = statuses.find((item) => item.targetId === platform)?.status;
                const enabled = enabledTargets.includes(platform);
                const state = chipStateFromStatus(status, enabled);
                return {
                  platform,
                  state,
                  tooltip: t("agents.chip.tooltip", { platform, state: t(`agents.chip.${state}` as never) }),
                };
              });
              return (
                <li key={entry.name}>
                  <button type="button" className="agents-vscode-list-item" onClick={() => openPreview(entry)}>
                    <span className="agents-vscode-list-name">{entry.name}</span>
                    {entry.description ? (
                      <span className="agents-vscode-list-description" title={entry.description}>{entry.description}</span>
                    ) : null}
                    <span className="agents-target-chips">
                      {chips.map((chip) => (
                        <span key={chip.platform} className={`agents-target-chip agents-target-chip--${chip.state}`} title={chip.tooltip}>
                          <span className="agents-target-chip-dot" />
                          {chip.platform}
                        </span>
                      ))}
                    </span>
                  </button>
                </li>
              );
            })}
          </ul>
        ) : null}
      </div>
    );
  };

  // ─── 同步 modal ──────────────────────────────────────────────────────────

  const renderSyncModal = () => {
    if (!syncModalTab || (syncModalTab !== "skills" && syncModalTab !== "commands")) return null;
    const tab = syncModalTab;
    return (
      <div className="dialog-backdrop">
        <article className="dialog-card agents-sync-modal">
          <div className="agents-sync-modal-header">
            <h3>{tab === "skills" ? t("agents.tab.skills") : t("agents.tab.commands")} — {t("agents.action.sync")}</h3>
            <button type="button" className="ghost-button agents-icon-button" onClick={() => setSyncModalTab(null)} title={t("dialog.cancel")}>
              ×
            </button>
          </div>
          <div className="agents-sync-modal-scroll">
            <div className="agents-sync-modal-body">
              {groups.map((data) => (
                <SyncModalGroup
                  key={scopeGroupKey(data.scope)}
                  tab={tab}
                  data={data}
                  label={scopeGroupLabel(data.scope)}
                  targetInfos={resolveTargetInfos(tab, data)}
                  entries={resolveMatrixEntries(tab, data)}
                  resolveTargetStatuses={(entry) => resolveTargetStatuses(tab, data, entry)}
                  onOpenExternal={onOpenExternal}
                  t={t}
                />
              ))}
            </div>
          </div>
        </article>
      </div>
    );
  };

  // ─── MCP 頁籤：provider 分頁共用一列，其下 scope 群組 ─────────────────────

  const renderMcpTab = () => (
    <McpConfigView
      groups={groups.map((data) => ({
        scope: data.scope,
        label: scopeGroupLabel(data.scope),
        providers: data.mcpProviders,
        isLoading: data.mcpLoading,
        onRefresh: data.onRefreshMcp,
        onUpsert: data.onUpsertMcpServer,
        onDelete: data.onDeleteMcpServer,
        onSetEnabled: data.onSetMcpServerEnabled,
        codexTrusted: data.codexTrusted,
      }))}
      onOpenExternal={onOpenExternal}
      onRevealPath={onRevealPath}
    />
  );

  const scopeGroupLabel = (scope: AgentsScope): string =>
    scope.kind === "global" ? t("agents.section.global") : t("agents.section.project");

  const groupExpandKey = (tab: AgentsTab) => `agents:groupExpanded:${tab}`;

  return (
    <section className="agents-config-card">
      <div className="agents-top-bar">
        <div className="sub-tab-bar agents-top-tabs">
          {([
            ["agents-md", t("agents.tab.agentsMd")],
            ["skills", t("agents.tab.skills")],
            ["commands", t("agents.tab.commands")],
            ["mcp", t("agents.tab.mcp")],
          ] as const).map(([value, label]) => (
            <button
              key={value}
              type="button"
              className={`sub-tab-item ${activeTab === value ? "sub-tab-item--active" : ""}`}
              onClick={() => {
                if (value === activeTab) return;
                setActiveTab(value);
              }}
            >
              {label}
            </button>
          ))}
        </div>
        {(activeTab === "skills" || activeTab === "commands") ? (
          <label className="agents-search-field">
            <SearchIcon size={14} />
            <input
              type="text"
              value={searchQuery}
              placeholder={t("agents.list.searchPlaceholder")}
              onChange={(event) => setSearchQuery(event.currentTarget.value)}
            />
          </label>
        ) : null}
      </div>

      {activeTab === "agents-md" ? (
        globalData ? (
          <div className="agents-scope-groups">
            {groups.map((data) => {
              const { actions, body } = renderAgentsMdGroup(data);
              return (
                <CollapsibleGroup key={scopeGroupKey(data.scope)} scope={data.scope} storageKey={groupExpandKey("agents-md")} count={data.agentsMdData?.entries.length ?? 0} label={scopeGroupLabel(data.scope)} actions={actions}>
                  {body}
                </CollapsibleGroup>
              );
            })}
          </div>
        ) : (
          (() => {
            const { actions, body } = renderAgentsMdGroup(primary);
            return (
              <>
                <div className="agents-inline-header">{actions}</div>
                {body}
              </>
            );
          })()
        )
      ) : null}

      {(activeTab === "skills" || activeTab === "commands") ? (
        globalData ? (
          <div className="agents-scope-groups">
            {groups.map((data) => {
              const count = resolveMatrixEntries(activeTab, data).length;
              return (
                <CollapsibleGroup key={scopeGroupKey(data.scope)} scope={data.scope} storageKey={groupExpandKey(activeTab)} count={count} label={scopeGroupLabel(data.scope)}>
                  {renderListGroup(activeTab, data)}
                </CollapsibleGroup>
              );
            })}
          </div>
        ) : (
          renderListGroup(activeTab, primary)
        )
      ) : null}

      {activeTab === "mcp" ? renderMcpTab() : null}

      {previewNode?.filePath ? (
        <div className="dialog-backdrop">
          <article className="dialog-card agents-preview-modal">
            <div className="agents-detail-header">
              <button type="button" className="ghost-button agents-detail-back" onClick={closeDetail}>
                <ChevronLeftIcon size={16} />
                {t("agents.action.back")}
              </button>
              <div className="agents-detail-meta">
                <strong>{previewNode.label}</strong>
                <span>{toScopeRelativePath(primary.scope, previewNode.filePath)}</span>
              </div>
              <div className="settings-actions agents-toolbar-actions">
                <button type="button" className="ghost-button agents-icon-button" onClick={() => previewNode.filePath && onOpenExternal(previewNode.filePath)} title={t("agents.action.openExternal")}>
                  <ExternalLinkIcon size={15} />
                </button>
                <button type="button" className="ghost-button agents-icon-button" onClick={() => previewNode.filePath && onRevealPath(previewNode.filePath)} title={t("agents.action.reveal")}>
                  <FolderIcon size={15} />
                </button>
                <button type="button" className="ghost-button agents-icon-button agents-detail-close" onClick={closeDetail} title={t("tabs.close")} aria-label={t("tabs.close")}>
                  ×
                </button>
              </div>
            </div>
            <ContentViewer
              content={content}
              filePath={previewNode.filePath}
              filePathType="absolute"
              isLoading={contentLoading}
              error={contentError}
              isTaskSaving={false}
              onToggleTask={async () => {}}
            />
          </article>
        </div>
      ) : null}

      {renderSyncModal()}
    </section>
  );
}

// ─── 收折群組（帶計數） ──────────────────────────────────────────────────

function CollapsibleGroup({
  scope,
  storageKey,
  count,
  label,
  actions,
  children,
}: {
  scope: AgentsScope;
  storageKey: string;
  count: number;
  label: string;
  actions?: React.ReactNode;
  children: React.ReactNode;
}) {
  const key = `${storageKey}:${scope.kind === "global" ? "global" : scope.projectCwd.toLowerCase()}:${scope.kind}`;
  const [expanded, setExpanded] = useState(() => {
    const stored = window.localStorage.getItem(key);
    if (stored === "true") return true;
    if (stored === "false") return false;
    return scope.kind === "project";
  });

  const toggle = () => {
    setExpanded((current) => {
      const next = !current;
      window.localStorage.setItem(key, String(next));
      return next;
    });
  };

  return (
    <CollapsibleSection title={`${label} (${count})`} expanded={expanded} onToggle={toggle} actions={actions}>
      {children}
    </CollapsibleSection>
  );
}

// ─── 同步 modal 內單一群組（矩陣 + 預覽/套用 + 報告） ─────────────────────

function SyncModalGroup({
  tab,
  data,
  label,
  targetInfos,
  entries,
  resolveTargetStatuses,
  onOpenExternal,
  t,
}: {
  tab: "skills" | "commands";
  data: AgentsScopeDataBundle;
  label: string;
  targetInfos: TargetInfo[];
  entries: MatrixEntry[];
  resolveTargetStatuses: (entry: MatrixEntry) => TargetStatus[];
  onOpenExternal: (path: string) => void;
  t: (key: MessageKey, params?: Record<string, string | number>) => string;
}) {
  const [syncMode, setSyncMode] = useState<SyncMode>("link");
  const [selectedNames, setSelectedNames] = useState<string[]>([]);
  const [report, setReport] = useState<SyncReport | null>(null);
  const [selectedActionKeys, setSelectedActionKeys] = useState<string[]>([]);

  const prefs = data.prefs ?? DEFAULT_PREFS;

  const toggleTarget = async (targetId: string, enabled: boolean) => {
    const nextEnabledTargets = enabled
      ? [...new Set([...(prefs.enabledTargets ?? []), targetId])]
      : (prefs.enabledTargets ?? []).filter((item) => item !== targetId);
    await data.onUpdatePrefs({ ...DEFAULT_PREFS, ...prefs, enabledTargets: nextEnabledTargets });
  };

  const buildRequest = (dryRun: boolean): SyncRequest => {
    const items: SyncItem[] = [];
    for (const entry of entries) {
      if (!selectedNames.includes(entry.name)) continue;
      for (const target of resolveTargetStatuses(entry)) {
        if (!(prefs.enabledTargets ?? []).includes(target.targetId)) continue;
        const targetInfo = targetInfos.find((item) => item.targetId === target.targetId);
        if (!targetInfo?.rootExists) continue;

        if (tab === "skills") {
          items.push({
            source: (entry as SkillEntry).sourceDir,
            target: joinPath(target.targetRoot, entry.name),
            itemKind: "directory",
            targetId: target.targetId,
          });
        } else {
          items.push({
            source: (entry as CommandsScanResult["commands"][number]).syncSourcePath,
            target: joinPath(target.targetRoot, commandFileName(entry.name, target.targetId)),
            itemKind: "file",
            targetId: target.targetId,
          });
        }
      }
    }
    return {
      items,
      dryRun,
      force: false,
      mode: tab === "skills" ? syncMode : "copy",
      projectCwd: data.scope.kind === "project" ? data.scope.projectCwd : null,
    };
  };

  const handlePreview = async () => {
    const nextReport = await data.onPreviewSync(buildRequest(true));
    setReport(nextReport);
    setSelectedActionKeys(nextReport.actions.map((action, index) => `${action.source}|${action.target}|${action.action}|${index}`));
  };

  const handleApply = async (request: SyncRequest) => {
    const nextReport = await data.onApplySync(request);
    setReport(nextReport);
    setSelectedActionKeys(nextReport.actions.map((action, index) => `${action.source}|${action.target}|${action.action}|${index}`));
  };

  const selectedItemsFromReport = useMemo<SyncItem[]>(() => {
    if (!report) return [];
    return report.actions.flatMap((action, index) => {
      const key = `${action.source}|${action.target}|${action.action}|${index}`;
      if (!selectedActionKeys.includes(key)) return [];
      return [{ source: action.source, target: action.target, itemKind: tab === "skills" ? "directory" : "file" as const }];
    });
  }, [report, selectedActionKeys, tab]);

  const reportSummary = report ? summarizeReport(report) : null;

  return (
    <section className="agents-sync-modal-group">
      <div className="agents-sync-modal-group-header">
        <strong>{label}</strong>
        {tab === "skills" ? (
          <label className="settings-field agents-mode-select-wrap">
            <span>{t("agents.action.syncMode")}</span>
            <select value={syncMode} onChange={(event) => setSyncMode(event.currentTarget.value as SyncMode)}>
              <option value="copy">{t("agents.action.syncMode.copy")}</option>
              <option value="link">{t("agents.action.syncMode.link")}</option>
            </select>
          </label>
        ) : null}
      </div>

      {entries.length === 0 ? (
        <div className="explorer-content-empty">{t(`agents.empty.${tab}` as never)}</div>
      ) : (
        <div className="agents-matrix-card">
          <table className="agents-matrix-table">
            <thead>
              <tr>
                <th>{t("agents.table.item")}</th>
                {targetInfos.map((target) => (
                  <th key={target.targetId}>
                    <div className="agents-target-header">
                      <button
                        type="button"
                        className="agents-target-header-name"
                        disabled={!target.rootExists}
                        onClick={() => onOpenExternal(target.root)}
                      >
                        {target.targetId}
                      </button>
                      {target.targetId === AGENTS_TARGET_ID && tab === "skills" ? null : (
                        <label className="checkbox-group checkbox-group--inline agents-target-toggle">
                          <input
                            type="checkbox"
                            checked={(prefs.enabledTargets ?? []).includes(target.targetId)}
                            disabled={!target.rootExists}
                            onChange={(event) => void toggleTarget(target.targetId, event.currentTarget.checked)}
                          />
                        </label>
                      )}
                    </div>
                  </th>
                ))}
              </tr>
            </thead>
            <tbody>
              {entries.map((entry) => (
                <tr key={entry.name}>
                  <td>
                    <div className="agents-matrix-item-cell">
                      <input
                        type="checkbox"
                        checked={selectedNames.includes(entry.name)}
                        onChange={(event) => {
                          const checked = event.currentTarget.checked;
                          setSelectedNames((current) => (checked ? [...current, entry.name] : current.filter((name) => name !== entry.name)));
                        }}
                      />
                      <span>{entry.name}</span>
                    </div>
                  </td>
                  {targetInfos.map((targetInfo) => {
                    const status = resolveTargetStatuses(entry).find((item) => item.targetId === targetInfo.targetId);
                    const statusValue = status?.status ?? "target-missing";
                    const pillClass = `agents-status-pill agents-status-pill--${getStatusTone(statusValue)}`;
                    return (
                      <td key={`${entry.name}:${targetInfo.targetId}`}>
                        <span className={pillClass}>
                          {status?.targetNewer ? `${t("agents.status.targetNewer")} ` : ""}{t(`agents.status.${statusValue}` as never)}
                        </span>
                      </td>
                    );
                  })}
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}

      <div className="agents-sync-actions">
        <button type="button" className="ghost-button" disabled={selectedNames.length === 0} onClick={() => void handlePreview()}>
          {t("agents.action.previewSync")}
        </button>
        <button type="button" className="primary-button" disabled={selectedNames.length === 0} onClick={() => void handleApply(buildRequest(false))}>
          {t("agents.action.applySync")}
        </button>
      </div>

      {report ? (
        <section className="agents-sync-report">
          <div className="section-heading">
            <h3>{t("agents.report.title")}</h3>
            <span>
              {reportSummary?.create ?? 0}/{reportSummary?.overwrite ?? 0}/{reportSummary?.["skip-in-sync"] ?? 0}/{reportSummary?.error ?? 0}
            </span>
          </div>
          <div className="agents-report-list">
            {report.actions.map((action, index) => {
              const key = `${action.source}|${action.target}|${action.action}|${index}`;
              return (
                <label key={key} className="agents-report-row">
                  <input
                    type="checkbox"
                    checked={selectedActionKeys.includes(key)}
                    onChange={(event) => {
                      const checked = event.currentTarget.checked;
                      setSelectedActionKeys((current) => (checked ? [...current, key] : current.filter((item) => item !== key)));
                    }}
                  />
                  <span className={`agents-status-pill agents-status-pill--${action.action === "error" ? "not_started" : "neutral"}`}>
                    {t(`agents.report.action.${action.action}` as never)}
                  </span>
                  <code>{action.source}</code>
                  <code>{action.target}</code>
                  {action.reason ? <small>{action.reason}</small> : null}
                </label>
              );
            })}
          </div>
          <div className="agents-sync-actions">
            <button
              type="button"
              className="primary-button"
              disabled={selectedItemsFromReport.length === 0}
              onClick={() => void handleApply({ items: selectedItemsFromReport, dryRun: false, force: false, mode: tab === "skills" ? syncMode : "copy" })}
            >
              {t("agents.action.applySync")}
            </button>
          </div>
        </section>
      ) : null}
    </section>
  );
}
