import { useI18n } from "../i18n/I18nProvider";
import type { SessionInfo } from "../types";
import { formatDateTime } from "../utils/formatDate";

type Props = {
  session: SessionInfo;
  onOpenTerminal: (session: SessionInfo) => void;
  onCopyCommand: (sessionId: string) => void;
  onEditNotes: (session: SessionInfo) => void;
  onEditTags: (session: SessionInfo) => void;
  onOpenPlan: (session: SessionInfo) => void;
  onArchive: (session: SessionInfo) => void;
  onDelete: (session: SessionInfo) => void;
};

function getSessionTitle(session: SessionInfo) {
  return session.summary?.trim() || session.id;
}

export function SessionCard({
  session,
  onOpenTerminal,
  onCopyCommand,
  onEditNotes,
  onEditTags,
  onOpenPlan,
  onArchive,
  onDelete,
}: Props) {
  const { t, locale } = useI18n();

  return (
    <article className="session-card">
      <div className="session-card-header">
        <div>
          <h3>{getSessionTitle(session)}</h3>
          <p>{session.id}</p>
        </div>

        <div className="session-chip-row">
          {session.isArchived ? (
            <span className="session-chip muted-chip">{t("session.archived")}</span>
          ) : null}
          {session.hasPlan ? (
            <span className="session-chip">{t("session.hasPlan")}</span>
          ) : null}
          {session.parseError ? (
            <span className="session-chip error-chip">{t("session.parseError")}</span>
          ) : null}
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

      {session.tags.length > 0 ? (
        <div className="session-chip-row">
          {session.tags.map((tag) => (
            <span key={tag} className="session-chip">
              #{tag}
            </span>
          ))}
        </div>
      ) : null}

      <div className="session-actions">
        <button type="button" className="ghost-button" onClick={() => onOpenTerminal(session)}>
          {t("session.actions.openTerminal")}
        </button>
        <button type="button" className="ghost-button" onClick={() => onCopyCommand(session.id)}>
          {t("session.actions.copyCommand")}
        </button>
        <button type="button" className="ghost-button" onClick={() => onEditNotes(session)}>
          {t("session.actions.editNotes")}
        </button>
        <button type="button" className="ghost-button" onClick={() => onEditTags(session)}>
          {t("session.actions.editTags")}
        </button>
        <button type="button" className="ghost-button" onClick={() => onOpenPlan(session)}>
          {t("session.actions.editPlan")}
        </button>
        {!session.isArchived ? (
          <button type="button" className="ghost-button" onClick={() => onArchive(session)}>
            {t("session.actions.archive")}
          </button>
        ) : null}
        <button type="button" className="danger-button" onClick={() => onDelete(session)}>
          {t("session.actions.delete")}
        </button>
      </div>
    </article>
  );
}
