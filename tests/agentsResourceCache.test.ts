import { expect, test } from "bun:test";
import { QueryClient, QueryObserver, focusManager, onlineManager } from "@tanstack/react-query";

import { RESOURCE_QUERY_OPTIONS, resourceCanLoad, resourceKeyId, resourceQueryKey, refreshResource, updateResourceAfterWrite, writtenSyncKinds } from "../src/utils/agentsResourceCache";
import type { AppSettings, SyncReport, SyncRequest } from "../src/types";

const settings: Pick<AppSettings, "enabledProviders" | "agentsSourceRoot" | "claudeRoot" | "codexRoot" | "copilotRoot" | "antigravityRoot"> = {
  enabledProviders: ["codex", "claude"], agentsSourceRoot: "C:\\Agents\\", claudeRoot: "C:\\Claude",
  codexRoot: "C:\\Codex", copilotRoot: "", antigravityRoot: "",
};

function deferred<T>() {
  let resolve!: (value: T) => void;
  const promise = new Promise<T>((done) => { resolve = done; });
  return { promise, resolve };
}

test("resource keys isolate sources and scopes without depending on provider order or session root", () => {
  const first = resourceQueryKey("skills", "C:\\Repo\\A\\", settings, { ignoredPaths: ["DIST/", "node_modules"], enabledTargets: [] });
  const reorderedSettings = { ...settings, enabledProviders: ["claude", "codex"], opencodeRoot: "elsewhere" };
  expect(first).toEqual(resourceQueryKey("skills", "c:/repo/a", reorderedSettings, { ignoredPaths: ["node_modules", "dist"], enabledTargets: [] }));
  expect(first).not.toEqual(resourceQueryKey("skills", "C:/Repo/B", settings));
  expect(first).not.toEqual(resourceQueryKey("commands", "C:/Repo/A", settings));
  expect(resourceQueryKey("skills", "global", settings)).not.toEqual(resourceQueryKey("skills", "global", { ...settings, agentsSourceRoot: "D:/agents" }));
  expect(resourceQueryKey("mcp", "global", settings)).toEqual(resourceQueryKey("mcp", "global", { ...settings, agentsSourceRoot: "D:/agents" }));
});

test("a successful empty result is cached across remount, focus and reconnect; refresh is single-flight", async () => {
  const client = new QueryClient();
  const key = resourceQueryKey("commands", "global", settings);
  let calls = 0;
  const pending = deferred<string[]>();
  const scan = () => { calls += 1; return calls === 1 ? Promise.resolve([]) : pending.promise; };
  const options = { queryKey: key, queryFn: scan, ...RESOURCE_QUERY_OPTIONS };
  const observer = new QueryObserver(client, options);
  const unsubscribe = observer.subscribe(() => undefined);
  await client.ensureQueryData(options);
  expect(calls).toBe(1);
  unsubscribe();
  focusManager.setFocused(false);
  focusManager.setFocused(true);
  onlineManager.setOnline(false);
  onlineManager.setOnline(true);
  const second = new QueryObserver(client, options);
  const stop = second.subscribe(() => undefined);
  expect(second.getCurrentResult().data).toEqual([]);
  expect(calls).toBe(1);
  const firstRefresh = refreshResource(client, key, scan);
  const sameRefresh = refreshResource(client, key, scan);
  expect(calls).toBe(2);
  pending.resolve(["updated"]);
  expect(await firstRefresh).toEqual(["updated"]);
  expect(await sameRefresh).toEqual(["updated"]);
  stop(); client.clear(); focusManager.setFocused(true); onlineManager.setOnline(true);
});

test("failed first attempt stays errored until explicit retry, while another key loads", async () => {
  const client = new QueryClient();
  const a = resourceQueryKey("skills", "C:/A", settings);
  const b = resourceQueryKey("skills", "C:/B", settings);
  let calls = 0;
  const scan = () => { calls += 1; return calls === 1 ? Promise.reject(new Error("unavailable")) : Promise.resolve(["ready"]); };
  const observer = new QueryObserver(client, { queryKey: a, queryFn: scan, ...RESOURCE_QUERY_OPTIONS });
  const stop = observer.subscribe(() => undefined);
  await expect(client.fetchQuery({ queryKey: a, queryFn: scan, retry: false })).rejects.toThrow("unavailable");
  expect(resourceCanLoad(client, a)).toBe(false);
  stop();
  expect(await client.fetchQuery({ queryKey: b, queryFn: scan })).toEqual(["ready"]);
  expect(calls).toBe(2);
  expect(await refreshResource(client, a, scan)).toEqual(["ready"]);
  expect(calls).toBe(3);
  client.clear();
});

