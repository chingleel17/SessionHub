import { expect, test } from "bun:test";
import { QueryClient, QueryObserver } from "@tanstack/react-query";

import type { AgentsMdScanResult, SkillEntry, SkillsScanResult } from "../src/types";
import { canLoadGlobalAgents, mergeAgentsMdLayers, mergeSkillsLayers, scanAgentsInLayers } from "../src/utils/agentsLayeredScan";
import { refreshResource, RESOURCE_QUERY_OPTIONS } from "../src/utils/agentsResourceCache";

function deferred<T>() {
  let resolve!: (value: T) => void;
  const promise = new Promise<T>((done) => { resolve = done; });
  return { promise, resolve };
}

function md(root: string, directories: string[]): AgentsMdScanResult {
  return {
    root, scannedDirs: directories.length, truncated: false,
    entries: directories.map((dir) => ({
      dir: `${root}/${dir}`, relDir: dir,
      source: { path: `${root}/${dir}/AGENTS.md`, exists: true, hash: "same", mtimeMs: 1, size: 1 },
      target: { path: `${root}/${dir}/CLAUDE.md`, exists: false, hash: null, mtimeMs: null, size: null },
      status: "target-missing", targetNewer: false,
    })),
  };
}

test("root is published while the same logical query keeps fetching, then nested completes it", async () => {
  const client = new QueryClient();
  const key = ["agents-md", "A"];
  const rootReady = deferred<AgentsMdScanResult>();
  const nestedReady = deferred<AgentsMdScanResult>();
  const started = deferred<void>();
  const calls: string[] = [];
  const queryFn = ({ signal }: { signal: AbortSignal }) => scanAgentsInLayers(client, key, (layer) => {
    calls.push(layer);
    if (layer === "root") return rootReady.promise;
    started.resolve();
    return nestedReady.promise;
  }, mergeAgentsMdLayers, signal);
  const observer = new QueryObserver(client, { queryKey: key, queryFn, ...RESOURCE_QUERY_OPTIONS });
  const stop = observer.subscribe(() => undefined);
  try {
    const pending = refreshResource(client, key, queryFn);
    expect(calls).toEqual(["root"]);
    rootReady.resolve(md("A", [""]));
    await started.promise;
    expect(client.getQueryState(key)?.fetchStatus).toBe("fetching");
    expect(observer.getCurrentResult().isSuccess).toBe(true);
    expect(observer.getCurrentResult().isFetching).toBe(true);
    expect(client.getQueryData<AgentsMdScanResult>(key)?.scanComplete).toBe(false);
    expect(client.getQueryData<AgentsMdScanResult>(key)?.entries.map((entry) => entry.relDir)).toEqual([""]);
    const sameRefresh = refreshResource(client, key, queryFn);
    nestedReady.resolve(md("A", ["packages/api", "packages/web"]));
    const complete = await pending;
    expect(await sameRefresh).toEqual(complete);
    expect(complete.scanComplete).toBe(true);
    expect(complete.entries.map((entry) => entry.relDir)).toEqual(["", "packages/api", "packages/web"]);
    expect(complete.scannedDirs).toBe(3);
    expect(calls).toEqual(["root", "nested"]);
  } finally { stop(); client.clear(); }
});

test("refresh preserves old nested entries until the final result removes deleted files", async () => {
  const client = new QueryClient();
  const key = ["agents-md", "A"];
  client.setQueryData(key, md("A", ["", "deleted"]));
  const nested = deferred<AgentsMdScanResult>();
  const started = deferred<void>();
  const pending = refreshResource(client, key, ({ signal }) => scanAgentsInLayers(client, key, async (layer) => {
    if (layer === "root") return md("A", []);
    started.resolve();
    return nested.promise;
  }, mergeAgentsMdLayers, signal));
  await started.promise;
  expect(client.getQueryData<AgentsMdScanResult>(key)?.entries.map((entry) => entry.relDir)).toEqual(["deleted"]);
  nested.resolve(md("A", []));
  expect((await pending).entries).toEqual([]);
  client.clear();
});

test("nested failure retains usable partial data without silently retrying", async () => {
  const client = new QueryClient();
  const key = ["agents-md", "A"];
  const pending = refreshResource(client, key, ({ signal }) => scanAgentsInLayers(client, key, async (layer) => {
    if (layer === "root") return md("A", [""]);
    throw new Error("nested inaccessible");
  }, mergeAgentsMdLayers, signal));
  await expect(pending).rejects.toThrow("nested inaccessible");
  expect(client.getQueryData<AgentsMdScanResult>(key)?.scanComplete).toBe(false);
  expect(client.getQueryData<AgentsMdScanResult>(key)?.entries).toHaveLength(1);
  expect(client.getQueryState(key)?.status).toBe("error");
  client.clear();
});

