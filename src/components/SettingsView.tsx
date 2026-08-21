import { useState } from "react";
import { useI18n } from "../i18n/I18nProvider";
import { useTheme } from "../theme/ThemeProvider";
import type {
  AppSettings,
  ProviderIntegrationState,
  ProviderIntegrationStatus,
  ToolAvailability,
} from "../types";
import { formatDateTime } from "../utils/formatDate";
import { compareProviders, PROVIDER_DISPLAY_ORDER } from "../utils/providerOrder";
import { CheckIcon, ChevronRightIcon, DeleteIcon, EditNotesIcon, FolderIcon, MoonIcon, RefreshIcon, SunIcon } from "./Icons";
import { Button } from "./ui/Button";
import { IconButton } from "./ui/IconButton";
import { Select } from "./ui/Select";

type ProviderIntegrationAction = "install" | "update" | "recheck" | "uninstall";
type ProviderRootField = "copilotRoot" | "opencodeRoot" | "codexRoot" | "claudeRoot" | "antigravityRoot";

type Props = {
  settingsForm: AppSettings;
  onFormChange: (next: AppSettings) => void;
  onSave: () => void;
  onBrowseDirectory: (field: ProviderRootField | "agentsSourceRoot") => void;
  onBrowseFile: (field: "terminalPath" | "externalEditorPath") => void;
  onDetectTerminal: () => void;
  onDetectVscode: () => void;
  onDetectTools: () => void;
  toolAvailability: ToolAvailability | null;
  providerDirectoryExists: Record<string, boolean | undefined>;
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

function getProviderRootField(provider: string): ProviderRootField | null {
  switch (provider) {
    case "copilot":
      return "copilotRoot";
    case "opencode":
      return "opencodeRoot";
    case "codex":
      return "codexRoot";
    case "claude":
      return "claudeRoot";
    case "antigravity":
      return "antigravityRoot";
    default:
      return null;
  }
}

function getProviderRootPath(settings: AppSettings, provider: string): string {
  switch (provider) {
    case "copilot":
      return settings.copilotRoot;
    case "opencode":
      return settings.opencodeRoot;
    case "codex":
      return settings.codexRoot;
    case "claude":
      return settings.claudeRoot ?? "";
    case "antigravity":
      return settings.antigravityRoot ?? "";
    default:
      return "";
  }
}

function sortProviderIntegrations(
  integrations: ProviderIntegrationStatus[],
): ProviderIntegrationStatus[] {
  return [...integrations].sort((left, right) => compareProviders(left.provider, right.provider));
}

export function SettingsView({
  settingsForm,
  onFormChange,
  onSave,
  onBrowseDirectory,
  onBrowseFile,
  onDetectTerminal,
  onDetectVscode,
  onDetectTools,
  toolAvailability,
  providerDirectoryExists,
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

          <div className="settings-field">
            <label htmlFor="terminal-launcher-select">
              {t("settings.fields.terminalLauncher")}
              <small className="settings-field-desc">{t("settings.fields.terminalLauncherDesc")}</small>
            </label>
            <div className="field-with-action">
              <Select
                id="terminal-launcher-select"
                className="settings-select"
                value={settingsForm.terminalLauncher ?? "shell"}
                onChange={(event) =>
                  onFormChange({ ...settingsForm, terminalLauncher: event.currentTarget.value })
                }
              >
                <option value="shell">{t("settings.launcher.shell")}</option>
                <option
                  value="herdr"
                  disabled={toolAvailability != null && !toolAvailability.herdr && settingsForm.terminalLauncher !== "herdr"}
                >
                  {t(
                    toolAvailability?.herdr
                      ? "settings.launcher.herdr"
                      : "settings.launcher.herdrMissing",
                  )}
                </option>
              </Select>
              <button type="button" className="ghost-button" onClick={onDetectTools}>
                {t("settings.actions.detectTools")}
              </button>
            </div>
          </div>

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
              <option value="terminal">{t("settings.launcher.defaultTerminal")}</option>
              <option value="vscode">{t("settings.launcher.defaultEditor")}</option>
              <option value="explorer">{t("settings.launcher.defaultExplorer")}</option>
              <option value="opencode">{t("settings.launcher.defaultOpencode")}</option>
              <option value="claude">{t("settings.launcher.defaultClaude")}</option>
              <option value="codex">{t("settings.launcher.defaultCodex")}</option>
              <option value="copilot">{t("settings.launcher.defaultCopilot")}</option>
              <option value="gemini">{t("settings.launcher.defaultGemini")}</option>
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

          <details className="advanced-settings">
            <summary className="advanced-settings-summary">
              <span>
                <strong>{t("settings.advanced.title")}</strong>
                <small>{t("settings.advanced.subtitle")}</small>
              </span>
              <ChevronRightIcon className="advanced-settings-chevron" size={16} />
            </summary>

            <div className="advanced-settings-body">
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
                  checked={settingsForm.showStatusBar ?? true}
                  onChange={(event) =>
                    onFormChange({ ...settingsForm, showStatusBar: event.currentTarget.checked })
                  }
                />
                <span>{t("statusBar.showStatusBar")}</span>
              </label>

              <label className="field-group">
                <span>{t("settings.fields.terminalPath")}</span>
                <div className="field-with-action field-with-action--dual-action">
                  <input
                    value={settingsForm.terminalPath ?? ""}
                    title={settingsForm.terminalPath ?? ""}
                    onChange={(event) =>
                      onFormChange({ ...settingsForm, terminalPath: event.currentTarget.value })
                    }
                  />
                  <IconButton
                    label={t("settings.actions.browseFile")}
                    className="path-picker-button"
                    onClick={() => onBrowseFile("terminalPath")}
                  >
                    <EditNotesIcon size={14} />
                  </IconButton>
                  <button
                    type="button"
                    className="ghost-button path-action-button"
                    onClick={onDetectTerminal}
                  >
                    {t("settings.actions.detectTerminal")}
                  </button>
                </div>
              </label>

              <label className="field-group">
                <span>{t("settings.fields.externalEditorPath")}</span>
                <div className="field-with-action field-with-action--dual-action">
                  <input
                    value={settingsForm.externalEditorPath ?? ""}
                    title={settingsForm.externalEditorPath ?? ""}
                    onChange={(event) =>
                      onFormChange({ ...settingsForm, externalEditorPath: event.currentTarget.value })
                    }
                  />
                  <IconButton
                    label={t("settings.actions.browseFile")}
                    className="path-picker-button"
                    onClick={() => onBrowseFile("externalEditorPath")}
                  >
                    <EditNotesIcon size={14} />
                  </IconButton>
                  <button
                    type="button"
                    className="ghost-button path-action-button"
                    onClick={onDetectVscode}
                  >
                    {t("settings.actions.detectEditor")}
                  </button>
                </div>
              </label>

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

              <div className="settings-field settings-field--stacked advanced-settings-agents">
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
                    <IconButton
                      label={t("settings.actions.browseDirectory")}
                      className="path-picker-button"
                      onClick={() => onBrowseDirectory("agentsSourceRoot")}
                    >
                      <EditNotesIcon size={14} />
                    </IconButton>
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
            </div>
          </details>

          <div className="settings-actions">
            <Button variant="primary" onClick={onSave}>
              {t("settings.actions.save")}
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
              const providerAvailable = providerDirectoryExists[integration.provider] !== false;
              const providerRootField = getProviderRootField(integration.provider);
              const providerRootPath = getProviderRootPath(settingsForm, integration.provider);
              const configPath = integration.configPath?.trim() || null;
              const bridgePath = integration.bridgePath?.trim() || null;
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
                  }${providerAvailable ? "" : " provider-integration-card--unavailable"} ${isExpanded ? "provider-integration-card--expanded" : "provider-integration-card--collapsed"}`}
                >
                  <div
                    className="provider-integration-header"
                    onClick={() => toggleProviderExpanded(integration.provider, isExpanded)}
                    aria-expanded={isExpanded}
                    title={t(isExpanded ? "settings.integrations.actions.collapse" : "settings.integrations.actions.expand")}
                  >
                    <div className="provider-integration-badges">
                        <input
                          type="checkbox"
                          className="provider-integration-enabled"
                          aria-label={providerLabel}
                          checked={settingsForm.enabledProviders.includes(integration.provider)}
                          disabled={!providerAvailable}
                          onClick={(event) => event.stopPropagation()}
                          onChange={(event) => {
                            const next = event.currentTarget.checked
                              ? [...settingsForm.enabledProviders, integration.provider]
                              : settingsForm.enabledProviders.filter((provider) => provider !== integration.provider);
                            onFormChange({ ...settingsForm, enabledProviders: next });
                            if (event.currentTarget.checked) {
                              onProviderAction(integration.provider, "install");
                            }
                          }}
                        />
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
                        <span
                          className="provider-detection-indicator"
                          title={providerDirectoryExists[integration.provider] === true ? t("settings.status.detected") : undefined}
                        >
                          {providerDirectoryExists[integration.provider] === true ? <CheckIcon size={14} /> : null}
                        </span>
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
                            disabled={!providerAvailable || Boolean(providerBusy)}
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
                            disabled={!providerAvailable || Boolean(providerBusy)}
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
                        <div className="provider-integration-meta provider-integration-meta--path">
                          <div className="provider-path-heading">
                            <span>{t("settings.integrations.fields.rootPath")}</span>
                            {providerRootField ? (
                              <IconButton
                                label={t("settings.actions.browseDirectory")}
                                className="path-picker-button"
                                onClick={() => onBrowseDirectory(providerRootField)}
                              >
                                <EditNotesIcon size={12} />
                              </IconButton>
                            ) : null}
                          </div>
                          <code className="path-text path-text--wrap" title={providerRootPath || undefined}>
                            {providerRootPath || t("settings.integrations.values.unavailable")}
                          </code>
                        </div>

                        {configPath ? (
                          <div className="provider-integration-meta provider-integration-meta--path">
                            <div className="provider-path-heading">
                              <span>{t(
                                integration.provider === "claude"
                                  ? "settings.integrations.fields.hookPath"
                                  : "settings.integrations.fields.configPath",
                              )}</span>
                            </div>
                            <code className="path-text path-text--wrap" title={configPath}>{configPath}</code>
                          </div>
                        ) : null}

                        {bridgePath ? (
                          <div className="provider-integration-meta provider-integration-meta--path">
                            <div className="provider-path-heading">
                              <span>{t("settings.integrations.fields.bridgePath")}</span>
                            </div>
                            <code className="path-text path-text--wrap" title={bridgePath}>{bridgePath}</code>
                          </div>
                        ) : null}

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
                  {PROVIDER_DISPLAY_ORDER.map((provider) => {
                    const enabledProviders =
                      settingsForm.quotaEnabledProviders ?? [...PROVIDER_DISPLAY_ORDER];
                    const checked = enabledProviders.includes(provider);
                    const providerAvailable = providerDirectoryExists[provider] !== false;
                    return (
                      <label
                        key={provider}
                        className={`checkbox-group checkbox-group--inline${providerAvailable ? "" : " checkbox-group--disabled"}`}
                      >
                        <input
                          type="checkbox"
                          checked={checked}
                          disabled={!providerAvailable}
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
                  {PROVIDER_DISPLAY_ORDER.filter((provider) =>
                    (settingsForm.quotaEnabledProviders ?? []).includes(provider),
                  ).map((provider) => (
                    <option
                      key={provider}
                      value={provider}
                      disabled={providerDirectoryExists[provider] === false}
                    >
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
                      {PROVIDER_DISPLAY_ORDER.map((provider) => {
                        const monitored = (settingsForm.quotaEnabledProviders ?? []).includes(provider);
                        const checked = (settingsForm.quotaOverlayProviders ?? []).includes(provider);
                        const providerAvailable = providerDirectoryExists[provider] !== false;
                        return (
                          <label
                            key={provider}
                            className={`checkbox-group checkbox-group--inline${monitored && providerAvailable ? "" : " checkbox-group--disabled"}`}
                          >
                            <input
                              type="checkbox"
                              checked={checked}
                              disabled={!monitored || !providerAvailable}
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

      </div>
    </section>
  );
}
