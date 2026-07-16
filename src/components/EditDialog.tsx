import { useEffect, useState } from "react";
import { useI18n } from "../i18n/I18nProvider";
import type { EditDialogState } from "../types";
import { Button } from "./ui/Button";

type Props = {
  dialog: EditDialogState;
  onCancel: () => void;
  onConfirm: (value: string) => void;
};

export function EditDialog({ dialog, onCancel, onConfirm }: Props) {
  const { t } = useI18n();
  const [value, setValue] = useState(dialog.initialValue);

  const handleSecondaryAction = () => {
    dialog.onSecondaryAction?.(value);
    onCancel();
  };

  useEffect(() => {
    setValue(dialog.initialValue);
  }, [dialog.initialValue]);

  return (
    <div className="dialog-backdrop">
      <article className="dialog-card">
        <h3>{dialog.title}</h3>
        <p>{dialog.message}</p>
        <div className="dialog-form">
          {dialog.multiline ? (
            <textarea
              className="dialog-input dialog-input-multiline"
              value={value}
              onChange={(event) => setValue(event.currentTarget.value)}
            />
          ) : (
            <input
              className="dialog-input"
              value={value}
              onChange={(event) => setValue(event.currentTarget.value)}
            />
          )}
        </div>
        <div className="dialog-actions">
          {dialog.secondaryActionLabel ? (
            <Button
              variant={dialog.secondaryActionTone === "danger" ? "danger" : "secondary"}
              onClick={handleSecondaryAction}
            >
              {dialog.secondaryActionLabel}
            </Button>
          ) : null}
          <Button variant="secondary" onClick={onCancel}>
            {t("dialog.cancel")}
          </Button>
          <Button variant="primary" onClick={() => onConfirm(value)}>
            {dialog.actionLabel}
          </Button>
        </div>
      </article>
    </div>
  );
}
