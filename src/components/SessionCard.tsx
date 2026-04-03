import { useState } from "react";
import { useI18n } from "../i18n/I18nProvider";
import type { IdeLauncherType, SessionActivityStatus, SessionInfo, SessionStats, ToolAvailability } from "../types";
import { formatDateTime } from "../utils/formatDate";
import {
  ArchiveIcon,
  CopyIcon,
  DeleteIcon,
  EditNotesIcon,
  EditTagsIcon,
  PlanIcon,
  StatsIcon,
  TerminalIcon,
  UnarchiveIcon,
} from "./Icons";
import { SessionStatsBadge } from "./SessionStatsBadge";
import { SessionStatsPanel } from "./SessionStatsPanel";

const LAUNCHER_OPTIONS: { type: IdeLauncherType; label: string; icon: string; availKey?: keyof ToolAvailability }[] = [
  { type: "terminal", label: "Terminal", icon: ">_" },
  { type: "copilot", label: "Copilot", icon: "C", availKey: "copilot" },
  { type: "opencode", label: "OpenCode", icon: "O", availKey: "opencode" },
  { type: "gemini", label: "Gemini", icon: "G", availKey: "gemini" },
  { type: "vscode", label: "VS Code", icon: "⌨", availKey: "vscode" },
  { type: "explorer", label: "Explorer", icon: "📁" },
];

type Props = {
  session: SessionInfo;
  onOpenTerminal: (session: SessionInfo) => void;
  onCopyCommand: (session: SessionInfo) => void;
  onEditNotes: (session: SessionInfo) => void;
  onEditTags: (session: SessionInfo) => void;
  onOpenPlan: (session: SessionInfo) => void;
  onArchive: (session: SessionInfo) => void;
  onUnarchive: (session: SessionInfo) => void;
  onDelete: (session: SessionInfo) => void;
  stats?: SessionStats;
  statsLoading: boolean;
  activityStatus?: SessionActivityStatus;
  onOpenInTool: (session: SessionInfo, tool: IdeLauncherType) => void;
  onFocusTerminal: (session: SessionInfo) => void;
  defaultLauncher: string | null;
  toolAvailability: ToolAvailability | null;
};

function getSessionTitle(session: SessionInfo) {
  return session.summary?.trim() || session.id;
}

function getProviderLabel(provider: string): string {
  switch (provider) {
    case "copilot":
      return "Copilot";
    case "opencode":
      return "OpenCode";
    default:
      return provider;
  }
}

