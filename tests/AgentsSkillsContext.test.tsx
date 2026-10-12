import { expect, test } from "bun:test";
import { act } from "react";
import { createRoot } from "react-dom/client";
import { Window } from "happy-dom";

import type { AgentsScopeDataBundle } from "../src/components/AgentsConfigView";
import { I18nProvider } from "../src/i18n/I18nProvider";
import type { SkillEntry, SyncRequest } from "../src/types";

test("sibling skills stay separate in list, preview and sync request", async () => {
  const browserWindow = new Window({ url: "http://localhost" });
  browserWindow.localStorage.setItem("session-hub.locale", "en-US");
  browserWindow.localStorage.setItem("agents:tab:c:\\project", "skills");
  const originals = new Map<string, PropertyDescriptor | undefined>();
  for (const [key, value] of Object.entries({ window: browserWindow, document: browserWindow.document,
    navigator: browserWindow.navigator, localStorage: browserWindow.localStorage,
    HTMLElement: browserWindow.HTMLElement, IS_REACT_ACT_ENVIRONMENT: true })) {
    originals.set(key, Object.getOwnPropertyDescriptor(globalThis, key));
    Object.defineProperty(globalThis, key, { configurable: true, value });
  }
  const container = document.createElement("div");
  document.body.append(container);
  const root = createRoot(container);
  const { AgentsConfigView } = await import("../src/components/AgentsConfigView");
  const entries = ["api", "web"].map((part): SkillEntry => ({
    name: "review", projectRelativeDir: `packages/${part}`, sourceDir: `C:\\Project\\packages\\${part}\\.agents\\skills\\review`,
    skillMdPath: `C:\\Project\\packages\\${part}\\.agents\\skills\\review\\SKILL.md`,
    fileCount: 1, targets: [{ targetId: "claude", targetRoot: `C:\\Project\\packages\\${part}\\.claude\\skills`,
      targetRootExists: true, status: "target-missing", targetNewer: false }],
    providerId: "codex", locations: [],
  }));
  let request: SyncRequest | undefined;
  const data: AgentsScopeDataBundle = {
    scope: { kind: "project", projectCwd: "C:\\Project" },
    skillsData: { sourceRoot: "C:\\Project", skills: entries, targets: [{ targetId: "claude", root: "C:\\Project\\.claude\\skills", rootExists: false }],
      enabledProviders: ["codex", "claude"], discoveries: [], diagnostics: [] },
    prefs: { ignoredPaths: [], enabledTargets: ["claude"] },
    isAgentsMdLoading: false, isSkillsLoading: false, isCommandsLoading: false, isPrefsLoading: false,
    onRefreshAgentsMd: async () => undefined, onRefreshSkills: async () => undefined, onRefreshCommands: async () => undefined,
    onPreviewSync: async (next) => { request = next; return { dryRun: true, actions: [], conflicts: 0, errors: 0 }; },
    onApplySync: async () => ({ dryRun: false, actions: [], conflicts: 0, errors: 0 }),
    onUpdatePrefs: async () => undefined,
    mcpProviders: [], mcpLoading: false, mcpFetching: false, onRefreshMcp: async () => undefined,
    onUpsertMcpServer: async () => undefined, onDeleteMcpServer: async () => undefined,
    onSetMcpServerEnabled: async () => undefined, onTestMcpConnection: async () => ({ kind: "connectionFailed", message: "" }),
  };
  try {
    await act(async () => root.render(<I18nProvider><AgentsConfigView {...data} onReadFile={async (path) => path}
      onWriteFile={async () => undefined} onOpenExternal={() => undefined} onRevealPath={() => undefined} /></I18nProvider>));
    const list = container.querySelectorAll<HTMLButtonElement>(".agents-vscode-list-item");
    expect(list.length).toBe(2);
    expect([...list].map((item) => item.textContent)).toEqual(expect.arrayContaining([expect.stringContaining("packages/api · review"), expect.stringContaining("packages/web · review")]));
    await act(async () => list[1].click());
    expect(document.querySelector(".agents-detail-meta")?.textContent).toContain("packages/web · review");
    await act(async () => document.querySelector<HTMLButtonElement>(".agents-detail-close")?.click());
    await act(async () => container.querySelector<HTMLButtonElement>(".agents-toolbar-actions .ghost-button")?.click());
    const rows = document.querySelectorAll<HTMLTableRowElement>(".agents-sync-modal-group tbody tr");
    expect(rows.length).toBe(2);
    await act(async () => rows[0].querySelector<HTMLInputElement>('input[type="checkbox"]')?.click());
    await act(async () => document.querySelector<HTMLButtonElement>(".agents-sync-actions button")?.click());
    expect(request?.items).toEqual([{ source: entries[0].sourceDir, target: `C:\\Project\\packages\\api\\.claude\\skills\\review`, itemKind: "directory", targetId: "claude" }]);
    data.skillsData = { ...data.skillsData!, diagnostics: [{ providerId: "codex", kind: "skill", scope: "project", message: "packages/limited: directory budget exceeded" }] };
    await act(async () => root.render(<I18nProvider><AgentsConfigView {...data} onReadFile={async (path) => path}
      onWriteFile={async () => undefined} onOpenExternal={() => undefined} onRevealPath={() => undefined} /></I18nProvider>));
    expect(container.querySelector(".agents-resource-diagnostic")?.textContent).toContain("packages/limited");
    data.skillsError = "refresh failed";
    await act(async () => root.render(<I18nProvider><AgentsConfigView {...data} onReadFile={async (path) => path}
      onWriteFile={async () => undefined} onOpenExternal={() => undefined} onRevealPath={() => undefined} /></I18nProvider>));
    expect(container.querySelector(".agents-resource-diagnostic")?.textContent).toContain("packages/limited");
    data.skillsData = { ...data.skillsData, diagnostics: [] };
    await act(async () => root.render(<I18nProvider><AgentsConfigView {...data} onReadFile={async (path) => path}
      onWriteFile={async () => undefined} onOpenExternal={() => undefined} onRevealPath={() => undefined} /></I18nProvider>));
    expect(container.querySelector(".agents-resource-diagnostic")).toBeNull();
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
