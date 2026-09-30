import { expect, test } from "bun:test";
import { act } from "react";
import { createRoot } from "react-dom/client";
import { Window } from "happy-dom";

import { ModelPricingView } from "../src/components/ModelPricingView";
import { I18nProvider } from "../src/i18n/I18nProvider";

test("pricing dialog floats outside the page and sync is an explicit action", async () => {
  const browserWindow = new Window({ url: "http://localhost" });
  browserWindow.localStorage.setItem("session-hub.locale", "en-US");
  const originals = new Map<string, PropertyDescriptor | undefined>();
  for (const [key, value] of Object.entries({
    window: browserWindow, document: browserWindow.document,
    navigator: browserWindow.navigator, localStorage: browserWindow.localStorage,
    HTMLElement: browserWindow.HTMLElement, IS_REACT_ACT_ENVIRONMENT: true,
  })) {
    originals.set(key, Object.getOwnPropertyDescriptor(globalThis, key));
    Object.defineProperty(globalThis, key, { configurable: true, value });
  }
  const container = document.createElement("div");
  document.body.append(container);
  const root = createRoot(container);
  let syncCount = 0;
  try {
    await act(async () => root.render(
      <I18nProvider><ModelPricingView entries={[]} isLoading={false} isSaving={false}
        isSyncing={false} errorMessage={null} onSync={() => { syncCount += 1; }}
        onSave={async () => undefined} onDelete={() => undefined} onVisibilityChange={() => undefined}
      /></I18nProvider>,
    ));
    const add = container.querySelector<HTMLButtonElement>(".pricing-toolbar > .pricing-actions button:last-child");
    expect(add).not.toBeNull();
    await act(async () => add?.click());
    const dialog = document.querySelector('[role="dialog"]');
    expect(dialog).not.toBeNull();
    expect(container.contains(dialog)).toBe(false);
    expect(dialog?.parentElement?.classList.contains("dialog-backdrop")).toBe(true);
    await act(async () => browserWindow.dispatchEvent(new browserWindow.KeyboardEvent("keydown", { key: "Escape" })));
    expect(document.querySelector('[role="dialog"]')).toBeNull();
    const sync = container.querySelector<HTMLButtonElement>(".pricing-actions button");
    await act(async () => sync?.click());
    expect(syncCount).toBe(1);
  } finally {
    await act(async () => root.unmount());
    container.remove();
    for (const [key, descriptor] of originals) {
      if (descriptor) Object.defineProperty(globalThis, key, descriptor);
      else Reflect.deleteProperty(globalThis, key);
    }
    await browserWindow.happyDOM.close();
  }
});
