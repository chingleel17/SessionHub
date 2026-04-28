import { useI18n } from "../i18n/I18nProvider";
import type {
  AppSettings,
  ProviderIntegrationState,
  ProviderIntegrationStatus,
} from "../types";
import { formatDateTime } from "../utils/formatDate";

type ProviderIntegrationAction = "install" | "update" | "recheck";

type Props = {
  settingsForm: AppSettings;
  onFormChange: (next: AppSettings) => void;
  onSave: () => void;
  onBrowseDirectory: (field: "copilotRoot" | "opencodeRoot") => void;
  onBrowseFile: (field: "terminalPath" | "externalEditorPath") => void;
  onDetectTerminal: () => void;
  onDetectVscode: () => void;
  onProviderAction: (provider: string, action: ProviderIntegrationAction) => void;
  onOpenProviderPath: (integration: ProviderIntegrationStatus) => void;
  onEditProviderPath: (integration: ProviderIntegrationStatus) => void;
  pendingProviderAction: string | null;
  onOpenEventMonitor: () => void;
};

function getProviderLabel(
  provider: string,
  providerCopilotLabel: string,
  providerOpencodeLabel: string,
): string {
  switch (provider) {
    case "copilot":
      return providerCopilotLabel;
    case "opencode":
      return providerOpencodeLabel;
    default:
      return provider;
  }
}

function getProviderStatusLabel(
  status: ProviderIntegrationState,
  labels: Record<ProviderIntegrationState, string>,
): string {
  return labels[status];
}

function getProviderStatusChipClass(status: ProviderIntegrationState): string {
  switch (status) {
    case "installed":
      return "provider-status-chip provider-status-chip--installed";
    case "outdated":
      return "provider-status-chip provider-status-chip--outdated";
    case "missing":
      return "provider-status-chip provider-status-chip--missing";
    case "manual_required":
      return "provider-status-chip provider-status-chip--manual";
    case "error":
      return "provider-status-chip provider-status-chip--error";
    default:
      return "";
  }
}

function getProviderPrimaryAction(
  status: ProviderIntegrationState,
): ProviderIntegrationAction | null {
  switch (status) {
    case "outdated":
      return "update";
    case "missing":
    case "manual_required":
    case "error":
      return "install";
    case "installed":
    default:
      return null;
  }
}

function getProviderPrimaryActionLabel(
  action: ProviderIntegrationAction | null,
  installLabel: string,
  updateLabel: string,
): string | null {
  if (action === "install") return installLabel;
  if (action === "update") return updateLabel;
  return null;
}

function getProviderTargetPath(integration: ProviderIntegrationStatus): string | null {
  const configPath = integration.configPath?.trim();
  if (configPath) return configPath;
  const bridgePath = integration.bridgePath?.trim();
  return bridgePath || null;
}

function sortProviderIntegrations(
  integrations: ProviderIntegrationStatus[],
): ProviderIntegrationStatus[] {
  const providerOrder = ["copilot", "opencode"];

  return [...integrations].sort((left, right) => {
    const leftIndex = providerOrder.indexOf(left.provider);
    const rightIndex = providerOrder.indexOf(right.provider);
    const normalizedLeft = leftIndex === -1 ? Number.MAX_SAFE_INTEGER : leftIndex;
    const normalizedRight = rightIndex === -1 ? Number.MAX_SAFE_INTEGER : rightIndex;
    return normalizedLeft - normalizedRight || left.provider.localeCompare(right.provider);
  });
}

