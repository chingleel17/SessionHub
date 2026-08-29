import { useEffect, useRef } from "react";
import { useI18n } from "../i18n/I18nProvider";
import type { ProjectGroup } from "../types";
import { CloseIcon } from "./Icons";
import { Button } from "./ui/Button";

type ProjectPickerDialogProps = {
  projects: ProjectGroup[];
  busyProjectKey: string | null;
  onClose: () => void;
  onOpenProject: (projectKey: string) => void;
  onPinProject: (projectKey: string) => void;
};

function getFocusableElements(container: HTMLElement): HTMLElement[] {
  return Array.from(
    container.querySelectorAll<HTMLElement>(
      'button:not([disabled]), [href], input:not([disabled]), select:not([disabled]), textarea:not([disabled]), [tabindex]:not([tabindex="-1"])',
    ),
  );
}

export function ProjectPickerDialog({
  projects,
  busyProjectKey,
  onClose,
  onOpenProject,
  onPinProject,
}: ProjectPickerDialogProps) {
  const { t } = useI18n();
  const dialogRef = useRef<HTMLElement | null>(null);
  const closeButtonRef = useRef<HTMLButtonElement | null>(null);
  const restoreFocusRef = useRef<HTMLElement | null>(null);

  useEffect(() => {
    restoreFocusRef.current = document.activeElement instanceof HTMLElement ? document.activeElement : null;
    closeButtonRef.current?.focus();

    const handleKeyDown = (event: KeyboardEvent) => {
      if (event.key === "Escape") {
        event.preventDefault();
        onClose();
        return;
      }

      if (event.key !== "Tab") return;
      const dialog = dialogRef.current;
      if (!dialog) return;

      const focusableElements = getFocusableElements(dialog);
      if (focusableElements.length === 0) {
        event.preventDefault();
        dialog.focus();
        return;
      }

      const first = focusableElements[0];
      const last = focusableElements[focusableElements.length - 1];
      if (event.shiftKey && document.activeElement === first) {
        event.preventDefault();
        last.focus();
      } else if (!event.shiftKey && document.activeElement === last) {
        event.preventDefault();
        first.focus();
      }
    };

    document.addEventListener("keydown", handleKeyDown);
    return () => {
      document.removeEventListener("keydown", handleKeyDown);
      restoreFocusRef.current?.focus();
    };
  }, [onClose]);

  return (
    <div
      className="dialog-backdrop"
      role="presentation"
      onClick={(event) => {
        if (event.target === event.currentTarget) onClose();
      }}
    >
      <article
        ref={dialogRef}
        className="dialog-card dialog-card--solid project-picker-dialog"
        role="dialog"
        aria-modal="true"
        aria-labelledby="project-picker-title"
        aria-describedby="project-picker-description"
        tabIndex={-1}
      >
        <header className="project-picker-header">
          <div>
            <h3 id="project-picker-title">{t("sidebar.projectPicker.title")}</h3>
            <p id="project-picker-description">{t("sidebar.projectPicker.description")}</p>
          </div>
          <button
            ref={closeButtonRef}
            type="button"
            className="project-picker-close"
            aria-label={t("sidebar.projectPicker.close")}
            title={t("sidebar.projectPicker.close")}
            onClick={onClose}
          >
            <CloseIcon size={16} />
          </button>
        </header>

        {projects.length === 0 ? (
          <p className="project-picker-empty">{t("sidebar.projectPicker.empty")}</p>
        ) : (
          <div className="project-picker-list">
            {projects.map((project) => (
              <div className="project-picker-item" key={project.key}>
                <div className="project-picker-item-info">
                  <strong title={project.title}>{project.title}</strong>
                  {project.branchLabel ? (
                    <span className="project-picker-branch">· {project.branchLabel}</span>
                  ) : null}
                  <code title={project.pathLabel}>{project.pathLabel}</code>
                </div>
                <div className="project-picker-item-actions">
                  <Button
                    variant="secondary"
                    disabled={Boolean(busyProjectKey)}
                    aria-label={`${t("sidebar.projectPicker.open")} ${project.title}`}
                    onClick={() => onOpenProject(project.key)}
                  >
                    {t("sidebar.projectPicker.open")}
                  </Button>
                  <Button
                    variant="primary"
                    disabled={Boolean(busyProjectKey)}
                    aria-label={`${t("sidebar.projectPicker.pin")} ${project.title}`}
                    onClick={() => onPinProject(project.key)}
                  >
                    {t("sidebar.projectPicker.pin")}
                  </Button>
                </div>
              </div>
            ))}
          </div>
        )}
      </article>
    </div>
  );
}
