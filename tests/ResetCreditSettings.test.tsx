import { expect, test } from "bun:test";
import { act, useState } from "react";
import { createRoot } from "react-dom/client";
import { renderToStaticMarkup } from "react-dom/server";
import { Window } from "happy-dom";

import { ResetCreditSettings } from "../src/components/ResetCreditSettings";
import { I18nProvider } from "../src/i18n/I18nProvider";
import { ThemeProvider } from "../src/theme/ThemeProvider";
import { SettingsView } from "../src/components/SettingsView";
import { enUsMessages } from "../src/locales/en-US";
import type { AppSettings, ResetCreditAutomationStatus } from "../src/types";
import { mergeAppSettings } from "../src/utils/appSettingsDefaults";
import { newerResetCreditStatus, resetCreditSettingsForSave, validResetCreditSettings } from "../src/utils/resetCreditAutomation";

const form: AppSettings = { copilotRoot: "", opencodeRoot: "", codexRoot: "", enabledProviders: ["codex"], showArchived: false, ...mergeAppSettings(undefined) };
const status: ResetCreditAutomationStatus = { accountScope: "a", generation: 2, reminderState: "ready", autoUseState: "paused", reason: "selection_unverified", notificationStatus: "dispatched", notificationError: null, nextReminderAt: null, lastAttemptAt: "2026-10-30T01:00:00Z", lastResult: "unknown" };

test("reset credit validation and unrelated autosave do not enable unsaved automation", () => {
  const draft = { ...form, codexResetCreditReminderEnabled: true, codexResetCreditAutoUseEnabled: true };
  expect(resetCreditSettingsForSave(draft, form, false).codexResetCreditAutoUseEnabled).toBe(false);
  expect(resetCreditSettingsForSave(draft, form, true).codexResetCreditAutoUseEnabled).toBe(true);
  for (const time of ["9:00", "24:00", "23:60", ""]) expect(validResetCreditSettings({ ...form, codexResetCreditReminderTime: time })).toBe(false);
  for (const days of [-1, 31, 0.5, Number.NaN]) expect(validResetCreditSettings({ ...form, codexResetCreditReminderDaysBefore: days })).toBe(false);
  expect(validResetCreditSettings({ ...form, codexResetCreditReminderTime: "00:00", codexResetCreditReminderDaysBefore: 30 })).toBe(true);
  expect(newerResetCreditStatus(status, { ...status, accountScope: "old", generation: 1 })).toBe(status);
});

test("reset credit controls are independent and preserve time when reminders are disabled", async () => {
  const browser = new Window({ url: "http://localhost" });
  browser.localStorage.setItem("app-locale", "en-US");
  const originals = new Map<string, PropertyDescriptor | undefined>();
  for (const [key, value] of Object.entries({ window: browser, document: browser.document, navigator: browser.navigator, localStorage: browser.localStorage, HTMLElement: browser.HTMLElement, IS_REACT_ACT_ENVIRONMENT: true })) {
    originals.set(key, Object.getOwnPropertyDescriptor(globalThis, key));
    Object.defineProperty(globalThis, key, { configurable: true, value });
  }
  const container = document.createElement("div"); const root = createRoot(container);
  let latest = form; let refreshes = 0;
  function Harness() {
    const [settings, setSettings] = useState<AppSettings>({ ...form, codexResetCreditReminderTime: "08:30", codexResetCreditReminderDaysBefore: 3 });
    latest = settings;
    return <I18nProvider><ResetCreditSettings settings={settings} onChange={setSettings} status={status} onRefresh={() => { refreshes += 1; }} /></I18nProvider>;
  }
  try {
    const structure = document.createElement("div");
    const noop = () => undefined;
    structure.innerHTML = renderToStaticMarkup(<I18nProvider><ThemeProvider><SettingsView
      settingsForm={{ ...form, enableQuotaMonitoring: false }} onFormChange={noop} onSave={noop} isSaving
      onBrowseDirectory={noop} onBrowseFile={noop} onDetectTerminal={noop} onDetectVscode={noop}
      onDetectTools={noop} toolAvailability={null} providerDirectoryExists={{}} onProviderAction={noop}
      onOpenProviderPath={noop} onEditProviderPath={noop} pendingProviderAction={null} onOpenEventMonitor={noop}
      resetCreditStatus={{ ...status, autoUseState: "disabled", reminderReason: "transient_error", autoUseReason: null }}
    /></ThemeProvider></I18nProvider>);
    const resetCard = structure.querySelector(".reset-credit-settings")!;
    const quotaCard = Array.from(structure.querySelectorAll("article")).find((article) => article.querySelector("h3")?.textContent === enUsMessages["quota.monitoring.title"]);
    expect(resetCard.tagName).toBe("ARTICLE");
    expect(resetCard.classList.contains("info-card")).toBe(true);
    expect(quotaCard).toBeDefined();
    expect(quotaCard!.contains(resetCard)).toBe(false);
    expect(resetCard.querySelectorAll("dl > div")[1].querySelector("dd")?.textContent).toBe("Off");
    expect(Array.from(resetCard.querySelectorAll<HTMLInputElement>('input[type="checkbox"]')).every((input) => input.disabled)).toBe(true);
    expect(Array.from(structure.querySelectorAll("button")).find((button) => button.textContent === enUsMessages["settings.actions.saving"])?.disabled).toBe(true);
    await act(async () => root.render(<Harness />));
    const inputs = container.querySelectorAll<HTMLInputElement>('input[type="checkbox"]');
    const time = container.querySelector<HTMLInputElement>('input[type="time"]')!;
    expect(time.disabled).toBe(true);
    await act(async () => inputs[1].click());
    expect(latest.codexResetCreditAutoUseEnabled).toBe(true);
    expect(latest.codexResetCreditReminderEnabled).toBe(false);
    await act(async () => inputs[0].click());
    expect(time.disabled).toBe(false);
    await act(async () => inputs[0].click());
    expect(time.disabled).toBe(true);
    expect(latest.codexResetCreditReminderTime).toBe("08:30");
    expect(latest.codexResetCreditReminderDaysBefore).toBe(3);
    expect(latest.codexResetCreditAutoUseEnabled).toBe(true);
    expect(container.textContent).toContain("Selection order");
    expect(container.textContent).toContain("previous result is unknown");
    await act(async () => container.querySelector<HTMLButtonElement>("button")?.click());
    expect(refreshes).toBe(1);
  } finally {
    await act(async () => root.unmount());
    for (const [key, descriptor] of originals) { if (descriptor) Object.defineProperty(globalThis, key, descriptor); else Reflect.deleteProperty(globalThis, key); }
    await browser.happyDOM.close();
  }
});