test("a canceled root response cannot overwrite a new scan or start nested work", async () => {
  const client = new QueryClient();
  const key = ["agents-md", "A"];
  const oldRoot = deferred<AgentsMdScanResult>();
  let nestedCalls = 0;
  const pending = refreshResource(client, key, ({ signal }) => scanAgentsInLayers(client, key, (layer) => {
    if (layer === "root") return oldRoot.promise;
    nestedCalls += 1;
    return Promise.resolve(md("A", ["stale"]));
  }, mergeAgentsMdLayers, signal)).catch(() => undefined);
  await client.cancelQueries({ queryKey: key });
  const latest = { ...md("A", ["new"]), scanComplete: true };
  client.setQueryData(key, latest);
  client.setQueryData(["agents-md", "B"], md("B", ["other"]));
  oldRoot.resolve(md("A", ["old"]));
  await pending;
  await new Promise((resolve) => setTimeout(resolve, 0));
  expect(client.getQueryData(key)).toEqual(latest);
  expect(client.getQueryData<AgentsMdScanResult>(["agents-md", "B"])?.root).toBe("B");
  expect(nestedCalls).toBe(0);
  client.clear();
});

test("a write during nested loading waits for the old scan then completes a fresh root/nested pair", async () => {
  const client = new QueryClient();
  const key = ["agents-md", "A"];
  const oldNested = deferred<AgentsMdScanResult>();
  const started = deferred<void>();
  const calls: string[] = [];
  let generation = 0;
  const queryFn = ({ signal }: { signal: AbortSignal }) => {
    const version = ++generation;
    return scanAgentsInLayers(client, key, async (layer) => {
      calls.push(`${version}:${layer}`);
      if (version === 1 && layer === "nested") { started.resolve(); return oldNested.promise; }
      return md("A", layer === "root" ? [""] : ["after-write"]);
    }, mergeAgentsMdLayers, signal);
  };
  const old = refreshResource(client, key, queryFn);
  await started.promise;
  const afterWrite = refreshResource(client, key, queryFn, true);
  expect(generation).toBe(1);
  oldNested.resolve(md("A", ["before-write"]));
  await old;
  const result = await afterWrite;
  expect(result.entries.map((entry) => entry.relDir)).toEqual(["", "after-write"]);
  expect(calls).toEqual(["1:root", "1:nested", "2:root", "2:nested"]);
  client.clear();
});

test("skill merging keeps context/provider identity and diagnostics while removing obsolete root entries", () => {
  const skill = (context: string, provider: string): SkillEntry => ({
    name: "review", projectRelativeDir: context, providerId: provider,
    sourceDir: `${context}/review`, skillMdPath: `${context}/review/SKILL.md`, fileCount: 1, targets: [], locations: [],
  });
  const root: SkillsScanResult = { sourceRoot: "A", skills: [skill("", "codex")], targets: [], enabledProviders: ["codex", "claude"], discoveries: [], diagnostics: [] };
  const nested: SkillsScanResult = { ...root, skills: [skill("packages/api", "codex"), skill("packages/api", "claude"), skill("packages/web", "codex")], diagnostics: [{ providerId: "codex", kind: "skill", scope: "project", message: "partial" }] };
  const combined = mergeSkillsLayers(root, nested);
  expect(combined.skills).toHaveLength(4);
  expect(combined.skills.map((entry) => [entry.projectRelativeDir, entry.providerId])).toEqual([["", "codex"], ["packages/api", "codex"], ["packages/api", "claude"], ["packages/web", "codex"]]);
  const preview = mergeSkillsLayers({ ...root, skills: [] }, combined, true);
  expect(preview.skills).toHaveLength(3);
  expect(preview.diagnostics).toEqual(nested.diagnostics);
});

test("global scanning waits for project completion or failure, but a direct global visit starts immediately", () => {
  expect(canLoadGlobalAgents(false, true, { isFetched: false, isFetching: true })).toBe(false);
  expect(canLoadGlobalAgents(false, true, { isFetched: true, isFetching: true })).toBe(false);
  expect(canLoadGlobalAgents(false, true, { isFetched: true, isFetching: false })).toBe(true);
  expect(canLoadGlobalAgents(true, false, { isFetched: false, isFetching: true })).toBe(true);
  expect(canLoadGlobalAgents(false, false, { isFetched: true, isFetching: false })).toBe(false);
});

test("background CLI discovery enriches the already published root without probing or fingerprinting it again", () => {
  const root: SkillsScanResult = { sourceRoot: "A", skills: [{
    name: "Review", projectRelativeDir: "", providerId: "opencode", sourceDir: "A/.agents/skills/review",
    skillMdPath: "A/.agents/skills/review/SKILL.md", fileCount: 5, targets: [], locations: [],
    metrics: { characterCount: 42, estimatedTokens: 12, estimatorVersion: "test" },
  }], targets: [], enabledProviders: ["opencode"], discoveries: [], diagnostics: [] };
  const nested: SkillsScanResult = { ...root, skills: [], cliSkills: [{ name: "review", effectivePath: "A/.agents/skills/review", source: null, enabled: true }] };
  const combined = mergeSkillsLayers(root, nested);
  expect(combined.skills[0].cliState).toBe("available");
  expect(combined.skills[0].cliSource).toBe("cli");
  expect(combined.skills[0].fileCount).toBe(5);
  expect(combined.skills[0].metrics).toEqual(root.skills[0].metrics);
  expect(combined.discoveries[0].source).toBe("cli");
  expect(mergeSkillsLayers(root, combined, true).skills[0].cliState).toBeUndefined();
});
