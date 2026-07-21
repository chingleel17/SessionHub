import { useState } from "react";
import packageJson from "../../package.json";
import { useI18n } from "../i18n/I18nProvider";
import type { ProjectGroup, RealtimeStatus } from "../types";
import { AgentsIcon, CloseIcon, DashboardIcon, PanelLeftIcon, PinIcon, RefreshIcon, SettingsIcon } from "./Icons";

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
  onPinProject: (key: string) => void;
  onCollapseToggle: () => void;
  onRefresh: () => void;
  onConfigurePath: () => void;
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
  onPinProject,
  onCollapseToggle,
  onRefresh,
  onConfigurePath,
}: Props) {
  const { t } = useI18n();

  const [dragKey, setDragKey] = useState<string | null>(null);
  const [dragOverKey, setDragOverKey] = useState<string | null>(null);
  const [isDraggingOverPinned, setIsDraggingOverPinned] = useState(false);

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

  const handleDragStart = (key: string, e: React.DragEvent) => {
    // setData 是必須的，否則瀏覽器會顯示禁止符號並取消 drag
    e.dataTransfer.setData("text/plain", key);
    e.dataTransfer.effectAllowed = "move";
    setDragKey(key);
  };

  const handleDragEnd = () => {
    setDragKey(null);
    setDragOverKey(null);
    setIsDraggingOverPinned(false);
  };

  const handleDragOverItem = (e: React.DragEvent, targetKey: string) => {
    e.preventDefault();
    e.dataTransfer.dropEffect = "move";
    if (dragKey && dragKey !== targetKey) setDragOverKey(targetKey);
  };

  const handleDropOnItem = (e: React.DragEvent, targetKey: string) => {
    e.preventDefault();
    if (!dragKey || dragKey === targetKey) return;
    const nonPinnedKeys = openProjectKeys.filter((k) => !pinnedProjects.includes(k));
    const from = nonPinnedKeys.indexOf(dragKey);
    const to = nonPinnedKeys.indexOf(targetKey);
    if (from === -1 || to === -1) return;
    const next = [...nonPinnedKeys];
    next.splice(from, 1);
    next.splice(to, 0, dragKey);
    onReorderOpenProjects(next);
    setDragKey(null);
    setDragOverKey(null);
  };

  const handleDragOverPinned = (e: React.DragEvent) => {
    if (!dragKey) return;
    e.preventDefault();
    e.dataTransfer.dropEffect = "move";
    setIsDraggingOverPinned(true);
  };

  const handleDragLeavePinned = () => {
    setIsDraggingOverPinned(false);
  };

  const handleDropOnPinned = (e: React.DragEvent) => {
    e.preventDefault();
    if (!dragKey) return;
    onPinProject(dragKey);
    setDragKey(null);
    setIsDraggingOverPinned(false);
  };

  const getGroupTooltip = (group: ProjectGroup) =>
    group.branchLabel ? `${group.title} · ${group.branchLabel}` : group.title;

  const renderGroupLabel = (group: ProjectGroup) => (
    <span className="sidebar-group-label-wrap">
      <span className="sidebar-pinned-item-label">{group.title}</span>
      {group.branchLabel ? <span className="sidebar-branch-label">· {group.branchLabel}</span> : null}
    </span>
  );

  return (
    <aside className="sidebar">
      <div className="sidebar-brand">
        <div className="sidebar-brand-icon">SH</div>
        <div className="sidebar-brand-copy">
          <h1 className="topbar-title">{t("app.title")}</h1>
        </div>
      </div>

      <button
        type="button"
        className="sidebar-collapse-button"
        onClick={onCollapseToggle}
        aria-label={isSidebarCollapsed ? t("sidebar.expand") : t("sidebar.collapse")}
        title={isSidebarCollapsed ? t("sidebar.expand") : t("sidebar.collapse")}
      >
        <PanelLeftIcon size={18} />
      </button>

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

        {visiblePinnedGroups.length > 0 ? (
          <div
            className={`sidebar-section sidebar-pinned-section ${isDraggingOverPinned ? "sidebar-section--drop-target" : ""}`}
            onDragOver={handleDragOverPinned}
            onDragLeave={handleDragLeavePinned}
            onDrop={handleDropOnPinned}
          >
            <div className="sidebar-section-divider" aria-hidden="true" />

            <div className="sidebar-section-list">
              {visiblePinnedGroups.map((group) => {
                const initial = group.title.charAt(0).toUpperCase();
                return (
                  <button
                    key={group.key}
                    type="button"
                    className={`sidebar-link ${activeView === group.key ? "active" : ""}`}
                    title={getGroupTooltip(group)}
                    onClick={() => onOpenProject(group.key)}
                  >
                    <span className="sidebar-link-icon sidebar-pinned-initial">
                      {initial}
                      <span className="sidebar-pin-badge" aria-hidden="true">
                        <PinIcon size={9} />
                      </span>
                    </span>
                    {renderGroupLabel(group)}
                  </button>
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
                const isBeingDraggedOver = dragOverKey === group.key;
                return (
                  <div
                    key={group.key}
                    className={`sidebar-open-item ${isBeingDraggedOver ? "sidebar-open-item--drag-over" : ""} ${dragKey === group.key ? "sidebar-open-item--dragging" : ""}`}
                    onDragOver={(e) => handleDragOverItem(e, group.key)}
                    onDrop={(e) => handleDropOnItem(e, group.key)}
                  >
                    <button
                      type="button"
                      draggable
                      className={`sidebar-link sidebar-open-item-label ${activeView === group.key ? "active" : ""}`}
                      title={getGroupTooltip(group)}
                      onClick={() => onOpenProject(group.key)}
                      onDragStart={(e) => handleDragStart(group.key, e)}
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
            onClick={onRefresh}
          >
            <RefreshIcon size={16} />
          </button>
        </div>
      </footer>
    </aside>
  );
}
