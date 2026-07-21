import React, { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { useI18n } from "../i18n/I18nProvider";
import type { OpenSpecChange, OpenSpecData, SisyphusData, TreeNode } from "../types";
import { ARCHIVED_CHANGES_GROUP_ID, buildOpenSpecTree, buildSisyphusTree } from "../utils/buildTree";
import { ContentViewer } from "./ContentViewer";
import { ExplorerTree } from "./ExplorerTree";

type Props = {
  sisyphusData: SisyphusData | undefined;
  openspecData: OpenSpecData | undefined;
  isLoading: boolean;
  isRefreshing: boolean;
  onReadFileContent: (filePath: string) => Promise<string>;
  onReadOpenspecFile: (projectCwd: string, relativePath: string) => Promise<string>;
  onWriteOpenspecFile: (projectCwd: string, relativePath: string, content: string) => Promise<void>;
  onRefresh: () => Promise<void>;
  refreshToken: string;
  projectCwd: string;
};

type ExplorerViewMode = "tree" | "list" | "cols";
type SortField = "progress" | "name" | "createdAt";
type SortDir = "asc" | "desc";

type ChangeAction = {
  label: string;
  command: string | null;
  tone: "not_started" | "in_progress" | "done";
};

function resolveChangeAction(entryNode: TreeNode, isArchived: boolean): ChangeAction {
  const changeName = entryNode.id.replace(/^openspec:(change|archived):/, "");
  const children = entryNode.children ?? [];
  const hasProposal = children.some((c) => c.icon === "proposal");
  const hasTasks = children.some((c) => c.icon === "tasks");
  const progress = entryNode.progress;

  if (!hasProposal) {
    return { label: "待 propose", command: `/opsx:propose ${changeName}`, tone: "not_started" };
  }
  if (!hasTasks || !progress) {
    return { label: "可 apply", command: `/opsx:apply ${changeName}`, tone: "not_started" };
  }
  if (progress.done >= progress.total && progress.total > 0) {
    if (isArchived) {
      return { label: "已封存", command: null, tone: "done" };
    }
    return { label: "可封存", command: `/opsx:archive ${changeName}`, tone: "done" };
  }
  return {
    label: `進行中 ${progress.done}/${progress.total}`,
    command: `/opsx:apply ${changeName}`,
    tone: "in_progress",
  };
}

function sortChanges(changes: OpenSpecChange[], field: SortField, dir: SortDir): OpenSpecChange[] {
  const sorted = [...changes].sort((a, b) => {
    let cmp = 0;
    if (field === "name") {
      cmp = a.name.localeCompare(b.name);
    } else if (field === "progress") {
      const aVal = a.taskProgress && a.taskProgress.total > 0
        ? a.taskProgress.done / a.taskProgress.total
        : -1;
      const bVal = b.taskProgress && b.taskProgress.total > 0
        ? b.taskProgress.done / b.taskProgress.total
        : -1;
      cmp = aVal - bVal;
    } else {
      const aTime = a.createdAt ? Date.parse(a.createdAt) : Number.NaN;
      const bTime = b.createdAt ? Date.parse(b.createdAt) : Number.NaN;
      const hasATime = !Number.isNaN(aTime);
      const hasBTime = !Number.isNaN(bTime);

      // 沒有有效時間時維持相對原序，避免排序結果跳動。
      if (!hasATime && !hasBTime) return 0;
      if (!hasATime) return 1;
      if (!hasBTime) return -1;
      cmp = aTime - bTime;
    }
    return dir === "asc" ? cmp : -cmp;
  });
  return sorted;
}

const DEFAULT_EXPLORER_WIDTH = 300;
const TASK_LINE_PATTERN = /^(\s*(?:[-*+]|\d+\.)\s+\[)( |x|X)(\].*)$/;

function toggleMarkdownTask(content: string, taskIndex: number, checked: boolean): string {
  const lineEnding = content.includes("\r\n") ? "\r\n" : "\n";
  const lines = content.split(/\r?\n/);
  const taskLineIndexes = lines.flatMap((line, index) => (
    TASK_LINE_PATTERN.test(line) ? [index] : []
  ));
  const lineIndex = taskLineIndexes[taskIndex];
  if (lineIndex === undefined) {
    throw new Error("Task item not found");
  }

  lines[lineIndex] = lines[lineIndex].replace(
    TASK_LINE_PATTERN,
    (_, prefix: string, __state: string, suffix: string) => `${prefix}${checked ? "x" : " "}${suffix}`,
  );

  return lines.join(lineEnding);
}

function findNodePath(nodes: TreeNode[], targetId: string): TreeNode[] | null {
  for (const node of nodes) {
    if (node.id === targetId) {
      return [node];
    }
    if (!node.children) continue;
    const childPath = findNodePath(node.children, targetId);
    if (childPath) {
      return [node, ...childPath];
    }
  }
  return null;
}

function getFirstSelectableDescendant(node: TreeNode): TreeNode | null {
  if (node.filePath) {
    return node;
  }
  for (const child of node.children ?? []) {
    const matched = getFirstSelectableDescendant(child);
    if (matched) return matched;
  }
  return null;
}



function getSelectableNode(node: TreeNode): TreeNode | null {
  if (node.filePath) {
    return node;
  }
  return getFirstSelectableDescendant(node);
}

function ListGroup({
  groupNode,
  renderItem,
}: {
  groupNode: TreeNode;
  renderItem: (item: TreeNode) => React.ReactNode;
}) {
  // Active Changes 預設展開，其餘預設折疊
  const [isOpen, setIsOpen] = useState(groupNode.defaultOpen ?? false);
  const children = groupNode.children ?? [];

  return (
    <section className="explorer-list-section">
      <button
        type="button"
        className="explorer-list-group-header"
        onClick={() => setIsOpen((v) => !v)}
      >
        <span className={`tree-group-arrow${isOpen ? " tree-group-arrow--open" : ""}`}>▶</span>
        <span className="explorer-list-group-label">{groupNode.label}</span>
        <span className="explorer-list-group-count">{children.length}</span>
      </button>
      {isOpen ? (
        <div className="explorer-list-rows">
          {children.map((item) => renderItem(item))}
        </div>
      ) : null}
    </section>
  );
}


const explorerViewLabels: Record<ExplorerViewMode, "plansSpecs.explorer.view.tree" | "plansSpecs.explorer.view.list" | "plansSpecs.explorer.view.cols"> = {
  tree: "plansSpecs.explorer.view.tree",
  list: "plansSpecs.explorer.view.list",
  cols: "plansSpecs.explorer.view.cols",
};

const explorerSortFields: Array<{ field: SortField; labelKey: "plansSpecs.explorer.sort.progress" | "plansSpecs.explorer.sort.name" | "plansSpecs.explorer.sort.createdAt" }> = [
  { field: "progress", labelKey: "plansSpecs.explorer.sort.progress" },
  { field: "name", labelKey: "plansSpecs.explorer.sort.name" },
  { field: "createdAt", labelKey: "plansSpecs.explorer.sort.createdAt" },
];

export function PlansSpecsView({
  sisyphusData,
  openspecData,
  isLoading,
  isRefreshing,
  onReadFileContent,
  onReadOpenspecFile,
  onWriteOpenspecFile,
  onRefresh,
  refreshToken,
  projectCwd,
}: Props) {
  const { t } = useI18n();
  const [selectedNode, setSelectedNode] = useState<TreeNode | null>(null);
  const [content, setContent] = useState<string | null>(null);
  const [contentFilePath, setContentFilePath] = useState<string | null>(null);
  const [contentFilePathType, setContentFilePathType] = useState<TreeNode["filePathType"] | null>(null);
  const [contentLoading, setContentLoading] = useState(false);
  const [contentError, setContentError] = useState<string | null>(null);
  const [taskSaving, setTaskSaving] = useState(false);
  const [explorerWidth, setExplorerWidth] = useState(() => {
    const stored = localStorage.getItem("explorer-width");
    const parsed = stored ? parseInt(stored, 10) : NaN;
    return Number.isNaN(parsed) ? DEFAULT_EXPLORER_WIDTH : Math.max(260, Math.min(560, parsed));
  });
  const [isCollapsed, setIsCollapsed] = useState(false);

  const viewModeStorageKey = `explorer-view-mode:${projectCwd}`;
  const [viewMode, setViewMode] = useState<ExplorerViewMode>(() => {
    const stored = localStorage.getItem(viewModeStorageKey);
    return (stored === "tree" || stored === "list" || stored === "cols") ? stored : "tree";
  });

  const sortStorageKey = `explorer-sort:${projectCwd}`;
  const [sortField, setSortField] = useState<SortField>(() => {
    try {
      const raw = localStorage.getItem(sortStorageKey);
      const parsed = raw ? JSON.parse(raw) : null;
      return (parsed?.field === "progress" || parsed?.field === "name" || parsed?.field === "createdAt")
        ? parsed.field
        : "name";
    } catch {
      return "name";
    }
  });
  const [sortDir, setSortDir] = useState<SortDir>(() => {
    try {
      const raw = localStorage.getItem(sortStorageKey);
      const parsed = raw ? JSON.parse(raw) : null;
      return parsed?.dir === "desc" ? "desc" : "asc";
    } catch {
      return "asc";
    }
  });

  const handleSortClick = useCallback((field: SortField) => {
    setSortField((prevField) => {
      const newDir = prevField === field
        ? (sortDir === "asc" ? "desc" : "asc")
        : "asc";
      setSortDir(newDir);
      localStorage.setItem(sortStorageKey, JSON.stringify({ field, dir: newDir }));
      return field;
    });
  }, [sortDir, sortStorageKey]);

  const handleSetViewMode = useCallback((mode: ExplorerViewMode) => {
    setViewMode(mode);
    localStorage.setItem(viewModeStorageKey, mode);
  }, [viewModeStorageKey]);
  const [columnsOpenGroups, setColumnsOpenGroups] = useState<Set<string>>(
    () => new Set(["openspec:active-changes"]),
  );
  const [columnsChangeId, setColumnsChangeId] = useState<string | null>(null);
  const [copiedEntryId, setCopiedEntryId] = useState<string | null>(null);
  const isDragging = useRef(false);
  const resizerRef = useRef<HTMLDivElement>(null);
  const cleanupDragRef = useRef<(() => void) | null>(null);
  const lastHandledRefreshTokenRef = useRef(refreshToken);
  const selfWrittenFilesRef = useRef<Set<string>>(new Set());

  useEffect(() => {
    return () => {
      cleanupDragRef.current?.();
    };
  }, []);

  const loadNodeContent = useCallback(
    async (node: TreeNode, resetContent: boolean) => {
      if (!node.filePath) return;
      setContentLoading(true);
      setContentError(null);
      if (resetContent) {
        setContent(null);
      }
      setContentFilePath(node.filePath);
      setContentFilePathType(node.filePathType ?? null);
      try {
        const text = node.filePathType === "openspec"
          ? await onReadOpenspecFile(projectCwd, node.filePath)
          : await onReadFileContent(node.filePath);
        setContent(text);
      } catch (e) {
        setContentError(e instanceof Error ? e.message : String(e));
      } finally {
        setContentLoading(false);
      }
    },
    [onReadFileContent, onReadOpenspecFile, projectCwd],
  );

  const handleSelect = useCallback(
    async (node: TreeNode) => {
      if (!node.filePath) return;
      setSelectedNode(node);
      await loadNodeContent(node, true);
    },
    [loadNodeContent],
  );

  const handleSelectTarget = useCallback(
    async (node: TreeNode) => {
      const selectableNode = getSelectableNode(node);
      if (!selectableNode) return;
      await handleSelect(selectableNode);
    },
    [handleSelect],
  );

  const handleToggleTask = useCallback(
    async (filePath: string, taskIndex: number, checked: boolean) => {
      if (content === null || contentFilePath !== filePath || contentFilePathType !== "openspec") {
        throw new Error("Task file is not loaded");
      }

      const previousContent = content;
      const nextContent = toggleMarkdownTask(content, taskIndex, checked);
      setTaskSaving(true);
      setContentError(null);
      setContent(nextContent);

      try {
        await onWriteOpenspecFile(projectCwd, filePath, nextContent);
        selfWrittenFilesRef.current.add(filePath);
      } catch (error) {
        setContent(previousContent);
        throw error;
      } finally {
        setTaskSaving(false);
      }
    },
    [content, contentFilePath, contentFilePathType, onWriteOpenspecFile, projectCwd],
  );

  const handleResizerMouseDown = useCallback((e: React.MouseEvent) => {
    e.preventDefault();
    isDragging.current = true;
    const startX = e.clientX;
    const startWidth = explorerWidth;

    const onMouseMove = (ev: MouseEvent) => {
      if (!isDragging.current) return;
      const delta = ev.clientX - startX;
      currentWidth = Math.max(260, Math.min(560, startWidth + delta));
      setExplorerWidth(currentWidth);
    };

    let currentWidth = startWidth;
    const onMouseUp = () => {
      if (isDragging.current) {
        localStorage.setItem("explorer-width", String(currentWidth));
      }
      isDragging.current = false;
      cleanupDragRef.current = null;
      resizerRef.current?.classList.remove("explorer-resizer--dragging");
      document.removeEventListener("mousemove", onMouseMove);
      document.removeEventListener("mouseup", onMouseUp);
    };

    cleanupDragRef.current = () => {
      isDragging.current = false;
      document.removeEventListener("mousemove", onMouseMove);
      document.removeEventListener("mouseup", onMouseUp);
    };

    resizerRef.current?.classList.add("explorer-resizer--dragging");
    document.addEventListener("mousemove", onMouseMove);
    document.addEventListener("mouseup", onMouseUp);
  }, [explorerWidth]);

  const hasSisyphus =
    sisyphusData &&
    (sisyphusData.plans.length > 0 ||
      sisyphusData.notepads.length > 0 ||
      sisyphusData.evidenceFiles.length > 0 ||
      sisyphusData.draftFiles.length > 0 ||
      sisyphusData.activePlan !== null);

  const hasOpenSpec =
    openspecData &&
    (openspecData.activeChanges.length > 0 ||
      openspecData.archivedChanges.length > 0 ||
      openspecData.specs.length > 0);

  const rootNodes = useMemo<TreeNode[]>(() => {
    const sisyphusNodes = hasSisyphus && sisyphusData ? buildSisyphusTree(sisyphusData, t) : [];
    const sortedOpenspecData = hasOpenSpec && openspecData
      ? {
          ...openspecData,
          activeChanges: sortChanges(openspecData.activeChanges, sortField, sortDir),
          archivedChanges: sortChanges(openspecData.archivedChanges, sortField, sortDir),
        }
      : openspecData;
    const openspecNodes = hasOpenSpec && sortedOpenspecData ? buildOpenSpecTree(sortedOpenspecData, t) : [];

    return [
      ...(sisyphusNodes.length > 0
        ? [
            {
              id: "root:sisyphus",
              label: t("plansSpecs.sisyphus.title"),
              icon: "folder" as const,
              defaultOpen: true,
              children: sisyphusNodes,
            },
          ]
        : []),
      ...(openspecNodes.length > 0
        ? [
            {
              id: "root:openspec",
              label: t("plansSpecs.openspec.title"),
              icon: "folder" as const,
              defaultOpen: true,
              children: openspecNodes,
            },
          ]
        : []),
    ];
  }, [hasOpenSpec, hasSisyphus, openspecData, sisyphusData, t, sortField, sortDir]);

  // 依目前顯示的檔案反查所屬 change 的建立日期（僅 openspec change 的 artifact 有值）
  const contentCreatedAt = useMemo<string | null>(() => {
    if (contentFilePathType !== "openspec" || !contentFilePath || !openspecData) {
      return null;
    }
    const normalized = contentFilePath.replace(/\\/g, "/");
    const match = normalized.match(/^changes\/(?:archive\/)?([^/]+)\//);
    if (!match) return null;
    const changeName = match[1];
    const isArchived = normalized.startsWith("changes/archive/");
    const pool = isArchived ? openspecData.archivedChanges : openspecData.activeChanges;
    return pool.find((c) => c.name === changeName)?.createdAt ?? null;
  }, [contentFilePath, contentFilePathType, openspecData]);

  // 蒐集所有狀態群組（扁平），供 Cols 模式使用
  const allColsStatusGroups = useMemo<TreeNode[]>(() => {
    const groups: TreeNode[] = [];
    for (const rootNode of rootNodes) {
      for (const child of rootNode.children ?? []) {
        groups.push(child);
      }
    }
    return groups;
  }, [rootNodes]);

  // 同步 selectedNode → 展開對應群組 + 選中對應 change
  useEffect(() => {
    if (!selectedNode?.id) return;
    const path = findNodePath(rootNodes, selectedNode.id);
    if (!path) return;
    // path[0] = root, path[1] = status group, path[2] = change
    const statusGroup = path[1];
    if (statusGroup && allColsStatusGroups.some((g) => g.id === statusGroup.id)) {
      setColumnsOpenGroups((prev) => {
        if (prev.has(statusGroup.id)) return prev;
        const next = new Set(prev);
        next.add(statusGroup.id);
        return next;
      });
    }
    const changeNode = path[2];
    if (changeNode) {
      setColumnsChangeId(changeNode.id);
    }
  }, [rootNodes, selectedNode?.id, allColsStatusGroups]);

  // Cols 模式：若 columnsChangeId 失效，自動選第一個可見 change
  useEffect(() => {
    if (!allColsStatusGroups.length) {
      setColumnsChangeId(null);
      return;
    }
    const allVisibleChanges = allColsStatusGroups
      .filter((g) => columnsOpenGroups.has(g.id))
      .flatMap((g) => g.children ?? []);
    if (!allVisibleChanges.length) return;
    setColumnsChangeId((current) => (
      current && allVisibleChanges.some((c) => c.id === current)
        ? current
        : allVisibleChanges[0].id
    ));
  }, [allColsStatusGroups, columnsOpenGroups]);

  useEffect(() => {
    if (!selectedNode?.id) return;
    if (lastHandledRefreshTokenRef.current === refreshToken) return;
    lastHandledRefreshTokenRef.current = refreshToken;

    if (contentFilePath && selfWrittenFilesRef.current.has(contentFilePath)) {
      selfWrittenFilesRef.current.delete(contentFilePath);
      return;
    }

    const matchedPath = findNodePath(rootNodes, selectedNode.id);
    const matchedNode = matchedPath?.[matchedPath.length - 1] ?? null;

    if (!matchedNode?.filePath) {
      setSelectedNode(null);
      setContent(null);
      setContentFilePath(null);
      setContentFilePathType(null);
      setContentError(null);
      return;
    }

    if (
      selectedNode.filePath !== matchedNode.filePath ||
      selectedNode.filePathType !== matchedNode.filePathType
    ) {
      setSelectedNode(matchedNode);
    }

    void loadNodeContent(matchedNode, false);
  }, [contentFilePath, loadNodeContent, refreshToken, rootNodes, selectedNode?.filePath, selectedNode?.filePathType, selectedNode?.id]);

  const renderListChangeRow = (item: TreeNode) => {
    const artifactNodes = item.children ?? [];
    const specsNode = artifactNodes.find((a) => a.id.endsWith(":specs"));
    const badgeArtifacts = artifactNodes.filter((a) => !a.id.endsWith(":specs"));
    const specsCount = specsNode?.children?.length ?? 0;
    const isAnyArtifactActive = badgeArtifacts.some((a) => {
      const leaf = getSelectableNode(a);
      return leaf && selectedNode?.id === leaf.id;
    });
    const isRowActive = isAnyArtifactActive
      || (Boolean(getSelectableNode(item)) && selectedNode?.id === getSelectableNode(item)?.id);

    return (
      <div
        key={item.id}
        role="button"
        tabIndex={0}
        className={`explorer-list-row${isRowActive ? " explorer-list-row--active" : ""}`}
        onClick={() => { void handleSelectTarget(item); }}
        onKeyDown={(e) => { if (e.key === "Enter" || e.key === " ") void handleSelectTarget(item); }}
      >
        <div className="explorer-list-row-header">
          <span className="explorer-list-row-name" title={item.label}>
            {item.label}
          </span>
          {specsCount > 0 ? (
            <span className="explorer-list-specs-count">{specsCount} specs</span>
          ) : null}
        </div>
        {badgeArtifacts.length > 0 ? (
          <div className="explorer-chip-row">
            {badgeArtifacts.map((artifact) => {
              const targetNode = getSelectableNode(artifact);
              const isActive = Boolean(targetNode && selectedNode?.id === targetNode.id);
              const chipLabel = artifact.icon === "proposal"
                ? "proposal"
                : artifact.icon === "design"
                  ? "design"
                  : artifact.icon === "tasks"
                    ? "tasks"
                    : artifact.label;
              return (
                <button
                  key={artifact.id}
                  type="button"
                  className={`explorer-chip${isActive ? " explorer-chip--active" : ""}`}
                  onClick={(e) => {
                    e.stopPropagation();
                    void handleSelectTarget(artifact);
                  }}
                >
                  <span>{chipLabel}</span>
                  {artifact.badge ? (
                    <span className={`tree-node-badge tree-node-badge--${artifact.tone ?? "neutral"}`}>
                      {artifact.badge}
                    </span>
                  ) : null}
                </button>
              );
            })}
          </div>
        ) : null}
      </div>
    );
  };

  const renderListView = () => {
    // 蒐集所有群組（Active Changes、Archived Changes、Specs 等）
    const groups: TreeNode[] = [];
    for (const rootNode of rootNodes) {
      for (const child of rootNode.children ?? []) {
        groups.push(child);
      }
    }

    return (
      <div className="explorer-list-view">
        {groups.map((groupNode) => (
          <ListGroup key={groupNode.id} groupNode={groupNode} renderItem={renderListChangeRow} />
        ))}
      </div>
    );
  };

  const renderColumnsPanel = () => {
    const groups = allColsStatusGroups;
    const activeChange = groups
      .flatMap((g) => g.children ?? [])
      .find((c) => c.id === columnsChangeId) ?? null;
    const detailNodes = activeChange?.children ?? [];

    return (
      <div className="explorer-cols-panel">
        <div className="explorer-cols-master-detail">
          {/* 左欄 master：手風琴群組 + change 進度列 */}
          <div className="explorer-cols-master">
            <div className="explorer-cols-entries">
              {groups.map((groupNode) => {
                const isOpen = columnsOpenGroups.has(groupNode.id);
                const entryNodes = groupNode.children ?? [];
                return (
                  <div key={groupNode.id} className="explorer-cols-group">
                    <button
                      type="button"
                      className="explorer-cols-group-header"
                      onClick={() => setColumnsOpenGroups((prev) => {
                        const next = new Set(prev);
                        if (next.has(groupNode.id)) {
                          next.delete(groupNode.id);
                        } else {
                          next.add(groupNode.id);
                        }
                        return next;
                      })}
                    >
                      <span className={`tree-group-arrow${isOpen ? " tree-group-arrow--open" : ""}`}>▶</span>
                      <span className="explorer-cols-group-label">{groupNode.label}</span>
                    </button>
                    {isOpen ? (
                      <div className="explorer-cols-group-items">
                        {entryNodes.map((entryNode) => {
                          const isActive = entryNode.id === columnsChangeId;
                          const progress = entryNode.progress;
                          const progressPct = progress && progress.total > 0
                            ? Math.round((progress.done / progress.total) * 100)
                            : null;
                          return (
                            <button
                              key={entryNode.id}
                              type="button"
                              className={`explorer-cols-entry${isActive ? " explorer-cols-entry--active" : ""}`}
                              onClick={() => {
                                setColumnsChangeId(entryNode.id);
                                if (entryNode.icon === "spec") {
                                  const selectableNode = getSelectableNode(entryNode);
                                  if (selectableNode) void handleSelect(selectableNode);
                                  return;
                                }
                                const tasksNode = (entryNode.children ?? []).find((c) => c.id.endsWith(":tasks"));
                                if (tasksNode) {
                                  const selectableNode = getSelectableNode(tasksNode);
                                  if (selectableNode) void handleSelect(selectableNode);
                                }
                              }}
                            >
                              <div className="explorer-cols-entry-top">
                                <span className="explorer-cols-entry-name">{entryNode.label}</span>
                                {entryNode.badge ? (
                                  <span className={`tree-node-badge tree-node-badge--${entryNode.tone ?? "neutral"}`}>
                                    {entryNode.badge}
                                  </span>
                                ) : null}
                              </div>
                              {progressPct !== null ? (
                                <div className="explorer-cols-progress">
                                  <div
                                    className={`explorer-cols-progress-bar explorer-cols-progress-bar--${entryNode.tone ?? "neutral"}`}
                                    style={{ width: `${progressPct}%` }}
                                  />
                                </div>
                              ) : null}
                              {entryNode.icon === "spec" ? null : (() => {
                                const action = resolveChangeAction(entryNode, groupNode.id === ARCHIVED_CHANGES_GROUP_ID);
                                const isCopied = copiedEntryId === entryNode.id;
                                return (
                                  <div className="explorer-cols-action">
                                    <span className={`explorer-cols-action-label explorer-cols-action-label--${action.tone}`}>
                                      {action.label}
                                    </span>
                                    {action.command ? (
                                      <button
                                        type="button"
                                        className={`explorer-cols-action-copy${isCopied ? " explorer-cols-action-copy--copied" : ""}`}
                                        title={action.command}
                                        onClick={(e) => {
                                          e.stopPropagation();
                                          navigator.clipboard.writeText(action.command!).then(() => {
                                            setCopiedEntryId(entryNode.id);
                                            setTimeout(() => setCopiedEntryId((prev) => prev === entryNode.id ? null : prev), 500);
                                          }).catch(() => { /* 靜默失敗 */ });
                                        }}
                                      >
                                        {isCopied ? "✓" : "⎘"}
                                      </button>
                                    ) : null}
                                  </div>
                                );
                              })()}
                            </button>
                          );
                        })}
                      </div>
                    ) : null}
                  </div>
                );
              })}
            </div>
          </div>
          {/* 右欄 detail：選中 change 的檔案清單 */}
          <div className="explorer-cols-detail">
            {activeChange ? (
              detailNodes.length > 0 ? (
                <ExplorerTree
                  nodes={detailNodes}
                  selectedId={selectedNode?.id ?? null}
                  onSelect={handleSelect}
                />
              ) : (
                <div className="explorer-cols-detail-empty">{t("plansSpecs.explorer.noFiles")}</div>
              )
            ) : (
              <div className="explorer-cols-detail-empty">{t("plansSpecs.explorer.selectChange")}</div>
            )}
          </div>
        </div>
      </div>
    );
  };

  if (isLoading) {
    return (
      <div className="plans-specs-empty">{t("plansSpecs.loading")}</div>
    );
  }

  if (!hasSisyphus && !hasOpenSpec) {
    return (
      <div className="plans-specs-empty">{t("plansSpecs.empty")}</div>
    );
  }

  return (
    <div
      className="explorer-layout"
      style={{ "--explorer-width": `${explorerWidth}px` } as React.CSSProperties}
    >
      <div
        className={`explorer-panel${isCollapsed ? " explorer-panel--collapsed" : ""}`}
        style={isCollapsed ? undefined : { width: explorerWidth }}
      >
        <div className="explorer-panel-header">
          {!isCollapsed ? (
            <div className="explorer-panel-heading">
              <span className="explorer-panel-title">{t("plansSpecs.explorer.title")}</span>
              <div className="explorer-view-switcher" aria-label={t("plansSpecs.explorer.title")}>
                {(["tree", "list", "cols"] as ExplorerViewMode[]).map((mode) => (
                  <button
                    key={mode}
                    type="button"
                    className={`explorer-view-btn${viewMode === mode ? " explorer-view-btn--active" : ""}`}
                    onClick={() => handleSetViewMode(mode)}
                  >
                    {t(explorerViewLabels[mode])}
                  </button>
                ))}
              </div>
              <div className="explorer-sort-switcher">
                {explorerSortFields.map(({ field, labelKey }) => {
                  const isActive = sortField === field;
                  const icon = isActive ? (sortDir === "asc" ? "↑" : "↓") : "⇅";
                  return (
                    <button
                      key={field}
                      type="button"
                      className={`explorer-sort-btn${isActive ? " explorer-sort-btn--active" : ""}`}
                      onClick={() => handleSortClick(field)}
                    >
                      {t(labelKey)}{icon}
                    </button>
                  );
                })}
              </div>
              <button
                type="button"
                className="explorer-refresh-btn"
                onClick={() => { void onRefresh(); }}
                title={t("app.actions.refresh")}
                aria-label={t("app.actions.refresh")}
                disabled={isRefreshing}
              >
                ↻
              </button>
            </div>
          ) : null}
          <button
            type="button"
            className="explorer-collapse-btn"
            onClick={() => setIsCollapsed((value) => !value)}
            title={isCollapsed ? t("plansSpecs.explorer.expandPanel") : t("plansSpecs.explorer.collapsePanel")}
          >
            {isCollapsed ? "»" : "«"}
          </button>
        </div>
        {!isCollapsed ? (
          viewMode === "tree" ? (
            <ExplorerTree
              nodes={rootNodes}
              selectedId={selectedNode?.id ?? null}
              onSelect={handleSelect}
            />
          ) : viewMode === "list" ? (
            renderListView()
          ) : (
            renderColumnsPanel()
          )
        ) : null}
      </div>

      {!isCollapsed ? (
        <div
          ref={resizerRef}
          className="explorer-resizer"
          onMouseDown={handleResizerMouseDown}
        />
      ) : null}

      <ContentViewer
        content={content}
        filePath={contentFilePath}
        filePathType={contentFilePathType}
        createdAt={contentCreatedAt}
        isLoading={contentLoading}
        error={contentError}
        isTaskSaving={taskSaving}
        onToggleTask={handleToggleTask}
      />
    </div>
  );
}
