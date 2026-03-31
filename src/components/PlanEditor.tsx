import { useI18n } from "../i18n/I18nProvider";
import type { SessionInfo } from "../types";

type Props = {
  session: SessionInfo;
  planDraft: string;
  planPreviewHtml: string;
  onDraftChange: (value: string) => void;
  onSave: () => void;
  onOpenExternal: (session: SessionInfo) => void;
  onClose: () => void;
};

function getSessionTitle(session: SessionInfo) {
  return session.summary?.trim() || session.id;
}

export function PlanEditor({
  session,
  planDraft,
  planPreviewHtml,
  onDraftChange,
  onSave,
  onOpenExternal,
  onClose,
}: Props) {
  const { t } = useI18n();

  return (
    <article className="info-card plan-editor-card">
      <div className="section-heading">
        <h3>{t("plan.title")}</h3>
        <span>{getSessionTitle(session)}</span>
      </div>

      <div className="plan-editor-layout">
        <label className="field-group">
          <span>{t("plan.editor")}</span>
          <textarea
            className="plan-textarea"
            value={planDraft}
            onChange={(event) => onDraftChange(event.currentTarget.value)}
          />
        </label>

        <div className="plan-preview">
          <span className="session-meta-label">{t("plan.preview")}</span>
          <div
            className="plan-preview-markdown"
            dangerouslySetInnerHTML={{ __html: planPreviewHtml }}
          />
        </div>
      </div>

      <div className="settings-actions">
        <button type="button" className="primary-button" onClick={onSave}>
          <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.2" strokeLinecap="round" strokeLinejoin="round">
            <path d="M19 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h11l5 5v11a2 2 0 0 1-2 2z"/>
            <polyline points="17 21 17 13 7 13 7 21"/>
            <polyline points="7 3 7 8 15 8"/>
          </svg>
          {t("plan.actions.save")}
        </button>
        <button type="button" className="ghost-button" onClick={() => onOpenExternal(session)}>
          {t("plan.actions.openExternal")}
        </button>
        <button type="button" className="ghost-button" onClick={onClose}>
          {t("plan.actions.close")}
        </button>
      </div>
    </article>
  );
}