test("post-write scan waits for an older scan and replaces it with the new result", async () => {
  const client = new QueryClient();
  const key = resourceQueryKey("mcp", "global", settings);
  const oldScan = deferred<string[]>();
  let calls = 0;
  const scan = () => { calls += 1; return calls === 1 ? oldScan.promise : Promise.resolve(["after-write"]); };
  const first = client.fetchQuery({ queryKey: key, queryFn: scan });
  const afterWrite = refreshResource(client, key, scan, true);
  expect(calls).toBe(1);
  oldScan.resolve(["before-write"]);
  await first;
  expect(await afterWrite).toEqual(["after-write"]);
  expect(client.getQueryData(key)).toEqual(["after-write"]);
  expect(calls).toBe(2);
  client.clear();
});

test("mock resource IPC only loads opened kinds and a manual project refresh leaves global untouched", async () => {
  const client = new QueryClient();
  const calls = new Map<string, number>();
  const scan = (name: string) => () => {
    calls.set(name, (calls.get(name) ?? 0) + 1);
    return Promise.resolve([name]);
  };
  const kinds = ["skills", "commands", "mcp"] as const;
  for (const kind of kinds) {
    const project = resourceQueryKey(kind, "C:/Project", settings);
    const global = resourceQueryKey(kind, "global", settings);
    const projectScan = scan(`project:${kind}`);
    const globalScan = scan(`global:${kind}`);
    const projectObserver = new QueryObserver(client, { queryKey: project, queryFn: projectScan, enabled: false, ...RESOURCE_QUERY_OPTIONS });
    const globalObserver = new QueryObserver(client, { queryKey: global, queryFn: globalScan, enabled: false, ...RESOURCE_QUERY_OPTIONS });
    const stopProject = projectObserver.subscribe(() => undefined);
    const stopGlobal = globalObserver.subscribe(() => undefined);
    expect(calls.get(`project:${kind}`)).toBeUndefined();
    expect(calls.get(`global:${kind}`)).toBeUndefined();
    projectObserver.setOptions({ queryKey: project, queryFn: projectScan, enabled: true, ...RESOURCE_QUERY_OPTIONS });
    globalObserver.setOptions({ queryKey: global, queryFn: globalScan, enabled: true, ...RESOURCE_QUERY_OPTIONS });
    await Promise.all([client.ensureQueryData({ queryKey: project, queryFn: projectScan, ...RESOURCE_QUERY_OPTIONS }),
      client.ensureQueryData({ queryKey: global, queryFn: globalScan, ...RESOURCE_QUERY_OPTIONS })]);
    expect(calls.get(`project:${kind}`)).toBe(1);
    expect(calls.get(`global:${kind}`)).toBe(1);
    projectObserver.setOptions({ queryKey: project, queryFn: projectScan, enabled: false, ...RESOURCE_QUERY_OPTIONS });
    projectObserver.setOptions({ queryKey: project, queryFn: projectScan, enabled: true, ...RESOURCE_QUERY_OPTIONS });
    expect(calls.get(`project:${kind}`)).toBe(1);
    if (kind === "skills") {
      await refreshResource(client, project, projectScan);
      expect(calls.get("project:skills")).toBe(2);
      expect(calls.get("global:skills")).toBe(1);
    }
    stopProject(); stopGlobal();
  }
  expect(calls.get("project:commands")).toBe(1);
  expect(calls.get("project:mcp")).toBe(1);
  client.clear();
});

