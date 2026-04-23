import { useState } from "react";
import packageJson from "../../package.json";
import { useI18n } from "../i18n/I18nProvider";
import { useTheme } from "../theme/ThemeProvider";
import type { ProjectGroup, RealtimeStatus } from "../types";
import { MoonIcon, PinIcon, SunIcon } from "./Icons";

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
  const { theme, setTheme } = useTheme();

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
        >
          {isSidebarCollapsed ? (
            <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
              <rect x="3" y="3" width="18" height="18" rx="2" />
              <line x1="9" y1="3" x2="9" y2="21" />
              <polyline points="12 9 15 12 12 15" />
            </svg>
          ) : (
            <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
              <rect x="3" y="3" width="18" height="18" rx="2" />
              <line x1="9" y1="3" x2="9" y2="21" />
              <polyline points="15 9 12 12 15 15" />
            </svg>
          )}
        </button>
      </div>

      <nav className="sidebar-menu">
        <button
          type="button"
          className={`sidebar-link ${activeView === "dashboard" ? "active" : ""}`}
          onClick={() => onNavigate("dashboard")}
        >
          <span className="sidebar-link-icon">◫</span>
          <span>{t("sidebar.menu.dashboard")}</span>
        </button>

        {visiblePinnedGroups.length > 0 ? (
          <div
            className={`sidebar-section sidebar-pinned-section ${isDraggingOverPinned ? "sidebar-section--drop-target" : ""}`}
            onDragOver={handleDragOverPinned}
            onDragLeave={handleDragLeavePinned}
            onDrop={handleDropOnPinned}
          >
            {isSidebarCollapsed && <div className="sidebar-section-divider" aria-hidden="true" />}

            <div className={`sidebar-section-list ${isSidebarCollapsed ? "sidebar-section-list--collapsed" : ""}`}>
              {visiblePinnedGroups.map((group) => {
                const initial = group.title.charAt(0).toUpperCase();
                return isSidebarCollapsed ? (
                  <button
                    key={group.key}
                    type="button"
                    className={`sidebar-icon-button ${activeView === group.key ? "active" : ""}`}
                    title={group.title}
                    onClick={() => onOpenProject(group.key)}
                  >
                    {initial}
                  </button>
                ) : (
                  <button
                    key={group.key}
                    type="button"
                    className={`sidebar-link ${activeView === group.key ? "active" : ""}`}
                    title={group.pathLabel}
                    onClick={() => onOpenProject(group.key)}
                  >
                    <span className="sidebar-link-icon sidebar-pin-icon">
                      <PinIcon size={13} />
                    </span>
                    <span className="sidebar-pinned-item-label">{group.title}</span>
                  </button>
                );
              })}
            </div>
          </div>
        ) : null}

        {openGroups.length > 0 ? (
          <>
            {/* 分隔線列：左側線條 + 右側全部關閉按鈕（collapsed 時只顯示分隔線） */}
            {!isSidebarCollapsed ? (
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
            ) : (
              visiblePinnedGroups.length > 0 ? (
                <div className="sidebar-section-divider" aria-hidden="true" />
              ) : null
            )}

            <div className={`sidebar-section-list ${isSidebarCollapsed ? "sidebar-section-list--collapsed" : ""}`}>
              {openGroups.map((group) => {
                const initial = group.title.charAt(0).toUpperCase();
                const isBeingDraggedOver = dragOverKey === group.key;
                return isSidebarCollapsed ? (
                  <div key={group.key} className="sidebar-open-icon-wrap">
                    <button
                      type="button"
                      className={`sidebar-icon-button ${activeView === group.key ? "active" : ""}`}
                      title={group.title}
                      onClick={() => onOpenProject(group.key)}
                    >
                      {initial}
                    </button>
                    <button
                      type="button"
                      className="sidebar-open-item-close sidebar-open-item-close--collapsed"
                      title={t("sidebar.closeProject")}
                      aria-label={`${t("sidebar.closeProject")} ${group.title}`}
                      onClick={(e) => { e.stopPropagation(); onCloseProject(group.key); }}
                    >
                      ×
                    </button>
                  </div>
                ) : (
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
                      title={group.pathLabel}
                      onClick={() => onOpenProject(group.key)}
                      onDragStart={(e) => handleDragStart(group.key, e)}
                      onDragEnd={handleDragEnd}
                    >
                      <span className="sidebar-link-icon sidebar-pinned-initial">{initial}</span>
                      <span className="sidebar-pinned-item-label">{group.title}</span>
                    </button>
                    <button
                      type="button"
                      className="sidebar-open-item-close"
                      title={t("sidebar.closeProject")}
                      aria-label={`${t("sidebar.closeProject")} ${group.title}`}
                      onClick={(e) => { e.stopPropagation(); onCloseProject(group.key); }}
                    >
                      ×
                    </button>
                  </div>
                );
              })}
            </div>
          </>
        ) : null}
      </nav>

      <footer className="sidebar-footer">
        {isSidebarCollapsed ? (
          <button
            type="button"
            className="sidebar-icon-button"
            title={theme === "light" ? t("sidebar.theme.dark") : t("sidebar.theme.light")}
            onClick={() => setTheme(theme === "light" ? "dark" : "light")}
          >
            {theme === "light" ? <SunIcon size={16} /> : <MoonIcon size={16} />}
          </button>
        ) : (
          <div className="theme-toggle-row">
            <span className={`theme-toggle-icon ${theme === "light" ? "active" : ""}`}>☀</span>
            <button
              type="button"
              role="switch"
              aria-checked={theme === "dark"}
              className={`theme-toggle-switch ${theme === "dark" ? "dark" : ""}`}
              title={theme === "light" ? t("sidebar.theme.dark") : t("sidebar.theme.light")}
              onClick={() => setTheme(theme === "light" ? "dark" : "light")}
            >
              <span className="theme-toggle-thumb" />
            </button>
            <span className={`theme-toggle-icon ${theme === "dark" ? "active" : ""}`}>☾</span>
            <span className="theme-toggle-label">
              {theme === "light" ? t("sidebar.theme.light") : t("sidebar.theme.dark")}
            </span>
          </div>
        )}

        {isSidebarCollapsed ? (
          <div className="sidebar-quick-actions">
            <button
              type="button"
              className={`sidebar-icon-button ${activeView === "settings" ? "active" : ""}`}
              title={t("sidebar.menu.settings")}
              onClick={onConfigurePath}
            >
              ⚙
            </button>
            <button
              type="button"
              className="sidebar-icon-button"
              title={t("app.actions.refresh")}
              onClick={onRefresh}
            >
              ↺
            </button>
          </div>
        ) : null}

        {!isSidebarCollapsed ? (
          <button
            type="button"
            className={`sidebar-link ${activeView === "settings" ? "active" : ""}`}
            onClick={onConfigurePath}
          >
            <span className="sidebar-link-icon">⚙</span>
            <span>{t("sidebar.menu.settings")}</span>
          </button>
        ) : null}

        <div className="sidebar-version">
          {isSidebarCollapsed ? (
            <strong title={`v${packageJson.version}`}>
              v{packageJson.version.split(".").slice(0, 2).join(".")}
            </strong>
          ) : (
            <strong>v{packageJson.version}</strong>
          )}
        </div>

        <div className="sidebar-realtime-row">
          <div className={`sidebar-realtime realtime-${realtimeStatus}`}>
            <span className="realtime-dot" />
            <span className="sidebar-realtime-label">
              {realtimeLabel}
              {lastRealtimeSyncAt ? ` · ${lastRealtimeSyncAt}` : ""}
            </span>
          </div>
          {!isSidebarCollapsed ? (
            <button
              type="button"
              className="sidebar-icon-button"
              title={t("app.actions.refresh")}
              onClick={onRefresh}
            >
              ↺
            </button>
          ) : null}
        </div>
      </footer>
    </aside>
  );
}
