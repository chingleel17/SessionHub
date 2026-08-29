import { useRef, useState, type DragEvent } from "react";
import packageJson from "../../package.json";
import { useI18n } from "../i18n/I18nProvider";
import type { ProjectGroup, RealtimeStatus } from "../types";
import {
  AgentsIcon,
  CloseIcon,
  DashboardIcon,
  PanelLeftCloseIcon,
  PanelLeftOpenIcon,
  PinIcon,
  RefreshIcon,
  SettingsIcon,
} from "./Icons";

type Props = {
  activeView: string;
  isSidebarCollapsed: boolean;
  realtimeStatus: RealtimeStatus;
  lastRealtimeSyncAt: string | null;
  sessionsIsFetching: boolean;
  pinnedProjects: string[];
  openProjectKeys: string[];
  projectGroups: ProjectGroup[];
  onNavigate: (view: string) => void;
  onOpenProject: (projectKey: string) => void;
  onCloseProject: (projectKey: string) => void;
  onClearOpenProjects: () => void;
  onReorderOpenProjects: (newKeys: string[]) => void;
  onReorderPinnedProjects: (newKeys: string[]) => void;
  onPinProject: (key: string) => void;
  onCollapseToggle: () => void;
  onRefresh: () => void;
  onConfigurePath: () => void;
  onRequestProjectPicker: () => void;
};

type DragSource = "pinned" | "open";
type DragState = { key: string; source: DragSource };

const serializeDragState = (dragState: DragState) => JSON.stringify(dragState);

const readDragState = (dataTransfer: DataTransfer): DragState | null => {
  try {
    const value: unknown = JSON.parse(dataTransfer.getData("text/plain"));
    if (
      typeof value === "object"
      && value !== null
      && "key" in value
      && typeof value.key === "string"
      && "source" in value
      && (value.source === "pinned" || value.source === "open")
    ) {
      return { key: value.key, source: value.source };
    }
  } catch {
    return null;
  }
  return null;
};

