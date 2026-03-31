import { useI18n } from "../i18n/I18nProvider";
import type { ConfirmDialogState } from "../types";

type Props = {
  dialog: ConfirmDialogState;
  onCancel: () => void;
};

export function ConfirmDialog({ dialog, onCancel }: Props) {
  const { t } = useI18n();

  return (
    <div className="dialog-backdrop">
      <article className="dialog-card">
        <h3>{dialog.title}</h3>
        <p>{dialog.message}</p>
        <div className="dialog-actions">
          <button type="button" className="ghost-button" onClick={onCancel}>
            {t("dialog.cancel")}
          </button>
          <button
            type="button"
            className={dialog.tone === "danger" ? "danger-button" : "dialog-confirm-button"}
            onClick={() => {
              dialog.onConfirm();
              onCancel();
            }}
          >
            {dialog.actionLabel}
          </button>
        </div>
      </article>
    </div>
  );
}
