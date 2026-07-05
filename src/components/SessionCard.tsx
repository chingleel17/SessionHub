import { useState } from "react";
import { useI18n } from "../i18n/I18nProvider";
import type {
  SessionActivityStatus,
  SessionInfo,
  SessionStats,
  SessionTodo,
} from "../types";
import { formatDateTime } from "../utils/formatDate";
import { getProviderLabel } from "../utils/providerLabel";
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
  onCopyCommand: (session: SessionInfo) => void;
  onEditNotes: (session: SessionInfo) => void;
  onEditTags: (session: SessionInfo) => void;
  onEditTag: (session: SessionInfo, tag: string, tagIndex: number) => void;
  onOpenPlan: (session: SessionInfo) => void;
  onOpenTodos: (session: SessionInfo) => void;
  onArchive: (session: SessionInfo) => void;
  onUnarchive: (session: SessionInfo) => void;
  onDelete: (session: SessionInfo) => void;
  stats?: SessionStats;
  statsLoading: boolean;
  todos: SessionTodo[];
  todosLoading: boolean;
  activityStatus?: SessionActivityStatus;
  onResumeSession: (session: SessionInfo) => void;
  onFocusTerminal: (session: SessionInfo) => void;
};

function getSessionTitle(session: SessionInfo) {
  return session.summary?.trim() || session.id;
}

function supportsSessionCommandCopy(provider: string): boolean {
  return ["copilot", "opencode", "codex", "claude"].includes(provider);
}

export function SessionCard({
  session,
  onCopyCommand,
  onEditNotes,
  onEditTags,
  onEditTag,
  onOpenPlan,
  onOpenTodos,
  onArchive,
  onUnarchive,
  onDelete,
  stats,
  statsLoading,
  todos,
  todosLoading,
  activityStatus,
  onResumeSession,
  onFocusTerminal,
}: Props) {
  const { t, locale } = useI18n();
  const [showStats, setShowStats] = useState(false);

  const activityStatusCls = activityStatus
    ? `activity-badge activity-badge--${activityStatus.status}`
    : null;
  const supportsCommandCopy = supportsSessionCommandCopy(session.provider);
  const supportsPlanEditing = session.provider !== "codex";
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
            <button
              type="button"
              className="session-chip session-chip-button"
              onClick={() => onOpenPlan(session)}
            >
              {t("session.hasPlan")}
            </button>
          ) : null}
          {session.parseError ? (
            <span className="session-chip error-chip">{t("session.parseError")}</span>
          ) : null}
          {activityStatusCls && activityLabel ? (
            <span className={activityStatusCls}>{activityLabel}</span>
          ) : null}
          {session.tags.map((tag, tagIndex) => (
            <button
              key={`${session.id}:${tag}:${tagIndex}`}
              type="button"
              className="session-chip session-chip-button tag-chip tag-chip-button"
              onClick={() => onEditTag(session, tag, tagIndex)}
              title={t("session.actions.editTags")}
              aria-label={`${t("session.actions.editTags")}: #${tag}`}
            >
              #{tag}
            </button>
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
        <div>
          <span className="session-meta-label">{t("session.repo")}</span>
          <p>{session.repoName ?? "-"}</p>
        </div>
      </div>

      {session.notes ? (
        <button
          type="button"
          className="session-notes session-notes-button"
          onClick={() => onEditNotes(session)}
          title={t("session.actions.editNotes")}
          aria-label={t("session.actions.editNotes")}
        >
          <strong>{t("session.notes")}</strong> {session.notes}
        </button>
      ) : null}

      <div className="session-actions">
        <button
          type="button"
          className="icon-button"
          title={t("session.actions.resumeWithProvider").replace("{provider}", getProviderLabel(session.provider))}
          aria-label={t("session.actions.resumeWithProvider").replace("{provider}", getProviderLabel(session.provider))}
          onClick={() => onResumeSession(session)}
        >
          <TerminalIcon size={16} />
        </button>
        <button
          type="button"
          className="icon-button"
          title={t("session.actions.focusTerminal")}
          aria-label={t("session.actions.focusTerminal")}
          onClick={() => onFocusTerminal(session)}
        >
          ⊙
        </button>

        {supportsCommandCopy ? (
          <button
            type="button"
            className="icon-button"
            title={t("session.actions.copyCommand")}
            aria-label={t("session.actions.copyCommand")}
            onClick={() => onCopyCommand(session)}
          >
            <CopyIcon size={16} />
          </button>
        ) : null}

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

        {supportsPlanEditing ? (
          <button
            type="button"
            className="icon-button"
            title={t("session.actions.editPlan")}
            aria-label={t("session.actions.editPlan")}
            onClick={() => onOpenPlan(session)}
          >
            <PlanIcon size={16} />
          </button>
        ) : null}

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

      <SessionStatsBadge
        session={session}
        stats={stats}
        isLoading={statsLoading}
        todos={todos}
        todosLoading={todosLoading}
        onOpenTodos={onOpenTodos}
      />
      {showStats ? (
        <>
          {stats ? <SessionStatsPanel stats={stats} provider={session.provider} /> : null}
        </>
      ) : null}
    </article>
  );
}
