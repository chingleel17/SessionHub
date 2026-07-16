import { useI18n } from "../i18n/I18nProvider";
import type { ConfirmDialogState } from "../types";
import { Button } from "./ui/Button";

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
          <Button variant="secondary" onClick={onCancel}>
            {t("dialog.cancel")}
          </Button>
          <Button
            variant={dialog.tone === "danger" ? "danger" : "primary"}
            onClick={() => {
              dialog.onConfirm();
              onCancel();
            }}
          >
            {dialog.actionLabel}
          </Button>
        </div>
      </article>
    </div>
  );
}
