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
  projectGroups: ProjectGroup[];
  onNavigate: (view: string) => void;
  onOpenProject: (projectKey: string) => void;
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
  projectGroups,
  onNavigate,
  onOpenProject,
  onCollapseToggle,
  onRefresh,
  onConfigurePath,
}: Props) {
  const { t } = useI18n();
  const { theme, setTheme } = useTheme();

  const realtimeLabel =
    sessionsIsFetching
      ? t("dashboard.status.scanning")
      : realtimeStatus === "error"
        ? t("dashboard.status.realtimeError")
        : realtimeStatus === "active"
          ? t("dashboard.status.realtimeActive")
          : t("dashboard.status.realtimeConnecting");

  // 只顯示有對應 projectGroup 的釘選項目
  const visiblePinnedGroups = pinnedProjects
    .map((key) => projectGroups.find((g) => g.key === key))
    .filter((g): g is ProjectGroup => Boolean(g));

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
          <div className="sidebar-section">
            {!isSidebarCollapsed ? (
              <div className="sidebar-section-title">
                <PinIcon size={12} />
                <span>{t("sidebar.pinnedProjects")}</span>
              </div>
            ) : (
              <div className="sidebar-section-divider" aria-hidden="true" />
            )}

            <div className={`sidebar-section-list ${isSidebarCollapsed ? "sidebar-section-list--collapsed" : ""}`}>
              {visiblePinnedGroups.map((group) => {
                const initial = group.title.charAt(0).toUpperCase();
                return isSidebarCollapsed ? (
                  <button
                    key={group.key}
                    type="button"
                    className={`sidebar-icon-button ${activeView === group.key ? "active" : ""}`}
                    title={`${t("sidebar.pinnedProjects")}: ${group.title}`}
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
                    <span className="sidebar-link-icon sidebar-pinned-initial">{initial}</span>
                    <span className="sidebar-pinned-item-label">{group.title}</span>
                  </button>
                );
              })}
            </div>
          </div>
        ) : null}
      </nav>

      <footer className="sidebar-footer">
        {/* Theme toggle: icon button when collapsed, switch row when expanded */}
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

        {/* Collapsed: icon buttons for settings + refresh */}
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

        {/* Settings row (expanded only) */}
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