export function Sidebar({
  activeView,
  isSidebarCollapsed,
  realtimeStatus,
  lastRealtimeSyncAt,
  sessionsIsFetching,
  pinnedProjects,
  openProjectKeys,
  projectGroups,
  onNavigate,
  onOpenProject,
  onCloseProject,
  onClearOpenProjects,
  onReorderOpenProjects,
  onReorderPinnedProjects,
  onPinProject,
  onCollapseToggle,
  onRefresh,
  onConfigurePath,
  onRequestProjectPicker,
}: Props) {
  const { t } = useI18n();

  const [dragState, setDragState] = useState<DragState | null>(null);
  const [dragOverKey, setDragOverKey] = useState<string | null>(null);
  const [isDraggingOverPinned, setIsDraggingOverPinned] = useState(false);
  const dragStateRef = useRef<DragState | null>(null);
  const suppressClickRef = useRef(false);

  const realtimeLabel =
    sessionsIsFetching
      ? t("dashboard.status.scanning")
      : realtimeStatus === "error"
        ? t("dashboard.status.realtimeError")
        : realtimeStatus === "active"
          ? t("dashboard.status.realtimeActive")
          : t("dashboard.status.realtimeConnecting");

  const visiblePinnedGroups = pinnedProjects
    .map((key) => projectGroups.find((g) => g.key === key))
    .filter((g): g is ProjectGroup => Boolean(g));

  // 僅顯示非釘選的已開啟項目
  const openGroups = openProjectKeys
    .filter((key) => !pinnedProjects.includes(key))
    .map((key) => projectGroups.find((g) => g.key === key))
    .filter((g): g is ProjectGroup => Boolean(g));

  const clearDragState = () => {
    dragStateRef.current = null;
    setDragState(null);
    setDragOverKey(null);
    setIsDraggingOverPinned(false);
  };

  const handleDragStart = (key: string, source: DragSource, e: DragEvent<HTMLButtonElement>) => {
    // setData 是必須的，否則瀏覽器會顯示禁止符號並取消 drag
    const nextDragState = { key, source };
    e.dataTransfer.setData("text/plain", serializeDragState(nextDragState));
    e.dataTransfer.effectAllowed = "move";
    dragStateRef.current = nextDragState;
    setDragState(nextDragState);
    setDragOverKey(null);
    setIsDraggingOverPinned(false);
    suppressClickRef.current = true;
  };

  const handleDragEnd = () => {
    clearDragState();
  };

  const handleDragOverItem = (
    e: DragEvent<HTMLDivElement>,
    targetKey: string,
    targetSource: DragSource,
  ) => {
    e.preventDefault();
    e.stopPropagation();
    e.dataTransfer.dropEffect = "move";

    const currentDragState = dragStateRef.current;
    const isPinningOpenProject = currentDragState?.source === "open" && targetSource === "pinned";
    if (!currentDragState || (currentDragState.source !== targetSource && !isPinningOpenProject)) return;
    if (isPinningOpenProject) {
      setDragOverKey(null);
      setIsDraggingOverPinned(true);
      return;
    }
    setDragOverKey(currentDragState.key === targetKey ? null : targetKey);
  };

  const handleDropOnItem = (
    e: DragEvent<HTMLDivElement>,
    targetKey: string,
    targetSource: DragSource,
  ) => {
    e.preventDefault();
    e.stopPropagation();

    const currentDragState = dragStateRef.current ?? readDragState(e.dataTransfer);
    const isPinningOpenProject = currentDragState?.source === "open" && targetSource === "pinned";
    if (!currentDragState || (currentDragState.source !== targetSource && !isPinningOpenProject)) return;

    if (isPinningOpenProject) {
      onPinProject(currentDragState.key);
      clearDragState();
      return;
    }

    const sourceKey = currentDragState.key;
    if (sourceKey === targetKey) {
      clearDragState();
      return;
    }

    const orderedKeys = targetSource === "pinned"
      ? visiblePinnedGroups.map((group) => group.key)
      : openProjectKeys.filter((key) => !pinnedProjects.includes(key));
    const from = orderedKeys.indexOf(sourceKey);
    const to = orderedKeys.indexOf(targetKey);
    if (from === -1 || to === -1) {
      clearDragState();
      return;
    }

    const next = [...orderedKeys];
    next.splice(from, 1);
    next.splice(from < to ? to - 1 : to, 0, sourceKey);
    if (targetSource === "pinned") {
      onReorderPinnedProjects(next);
    } else {
      onReorderOpenProjects(next);
    }
    clearDragState();
  };

  const handleDragOverPinned = (e: DragEvent<HTMLDivElement>) => {
    e.preventDefault();
    e.dataTransfer.dropEffect = "move";

    const currentDragState = dragStateRef.current;
    if (!currentDragState || currentDragState.source !== "open") return;
    setDragOverKey(null);
    setIsDraggingOverPinned(true);
  };

  const handleDragLeavePinned = (e: DragEvent<HTMLDivElement>) => {
    const relatedTarget = e.relatedTarget;
    if (!(relatedTarget instanceof Node) || !e.currentTarget.contains(relatedTarget)) {
      setIsDraggingOverPinned(false);
    }
  };

  const handleDropOnPinned = (e: DragEvent<HTMLDivElement>) => {
    e.preventDefault();

    const currentDragState = dragStateRef.current ?? readDragState(e.dataTransfer);
    if (!currentDragState || currentDragState.source !== "open") return;
    onPinProject(currentDragState.key);
    clearDragState();
  };

  const handleOpenProject = (projectKey: string) => {
    if (suppressClickRef.current) {
      suppressClickRef.current = false;
      return;
    }
    onOpenProject(projectKey);
  };

  const getGroupTooltip = (group: ProjectGroup) => {
    const branchLabel = group.branchLabel?.trim();
    return branchLabel ? `${group.title} · ${branchLabel}` : group.title;
  };

  const renderGroupLabel = (group: ProjectGroup) => {
    const branchLabel = group.branchLabel?.trim();
    return (
      <span className={`sidebar-group-label-wrap${branchLabel ? " sidebar-group-label-wrap--with-branch" : ""}`}>
        <span className="sidebar-pinned-item-label">{group.title}</span>
        {branchLabel ? <span className="sidebar-branch-label">· {branchLabel}</span> : null}
      </span>
    );
  };

  return (
    <aside className="sidebar">
      <div className="sidebar-brand">
        <div className="sidebar-brand-icon">SH</div>
        <div className="sidebar-brand-copy">
          <h1 className="topbar-title">{t("app.title")}</h1>
        </div>
        <button
          type="button"
          className="sidebar-collapse-button"
          onClick={onCollapseToggle}
          aria-label={isSidebarCollapsed ? t("sidebar.expand") : t("sidebar.collapse")}
          title={isSidebarCollapsed ? t("sidebar.expand") : t("sidebar.collapse")}
          aria-expanded={!isSidebarCollapsed}
        >
          {isSidebarCollapsed ? <PanelLeftOpenIcon size={18} /> : <PanelLeftCloseIcon size={18} />}
        </button>
      </div>

      <nav className="sidebar-menu">
        <button
          type="button"
          className={`sidebar-link ${activeView === "dashboard" ? "active" : ""}`}
          title={t("sidebar.menu.dashboard")}
          onClick={() => onNavigate("dashboard")}
        >
          <span className="sidebar-link-icon"><DashboardIcon size={18} /></span>
          <span>{t("sidebar.menu.dashboard")}</span>
        </button>

        <div className="sidebar-project-divider">
          <div className="sidebar-section-divider" aria-hidden="true" />
          <button
            type="button"
            className="sidebar-project-new"
            title={t("sidebar.newProject")}
            aria-label={t("sidebar.newProject")}
            onClick={onRequestProjectPicker}
          >
            <span className="sidebar-project-new-label">{t("sidebar.newProjectLabel")}</span>
          </button>
        </div>

        {visiblePinnedGroups.length > 0 ? (
          <div
            className={`sidebar-section sidebar-pinned-section ${isDraggingOverPinned ? "sidebar-section--drop-target" : ""}`}
            onDragOver={handleDragOverPinned}
            onDragLeave={handleDragLeavePinned}
            onDrop={handleDropOnPinned}
          >
            <div className="sidebar-section-list">
              {visiblePinnedGroups.map((group) => {
                const initial = group.title.charAt(0).toUpperCase();
                const isBeingDraggedOver = dragState?.source === "pinned" && dragOverKey === group.key;
                const isBeingDragged = dragState?.source === "pinned" && dragState.key === group.key;
                return (
                  <div
                    key={group.key}
                    className={`sidebar-pinned-item ${isBeingDraggedOver ? "sidebar-pinned-item--drag-over" : ""} ${isBeingDragged ? "sidebar-pinned-item--dragging" : ""}`}
                    onDragOver={(e) => handleDragOverItem(e, group.key, "pinned")}
                    onDrop={(e) => handleDropOnItem(e, group.key, "pinned")}
                  >
                    <button
                      type="button"
                      draggable
                      className={`sidebar-link ${activeView === group.key ? "active" : ""}`}
                      title={getGroupTooltip(group)}
                      onClick={() => handleOpenProject(group.key)}
                      onPointerDown={() => { suppressClickRef.current = false; }}
                      onDragStart={(e) => handleDragStart(group.key, "pinned", e)}
                      onDragEnd={handleDragEnd}
                    >
                      <span className="sidebar-link-icon sidebar-pinned-initial">
                        {initial}
                        <span className="sidebar-pin-badge" aria-hidden="true">
                          <PinIcon size={9} />
                        </span>
                      </span>
                      {renderGroupLabel(group)}
                    </button>
                  </div>
                );
              })}
            </div>
          </div>
        ) : null}

        {openGroups.length > 0 ? (
          <>
            {/* 分隔線列：左側線條 + 右側全部關閉按鈕（collapsed 時按鈕淡出、只留分隔線） */}
            <div className="sidebar-open-section-header">
              <div className="sidebar-open-section-divider" aria-hidden="true" />
              <button
                type="button"
                className="sidebar-open-section-clear"
                title={t("sidebar.clearOpen")}
                onClick={onClearOpenProjects}
              >
                ↓ {t("sidebar.clearOpen")}
              </button>
            </div>

            <div className="sidebar-section-list">
              {openGroups.map((group) => {
                const initial = group.title.charAt(0).toUpperCase();
                const isBeingDraggedOver = dragState?.source === "open" && dragOverKey === group.key;
                const isBeingDragged = dragState?.source === "open" && dragState.key === group.key;
                return (
                  <div
                    key={group.key}
                    className={`sidebar-open-item ${isBeingDraggedOver ? "sidebar-open-item--drag-over" : ""} ${isBeingDragged ? "sidebar-open-item--dragging" : ""}`}
                    onDragOver={(e) => handleDragOverItem(e, group.key, "open")}
                    onDrop={(e) => handleDropOnItem(e, group.key, "open")}
                  >
                    <button
                      type="button"
                      draggable
                      className={`sidebar-link sidebar-open-item-label ${activeView === group.key ? "active" : ""}`}
                      title={getGroupTooltip(group)}
                      onClick={() => handleOpenProject(group.key)}
                      onPointerDown={() => { suppressClickRef.current = false; }}
                      onDragStart={(e) => handleDragStart(group.key, "open", e)}
                      onDragEnd={handleDragEnd}
                    >
                      <span className="sidebar-link-icon sidebar-pinned-initial">{initial}</span>
                      {renderGroupLabel(group)}
                    </button>
                    <button
                      type="button"
                      className="sidebar-open-item-close"
                      title={t("sidebar.closeProject")}
                      aria-label={`${t("sidebar.closeProject")} ${group.title}`}
                      onClick={(e) => { e.stopPropagation(); onCloseProject(group.key); }}
                    >
                      <CloseIcon size={14} />
                    </button>
                  </div>
                );
              })}
            </div>
          </>
        ) : null}
      </nav>

      <footer className="sidebar-footer">
        <button
          type="button"
          className={`sidebar-link ${activeView === "agents-global" ? "active" : ""}`}
          title={t("agents.nav")}
          onClick={() => onNavigate("agents-global")}
        >
          <span className="sidebar-link-icon"><AgentsIcon size={16} /></span>
          <span>{t("agents.nav")}</span>
        </button>
        <button
          type="button"
          className={`sidebar-link ${activeView === "settings" ? "active" : ""}`}
          title={t("sidebar.menu.settings")}
          onClick={onConfigurePath}
        >
          <span className="sidebar-link-icon"><SettingsIcon size={16} /></span>
          <span>{t("sidebar.menu.settings")}</span>
        </button>

        <div className="sidebar-version">
          <strong>v{packageJson.version}</strong>
        </div>

        <div className="sidebar-realtime-row">
          <div className={`sidebar-realtime realtime-${realtimeStatus}`}>
            <span className="realtime-dot" />
            <span className="sidebar-realtime-label">
              {realtimeLabel}
              {lastRealtimeSyncAt ? ` · ${lastRealtimeSyncAt}` : ""}
            </span>
          </div>
          <button
            type="button"
            className="sidebar-icon-button"
            title={t("app.actions.refresh")}
            aria-label={t("app.actions.refresh")}
            onClick={onRefresh}
          >
            <RefreshIcon size={16} />
          </button>
        </div>
      </footer>
    </aside>
  );
}
