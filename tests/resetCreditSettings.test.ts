import { describe, expect, test } from "bun:test";
import { mergeAppSettings } from "../src/utils/appSettingsDefaults";
import { resetCreditMessage } from "../src/utils/resetCreditAutomation";
import { zhTwMessages } from "../src/locales/zh-TW";
import { enUsMessages } from "../src/locales/en-US";

describe("reset credit settings merge", () => {
  test("missing credit ID has localized recovery guidance", () => {
    expect(resetCreditMessage((key) => zhTwMessages[key], "unknown_credit_id")).toContain("識別碼");
    expect(resetCreditMessage((key) => enUsMessages[key], "unknown_credit_id")).toContain("IDs");
  });
  test("old form data retains defaults through a save round trip", () => {
    const old = { copilotRoot: "copilot", opencodeRoot: "open", codexRoot: "codex", enabledProviders: ["codex"], showArchived: false };
    const saved = JSON.parse(JSON.stringify({ ...old, ...mergeAppSettings(old), showArchived: true }));
    const reloaded = { ...saved, ...mergeAppSettings(saved) };
    expect(reloaded.codexResetCreditReminderEnabled).toBe(false);
    expect(reloaded.codexResetCreditAutoUseEnabled).toBe(false);
    expect(reloaded.codexResetCreditReminderTime).toBe("09:00");
    expect(reloaded.codexResetCreditReminderDaysBefore).toBe(0);
    expect(reloaded.showArchived).toBe(true);
    expect(reloaded.codexRoot).toBe("codex");
  });

  test("independent enabled settings survive form merging", () => {
    for (const reminder of [false, true]) {
      for (const auto of [false, true]) {
        const saved = mergeAppSettings({ codexResetCreditReminderEnabled: reminder, codexResetCreditAutoUseEnabled: auto, codexResetCreditReminderTime: "08:30", codexResetCreditReminderDaysBefore: 3 });
        expect(mergeAppSettings(saved)).toMatchObject({ codexResetCreditReminderEnabled: reminder, codexResetCreditAutoUseEnabled: auto, codexResetCreditReminderTime: "08:30", codexResetCreditReminderDaysBefore: 3 });
      }
    }
  });
});