export function SettingsView({
  settingsForm,
  onFormChange,
  onSave,
  onBrowseDirectory,
  onBrowseFile,
  onDetectTerminal,
  onDetectVscode,
  onProviderAction,
  onOpenProviderPath,
  onEditProviderPath,
  pendingProviderAction,
  onOpenEventMonitor,
}: Props) {
  const { t, locale, setLocale } = useI18n();
  const providerIntegrations = sortProviderIntegrations(settingsForm.providerIntegrations ?? []);
  const providerLabels = {
    copilot: t("settings.fields.providerCopilot"),
    opencode: t("settings.fields.providerOpencode"),
  };
  const statusLabels: Record<ProviderIntegrationState, string> = {
    installed: t("settings.integrations.status.installed"),
    outdated: t("settings.integrations.status.outdated"),
    missing: t("settings.integrations.status.missing"),
    manual_required: t("settings.integrations.status.manual_required"),
    error: t("settings.integrations.status.error"),
  };

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
            <span>{t("settings.fields.opencodeRoot")}</span>
            <div className="field-with-action">
              <input
                value={settingsForm.opencodeRoot}
                onChange={(event) =>
                  onFormChange({ ...settingsForm, opencodeRoot: event.currentTarget.value })
                }
              />
              <button
                type="button"
                className="ghost-button"
                onClick={() => onBrowseDirectory("opencodeRoot")}
              >
                {t("settings.actions.browseDirectory")}
              </button>
            </div>
          </label>

          <div className="field-group">
            <span>{t("settings.fields.enabledProviders")}</span>
            <div className="checkbox-list">
              <label className="checkbox-group">
                <input
                  type="checkbox"
                  checked={settingsForm.enabledProviders.includes("copilot")}
                  onChange={(event) => {
                    const next = event.currentTarget.checked
                      ? [...settingsForm.enabledProviders, "copilot"]
                      : settingsForm.enabledProviders.filter((p) => p !== "copilot");
                    onFormChange({ ...settingsForm, enabledProviders: next });
                  }}
                />
                <span>{t("settings.fields.providerCopilot")}</span>
              </label>
              <label className="checkbox-group">
                <input
                  type="checkbox"
                  checked={settingsForm.enabledProviders.includes("opencode")}
                  onChange={(event) => {
                    const next = event.currentTarget.checked
                      ? [...settingsForm.enabledProviders, "opencode"]
                      : settingsForm.enabledProviders.filter((p) => p !== "opencode");
                    onFormChange({ ...settingsForm, enabledProviders: next });
                  }}
                />
                <span>{t("settings.fields.providerOpencode")}</span>
              </label>
            </div>
          </div>

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

          <label className="checkbox-group">
            <input
              type="checkbox"
              checked={settingsForm.enableInterventionNotification ?? true}
              onChange={(event) =>
                onFormChange({ ...settingsForm, enableInterventionNotification: event.currentTarget.checked })
              }
            />
            <span>
              {t("settings.fields.enableInterventionNotification")}
              <small className="settings-field-desc">{t("settings.fields.enableInterventionNotificationDesc")}</small>
            </span>
          </label>

          <label className="checkbox-group">
            <input
              type="checkbox"
              checked={settingsForm.enableSessionEndNotification ?? false}
              onChange={(event) =>
                onFormChange({ ...settingsForm, enableSessionEndNotification: event.currentTarget.checked })
              }
            />
            <span>
              {t("settings.fields.enableSessionEndNotification")}
              <small className="settings-field-desc">{t("settings.fields.enableSessionEndNotificationDesc")}</small>
            </span>
          </label>

          <label className="checkbox-group">
            <input
              type="checkbox"
              checked={settingsForm.showStatusBar ?? true}
              onChange={(event) =>
                onFormChange({ ...settingsForm, showStatusBar: event.currentTarget.checked })
              }
            />
            <span>{t("statusBar.showStatusBar")}</span>
          </label>

          <div className="settings-field">
            <label htmlFor="default-launcher-select">{t("settings.fields.defaultLauncher")}</label>
            <select
              id="default-launcher-select"
              className="settings-select"
              value={settingsForm.defaultLauncher ?? "terminal"}
              onChange={(e) =>
                onFormChange({ ...settingsForm, defaultLauncher: e.currentTarget.value })
              }
            >
              <option value="terminal">Terminal</option>
              <option value="copilot">Copilot</option>
              <option value="opencode">OpenCode</option>
              <option value="gemini">Gemini</option>
              <option value="vscode">VS Code</option>
              <option value="explorer">Explorer</option>
            </select>
          </div>

          <div className="settings-field">
            <label htmlFor="analytics-refresh-interval-select">
              {t("settings.fields.analyticsRefreshInterval")}
            </label>
            <select
              id="analytics-refresh-interval-select"
              className="settings-select"
              value={settingsForm.analyticsRefreshInterval ?? 30}
              onChange={(event) =>
                onFormChange({
                  ...settingsForm,
                  analyticsRefreshInterval: Number(event.currentTarget.value) as 10 | 30,
                })
              }
            >
              <option value="10">{t("settings.fields.analyticsRefreshInterval.10")}</option>
              <option value="30">{t("settings.fields.analyticsRefreshInterval.30")}</option>
            </select>
          </div>

          <div className="settings-field">
            <label htmlFor="language-select">{t("sidebar.language.label")}</label>
            <select
              id="language-select"
              className="settings-select"
              value={locale}
              onChange={(e) => setLocale(e.currentTarget.value as "zh-TW" | "en-US")}
            >
              <option value="zh-TW">{t("sidebar.language.zhTW")}</option>
              <option value="en-US">{t("sidebar.language.enUS")}</option>
            </select>
          </div>

          <div className="settings-actions">
            <button type="button" className="primary-button" onClick={onSave}>
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

      <article className="info-card">
        <div className="section-heading">
          <h3>{t("settings.integrations.title")}</h3>
          <div style={{ display: "flex", alignItems: "center", gap: 12 }}>
            <span>{t("settings.integrations.subtitle")}</span>
            <button
              type="button"
              className="ghost-button"
              onClick={onOpenEventMonitor}
            >
              {t("eventMonitor.openButton")}
            </button>
          </div>
        </div>

        {providerIntegrations.length === 0 ? (
          <div className="provider-integration-empty">{t("settings.integrations.empty")}</div>
        ) : (
          <div className="provider-integrations-list">
            {providerIntegrations.map((integration) => {
              const providerLabel = getProviderLabel(
                integration.provider,
                providerLabels.copilot,
                providerLabels.opencode,
              );
              const providerBusy = pendingProviderAction?.startsWith(`${integration.provider}:`);
              const primaryAction = getProviderPrimaryAction(integration.status);
              const primaryActionLabel = getProviderPrimaryActionLabel(
                primaryAction,
                t("settings.integrations.actions.install"),
                t("settings.integrations.actions.update"),
              );
              const targetPath = getProviderTargetPath(integration);

              return (
                <article
                  key={integration.provider}
                  className={`provider-integration-card ${
                    integration.lastError ? "provider-integration-card--error" : ""
                  }`}
                >
                  <div className="provider-integration-header">
                    <div className="provider-integration-badges">
                      <span className={`provider-tag provider-tag--${integration.provider}`}>
                        {providerLabel}
                      </span>
                      <span
                        className={`session-chip ${getProviderStatusChipClass(integration.status)}`}
                      >
                        {getProviderStatusLabel(integration.status, statusLabels)}
                      </span>
                      {integration.installedVersion != null ? (
                        <span className="provider-version-badge">
                          v{integration.installedVersion}
                        </span>
                      ) : null}
                    </div>

                    <div className="provider-integration-actions">
                      {primaryAction && primaryActionLabel ? (
                        <button
                          type="button"
                          className="primary-button"
                          disabled={Boolean(providerBusy)}
                          onClick={() => onProviderAction(integration.provider, primaryAction)}
                        >
                          {primaryActionLabel}
                        </button>
                      ) : null}
                      <button
                        type="button"
                        className="ghost-button"
                        disabled={Boolean(providerBusy)}
                        onClick={() => onProviderAction(integration.provider, "recheck")}
                      >
                        {t("settings.integrations.actions.recheck")}
                      </button>
                      <button
                        type="button"
                        className="ghost-button"
                        disabled={!targetPath || Boolean(providerBusy)}
                        onClick={() => onOpenProviderPath(integration)}
                      >
                        {t("settings.integrations.actions.open")}
                      </button>
                      <button
                        type="button"
                        className="ghost-button"
                        disabled={!targetPath || Boolean(providerBusy)}
                        onClick={() => onEditProviderPath(integration)}
                      >
                        {t("settings.integrations.actions.edit")}
                      </button>
                    </div>
                  </div>

                  <div className="provider-integration-grid">
                    {(
                      [
                        {
                          label: t("settings.integrations.fields.configPath"),
                          value: integration.configPath?.trim() || null,
                        },
                        {
                          label: t("settings.integrations.fields.bridgePath"),
                          value: integration.bridgePath?.trim() || null,
                        },
                      ] as { label: string; value: string | null }[]
                    ).map(({ label, value }) => (
                      <div key={label} className="provider-integration-meta">
                        <details>
                          <summary className="provider-path-summary">
                            <span className="provider-path-label">{label}</span>
                            <button
                              type="button"
                              className="provider-path-copy"
                              disabled={!value}
                              onClick={(e) => {
                                e.preventDefault();
                                if (value) navigator.clipboard.writeText(value);
                              }}
                            >
                              {t("settings.integrations.actions.copy")}
                            </button>
                          </summary>
                          <code title={value ?? undefined}>
                            {value ?? t("settings.integrations.values.unavailable")}
                          </code>
                        </details>
                      </div>
                    ))}

                    <div className="provider-integration-meta">
                      <span>{t("settings.integrations.fields.lastEventAt")}</span>
                      <p>
                        {integration.lastEventAt
                          ? formatDateTime(integration.lastEventAt, locale)
                          : t("settings.integrations.values.noEvent")}
                      </p>
                    </div>

                    {integration.installedVersion != null ? (
                      <div className="provider-integration-meta">
                        <span>{t("settings.integrations.fields.version")}</span>
                        <p>v{integration.installedVersion}</p>
                      </div>
                    ) : null}
                  </div>

                  {integration.lastError ? (
                    <div className="provider-integration-error">
                      <span>{t("settings.integrations.fields.lastError")}</span>
                      <p>{integration.lastError}</p>
                    </div>
                  ) : null}
                </article>
              );
            })}
          </div>
        )}
      </article>
    </section>
  );
}
