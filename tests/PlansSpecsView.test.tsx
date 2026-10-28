import { expect, test } from "bun:test";
import DOMPurify from "dompurify";
import { Window } from "happy-dom";
import { act, type ComponentProps } from "react";
import { createRoot } from "react-dom/client";

import { PlansSpecsView } from "../src/components/PlansSpecsView";
import { I18nProvider } from "../src/i18n/I18nProvider";
import type { OpenSpecData } from "../src/types";

const activeData: OpenSpecData = {
  schema: "spec-driven",
  activeChanges: [{
    name: "current-change", hasProposal: true, hasDesign: false, hasTasks: false,
    specsCount: 0, specs: [], extraFiles: [],
  }],
  specs: [], archivedChanges: [],
};

test("分層載入時 active 可操作，背景資料加入保留選取與收折狀態", async () => {
  const browserWindow = new Window({ url: "http://localhost" });
  browserWindow.localStorage.setItem("app-locale", "en-US");
  const originals = new Map<string, PropertyDescriptor | undefined>();
  for (const [key, value] of Object.entries({
    window: browserWindow, document: browserWindow.document,
    navigator: browserWindow.navigator, localStorage: browserWindow.localStorage,
    HTMLElement: browserWindow.HTMLElement, IS_REACT_ACT_ENVIRONMENT: true,
  })) {
    originals.set(key, Object.getOwnPropertyDescriptor(globalThis, key));
    Object.defineProperty(globalThis, key, { configurable: true, value });
  }
  const originalSanitize = DOMPurify.sanitize;
  DOMPurify.sanitize = DOMPurify(browserWindow as unknown as Parameters<typeof DOMPurify>[0]).sanitize;
  const container = document.createElement("div");
  document.body.append(container);
  const root = createRoot(container);
  const reads: string[] = [];
  const props: ComponentProps<typeof PlansSpecsView> = {
    sisyphusData: undefined, openspecData: activeData,
    isLoading: true, isRefreshing: true, loadingMessage: "Loading main specs in the background…",
    onReadFileContent: async () => "",
    onReadOpenspecFile: async (_, path) => { reads.push(path); return "# Active proposal"; },
    onWriteOpenspecFile: async () => undefined,
    onRefresh: async () => undefined, refreshToken: "1:0:0",
    projectCwd: "C:/project", onOpenExternal: () => undefined,
  };
  const render = async (updates: Partial<typeof props>) => {
    Object.assign(props, updates);
    await act(async () => root.render(<I18nProvider><PlansSpecsView {...props} /></I18nProvider>));
  };
  const click = async (selector: string) => {
    const button = container.querySelector<HTMLButtonElement>(selector);
    expect(button).not.toBeNull();
    await act(async () => button?.click());
  };

  try {
    await render({});
    expect(container.textContent).toContain("current-change");
    expect(container.querySelector('[role="status"]')?.textContent).toContain("main specs");
    const change = Array.from(container.querySelectorAll<HTMLButtonElement>(".tree-subgroup-btn"))
      .find((button) => button.textContent?.includes("current-change"));
    expect(change).toBeDefined();
    await act(async () => change?.click());
    await click('button[title="proposal.md"]');
    expect(reads).toContain("changes/current-change/proposal.md");
    expect(container.textContent).toContain("Active proposal");

    await click(".explorer-view-btn:nth-child(2)");
    await click(".explorer-list-source-header");
    expect(container.querySelector(".explorer-list-section")).toBeNull();
    await render({
      isLoading: false,
      openspecData: { ...activeData, specs: [{ name: "main-spec", path: "C:/project/openspec/specs/main-spec/spec.md" }] },
      loadingMessage: "Loading archived changes in the background…", refreshToken: "1:2:0",
    });
    expect(container.querySelector(".explorer-list-section")).toBeNull();
    expect(container.textContent).toContain("Active proposal");
    await click(".explorer-list-source-header");
    expect(container.querySelector(".explorer-list-row-name")?.textContent).toBe("current-change");

    await render({ isRefreshing: false, loadingMessage: undefined, error: "Archive unavailable" });
    expect(container.querySelector('[role="alert"]')?.textContent).toContain("Archive unavailable");
    expect(container.textContent).toContain("current-change");
    expect(container.textContent).toContain("Active proposal");

    const empty: OpenSpecData = { schema: null, activeChanges: [], specs: [], archivedChanges: [] };
    await render({ openspecData: empty, loadingMessage: "Loading main specs in the background…", error: undefined });
    expect(container.textContent).not.toContain("No plan or spec data");
    expect(container.querySelector('[role="status"]')).not.toBeNull();
    await render({ loadingMessage: undefined });
    expect(container.textContent).toContain("No plan or spec data");
  } finally {
    await act(async () => root.unmount());
    DOMPurify.sanitize = originalSanitize;
    container.remove();
    for (const [key, descriptor] of originals) {
      if (descriptor) Object.defineProperty(globalThis, key, descriptor);
      else Reflect.deleteProperty(globalThis, key);
    }
    await browserWindow.happyDOM.close();
  }
});
