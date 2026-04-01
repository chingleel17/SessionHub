import { useEffect, useRef, useState } from "react";
import { createPortal } from "react-dom";
import packageJson from "../../package.json";
import { useI18n } from "../i18n/I18nProvider";
import { useTheme } from "../theme/ThemeProvider";
import type { ProjectGroup, RealtimeStatus } from "../types";
import { MoonIcon, SunIcon } from "./Icons";

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

type PopoverPos = { left: number; bottom: number };

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
  const { t, locale, setLocale } = useI18n();
  const { theme, setTheme } = useTheme();
  const [openPopover, setOpenPopover] = useState<"language" | null>(null);
  const [popoverPos, setPopoverPos] = useState<PopoverPos>({ left: 0, bottom: 0 });
  const langBtnRef = useRef<HTMLButtonElement>(null);

  const realtimeLabel =
    sessionsIsFetching
      ? t("dashboard.status.scanning")
      : realtimeStatus === "error"
        ? t("dashboard.status.realtimeError")
        : realtimeStatus === "active"
          ? t("dashboard.status.realtimeActive")
          : t("dashboard.status.realtimeConnecting");

  const togglePopover = (name: "language") => {
    const btnRef = langBtnRef;
    if (btnRef.current) {
      const rect = btnRef.current.getBoundingClientRect();
      setPopoverPos({ left: rect.left, bottom: window.innerHeight - rect.top + 8 });
    }
    setOpenPopover((current) => (current === name ? null : name));
  };

  useEffect(() => {
    if (!openPopover) return;
    const close = () => setOpenPopover(null);
    document.addEventListener("click", close);
    return () => document.removeEventListener("click", close);
  }, [openPopover]);

  // 只顯示有對應 projectGroup 的釘選項目
  const visiblePinnedGroups = pinnedProjects
    .map((key) => projectGroups.find((g) => g.key === key))
    .filter((g): g is ProjectGroup => Boolean(g));

  return (
    <aside className="sidebar">
      <div className="sidebar-brand">
        <div className="sidebar-brand-icon">CS</div>
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

        {/* 釘選專案：接在 Dashboard 下方，展開/收折都顯示 */}
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
              <span className="sidebar-link-icon sidebar-pinned-initial">{initial}</span>
              <span className="sidebar-pinned-item-label">{group.title}</span>
            </button>
          );
        })}
      </nav>

      {openPopover === "language"
        ? createPortal(
            <div
              className="sidebar-popover sidebar-popover-fixed"
              style={{ left: popoverPos.left, bottom: popoverPos.bottom }}
              onClick={(e) => e.stopPropagation()}
            >
              <div className="sidebar-popover-header">{t("sidebar.language.label")}</div>
              <button
                type="button"
                className={`sidebar-popover-item ${locale === "zh-TW" ? "active" : ""}`}
                onClick={() => { setLocale("zh-TW"); setOpenPopover(null); }}
              >
                {t("sidebar.language.zhTW")}
              </button>
              <button
                type="button"
                className={`sidebar-popover-item ${locale === "en-US" ? "active" : ""}`}
                onClick={() => { setLocale("en-US"); setOpenPopover(null); }}
              >
                {t("sidebar.language.enUS")}
              </button>
            </div>,
            document.body,
          )
        : null}

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

        {/* Language row */}
        {!isSidebarCollapsed ? (
          <div className="sidebar-quick-action-item">
            <button
              ref={langBtnRef}
              type="button"
              className={`sidebar-link ${openPopover === "language" ? "active" : ""}`}
              onClick={(e) => { e.stopPropagation(); togglePopover("language"); }}
            >
              <span className="sidebar-link-icon">文</span>
              <span>{t("sidebar.language.label")}</span>
            </button>
          </div>
        ) : (
          <div className="sidebar-quick-actions">
            <div className="sidebar-quick-action-item">
              <button
                ref={langBtnRef}
                type="button"
                className={`sidebar-icon-button ${openPopover === "language" ? "active" : ""}`}
                title={t("sidebar.language.label")}
                onClick={(e) => { e.stopPropagation(); togglePopover("language"); }}
              >
                文
              </button>
            </div>
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
        )}

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