export function SessionCard({
  session,
  onOpenTerminal,
  onCopyCommand,
  onEditNotes,
  onEditTags,
  onOpenPlan,
  onArchive,
  onUnarchive,
  onDelete,
  stats,
  statsLoading,
  activityStatus,
  onOpenInTool,
  onFocusTerminal,
  defaultLauncher,
  toolAvailability,
}: Props) {
  const { t, locale } = useI18n();
  const [showStats, setShowStats] = useState(false);
  const [showLauncher, setShowLauncher] = useState(false);

  const activityStatusCls = activityStatus
    ? `activity-badge activity-badge--${activityStatus.status}`
    : null;
  const activityLabel = activityStatus
    ? activityStatus.status === "active"
      ? (t as (k: string) => string)(`dashboard.kanban.detail.${activityStatus.detail ?? "working"}`)
      : (t as (k: string) => string)(`dashboard.kanban.status.${activityStatus.status}`)
    : null;

  return (
    <article className="session-card">
      <div className="session-card-header">
        <div>
          <h3>{getSessionTitle(session)}</h3>
          <p>{session.id}</p>
        </div>

        <div className="session-chip-row">
          <span className={`provider-tag provider-tag--${session.provider}`}>
            {getProviderLabel(session.provider)}
          </span>
          {session.isArchived ? (
            <span className="session-chip muted-chip">{t("session.archived")}</span>
          ) : null}
          {session.hasPlan ? (
            <span className="session-chip">{t("session.hasPlan")}</span>
          ) : null}
          {session.parseError ? (
            <span className="session-chip error-chip">{t("session.parseError")}</span>
          ) : null}
          {activityStatusCls && activityLabel ? (
            <span className={activityStatusCls}>{activityLabel}</span>
          ) : null}
          {session.tags.map((tag) => (
            <span key={tag} className="session-chip tag-chip">
              #{tag}
            </span>
          ))}
        </div>
      </div>

      <div className="session-meta-grid">
        <div>
          <span className="session-meta-label">{t("session.updatedAt")}</span>
          <p>{formatDateTime(session.updatedAt, locale)}</p>
        </div>
        <div>
          <span className="session-meta-label">{t("session.createdAt")}</span>
          <p>{formatDateTime(session.createdAt, locale)}</p>
        </div>
        <div>
          <span className="session-meta-label">{t("session.summaryCount")}</span>
          <p>{session.summaryCount ?? 0}</p>
        </div>
      </div>

      {session.notes ? (
        <p className="session-notes">
          <strong>{t("session.notes")}</strong> {session.notes}
        </p>
      ) : null}

      <div className="session-actions">
        <button
          type="button"
          className="icon-button"
          title={t("session.actions.openTerminal")}
          aria-label={t("session.actions.openTerminal")}
          onClick={() => onOpenTerminal(session)}
        >
          <TerminalIcon size={16} />
        </button>
        <div className="launcher-dropdown">
          <button
            type="button"
            className="icon-button"
            title={t("session.actions.chooseTool")}
            aria-label={t("session.actions.chooseTool")}
            onClick={() => setShowLauncher((v) => !v)}
          >
            ⋯
          </button>
          {showLauncher ? (
            <div className="launcher-menu">
              {LAUNCHER_OPTIONS.map((opt) => {
                const available = !opt.availKey || !toolAvailability ? true : toolAvailability[opt.availKey];
                return (
                  <button
                    key={opt.type}
                    type="button"
                    className={`launcher-menu-item${defaultLauncher === opt.type ? " launcher-menu-item--default" : ""}${!available ? " launcher-menu-item--disabled" : ""}`}
                    disabled={!available}
                    onClick={() => { onOpenInTool(session, opt.type); setShowLauncher(false); }}
                  >
                    <span className="launcher-option-icon">{opt.icon}</span>
                    {opt.label}
                    {!available ? <span className="launcher-option-unavail"> (未安裝)</span> : null}
                  </button>
                );
              })}
            </div>
          ) : null}
        </div>
        <button
          type="button"
          className="icon-button"
          title={t("session.actions.focusTerminal")}
          aria-label={t("session.actions.focusTerminal")}
          onClick={() => onFocusTerminal(session)}
        >
          ⊙
        </button>

        <button
          type="button"
          className="icon-button"
          title={t("session.actions.copyCommand")}
          aria-label={t("session.actions.copyCommand")}
          onClick={() => onCopyCommand(session)}
        >
          <CopyIcon size={16} />
        </button>

        <button
          type="button"
          className="icon-button"
          title={t("session.actions.editNotes")}
          aria-label={t("session.actions.editNotes")}
          onClick={() => onEditNotes(session)}
        >
          <EditNotesIcon size={16} />
        </button>

        <button
          type="button"
          className="icon-button"
          title={t("session.actions.editTags")}
          aria-label={t("session.actions.editTags")}
          onClick={() => onEditTags(session)}
        >
          <EditTagsIcon size={16} />
        </button>

        <button
          type="button"
          className="icon-button"
          title={t("session.actions.editPlan")}
          aria-label={t("session.actions.editPlan")}
          onClick={() => onOpenPlan(session)}
        >
          <PlanIcon size={16} />
        </button>

        {session.isArchived ? (
          <button
            type="button"
            className="icon-button"
            title={t("session.actions.unarchive")}
            aria-label={t("session.actions.unarchive")}
            onClick={() => onUnarchive(session)}
          >
            <UnarchiveIcon size={16} />
          </button>
        ) : (
          <button
            type="button"
            className="icon-button"
            title={t("session.actions.archive")}
            aria-label={t("session.actions.archive")}
            onClick={() => onArchive(session)}
          >
            <ArchiveIcon size={16} />
          </button>
        )}

        <button
          type="button"
          className="icon-button"
          title={t("stats.detail.title")}
          aria-label={t("stats.detail.title")}
          onClick={() => setShowStats((value) => !value)}
        >
          <StatsIcon size={16} />
        </button>

        <button
          type="button"
          className="icon-button icon-button--danger"
          title={t("session.actions.delete")}
          aria-label={t("session.actions.delete")}
          onClick={() => onDelete(session)}
        >
          <DeleteIcon size={16} />
        </button>
      </div>

      <SessionStatsBadge stats={stats} isLoading={statsLoading} />
      {showStats && stats ? <SessionStatsPanel stats={stats} /> : null}
    </article>
  );
}
