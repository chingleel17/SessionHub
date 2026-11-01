import { expect, test } from "bun:test";
import type { AppSettings } from "../src/types";
import { mergeAppSettings } from "../src/utils/appSettingsDefaults";
import { settingsSaveEffects } from "../src/utils/settingsSaveEffects";

const previous: AppSettings = { copilotRoot: "copilot", opencodeRoot: "open", codexRoot: "codex", enabledProviders: ["codex"], showArchived: false, ...mergeAppSettings(undefined) };

test("saving reset credits does not restart watchers or refresh tools and sessions", () => {
  expect(settingsSaveEffects(previous, { ...previous, codexResetCreditReminderEnabled: true, codexResetCreditAutoUseEnabled: true, codexResetCreditReminderTime: "08:30" }))
    .toEqual({ restartWatchers: false, refreshTools: false, refreshSessions: false });
});

test("source and tool changes only refresh their affected resources", () => {
  expect(settingsSaveEffects(previous, { ...previous, claudeRoot: "new-root" })).toEqual({ restartWatchers: true, refreshSessions: true, refreshTools: false });
  expect(settingsSaveEffects(previous, { ...previous, externalEditorPath: "new-editor" })).toEqual({ restartWatchers: false, refreshSessions: false, refreshTools: true });
  expect(settingsSaveEffects(previous, { ...previous, showArchived: true })).toEqual({ restartWatchers: false, refreshSessions: true, refreshTools: false });
});