test("a new project never shows the previous project while a failed refresh preserves its successful data", async () => {
  const client = new QueryClient();
  const a = resourceQueryKey("skills", "C:/A", settings);
  const b = resourceQueryKey("skills", "C:/B", settings);
  await client.fetchQuery({ queryKey: a, queryFn: async () => ["A"] });
  const observer = new QueryObserver(client, { queryKey: a, queryFn: async () => ["A"], ...RESOURCE_QUERY_OPTIONS });
  const stop = observer.subscribe(() => undefined);
  observer.setOptions({ queryKey: b, queryFn: async () => ["B"], enabled: false, ...RESOURCE_QUERY_OPTIONS });
  expect(observer.getCurrentResult().data).toBeUndefined();
  await client.fetchQuery({ queryKey: b, queryFn: async () => ["B"] });
  expect(observer.getCurrentResult().data).toEqual(["B"]);
  observer.setOptions({ queryKey: a, queryFn: async () => ["A"], ...RESOURCE_QUERY_OPTIONS });
  expect(observer.getCurrentResult().data).toEqual(["A"]);
  await expect(refreshResource(client, a, async () => { throw new Error("read failed"); })).rejects.toThrow("read failed");
  expect(client.getQueryData(a)).toEqual(["A"]);
  expect(resourceCanLoad(client, a)).toBe(false);
  stop(); client.clear();
});

test("successful writes refresh only the active loaded scope and mark inactive loaded data dirty", async () => {
  const client = new QueryClient();
  const global = resourceQueryKey("mcp", "global", settings);
  const project = resourceQueryKey("mcp", "C:/Project", settings);
  const unvisited = resourceQueryKey("commands", "C:/Project", settings);
  const dirty = new Set<string>();
  const active = new Set([resourceKeyId(global)]);
  const calls = { global: 0, project: 0, unvisited: 0 };
  const scanGlobal = async () => { calls.global += 1; return [calls.global]; };
  const scanProject = async () => { calls.project += 1; return [calls.project]; };
  await client.fetchQuery({ queryKey: global, queryFn: scanGlobal });
  await client.fetchQuery({ queryKey: project, queryFn: scanProject });
  await updateResourceAfterWrite(client, unvisited, active, dirty, async () => { calls.unvisited += 1; return []; });
  expect(calls.unvisited).toBe(0);
  await updateResourceAfterWrite(client, project, active, dirty, scanProject);
  expect(calls.project).toBe(1);
  expect(dirty.has(resourceKeyId(project))).toBe(true);
  await updateResourceAfterWrite(client, global, active, dirty, scanGlobal);
  expect(calls.global).toBe(2);
  expect(calls.project).toBe(1);
  dirty.delete(resourceKeyId(project));
  await refreshResource(client, project, scanProject, true);
  expect(calls.project).toBe(2);
  expect(client.getQueryState(unvisited)).toBeUndefined();
  client.clear();
});

test("sync preview, all-skip, failed actions and AGENTS.md edits never rescan resources", () => {
  const request: SyncRequest = { projectCwd: "C:/Project", items: [
    { source: "C:/Project/.agents/skills/demo", target: "C:/Project/.claude/skills/demo", itemKind: "directory" },
    { source: "C:/Project/.agents/command/review.md", target: "C:/Project/.claude/commands/review.md", itemKind: "file" },
    { source: "C:/Project/AGENTS.md", target: "C:/Project/CLAUDE.md", itemKind: "file" },
  ], dryRun: false, force: false, mode: "copy" };
  const actions: SyncReport["actions"] = request.items.map((item) => ({ source: item.source, target: item.target, action: "skip-in-sync" }));
  expect(writtenSyncKinds(request, { dryRun: false, actions, conflicts: 0, errors: 0 })).toEqual([]);
  const created = actions.map((action, index) => ({ ...action, action: index === 0 ? "create" : index === 1 ? "link-fallback-copy" : "overwrite" }));
  const report: SyncReport = { dryRun: false, actions: created, conflicts: 0, errors: 0 };
  expect(writtenSyncKinds(request, report)).toEqual(["skills", "commands"]);
  expect(writtenSyncKinds({ ...request, dryRun: true }, report)).toEqual([]);
  expect(writtenSyncKinds(request, { ...report, dryRun: true })).toEqual([]);
  expect(writtenSyncKinds(request, { ...report, actions: created.map((action) => ({ ...action, action: "error" })) })).toEqual([]);
});
