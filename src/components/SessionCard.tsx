import { useState } from "react";
import { useI18n } from "../i18n/I18nProvider";
import type { SessionInfo, SessionStats } from "../types";
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

type Props = {
  session: SessionInfo;
  onOpenTerminal: (session: SessionInfo) => void;
  onCopyCommand: (sessionId: string) => void;
  onEditNotes: (session: SessionInfo) => void;
  onEditTags: (session: SessionInfo) => void;
  onOpenPlan: (session: SessionInfo) => void;
  onArchive: (session: SessionInfo) => void;
  onUnarchive: (session: SessionInfo) => void;
  onDelete: (session: SessionInfo) => void;
  stats?: SessionStats;
  statsLoading: boolean;
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
}: Props) {
  const { t, locale } = useI18n();
  const [showStats, setShowStats] = useState(false);

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

        <button
          type="button"
          className="icon-button"
          title={t("session.actions.copyCommand")}
          aria-label={t("session.actions.copyCommand")}
          onClick={() => onCopyCommand(session.id)}
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
