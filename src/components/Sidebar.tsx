import { useRef, useState } from "react";
import packageJson from "../../package.json";
import { useI18n } from "../i18n/I18nProvider";
import type { RealtimeStatus } from "../types";

type Props = {
  activeView: string;
  isSidebarCollapsed: boolean;
  realtimeStatus: RealtimeStatus;
  lastRealtimeSyncAt: string | null;
  onNavigate: (view: string) => void;
  onCollapseToggle: () => void;
  onRefresh: () => void;
  onConfigurePath: () => void;
};

export function Sidebar({
  activeView,
  isSidebarCollapsed,
  realtimeStatus,
  lastRealtimeSyncAt,
  onNavigate,
  onCollapseToggle,
  onRefresh,
  onConfigurePath,
}: Props) {
  const { t } = useI18n();
  const [openPopover, setOpenPopover] = useState<"language" | "iconStyle" | null>(null);
  const langBtnRef = useRef<HTMLButtonElement>(null);
  const iconBtnRef = useRef<HTMLButtonElement>(null);

  const realtimeLabel =
    realtimeStatus === "error"
      ? t("dashboard.status.realtimeError")
      : realtimeStatus === "active"
        ? t("dashboard.status.realtimeActive")
        : t("dashboard.status.realtimeConnecting");

  const togglePopover = (name: "language" | "iconStyle") => {
    setOpenPopover((current) => (current === name ? null : name));
  };

  return (
    <aside className="sidebar">
      <div className="sidebar-brand">
        <div className="sidebar-brand-icon">CS</div>
        <div className="sidebar-brand-copy">
          <span className="topbar-badge">{t("app.badge")}</span>
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
        {isSidebarCollapsed ? "»" : "«"}
      </button>

      <nav className="sidebar-menu">
        <button
          type="button"
          className={`sidebar-link ${activeView === "dashboard" ? "active" : ""}`}
          onClick={() => onNavigate("dashboard")}
        >
          <span className="sidebar-link-icon">◫</span>
          <span>{t("sidebar.menu.dashboard")}</span>
        </button>

        <button
          type="button"
          className={`sidebar-link ${activeView === "settings" ? "active" : ""}`}
          onClick={() => onNavigate("settings")}
        >
          <span className="sidebar-link-icon">⚙</span>
          <span>{t("sidebar.menu.settings")}</span>
        </button>
      </nav>

      <footer className="sidebar-footer">
        <div className="sidebar-quick-actions">
          <div className="sidebar-quick-action-item">
            <button
              ref={langBtnRef}
              type="button"
              className={`sidebar-icon-button ${openPopover === "language" ? "active" : ""}`}
              title={t("sidebar.language.label")}
              onClick={() => togglePopover("language")}
            >
              文
            </button>
            {openPopover === "language" ? (
              <div className="sidebar-popover">
                <div className="sidebar-popover-header">{t("sidebar.language.label")}</div>
                <button
                  type="button"
                  className="sidebar-popover-item active"
                  onClick={() => setOpenPopover(null)}
                >
                  {t("sidebar.language.current")}
                </button>
              </div>
            ) : null}
          </div>

          <div className="sidebar-quick-action-item">
            <button
              ref={iconBtnRef}
              type="button"
              className={`sidebar-icon-button ${openPopover === "iconStyle" ? "active" : ""}`}
              title={t("sidebar.iconStyle.label")}
              onClick={() => togglePopover("iconStyle")}
            >
              ◌
            </button>
            {openPopover === "iconStyle" ? (
              <div className="sidebar-popover">
                <div className="sidebar-popover-header">{t("sidebar.iconStyle.label")}</div>
                <button
                  type="button"
                  className="sidebar-popover-item active"
                  onClick={() => setOpenPopover(null)}
                >
                  {t("sidebar.iconStyle.current")}
                </button>
              </div>
            ) : null}
          </div>

          <button
            type="button"
            className="sidebar-icon-button"
            title={t("app.actions.configureCopilotPath")}
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

        <div className="sidebar-version">
          <span>{t("sidebar.version")}</span>
          <strong>v{packageJson.version}</strong>
        </div>

        <div className={`sidebar-realtime realtime-${realtimeStatus}`}>
          <span className="realtime-dot" />
          <span className="sidebar-realtime-label">
            {realtimeLabel}
            {lastRealtimeSyncAt ? ` · ${lastRealtimeSyncAt}` : ""}
          </span>
        </div>
      </footer>
    </aside>
  );
}
