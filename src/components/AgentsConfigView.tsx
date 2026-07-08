import DOMPurify from "dompurify";
import { marked } from "marked";
import { useCallback, useEffect, useMemo, useState } from "react";
import { useI18n } from "../i18n/I18nProvider";
import { prepareMarkdownForPreview } from "../utils/splitFrontmatter";
import type {
  AgentsMdEntry,
  AgentsMdScanResult,
  AgentsScope,
  CommandsScanResult,
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
import { ContentViewer } from "./ContentViewer";
import { ExplorerTree } from "./ExplorerTree";
import {
  ChevronLeftIcon,
  EditNotesIcon,
  ExternalLinkIcon,
  EyeIcon,
  FolderIcon,
  RefreshIcon,
  SyncIcon,
} from "./Icons";

type AgentsTab = "agents-md" | "skills" | "commands";

type Props = {
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
  onReadFile: (filePath: string) => Promise<string>;
  onWriteFile: (filePath: string, content: string) => Promise<void>;
  onOpenExternal: (path: string) => void;
  onRevealPath: (path: string) => void;
  onPreviewSync: (request: SyncRequest) => Promise<SyncReport>;
  onApplySync: (request: SyncRequest) => Promise<SyncReport>;
  onUpdatePrefs: (prefs: ProjectAgentsPrefs) => Promise<void>;
};

type MatrixEntry = SkillEntry | SkillsScanResult["skills"][number] | CommandsScanResult["commands"][number];

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
      return "done";
    case "target-missing":
    case "source-missing":
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

function getSelectableNode(nodes: TreeNode[]): TreeNode | null {
  for (const node of nodes) {
    if (node.filePath) return node;
    if (node.children?.length) {
      const matched = getSelectableNode(node.children);
      if (matched) return matched;
    }
  }
  return null;
}

export function AgentsConfigView({
  scope,
  agentsMdData,
  skillsData,
  commandsData,
  prefs = DEFAULT_PREFS,
  isAgentsMdLoading,
  isSkillsLoading,
  isCommandsLoading,
  isPrefsLoading,
  onRefreshAgentsMd,
  onRefreshSkills,
  onRefreshCommands,
  onReadFile,
  onWriteFile,
  onOpenExternal,
  onRevealPath,
  onPreviewSync,
  onApplySync,
  onUpdatePrefs,
}: Props) {
  const { t } = useI18n();
  const scopeStoragePrefix = getScopeStorageKey(scope, "");
  const [activeTab, setActiveTab] = useState<AgentsTab>(() => {
    const stored = window.localStorage.getItem(getScopeStorageKey(scope, "tab"));
    return stored === "skills" || stored === "commands" ? stored : "agents-md";
  });
  const [selectedNode, setSelectedNode] = useState<TreeNode | null>(null);
  const [content, setContent] = useState<string | null>(null);
  const [contentLoading, setContentLoading] = useState(false);
  const [contentError, setContentError] = useState<string | null>(null);
  const [isEditing, setIsEditing] = useState(false);
  const [draft, setDraft] = useState("");
  const [syncMode, setSyncMode] = useState<SyncMode>("copy");
  const [selectedMatrixNames, setSelectedMatrixNames] = useState<string[]>([]);
  const [syncReport, setSyncReport] = useState<SyncReport | null>(null);
  const [selectedActionKeys, setSelectedActionKeys] = useState<string[]>([]);

  const treeNodes = useMemo(() => (agentsMdData ? buildAgentsMdTree(agentsMdData, t) : []), [agentsMdData, t]);

  useEffect(() => {
    window.localStorage.setItem(getScopeStorageKey(scope, "tab"), activeTab);
  }, [activeTab, scopeStoragePrefix]);

  useEffect(() => {
    const stored = window.localStorage.getItem(getScopeStorageKey(scope, "tab"));
    setActiveTab(stored === "skills" || stored === "commands" ? stored : "agents-md");
    setSelectedNode(null);
    setContent(null);
    setContentError(null);
    setSelectedMatrixNames([]);
    setSyncReport(null);
    setSelectedActionKeys([]);
  }, [scopeStoragePrefix]);

  useEffect(() => {
    if (activeTab !== "agents-md") return;
    if (!treeNodes.length) {
      setSelectedNode(null);
      setContent(null);
      return;
    }

    setSelectedNode((current) => {
      if (current && treeNodes.some((node) => JSON.stringify(node).includes(current.id))) {
        return current;
      }
      return getSelectableNode(treeNodes);
    });
  }, [activeTab, treeNodes]);

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

  useEffect(() => {
    if (!selectedNode?.filePath || activeTab !== "agents-md") return;
    void loadContent(selectedNode.filePath);
  }, [activeTab, loadContent, selectedNode]);

  const selectedMdEntry = useMemo<AgentsMdEntry | null>(() => {
    if (!selectedNode?.filePath || !agentsMdData) return null;
    return agentsMdData.entries.find((entry) => entry.source.path === selectedNode.filePath || entry.target.path === selectedNode.filePath) ?? null;
  }, [agentsMdData, selectedNode]);

  const targetInfos = useMemo<TargetInfo[]>(() => {
    if (activeTab === "skills") return skillsData?.targets ?? [];
    if (activeTab === "commands") return commandsData?.targets ?? [];
    return [];
  }, [activeTab, commandsData?.targets, skillsData?.targets]);

  const matrixEntries = useMemo<(SkillEntry | CommandsScanResult["commands"][number])[]>(() => {
    if (activeTab === "skills") return skillsData?.skills ?? [];
    if (activeTab === "commands") return commandsData?.commands ?? [];
    return [];
  }, [activeTab, commandsData?.commands, skillsData?.skills]);

  const previewHtml = useMemo(() => buildPreviewHtml(draft), [draft]);
  const reportSummary = useMemo(() => (syncReport ? summarizeReport(syncReport) : null), [syncReport]);

  const resolveTargetStatuses = (entry: MatrixEntry): TargetStatus[] => entry.targets ?? [];

  const previewNode = selectedNode?.id.startsWith("agents-preview:") ? selectedNode : null;

  const toScopeRelativePath = (path: string): string => {
    if (scope.kind !== "project") return path;
    const normalized = path.replace(/\//g, "\\");
    const normalizedRoot = scope.projectCwd.replace(/\//g, "\\").replace(/[\\]+$/, "");
    if (normalized.toLowerCase().startsWith(`${normalizedRoot.toLowerCase()}\\`)) {
      return normalized.slice(normalizedRoot.length + 1);
    }
    return path;
  };

  const handlePrefsChange = async (nextPrefs: ProjectAgentsPrefs) => {
    await onUpdatePrefs({ ...DEFAULT_PREFS, ...nextPrefs });
  };

  const toggleTarget = async (targetId: string, enabled: boolean) => {
    const nextEnabledTargets = enabled
      ? [...new Set([...(prefs.enabledTargets ?? []), targetId])]
      : (prefs.enabledTargets ?? []).filter((item) => item !== targetId);
    await handlePrefsChange({ ...prefs, enabledTargets: nextEnabledTargets });
  };

  const buildAgentsMdSyncRequest = (entry: AgentsMdEntry, dryRun: boolean): SyncRequest => ({
    items: [{ source: entry.source.path, target: entry.target.path }],
    dryRun,
    force: false,
    mode: "copy",
  });

  const buildMatrixSyncRequest = (dryRun: boolean): SyncRequest => {
    const items: SyncItem[] = [];

    for (const entry of matrixEntries) {
      if (!selectedMatrixNames.includes(entry.name)) continue;
      for (const target of resolveTargetStatuses(entry)) {
        if (!(prefs.enabledTargets ?? []).includes(target.targetId)) continue;
        const targetInfo = targetInfos.find((item) => item.targetId === target.targetId);
        if (!targetInfo?.rootExists) continue;

        if (activeTab === "skills") {
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
      mode: activeTab === "skills" ? syncMode : "copy",
      projectCwd: scope.kind === "project" ? scope.projectCwd : null,
    };
  };

  const handlePreview = async (request: SyncRequest) => {
    const report = await onPreviewSync(request);
    setSyncReport(report);
    setSelectedActionKeys(report.actions.map((action, index) => `${action.source}|${action.target}|${action.action}|${index}`));
  };

  const handleApply = async (request: SyncRequest) => {
    const nextReport = await onApplySync(request);
    setSyncReport(nextReport);
    setSelectedActionKeys(nextReport.actions.map((action, index) => `${action.source}|${action.target}|${action.action}|${index}`));
  };

  const selectedItemsFromReport = useMemo<SyncItem[]>(() => {
    if (!syncReport) return [];
    return syncReport.actions.flatMap((action, index) => {
      const key = `${action.source}|${action.target}|${action.action}|${index}`;
      if (!selectedActionKeys.includes(key)) return [];
      return [{
        source: action.source,
        target: action.target,
        itemKind: activeTab === "skills" ? "directory" : "file",
      }];
    });
  }, [activeTab, selectedActionKeys, syncReport]);

  const closeDetail = () => {
    setSelectedNode(null);
    setContent(null);
    setContentError(null);
  };

  const renderMatrix = (
    entries: (SkillEntry | CommandsScanResult["commands"][number])[],
    isLoading: boolean,
    onRefresh: () => Promise<void>,
  ) => {
    // 詳情頁存在時整頁切換：隱藏列表工具列、矩陣與同步報告，僅顯示返回鈕與內容檢視。
    if (previewNode?.filePath) {
      return (
        <div className="agents-detail-view">
          <div className="agents-detail-header">
            <button type="button" className="ghost-button agents-detail-back" onClick={closeDetail}>
              <ChevronLeftIcon size={16} />
              {t("agents.action.back")}
            </button>
            <div className="agents-detail-meta">
              <strong>{previewNode.label}</strong>
              <span>{toScopeRelativePath(previewNode.filePath)}</span>
            </div>
            <div className="settings-actions agents-toolbar-actions">
              <button type="button" className="ghost-button agents-icon-button" onClick={() => previewNode.filePath && onOpenExternal(previewNode.filePath)} title={t("agents.action.openExternal")}>
                <ExternalLinkIcon size={15} />
              </button>
              <button type="button" className="ghost-button agents-icon-button" onClick={() => previewNode.filePath && onRevealPath(previewNode.filePath)} title={t("agents.action.reveal")}>
                <FolderIcon size={15} />
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
        </div>
      );
    }

    return (
    <div className="agents-matrix-layout">
        <div className="agents-toolbar agents-toolbar--compact">
          <div className="agents-toolbar-meta">
            <strong>{activeTab === "skills" ? t("agents.tab.skills") : t("agents.tab.commands")}</strong>
            <span>{isPrefsLoading ? t("agents.prefs.loading") : t("agents.prefs.targetsHint")}</span>
          </div>
          <div className="settings-actions agents-toolbar-actions">
            {activeTab === "skills" ? (
              <label className="settings-field agents-mode-select-wrap">
                <span>{t("agents.action.syncMode")}</span>
              <select value={syncMode} onChange={(event) => setSyncMode(event.currentTarget.value as SyncMode)}>
                <option value="copy">{t("agents.action.syncMode.copy")}</option>
                <option value="link">{t("agents.action.syncMode.link")}</option>
              </select>
            </label>
          ) : null}
            <button
              type="button"
              className="ghost-button"
              disabled={selectedMatrixNames.length === 0}
              onClick={() => void handlePreview(buildMatrixSyncRequest(true))}
            >
              {t("agents.action.previewSync")}
            </button>
            <button
              type="button"
              className="primary-button"
              disabled={selectedMatrixNames.length === 0}
              onClick={() => void handleApply(buildMatrixSyncRequest(false))}
            >
              {t("agents.action.applySync")}
            </button>
            <button type="button" className="ghost-button agents-icon-button" onClick={() => void onRefresh()} title={t("app.actions.refresh")}>
              <RefreshIcon size={15} />
            </button>
          </div>
        </div>

      {isLoading ? <div className="explorer-content-loading">{t("plansSpecs.loading")}</div> : null}
      {!isLoading && entries.length === 0 ? <div className="explorer-content-empty">{t(`agents.empty.${activeTab}` as never)}</div> : null}

      {!isLoading && entries.length > 0 ? (
        <div className="agents-matrix-card">
          <table className="agents-matrix-table">
            <thead>
              <tr>
                <th>{t("agents.table.item")}</th>
                {targetInfos.map((target) => (
                  <th key={target.targetId}>
                    <label className="checkbox-group checkbox-group--inline agents-target-toggle">
                      <input
                        type="checkbox"
                        checked={(prefs.enabledTargets ?? []).includes(target.targetId)}
                        disabled={!target.rootExists}
                        onChange={(event) => void toggleTarget(target.targetId, event.currentTarget.checked)}
                      />
                      <span>{target.targetId}</span>
                    </label>
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
                        checked={selectedMatrixNames.includes(entry.name)}
                        onChange={(event) => {
                          // currentTarget 在事件派發結束後會被設為 null，必須先取值再進 updater。
                          const checked = event.currentTarget.checked;
                          setSelectedMatrixNames((current) => checked
                            ? [...current, entry.name]
                            : current.filter((name) => name !== entry.name));
                        }}
                      />
                      <button
                        type="button"
                        className="agents-link-button"
                        onClick={() => {
                          const previewPath = activeTab === "skills"
                            ? (entry as SkillEntry).skillMdPath
                            : (entry as CommandsScanResult["commands"][number]).sourcePath;
                          setSelectedNode({
                            id: `agents-preview:${previewPath}`,
                            label: entry.name,
                            filePath: previewPath,
                            filePathType: "absolute",
                          });
                          void loadContent(previewPath);
                        }}
                      >
                        {entry.name}
                      </button>
                    </div>
                  </td>
                  {targetInfos.map((targetInfo) => {
                    const status = resolveTargetStatuses(entry).find((item) => item.targetId === targetInfo.targetId);
                    const statusValue = status?.status ?? "target-missing";
                    const isClickable = statusValue !== "target-missing";
                    const handleTargetPreview = () => {
                      if (!status) return;
                      const targetPath = activeTab === "skills"
                        ? joinPath(joinPath(status.targetRoot, entry.name), "SKILL.md")
                        : joinPath(status.targetRoot, commandFileName(entry.name, targetInfo.targetId));
                      setSelectedNode({
                        id: `agents-preview:${targetPath}`,
                        label: `${entry.name} (${targetInfo.targetId})`,
                        filePath: targetPath,
                        filePathType: "absolute",
                      });
                      void loadContent(targetPath);
                    };
                    const pillClass = `agents-status-pill agents-status-pill--${getStatusTone(statusValue)}`;
                    return (
                      <td key={`${entry.name}:${targetInfo.targetId}`}>
                        {isClickable ? (
                          <button type="button" className={pillClass} title={t(`agents.status.${statusValue}` as never)} onClick={handleTargetPreview}>
                            {status?.targetNewer ? `${t("agents.status.targetNewer")} ` : ""}{t(`agents.status.${statusValue}` as never)}
                          </button>
                        ) : (
                          <span className={pillClass} title={t(`agents.status.${statusValue}` as never)}>
                            {status?.targetNewer ? `${t("agents.status.targetNewer")} ` : ""}{t(`agents.status.${statusValue}` as never)}
                          </span>
                        )}
                      </td>
                    );
                  })}
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      ) : null}

    </div>
    );
  };

  return (
    <section className="info-card agents-config-card">
      <div className="sub-tab-bar agents-top-tabs">
        {([
          ["agents-md", t("agents.tab.agentsMd")],
          ["skills", t("agents.tab.skills")],
          ["commands", t("agents.tab.commands")],
        ] as const).map(([value, label]) => (
          <button
            key={value}
            type="button"
            className={`sub-tab-item ${activeTab === value ? "sub-tab-item--active" : ""}`}
            onClick={() => {
              if (value === activeTab) return;
              setActiveTab(value);
              setSelectedNode(null);
              setContent(null);
              setContentError(null);
              setSyncReport(null);
              setSelectedActionKeys([]);
            }}
          >
            {label}
          </button>
        ))}
      </div>

      {activeTab === "agents-md" ? (
        <div className="agents-md-layout">
          <div className="agents-toolbar">
            <div className="agents-toolbar-meta">
              <strong>{t("agents.tab.agentsMd")}</strong>
              <span>{agentsMdData?.truncated ? t("agents.report.truncated") : t("agents.empty.agentsMdHint")}</span>
            </div>
            <div className="settings-actions agents-toolbar-actions">
              <button type="button" className="ghost-button agents-icon-button" onClick={() => void onRefreshAgentsMd()} title={t("app.actions.refresh")}>
                <RefreshIcon size={15} />
              </button>
              <button type="button" className="ghost-button agents-icon-button" disabled={!selectedNode?.filePath} onClick={() => selectedNode?.filePath && onOpenExternal(selectedNode.filePath)} title={t("agents.action.openExternal")}>
                <ExternalLinkIcon size={15} />
              </button>
              <button type="button" className="ghost-button agents-icon-button" disabled={!selectedNode?.filePath} onClick={() => selectedNode?.filePath && onRevealPath(selectedNode.filePath)} title={t("agents.action.reveal")}>
                <FolderIcon size={15} />
              </button>
              <button type="button" className="ghost-button agents-icon-button" disabled={!selectedMdEntry} onClick={() => selectedMdEntry && void handlePreview(buildAgentsMdSyncRequest(selectedMdEntry, true))} title={t("agents.action.syncThisDir")}>
                <SyncIcon size={15} />
              </button>
            </div>
          </div>

          {treeNodes.length === 0 && !isAgentsMdLoading ? (
            <div className="explorer-content-empty">{t("agents.empty.agentsMd")}</div>
          ) : (
            <div className="explorer-layout" style={{ ["--explorer-width" as string]: "320px" }}>
              <div className="explorer-panel">
                <div className="explorer-panel-header">
                  <div className="explorer-panel-heading">
                    <strong className="explorer-panel-title">{t("agents.tree.title")}</strong>
                  </div>
                </div>
                <ExplorerTree nodes={treeNodes} selectedId={selectedNode?.id ?? null} onSelect={setSelectedNode} />
              </div>
              <div className="explorer-content agents-content-pane">
                <div className="agents-content-actions">
                  <button type="button" className="ghost-button agents-inline-button" disabled={!selectedNode?.filePath} onClick={() => setIsEditing((current) => !current)}>
                    {isEditing ? <EyeIcon size={15} /> : <EditNotesIcon size={15} />}
                    {isEditing ? t("agents.action.preview") : t("agents.action.edit")}
                  </button>
                  <button
                    type="button"
                    className="primary-button agents-inline-button"
                    disabled={!selectedNode?.filePath || !isEditing}
                    onClick={async () => {
                      if (!selectedNode?.filePath) return;
                      await onWriteFile(selectedNode.filePath, draft);
                      setContent(draft);
                      setIsEditing(false);
                    }}
                  >
                    <SyncIcon size={15} />
                    {t("agents.action.save")}
                  </button>
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
                    content={content}
                    filePath={selectedNode?.filePath ?? null}
                    filePathType="absolute"
                    isLoading={contentLoading || isAgentsMdLoading}
                    error={contentError}
                    isTaskSaving={false}
                    onToggleTask={async () => {}}
                  />
                )}
              </div>
            </div>
          )}
        </div>
      ) : null}

      {activeTab === "skills" ? renderMatrix(skillsData?.skills ?? [], isSkillsLoading, onRefreshSkills) : null}
      {activeTab === "commands" ? renderMatrix(commandsData?.commands ?? [], isCommandsLoading, onRefreshCommands) : null}

      {syncReport && !(activeTab !== "agents-md" && previewNode?.filePath) ? (
        <section className="agents-sync-report">
          <div className="section-heading">
            <h3>{t("agents.report.title")}</h3>
            <span>
              {reportSummary?.create ?? 0}/{reportSummary?.overwrite ?? 0}/{reportSummary?.["skip-in-sync"] ?? 0}/{reportSummary?.error ?? 0}
            </span>
          </div>
          <div className="agents-report-list">
            {syncReport.actions.map((action, index) => {
              const key = `${action.source}|${action.target}|${action.action}|${index}`;
              return (
                <label key={key} className="agents-report-row">
                  <input
                    type="checkbox"
                    checked={selectedActionKeys.includes(key)}
                    onChange={(event) => {
                      const checked = event.currentTarget.checked;
                      setSelectedActionKeys((current) => checked ? [...current, key] : current.filter((item) => item !== key));
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
              onClick={() => void handleApply({
                items: selectedItemsFromReport,
                dryRun: false,
                force: false,
                mode: activeTab === "skills" ? syncMode : "copy",
              })}
            >
              {t("agents.action.applySync")}
            </button>
          </div>
        </section>
      ) : null}
    </section>
  );
}
