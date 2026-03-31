import { useEffect, useState } from "react";
import { useI18n } from "../i18n/I18nProvider";
import type { EditDialogState } from "../types";

type Props = {
  dialog: EditDialogState;
  onCancel: () => void;
  onConfirm: (value: string) => void;
};

export function EditDialog({ dialog, onCancel, onConfirm }: Props) {
  const { t } = useI18n();
  const [value, setValue] = useState(dialog.initialValue);

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
          <button type="button" className="ghost-button" onClick={onCancel}>
            {t("dialog.cancel")}
          </button>
          <button type="button" className="dialog-confirm-button" onClick={() => onConfirm(value)}>
            {dialog.actionLabel}
          </button>
        </div>
      </article>
    </div>
  );
}
