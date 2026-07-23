import { useState } from "react";
import { useI18n } from "../i18n/I18nProvider";
import { useTheme } from "../theme/ThemeProvider";
import type {
  AppSettings,
  ProviderIntegrationState,
  ProviderIntegrationStatus,
} from "../types";
import { formatDateTime } from "../utils/formatDate";
import { ChevronRightIcon, DeleteIcon, EditNotesIcon, FolderIcon, MoonIcon, RefreshIcon, SunIcon } from "./Icons";
import { Button } from "./ui/Button";
import { IconButton } from "./ui/IconButton";
import { Select } from "./ui/Select";

type ProviderIntegrationAction = "install" | "update" | "recheck" | "uninstall";

/** quota provider 的固定顯示順序，個別平台監控與 Overlay 清單共用 */
const QUOTA_PROVIDER_ORDER = ["claude", "copilot", "codex", "opencode", "antigravity"] as const;

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
  providerAntigravityLabel: string,
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
    case "antigravity":
      return providerAntigravityLabel;
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
  const [expandedProviders, setExpandedProviders] = useState<Record<string, boolean>>({});
  const toggleProviderExpanded = (provider: string, currentlyExpanded: boolean) => {
    setExpandedProviders((prev) => ({ ...prev, [provider]: !currentlyExpanded }));
  };
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
                  <IconButton
                    label={t("settings.actions.browseDirectory")}
                    className="checkbox-group-edit"
                    onClick={() => onBrowseDirectory(field)}
                  >
                    <EditNotesIcon size={12} />
                  </IconButton>
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

          <label className="checkbox-group">
            <input
              type="checkbox"
              checked={settingsForm.launchOnStartup ?? false}
              onChange={(event) =>
                onFormChange({ ...settingsForm, launchOnStartup: event.currentTarget.checked })
              }
            />
            <span>
              {t("settings.fields.launchOnStartup")}
              <small className="settings-field-desc">{t("settings.fields.launchOnStartupDesc")}</small>
            </span>
          </label>

          <label className="checkbox-group">
            <input
              type="checkbox"
              checked={settingsForm.startMinimizedOnStartup ?? true}
              disabled={!(settingsForm.launchOnStartup ?? false)}
              onChange={(event) =>
                onFormChange({ ...settingsForm, startMinimizedOnStartup: event.currentTarget.checked })
              }
            />
            <span>
              {t("settings.fields.startMinimizedOnStartup")}
              <small className="settings-field-desc">{t("settings.fields.startMinimizedOnStartupDesc")}</small>
            </span>
          </label>

          <div className="settings-field">
            <label htmlFor="default-launcher-select">
              {t("settings.fields.defaultLauncher")}
              <small className="settings-field-desc">{t("settings.fields.defaultLauncherDesc")}</small>
            </label>
            <Select
              id="default-launcher-select"
              className="settings-select"
              value={settingsForm.defaultLauncher ?? "terminal"}
              onChange={(e) =>
                onFormChange({ ...settingsForm, defaultLauncher: e.currentTarget.value })
              }
            >
              <option value="terminal">Terminal</option>
              <option value="vscode">VS Code</option>
              <option value="explorer">Explorer</option>
              <option value="opencode">OpenCode</option>
              <option value="claude">Claude</option>
              <option value="codex">Codex</option>
              <option value="copilot">Copilot</option>
              <option value="gemini">Gemini</option>
            </Select>
          </div>

          <div className="settings-field">
            <label htmlFor="analytics-refresh-interval-select">
              {t("settings.fields.analyticsRefreshInterval")}
            </label>
            <Select
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
            </Select>
          </div>

          <div className="settings-field">
            <label htmlFor="language-select">{t("sidebar.language.label")}</label>
            <Select
              id="language-select"
              className="settings-select"
              value={locale}
              onChange={(e) => setLocale(e.currentTarget.value as "zh-TW" | "en-US")}
            >
              <option value="zh-TW">{t("sidebar.language.zhTW")}</option>
              <option value="en-US">{t("sidebar.language.enUS")}</option>
            </Select>
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
            <Button variant="primary" onClick={onSave}>
              {t("settings.actions.save")}
            </Button>
            <Button variant="secondary" onClick={onDetectTerminal}>
              {t("settings.actions.detectTerminal")}
            </Button>
            <Button variant="secondary" onClick={onDetectVscode}>
              {t("settings.actions.detectEditor")}
            </Button>
          </div>
        </div>
      </article>

      <div className="settings-layout-column">

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
                providerLabels.antigravity,
              );
              const providerBusy = pendingProviderAction?.startsWith(`${integration.provider}:`);
              const primaryAction = getProviderPrimaryAction(integration.status);
              const primaryActionLabel = getProviderPrimaryActionLabel(
                primaryAction,
                t("settings.integrations.actions.install"),
                t("settings.integrations.actions.update"),
              );
              const targetPath = getProviderTargetPath(integration);
              const isExpanded = expandedProviders[integration.provider] ?? Boolean(integration.lastError);
              const summaryTime = integration.lastEventAt
                ? formatDateTime(integration.lastEventAt, locale)
                : t("settings.integrations.values.noEvent");

              return (
                <article
                  key={integration.provider}
                  className={`provider-integration-card ${
                    integration.lastError ? "provider-integration-card--error" : ""
                  } ${isExpanded ? "provider-integration-card--expanded" : "provider-integration-card--collapsed"}`}
                >
                  <div
                    className="provider-integration-header"
                    onClick={() => toggleProviderExpanded(integration.provider, isExpanded)}
                    aria-expanded={isExpanded}
                    title={t(isExpanded ? "settings.integrations.actions.collapse" : "settings.integrations.actions.expand")}
                  >
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
                      {!isExpanded ? (
                        <span className="provider-integration-summary-time">{summaryTime}</span>
                      ) : null}
                    </div>

                    {isExpanded ? (
                      <div
                        className="provider-integration-actions"
                        onClick={(e) => e.stopPropagation()}
                      >
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
                          <IconButton
                            label={t("settings.integrations.actions.open")}
                            className="icon-button"
                          disabled={!targetPath || Boolean(providerBusy)}
                          onClick={() => onOpenProviderPath(integration)}
                          >
                            <FolderIcon size={14} />
                          </IconButton>
                          <IconButton
                            label={t("settings.integrations.actions.edit")}
                            className="icon-button"
                          disabled={!targetPath || Boolean(providerBusy)}
                          onClick={() => onEditProviderPath(integration)}
                          >
                            <EditNotesIcon size={14} />
                          </IconButton>
                        {integration.status === "installed" ? (
                          <IconButton
                            label={t("settings.integrations.actions.uninstall")}
                            className="icon-button"
                            danger
                            disabled={Boolean(providerBusy)}
                            onClick={() => onProviderAction(integration.provider, "uninstall")}
                          >
                            <DeleteIcon size={14} />
                          </IconButton>
                        ) : null}
                      </div>
                    ) : null}

                    <ChevronRightIcon className="provider-integration-chevron" size={14} />
                  </div>

                  {isExpanded ? (
                    <>
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
                          <p>{summaryTime}</p>
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
                    </>
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
              <IconButton label={t("quota.monitoring.manualRefresh")} onClick={() => onRefreshQuota?.()}>
                <RefreshIcon />
              </IconButton>
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
              <>
                <div className="settings-field">
                <label>{t("quota.monitoring.perProvider")}</label>
                <div className="quota-provider-toggle-list">
                  {QUOTA_PROVIDER_ORDER.map((provider) => {
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

              <div className="settings-section-divider" />

              <div className="settings-field settings-field--stacked">
                <label htmlFor="tray-quota-mode-select">{t("quota.settings.trayMode")}</label>
                <Select
                  id="tray-quota-mode-select"
                  className="settings-select"
                  value={settingsForm.trayQuotaMode ?? "icon_only"}
                  onChange={(event) =>
                    onFormChange({
                      ...settingsForm,
                      trayQuotaMode: event.currentTarget.value as AppSettings["trayQuotaMode"],
                    })
                  }
                >
                  <option value="icon_only">{t("quota.settings.trayMode.iconOnly")}</option>
                  <option value="percentage">{t("quota.settings.trayMode.percentage")}</option>
                  <option value="bar">{t("quota.settings.trayMode.bar")}</option>
                  <option value="hidden">{t("quota.settings.trayMode.hidden")}</option>
                </Select>
              </div>

              <div className="settings-field settings-field--stacked">
                <label htmlFor="tray-quota-primary-provider-select">{t("quota.settings.primaryProvider")}</label>
                <Select
                  id="tray-quota-primary-provider-select"
                  className="settings-select"
                  value={settingsForm.trayQuotaPrimaryProvider ?? ""}
                  onChange={(event) =>
                    onFormChange({
                      ...settingsForm,
                      trayQuotaPrimaryProvider: event.currentTarget.value || null,
                    })
                  }
                >
                  <option value="">{t("quota.settings.primaryProvider.auto")}</option>
                  {QUOTA_PROVIDER_ORDER.filter((provider) =>
                    (settingsForm.quotaEnabledProviders ?? []).includes(provider),
                  ).map((provider) => (
                    <option key={provider} value={provider}>
                      {providerLabels[provider]}
                    </option>
                  ))}
                </Select>
              </div>

              <label className="checkbox-group">
                <input
                  type="checkbox"
                  checked={settingsForm.trayQuotaPanelEnabled ?? true}
                  onChange={(event) =>
                    onFormChange({ ...settingsForm, trayQuotaPanelEnabled: event.currentTarget.checked })
                  }
                />
                <span>{t("quota.settings.trayPanelEnabled")}</span>
              </label>

              <label className="checkbox-group">
                <input
                  type="checkbox"
                  checked={settingsForm.quotaOverlayEnabled ?? false}
                  onChange={(event) =>
                    onFormChange({ ...settingsForm, quotaOverlayEnabled: event.currentTarget.checked })
                  }
                />
                <span>
                  {t("quota.settings.overlayEnabled")}
                  <small className="settings-field-desc">{t("quota.settings.overlayEnabledDesc")}</small>
                </span>
              </label>

              {(settingsForm.quotaOverlayEnabled ?? false) ? (
                <>
                  <label className="checkbox-group">
                    <input
                      type="checkbox"
                      checked={settingsForm.quotaOverlayLocked ?? true}
                      onChange={(event) =>
                        onFormChange({ ...settingsForm, quotaOverlayLocked: event.currentTarget.checked })
                      }
                    />
                    <span>{t("quota.settings.overlayLocked")}</span>
                  </label>

                  <div className="settings-field settings-field--stacked">
                    <label htmlFor="quota-overlay-opacity-range">{t("quota.settings.overlayOpacity")}</label>
                    <div className="settings-range-row">
                      <input
                        id="quota-overlay-opacity-range"
                        type="range"
                        min="0.3"
                        max="1"
                        step="0.05"
                        value={settingsForm.quotaOverlayOpacity ?? 0.3}
                        onChange={(event) =>
                          onFormChange({
                            ...settingsForm,
                            quotaOverlayOpacity: Number(event.currentTarget.value),
                          })
                        }
                      />
                      <span className="settings-range-value">{Math.round((settingsForm.quotaOverlayOpacity ?? 0.3) * 100)}%</span>
                    </div>
                  </div>

                  <div className="settings-field settings-field--stacked">
                    <label htmlFor="quota-overlay-theme-select">{t("quota.settings.overlayTheme")}</label>
                    <Select
                      id="quota-overlay-theme-select"
                      className="settings-select"
                      value={settingsForm.quotaOverlayTheme ?? "dark"}
                      onChange={(event) =>
                        onFormChange({
                          ...settingsForm,
                          quotaOverlayTheme: event.currentTarget.value as AppSettings["quotaOverlayTheme"],
                        })
                      }
                    >
                      <option value="dark">{t("quota.settings.overlayTheme.dark")}</option>
                      <option value="light">{t("quota.settings.overlayTheme.light")}</option>
                    </Select>
                  </div>

                  <div className="settings-field settings-field--stacked">
                    <label htmlFor="quota-overlay-style-select">{t("quota.settings.overlayStyle")}</label>
                    <Select
                      id="quota-overlay-style-select"
                      className="settings-select"
                      value={settingsForm.quotaOverlayStyle ?? "compact"}
                      onChange={(event) =>
                        onFormChange({
                          ...settingsForm,
                          quotaOverlayStyle: event.currentTarget.value as AppSettings["quotaOverlayStyle"],
                        })
                      }
                    >
                      <option value="full">{t("quota.settings.overlayStyle.full")}</option>
                      <option value="compact">{t("quota.settings.overlayStyle.compact")}</option>
                    </Select>
                  </div>

                  <div className="settings-field settings-field--stacked">
                    <label>{t("quota.settings.overlayProviders")}</label>
                    <div className="quota-provider-toggle-list">
                      {QUOTA_PROVIDER_ORDER.map((provider) => {
                        const monitored = (settingsForm.quotaEnabledProviders ?? []).includes(provider);
                        const checked = (settingsForm.quotaOverlayProviders ?? []).includes(provider);
                        return (
                          <label
                            key={provider}
                            className={`checkbox-group checkbox-group--inline${monitored ? "" : " checkbox-group--disabled"}`}
                          >
                            <input
                              type="checkbox"
                              checked={checked}
                              disabled={!monitored}
                              onChange={(event) => {
                                const current = settingsForm.quotaOverlayProviders ?? [];
                                const next = event.currentTarget.checked
                                  ? [...current, provider]
                                  : current.filter((item) => item !== provider);
                                onFormChange({ ...settingsForm, quotaOverlayProviders: next });
                              }}
                            />
                            <span>{providerLabels[provider]}</span>
                          </label>
                        );
                      })}
                    </div>
                  </div>
                </>
              ) : null}
              </>
            ) : null}
          </div>
        </article>
      ) : null}

      </div>
    </section>
  );
}
