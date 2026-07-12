import { useI18n } from "../i18n/I18nProvider";
import { useTheme } from "../theme/ThemeProvider";
import type {
  AppSettings,
  ProviderIntegrationState,
  ProviderIntegrationStatus,
} from "../types";
import { formatDateTime } from "../utils/formatDate";
import { MoonIcon, SunIcon } from "./Icons";

type ProviderIntegrationAction = "install" | "update" | "recheck" | "uninstall";

type Props = {
  settingsForm: AppSettings;
  onFormChange: (next: AppSettings) => void;
  onSave: () => void;
  onBrowseDirectory: (field: "copilotRoot" | "opencodeRoot" | "codexRoot" | "claudeRoot" | "antigravityRoot" | "agentsSourceRoot") => void;
  onBrowseFile: (field: "terminalPath" | "externalEditorPath") => void;
  onDetectTerminal: () => void;
  onDetectVscode: () => void;
  onProviderAction: (provider: string, action: ProviderIntegrationAction) => void;
  onOpenProviderPath: (integration: ProviderIntegrationStatus) => void;
  onEditProviderPath: (integration: ProviderIntegrationStatus) => void;
  pendingProviderAction: string | null;
  onOpenEventMonitor: () => void;
  jqAvailable?: boolean | null;
  onRefreshQuota?: (provider?: string) => void;
};

