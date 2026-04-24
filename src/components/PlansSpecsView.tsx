import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { useI18n } from "../i18n/I18nProvider";
import type { OpenSpecData, SisyphusData, TreeNode } from "../types";
import { buildOpenSpecTree, buildSisyphusTree } from "../utils/buildTree";
import { ContentViewer } from "./ContentViewer";
import { ExplorerTree } from "./ExplorerTree";

type Props = {
  sisyphusData: SisyphusData | undefined;
  openspecData: OpenSpecData | undefined;
  isLoading: boolean;
  isRefreshing: boolean;
  onReadFileContent: (filePath: string) => Promise<string>;
  onReadOpenspecFile: (projectCwd: string, relativePath: string) => Promise<string>;
  onRefresh: () => Promise<void>;
  refreshToken: string;
  projectCwd: string;
};

const DEFAULT_EXPLORER_WIDTH = 215;

export function PlansSpecsView({
  sisyphusData,
  openspecData,
  isLoading,
  isRefreshing,
  onReadFileContent,
  onReadOpenspecFile,
  onRefresh,
  refreshToken,
  projectCwd,
}: Props) {
  const { t } = useI18n();
  const [selectedNode, setSelectedNode] = useState<TreeNode | null>(null);
  const [content, setContent] = useState<string | null>(null);
  const [contentFilePath, setContentFilePath] = useState<string | null>(null);
  const [contentLoading, setContentLoading] = useState(false);
  const [contentError, setContentError] = useState<string | null>(null);
  const [explorerWidth, setExplorerWidth] = useState(DEFAULT_EXPLORER_WIDTH);
  const [isCollapsed, setIsCollapsed] = useState(false);
  const isDragging = useRef(false);
  const resizerRef = useRef<HTMLDivElement>(null);
  const cleanupDragRef = useRef<(() => void) | null>(null);
  const lastHandledRefreshTokenRef = useRef(refreshToken);

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

  const handleResizerMouseDown = useCallback((e: React.MouseEvent) => {
    e.preventDefault();
    isDragging.current = true;
    const startX = e.clientX;
    const startWidth = explorerWidth;

    const onMouseMove = (ev: MouseEvent) => {
      if (!isDragging.current) return;
      const delta = ev.clientX - startX;
      const newWidth = Math.max(160, Math.min(480, startWidth + delta));
      setExplorerWidth(newWidth);
    };

    const onMouseUp = () => {
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
    const openspecNodes = hasOpenSpec && openspecData ? buildOpenSpecTree(openspecData, t) : [];

    return [
      ...(sisyphusNodes.length > 0
        ? [
            {
              id: "root:sisyphus",
              label: t("plansSpecs.sisyphus.title"),
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
              defaultOpen: true,
              children: openspecNodes,
            },
          ]
        : []),
    ];
  }, [hasOpenSpec, hasSisyphus, openspecData, sisyphusData, t]);

  useEffect(() => {
    if (!selectedNode?.id) return;
    if (lastHandledRefreshTokenRef.current === refreshToken) return;
    lastHandledRefreshTokenRef.current = refreshToken;

    const stack = [...rootNodes];
    let matchedNode: TreeNode | null = null;
    while (stack.length > 0) {
      const node = stack.pop()!;
      if (node.id === selectedNode.id) {
        matchedNode = node;
        break;
      }
      if (node.children) {
        stack.push(...node.children);
      }
    }

    if (!matchedNode?.filePath) {
      setSelectedNode(null);
      setContent(null);
      setContentFilePath(null);
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
  }, [loadNodeContent, refreshToken, rootNodes, selectedNode?.filePath, selectedNode?.filePathType, selectedNode?.id]);

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
      {/* Left panel */}
      <div
        className={`explorer-panel${isCollapsed ? " explorer-panel--collapsed" : ""}`}
        style={isCollapsed ? undefined : { width: explorerWidth }}
      >
        <div className="explorer-panel-header">
          {!isCollapsed ? (
            <div className="explorer-panel-heading">
              <span className="explorer-panel-title">{t("plansSpecs.explorer.title")}</span>
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
            onClick={() => setIsCollapsed((v) => !v)}
            title={isCollapsed ? t("plansSpecs.explorer.expandPanel") : t("plansSpecs.explorer.collapsePanel")}
          >
            {isCollapsed ? "»" : "«"}
          </button>
        </div>
        {!isCollapsed ? (
          <ExplorerTree
            nodes={rootNodes}
            selectedId={selectedNode?.id ?? null}
            onSelect={handleSelect}
          />
        ) : null}
      </div>

      {/* Resizer */}
      {!isCollapsed ? (
        <div
          ref={resizerRef}
          className="explorer-resizer"
          onMouseDown={handleResizerMouseDown}
        />
      ) : null}

      {/* Right panel */}
      <ContentViewer
        content={content}
        filePath={contentFilePath}
        isLoading={contentLoading}
        error={contentError}
      />
    </div>
  );
}
