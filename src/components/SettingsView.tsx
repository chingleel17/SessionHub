import { useI18n } from "../i18n/I18nProvider";
import type { AppSettings } from "../types";

type Props = {
  settingsForm: AppSettings;
  onFormChange: (next: AppSettings) => void;
  onSave: () => void;
  onBrowseDirectory: (field: "copilotRoot") => void;
  onBrowseFile: (field: "terminalPath" | "externalEditorPath") => void;
  onDetectTerminal: () => void;
  onDetectVscode: () => void;
};

export function SettingsView({
  settingsForm,
  onFormChange,
  onSave,
  onBrowseDirectory,
  onBrowseFile,
  onDetectTerminal,
  onDetectVscode,
}: Props) {
  const { t } = useI18n();

  return (
    <section className="settings-layout">
      <article className="info-card">
        <div className="section-heading">
          <h3>{t("settings.general.title")}</h3>
          <span>{t("settings.general.subtitle")}</span>
        </div>

        <div className="settings-form">
          <label className="field-group">
            <span>{t("settings.fields.copilotRoot")}</span>
            <div className="field-with-action">
              <input
                value={settingsForm.copilotRoot}
                onChange={(event) =>
                  onFormChange({ ...settingsForm, copilotRoot: event.currentTarget.value })
                }
              />
              <button
                type="button"
                className="ghost-button"
                onClick={() => onBrowseDirectory("copilotRoot")}
              >
                {t("settings.actions.browseDirectory")}
              </button>
            </div>
          </label>

          <label className="field-group">
            <span>{t("settings.fields.terminalPath")}</span>
            <div className="field-with-action">
              <input
                value={settingsForm.terminalPath ?? ""}
                onChange={(event) =>
                  onFormChange({ ...settingsForm, terminalPath: event.currentTarget.value })
                }
              />
              <button
                type="button"
                className="ghost-button"
                onClick={() => onBrowseFile("terminalPath")}
              >
                {t("settings.actions.browseFile")}
              </button>
            </div>
          </label>

          <label className="field-group">
            <span>{t("settings.fields.externalEditorPath")}</span>
            <div className="field-with-action">
              <input
                value={settingsForm.externalEditorPath ?? ""}
                onChange={(event) =>
                  onFormChange({ ...settingsForm, externalEditorPath: event.currentTarget.value })
                }
              />
              <button
                type="button"
                className="ghost-button"
                onClick={() => onBrowseFile("externalEditorPath")}
              >
                {t("settings.actions.browseFile")}
              </button>
            </div>
          </label>

          <label className="checkbox-group">
            <input
              type="checkbox"
              checked={settingsForm.showArchived}
              onChange={(event) =>
                onFormChange({ ...settingsForm, showArchived: event.currentTarget.checked })
              }
            />
            <span>{t("settings.fields.showArchived")}</span>
          </label>

          <div className="settings-actions">
            <button type="button" onClick={onSave}>
              {t("settings.actions.save")}
            </button>
            <button type="button" className="ghost-button" onClick={onDetectTerminal}>
              {t("settings.actions.detectTerminal")}
            </button>
            <button type="button" className="ghost-button" onClick={onDetectVscode}>
              {t("settings.actions.detectEditor")}
            </button>
          </div>
        </div>
      </article>
    </section>
  );
}