function getProviderLabel(
  provider: string,
  providerCopilotLabel: string,
  providerOpencodeLabel: string,
  providerCodexLabel: string,
  providerClaudeLabel: string,
): string {
  switch (provider) {
    case "copilot":
      return providerCopilotLabel;
    case "opencode":
      return providerOpencodeLabel;
    case "codex":
      return providerCodexLabel;
    case "claude":
      return providerClaudeLabel;
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
  const providerOrder = ["copilot", "opencode", "codex"];

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
  jqAvailable,
  onRefreshQuota,
}: Props) {
  const { t, locale, setLocale } = useI18n();
  const { theme, setTheme } = useTheme();
  const providerIntegrations = sortProviderIntegrations(settingsForm.providerIntegrations ?? []);
  const providerLabels = {
    copilot: t("settings.fields.providerCopilot"),
    opencode: t("settings.fields.providerOpencode"),
    codex: t("settings.fields.providerCodex"),
    claude: t("settings.fields.providerClaude"),
    antigravity: t("settings.fields.providerAntigravity"),
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
          <div className="field-group">
            <span>{t("settings.fields.enabledProviders")}</span>
            <div className="checkbox-list">
              {(
                [
                  { id: "copilot", labelKey: "settings.fields.providerCopilot", field: "copilotRoot", path: settingsForm.copilotRoot },
                  { id: "opencode", labelKey: "settings.fields.providerOpencode", field: "opencodeRoot", path: settingsForm.opencodeRoot },
                  { id: "codex", labelKey: "settings.fields.providerCodex", field: "codexRoot", path: settingsForm.codexRoot },
                  { id: "claude", labelKey: "settings.fields.providerClaude", field: "claudeRoot", path: settingsForm.claudeRoot ?? "" },
                  { id: "antigravity", labelKey: "settings.fields.providerAntigravity", field: "antigravityRoot", path: settingsForm.antigravityRoot ?? "" },
                ] as const
              ).map(({ id, labelKey, field, path }) => (
                <div key={id} className="checkbox-group">
                  <label className="checkbox-group-label">
                    <input
                      type="checkbox"
                      checked={settingsForm.enabledProviders.includes(id)}
                      onChange={(event) => {
                        const next = event.currentTarget.checked
                          ? [...settingsForm.enabledProviders, id]
                          : settingsForm.enabledProviders.filter((p) => p !== id);
                        onFormChange({ ...settingsForm, enabledProviders: next });
                        if (event.currentTarget.checked) {
                          onProviderAction(id, "install");
                        }
                      }}
                    />
                    <span>{t(labelKey)}</span>
                  </label>
                  <span className="checkbox-group-path" title={path}>{path}</span>
                  <button
                    type="button"
                    className="checkbox-group-edit"
                    onClick={() => onBrowseDirectory(field)}
                    title={t("settings.actions.browseDirectory")}
                  >
                    <svg width="12" height="12" viewBox="0 0 16 16" fill="currentColor">
                      <path d="M11.013 1.427a1.75 1.75 0 0 1 2.474 0l1.086 1.086a1.75 1.75 0 0 1 0 2.474l-8.61 8.61c-.21.21-.47.364-.756.445l-3.251.93a.75.75 0 0 1-.927-.928l.929-3.25c.081-.286.235-.547.445-.758l8.61-8.61zm1.414 1.06a.25.25 0 0 0-.354 0L10.811 3.75l1.439 1.44 1.263-1.263a.25.25 0 0 0 0-.354l-1.086-1.086zM11.189 6.25 9.75 4.81l-6.286 6.287a.25.25 0 0 0-.064.108l-.558 1.953 1.953-.558a.25.25 0 0 0 .108-.064z" />
                    </svg>
                  </button>
                </div>
              ))}
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

          <label className="checkbox-group">
            <input
              type="checkbox"
              checked={settingsForm.minimizeToTray ?? false}
              onChange={(event) =>
                onFormChange({ ...settingsForm, minimizeToTray: event.currentTarget.checked })
              }
            />
            <span>
              {t("settings.fields.minimizeToTray")}
              <small className="settings-field-desc">{t("settings.fields.minimizeToTrayDesc")}</small>
            </span>
          </label>

          <div className="settings-field">
            <label htmlFor="default-launcher-select">
              {t("settings.fields.defaultLauncher")}
              <small className="settings-field-desc">{t("settings.fields.defaultLauncherDesc")}</small>
            </label>
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

          <div className="settings-section-divider" />

          <div className="settings-field settings-field--stacked">
            <label>{t("sidebar.iconStyle.label")}</label>
            <div className="theme-toggle-row theme-toggle-row--settings">
              <span className={`theme-toggle-icon ${theme === "light" ? "active" : ""}`}><SunIcon size={15} /></span>
              <button
                type="button"
                role="switch"
                aria-checked={theme === "dark"}
                className={`theme-toggle-switch ${theme === "dark" ? "dark" : ""}`}
                title={theme === "light" ? t("sidebar.theme.dark") : t("sidebar.theme.light")}
                onClick={() => setTheme(theme === "light" ? "dark" : "light")}
              >
                <span className="theme-toggle-thumb" />
              </button>
              <span className={`theme-toggle-icon ${theme === "dark" ? "active" : ""}`}><MoonIcon size={15} /></span>
              <span className="theme-toggle-label">
                {theme === "light" ? t("sidebar.theme.light") : t("sidebar.theme.dark")}
              </span>
            </div>
          </div>

          <div className="settings-field settings-field--stacked">
            <label>{t("settings.agents.title")}</label>
            <p className="settings-field-desc settings-field-desc--block">{t("settings.agents.subtitle")}</p>

            <label className="field-group">
              <span>{t("settings.fields.agentsSourceRoot")}</span>
              <p className="settings-field-desc settings-field-desc--block">{t("settings.fields.agentsSourceRootDesc")}</p>
              <div className="field-with-action">
                <input
                  value={settingsForm.agentsSourceRoot ?? ""}
                  placeholder={t("settings.fields.agentsSourceRootPlaceholder")}
                  onChange={(event) =>
                    onFormChange({ ...settingsForm, agentsSourceRoot: event.currentTarget.value })
                  }
                />
                <button
                  type="button"
                  className="ghost-button"
                  onClick={() => onBrowseDirectory("agentsSourceRoot")}
                >
                  {t("settings.actions.browseDirectory")}
                </button>
              </div>
            </label>

            <label className="checkbox-group">
              <input
                type="checkbox"
                checked={settingsForm.allowCreateProjectConfigDir ?? false}
                onChange={(event) =>
                  onFormChange({ ...settingsForm, allowCreateProjectConfigDir: event.currentTarget.checked })
                }
              />
              <span>
                {t("settings.agents.allowCreateProjectConfigDir")}
                <small className="settings-field-desc">{t("settings.agents.allowCreateProjectConfigDirDesc")}</small>
              </span>
            </label>
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

        {jqAvailable === false && providerIntegrations.some((i) => i.provider === "claude") ? (
          <div className="jq-missing-banner">
            <strong>{t("settings.jqNotFound.title")}</strong>
            <span>{t("settings.jqNotFound.body")}</span>
            <code>{t("settings.jqNotFound.winget")}</code>
          </div>
        ) : null}

        {providerIntegrations.length === 0 ? (
          <div className="provider-integration-empty">{t("settings.integrations.empty")}</div>
        ) : (
          <div className="provider-integrations-list">
            {providerIntegrations.map((integration) => {
              const providerLabel = getProviderLabel(
                integration.provider,
                providerLabels.copilot,
                providerLabels.opencode,
                providerLabels.codex,
                providerLabels.claude,
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
                        className="icon-button"
                        disabled={!targetPath || Boolean(providerBusy)}
                        onClick={() => onOpenProviderPath(integration)}
                        title={t("settings.integrations.actions.open")}
                      >
                        <svg width="14" height="14" viewBox="0 0 16 16" fill="currentColor">
                          <path d="M1.75 1A1.75 1.75 0 0 0 0 2.75v10.5C0 14.216.784 15 1.75 15h12.5A1.75 1.75 0 0 0 16 13.25v-8.5A1.75 1.75 0 0 0 14.25 3H7.5a.25.25 0 0 1-.2-.1l-.9-1.2C6.07 1.26 5.55 1 5 1H1.75z"/>
                        </svg>
                      </button>
                      <button
                        type="button"
                        className="icon-button"
                        disabled={!targetPath || Boolean(providerBusy)}
                        onClick={() => onEditProviderPath(integration)}
                        title={t("settings.integrations.actions.edit")}
                      >
                        <svg width="14" height="14" viewBox="0 0 16 16" fill="currentColor">
                          <path d="M11.013 1.427a1.75 1.75 0 0 1 2.474 0l1.086 1.086a1.75 1.75 0 0 1 0 2.474l-8.61 8.61c-.21.21-.47.364-.756.445l-3.251.93a.75.75 0 0 1-.927-.928l.929-3.25c.081-.286.235-.547.445-.758l8.61-8.61zm1.414 1.06a.25.25 0 0 0-.354 0L10.811 3.75l1.439 1.44 1.263-1.263a.25.25 0 0 0 0-.354l-1.086-1.086zM11.189 6.25 9.75 4.81l-6.286 6.287a.25.25 0 0 0-.064.108l-.558 1.953 1.953-.558a.25.25 0 0 0 .108-.064z"/>
                        </svg>
                      </button>
                      {integration.status === "installed" ? (
                        <button
                          type="button"
                          className="icon-button icon-button--danger"
                          disabled={Boolean(providerBusy)}
                          onClick={() => onProviderAction(integration.provider, "uninstall")}
                          title={t("settings.integrations.actions.uninstall")}
                        >
                          <svg width="14" height="14" viewBox="0 0 16 16" fill="currentColor">
                            <path d="M11 1.75V3h2.25a.75.75 0 0 1 0 1.5H2.75a.75.75 0 0 1 0-1.5H5V1.75C5 .784 5.784 0 6.75 0h2.5C10.216 0 11 .784 11 1.75ZM4.496 6.675l.66 6.6a.25.25 0 0 0 .249.225h5.19a.25.25 0 0 0 .249-.225l.66-6.6a.75.75 0 0 1 1.492.149l-.66 6.6A1.748 1.748 0 0 1 10.595 15h-5.19a1.75 1.75 0 0 1-1.741-1.576l-.66-6.6a.75.75 0 1 1 1.492-.149ZM6.5 1.75V3h3V1.75a.25.25 0 0 0-.25-.25h-2.5a.25.25 0 0 0-.25.25Z"/>
                          </svg>
                        </button>
                      ) : null}
                    </div>
                  </div>

                  <div className="provider-integration-grid">
                    {(
                      [
                        {
                          label: t(
                            integration.provider === "claude"
                              ? "settings.integrations.fields.hookPath"
                              : "settings.integrations.fields.configPath",
                          ),
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

      {(settingsForm.enableQuotaMonitoring ?? true) || true ? (
        <article className="info-card">
          <div className="section-heading">
            <h3>{t("quota.monitoring.title")}</h3>
            {(settingsForm.enableQuotaMonitoring ?? true) ? (
              <button type="button" className="ghost-button" onClick={() => onRefreshQuota?.()}>
                {t("quota.monitoring.manualRefresh")}
              </button>
            ) : null}
          </div>

          <div className="settings-form">
            <label className="checkbox-group">
              <input
                type="checkbox"
                checked={settingsForm.enableQuotaMonitoring ?? true}
                onChange={(event) =>
                  onFormChange({ ...settingsForm, enableQuotaMonitoring: event.currentTarget.checked })
                }
              />
              <span>
                {t("quota.monitoring.enable")}
                <small className="settings-field-desc">{t("quota.monitoring.enableDesc")}</small>
              </span>
            </label>

            {(settingsForm.enableQuotaMonitoring ?? true) ? (
              <div className="settings-field">
                <label>{t("quota.monitoring.perProvider")}</label>
                <div className="quota-provider-toggle-list">
                  {(["claude", "copilot", "codex", "opencode", "antigravity"] as const).map((provider) => {
                    const enabledProviders =
                      settingsForm.quotaEnabledProviders ?? ["claude", "copilot", "opencode", "codex", "antigravity"];
                    const checked = enabledProviders.includes(provider);
                    return (
                      <label key={provider} className="checkbox-group checkbox-group--inline">
                        <input
                          type="checkbox"
                          checked={checked}
                          onChange={(event) => {
                            const next = event.currentTarget.checked
                              ? [...enabledProviders, provider]
                              : enabledProviders.filter((p) => p !== provider);
                            onFormChange({ ...settingsForm, quotaEnabledProviders: next });
                          }}
                        />
                        <span>{providerLabels[provider]}</span>
                      </label>
                    );
                  })}
                </div>
              </div>
            ) : null}
          </div>
        </article>
      ) : null}
    </section>
  );
}
