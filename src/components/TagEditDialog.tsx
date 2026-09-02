import { useEffect, useState, type KeyboardEvent } from "react";
import { useI18n } from "../i18n/I18nProvider";
import { Button } from "./ui/Button";
import { Modal } from "./ui/Modal";

type Props = {
  title: string;
  message: string;
  actionLabel: string;
  initialTags: string[];
  onCancel: () => void;
  onConfirm: (tags: string[]) => void;
};

function normalizeTag(value: string): string {
  return value.trim();
}

function addTag(tags: string[], value: string): string[] {
  const tag = normalizeTag(value);
  if (!tag || tags.some((current) => current.toLowerCase() === tag.toLowerCase())) return tags;
  return [...tags, tag];
}

export function TagEditDialog({
  title,
  message,
  actionLabel,
  initialTags,
  onCancel,
  onConfirm,
}: Props) {
  const { t } = useI18n();
  const [tags, setTags] = useState(initialTags);
  const [draft, setDraft] = useState("");

  useEffect(() => {
    setTags(initialTags);
    setDraft("");
  }, [initialTags]);

  const handleKeyDown = (event: KeyboardEvent<HTMLInputElement>) => {
    if (event.key !== "Enter") return;
    event.preventDefault();
    setTags((current) => addTag(current, draft));
    setDraft("");
  };

  return (
    <Modal panelClassName="tag-edit-dialog" ariaLabel={title}>
      <h3>{title}</h3>
      <p>{message}</p>
      <div className="tag-edit-form">
        <div className="tag-edit-list" aria-live="polite">
          {tags.map((tag) => (
            <span className="tag-edit-chip" key={tag}>
              <span>#{tag}</span>
              <button
                type="button"
                className="tag-edit-chip-remove"
                onClick={() => setTags((current) => current.filter((item) => item !== tag))}
                aria-label={`${t("session.actions.removeTag")}: #${tag}`}
              >
                ×
              </button>
            </span>
          ))}
        </div>
        <input
          className="dialog-input"
          value={draft}
          onChange={(event) => setDraft(event.currentTarget.value)}
          onKeyDown={handleKeyDown}
          placeholder={t("session.prompt.tagInput")}
          autoFocus
        />
      </div>
      <div className="dialog-actions">
        <Button variant="secondary" onClick={onCancel}>
          {t("dialog.cancel")}
        </Button>
        <Button variant="primary" onClick={() => onConfirm(addTag(tags, draft))}>
          {actionLabel}
        </Button>
      </div>
    </Modal>
  );
}
