import type { QueryClient, QueryKey } from "@tanstack/react-query";
import type { AgentsMdScanResult, SkillsScanResult } from "../types";

export type AgentsScanLayer = "root" | "nested";

/** 同一快取中的兩階段偵測；中間結果不結束 query 的 fetching 狀態。 */
export async function scanAgentsInLayers<T extends { scanComplete?: boolean }>(
  client: QueryClient,
  key: QueryKey,
  scan: (layer: AgentsScanLayer) => Promise<T>,
  merge: (root: T, nested: T, retainPrevious?: boolean) => T,
  signal: AbortSignal,
): Promise<T> {
  signal.throwIfAborted();
  const root = await scan("root");
  signal.throwIfAborted();
  const previous = client.getQueryData<T>(key);
  client.setQueryData<T>(key, {
    ...(previous ? merge(root, previous, true) : root),
    scanComplete: false,
  });
  const nested = await scan("nested");
  signal.throwIfAborted();
  return { ...merge(root, nested), scanComplete: true };
}

export function mergeAgentsMdLayers(
  root: AgentsMdScanResult,
  nested: AgentsMdScanResult,
  retainPrevious = false,
): AgentsMdScanResult {
  return {
    ...root,
    entries: [...root.entries, ...nested.entries.filter((entry) => entry.relDir !== "")]
      .sort((a, b) => a.dir < b.dir ? -1 : a.dir > b.dir ? 1 : 0),
    truncated: root.truncated || nested.truncated,
    scannedDirs: retainPrevious ? Math.max(root.scannedDirs, nested.scannedDirs) : root.scannedDirs + nested.scannedDirs,
  };
}

export function mergeSkillsLayers(
  root: SkillsScanResult,
  nested: SkillsScanResult,
  retainPrevious = false,
): SkillsScanResult {
  const nestedSkills = nested.skills.filter((entry) => Boolean(entry.projectRelativeDir));
  const cliSkills = new Map((retainPrevious ? [] : nested.cliSkills ?? []).map((entry) => [entry.name.toLowerCase(), entry]));
  const rootSkills = root.skills.map((entry) => {
    const cli = entry.providerId === "opencode" ? cliSkills.get(entry.name.toLowerCase()) : undefined;
    return cli ? { ...entry, effectivePath: cli.effectivePath, cliSource: "cli" as const, cliState: "available" as const } : entry;
  });
  const skills = [...rootSkills, ...nestedSkills];
  const diagnostics = new Map([...root.diagnostics, ...nested.diagnostics].map((entry) => [JSON.stringify(entry), entry]));
  return {
    ...root,
    skills,
    discoveries: skills.flatMap((entry) => entry.providerId ? [{
      providerId: entry.providerId, kind: "skill" as const,
      scope: entry.projectRelativeDir === null ? "global" as const : "project" as const,
      locations: entry.locations, effectivePath: entry.effectivePath ?? null,
      source: entry.cliSource ?? "file" as const, state: entry.cliState ?? "configured" as const, editable: true,
    }] : []),
    diagnostics: [...diagnostics.values()],
    cliSkills: retainPrevious ? root.cliSkills : nested.cliSkills,
  };
}

/** 專案成功或失敗結束後皆可載入附帶全域資料，獨立全域頁不必等待。 */
export function canLoadGlobalAgents(direct: boolean, requested: boolean, project: { isFetched: boolean; isFetching: boolean }): boolean {
  return direct || (requested && project.isFetched && !project.isFetching);
}
