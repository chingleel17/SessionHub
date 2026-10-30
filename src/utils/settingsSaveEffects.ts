import type { AppSettings } from "../types";

export type SettingsSaveEffects = {
  restartWatchers: boolean;
  refreshSessions: boolean;
  refreshTools: boolean;
};

export function settingsSaveEffects(previous: AppSettings | undefined, next: AppSettings): SettingsSaveEffects {
  if (!previous) return { restartWatchers: true, refreshSessions: true, refreshTools: true };
  const roots = ["copilotRoot", "opencodeRoot", "codexRoot", "claudeRoot", "antigravityRoot", "hookScriptsPath"] as const;
  const restartWatchers = roots.some((key) => (previous[key] ?? "") !== (next[key] ?? ""))
    || JSON.stringify(previous.enabledProviders) !== JSON.stringify(next.enabledProviders);
  const refreshTools = (["terminalPath", "terminalLauncher", "externalEditorPath"] as const)
    .some((key) => (previous[key] ?? "") !== (next[key] ?? ""));
  return { restartWatchers, refreshSessions: restartWatchers || previous.showArchived !== next.showArchived, refreshTools };
}
