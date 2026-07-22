import type { AppSettings } from "../types";

type RequiredKeys = "copilotRoot" | "opencodeRoot" | "codexRoot" | "showArchived" | "enabledProviders";

export const DEFAULT_APP_SETTINGS: Required<Omit<AppSettings, RequiredKeys>> = {
  terminalPath: "",
  externalEditorPath: "",
  pinnedProjects: [],
  providerIntegrations: [],
  defaultLauncher: "terminal",
  enableInterventionNotification: true,
  enableSessionEndNotification: false,
  showStatusBar: true,
  analyticsRefreshInterval: 30,
  analyticsPanelCollapsed: false,
  minimizeToTray: false,
  claudeRoot: "",
  antigravityRoot: "",
  hookScriptsPath: "",
  claudeQuotaResetDay: 1,
  claudeMonthlyLimitTokens: null,
  claudeMonthlyLimitUsd: null,
  enableQuotaMonitoring: true,
  quotaEnabledProviders: ["claude", "copilot", "opencode", "codex", "antigravity"],
  allowCreateProjectConfigDir: false,
  agentsSourceRoot: "",
  trayQuotaMode: "icon_only",
  trayQuotaPrimaryProvider: null,
  trayQuotaPanelEnabled: true,
  quotaOverlayEnabled: false,
  quotaOverlayLocked: true,
  quotaOverlayOpacity: 0.3,
  quotaOverlayProviders: [],
  quotaOverlayTheme: "dark",
  quotaOverlayStyle: "compact",
};

export function mergeAppSettings(
  source: Partial<AppSettings> | undefined,
): typeof DEFAULT_APP_SETTINGS {
  const result = { ...DEFAULT_APP_SETTINGS };
  for (const key of Object.keys(DEFAULT_APP_SETTINGS) as Array<keyof typeof DEFAULT_APP_SETTINGS>) {
    (result as Record<string, unknown>)[key] = source?.[key] ?? DEFAULT_APP_SETTINGS[key];
  }
  return result;
}
