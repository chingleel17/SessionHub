import { useI18n } from "../i18n/I18nProvider";
import type { SessionInfo } from "../types";
import { SaveIcon } from "./Icons";
import { Button } from "./ui/Button";

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
        <Button variant="primary" onClick={onSave}>
          <SaveIcon size={15} />
          {t("plan.actions.save")}
        </Button>
        <Button variant="secondary" onClick={() => onOpenExternal(session)}>
          {t("plan.actions.openExternal")}
        </Button>
        <Button variant="secondary" onClick={onClose}>
          {t("plan.actions.close")}
        </Button>
      </div>
    </article>
  );
}
