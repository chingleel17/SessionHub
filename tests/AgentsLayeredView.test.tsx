import { expect, test } from "bun:test";
import DOMPurify from "dompurify";
import { Window } from "happy-dom";
import { act, type ComponentProps } from "react";
import { createRoot } from "react-dom/client";

import { AgentsConfigView } from "../src/components/AgentsConfigView";
import { I18nProvider } from "../src/i18n/I18nProvider";
import { enUsMessages } from "../src/locales/en-US";
import type { AgentsMdScanResult, SkillEntry } from "../src/types";

const rootMd: AgentsMdScanResult = {
  root: "C:/Project", scanComplete: false, scannedDirs: 1, truncated: false,
  entries: [{ dir: "C:/Project", relDir: "", status: "target-missing", targetNewer: false,
    source: { path: "C:/Project/AGENTS.md", exists: true, hash: "hash", mtimeMs: 1, size: 1 },
    target: { path: "C:/Project/CLAUDE.md", exists: false, hash: null, mtimeMs: null, size: null } }],
};

async function withView(theme: "light" | "dark", run: (
  container: HTMLDivElement,
  render: (updates: Partial<ComponentProps<typeof AgentsConfigView>>) => Promise<void>,
) => Promise<void>) {
  const browserWindow = new Window({ url: "http://localhost" });
  browserWindow.localStorage.setItem("app-locale", "en-US");
  browserWindow.document.documentElement.setAttribute("data-theme", theme);
  const originals = new Map<string, PropertyDescriptor | undefined>();
  for (const [key, value] of Object.entries({
    window: browserWindow, document: browserWindow.document, navigator: browserWindow.navigator,
    localStorage: browserWindow.localStorage, HTMLElement: browserWindow.HTMLElement, IS_REACT_ACT_ENVIRONMENT: true,
  })) {
    originals.set(key, Object.getOwnPropertyDescriptor(globalThis, key));
    Object.defineProperty(globalThis, key, { configurable: true, value });
  }
  const originalSanitize = DOMPurify.sanitize;
  DOMPurify.sanitize = DOMPurify(browserWindow as unknown as Parameters<typeof DOMPurify>[0]).sanitize;
  const container = document.createElement("div");
  document.body.append(container);
  const root = createRoot(container);
  const props: ComponentProps<typeof AgentsConfigView> = {
    scope: { kind: "project", projectCwd: "C:/Project" }, agentsMdData: rootMd,
    prefs: { ignoredPaths: [], enabledTargets: [] },
    isAgentsMdLoading: true, isSkillsLoading: false, isCommandsLoading: false, isPrefsLoading: false,
    onRefreshAgentsMd: async () => undefined, onRefreshSkills: async () => undefined, onRefreshCommands: async () => undefined,
    onPreviewSync: async () => ({ dryRun: true, actions: [], conflicts: 0, errors: 0 }),
    onApplySync: async () => ({ dryRun: false, actions: [], conflicts: 0, errors: 0 }), onUpdatePrefs: async () => undefined,
    mcpProviders: [], mcpLoading: false, mcpFetching: false, onRefreshMcp: async () => undefined,
    onUpsertMcpServer: async () => undefined, onDeleteMcpServer: async () => undefined,
    onSetMcpServerEnabled: async () => undefined, onTestMcpConnection: async () => ({ kind: "connectionFailed", message: "" }),
    onReadFile: async () => "# Ready to read", onWriteFile: async () => undefined,
    onOpenExternal: () => undefined, onRevealPath: () => undefined,
  };
  const render = async (updates: Partial<typeof props>) => {
    Object.assign(props, updates);
    await act(async () => root.render(<I18nProvider><AgentsConfigView {...props} /></I18nProvider>));
  };
  try { await render({}); await run(container, render); }
  finally {
    await act(async () => root.unmount());
    DOMPurify.sanitize = originalSanitize;
    container.remove();
    for (const [key, descriptor] of originals) {
      if (descriptor) Object.defineProperty(globalThis, key, descriptor);
      else Reflect.deleteProperty(globalThis, key);
    }
    await browserWindow.happyDOM.close();
  }
}

for (const theme of ["light", "dark"] as const) {
  test(`${theme}: AGENTS root preview stays usable while nested loads or fails`, async () => {
    await withView(theme, async (container, render) => {
      expect(container.querySelector('[role="status"]')?.textContent).toContain(enUsMessages["agents.scan.loadingNested"]);
      const file = container.querySelector<HTMLButtonElement>(".tree-leaf-btn");
      expect(file).not.toBeNull();
      await act(async () => file?.click());
      expect(container.textContent).toContain("Ready to read");
      const nestedEntry = { ...rootMd.entries[0], dir: "C:/Project/packages/api", relDir: "packages/api",
        source: { ...rootMd.entries[0].source, path: "C:/Project/packages/api/AGENTS.md" } };
      await render({ agentsMdData: { ...rootMd, entries: [...rootMd.entries, nestedEntry], scanComplete: true }, isAgentsMdLoading: false });
      expect(container.textContent).toContain("Ready to read");
      expect(container.querySelector('[role="status"]')).toBeNull();

      await render({ agentsMdData: { ...rootMd, entries: [] }, isAgentsMdLoading: true });
      expect(container.textContent).not.toContain(enUsMessages["agents.empty.agentsMd"]);
      await render({ isAgentsMdLoading: false, agentsMdError: "Nested permission denied" });
      expect(container.querySelector('[role="alert"]')?.textContent).toContain("Nested permission denied");
      expect(container.textContent).toContain(enUsMessages["agents.scan.partial"]);
      expect(container.textContent).not.toContain(enUsMessages["agents.empty.agentsMd"]);
    });
  });

  test(`${theme}: skills remain selectable while nested loads and no empty state is shown prematurely`, async () => {
    await withView(theme, async (container, render) => {
      const skillsTab = [...container.querySelectorAll<HTMLButtonElement>(".agents-top-tabs button")]
        .find((button) => button.textContent === enUsMessages["agents.tab.skills"]);
      await act(async () => skillsTab?.click());
      const skill: SkillEntry = { name: "root-review", projectRelativeDir: "", providerId: "codex", sourceDir: "C:/Project/.agents/skills/review",
        skillMdPath: "C:/Project/.agents/skills/review/SKILL.md", fileCount: 1, targets: [], locations: [] };
      const skillsData = { sourceRoot: "C:/Project", scanComplete: false, skills: [skill], targets: [], enabledProviders: ["codex"], discoveries: [], diagnostics: [] };
      await render({ skillsData, isSkillsLoading: true });
      expect(container.querySelector(".agents-vscode-list-item")?.textContent).toContain("root-review");
      await act(async () => container.querySelector<HTMLButtonElement>(".agents-vscode-list-item")?.click());
      expect(document.querySelector(".agents-preview-modal")?.textContent).toContain("Ready to read");
      await render({ skillsData: { ...skillsData, skills: [] }, skillsError: "Nested unavailable", isSkillsLoading: false });
      expect(container.textContent).toContain(enUsMessages["agents.scan.partial"]);
      expect(container.querySelector('[role="alert"]')?.textContent).toContain("Nested unavailable");
      expect(container.textContent).not.toContain(enUsMessages["agents.empty.skills"]);
    });
  });
}
