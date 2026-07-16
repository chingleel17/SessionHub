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
  FocusIcon,
  PlanIcon,
  StatsIcon,
  TerminalIcon,
  UnarchiveIcon,
} from "./Icons";
import { IconButton } from "./ui/IconButton";
import { ProviderIcon } from "./ProviderIcon";
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
            <ProviderIcon provider={session.provider} label={getProviderLabel(session.provider)} />
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
        <IconButton
          label={t("session.actions.resumeWithProvider").replace("{provider}", getProviderLabel(session.provider))}
          className="session-action-button"
          onClick={() => onResumeSession(session)}
        >
          <TerminalIcon size={16} />
        </IconButton>
        <IconButton
          label={t("session.actions.focusTerminal")}
          className="session-action-button"
          onClick={() => onFocusTerminal(session)}
        >
          <FocusIcon size={16} />
        </IconButton>

        {supportsCommandCopy ? (
          <IconButton
            label={t("session.actions.copyCommand")}
            className="session-action-button"
            onClick={() => onCopyCommand(session)}
          >
            <CopyIcon size={16} />
          </IconButton>
        ) : null}

        <IconButton
          label={t("session.actions.editNotes")}
          className="session-action-button"
          onClick={() => onEditNotes(session)}
        >
          <EditNotesIcon size={16} />
        </IconButton>

        <IconButton
          label={t("session.actions.editTags")}
          className="session-action-button"
          onClick={() => onEditTags(session)}
        >
          <EditTagsIcon size={16} />
        </IconButton>

        {supportsPlanEditing ? (
          <IconButton
            label={t("session.actions.editPlan")}
            className="session-action-button"
            onClick={() => onOpenPlan(session)}
          >
            <PlanIcon size={16} />
          </IconButton>
        ) : null}

        {session.isArchived ? (
          <IconButton
            label={t("session.actions.unarchive")}
            className="session-action-button"
            onClick={() => onUnarchive(session)}
          >
            <UnarchiveIcon size={16} />
          </IconButton>
        ) : (
          <IconButton
            label={t("session.actions.archive")}
            className="session-action-button"
            onClick={() => onArchive(session)}
          >
            <ArchiveIcon size={16} />
          </IconButton>
        )}

        <IconButton
          label={t("stats.detail.title")}
          className="session-action-button"
          onClick={() => setShowStats((value) => !value)}
        >
          <StatsIcon size={16} />
        </IconButton>

        <IconButton
          label={t("session.actions.delete")}
          className="session-action-button"
          onClick={() => onDelete(session)}
          danger
        >
          <DeleteIcon size={16} />
        </IconButton>
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
