import { hashKey, type QueryClient, type QueryKey } from "@tanstack/react-query";
import type { AppSettings, ProjectAgentsPrefs, SyncReport, SyncRequest } from "../types";

type ResourceSettings = Pick<AppSettings, "enabledProviders" | "agentsSourceRoot" | "claudeRoot" | "codexRoot" | "copilotRoot" | "antigravityRoot">;

export type AgentsResourceKind = "skills" | "commands" | "mcp";

export const RESOURCE_QUERY_OPTIONS = {
  staleTime: Infinity,
  gcTime: Infinity,
  refetchOnMount: false,
  refetchOnWindowFocus: false,
  refetchOnReconnect: false,
  refetchInterval: false,
  retry: false,
  retryOnMount: false,
} as const;

export function normalizeResourcePath(path: string): string {
  const normalized = path.replace(/\\/g, "/").toLowerCase();
  if (/^[a-z]:\/$/.test(normalized) || normalized === "/") return normalized;
  return normalized.replace(/\/+$/, "");
}

export function resourceQueryKey(
  kind: AgentsResourceKind,
  scope: string,
  settings: ResourceSettings,
  prefs?: ProjectAgentsPrefs,
): QueryKey {
  const providers = [...new Set(settings.enabledProviders.map((provider) => provider.toLowerCase()))].sort();
  const project = scope !== "global";
  const sourcePaths = kind === "mcp"
    ? { codexRoot: settings.codexRoot, copilotRoot: settings.copilotRoot }
    : project ? {} : {
      agentsSourceRoot: settings.agentsSourceRoot,
      claudeRoot: settings.claudeRoot,
      codexRoot: settings.codexRoot,
      copilotRoot: settings.copilotRoot,
      antigravityRoot: settings.antigravityRoot,
    };
  const normalizedPaths = Object.fromEntries(Object.entries(sourcePaths).map(([key, value]) => [key, normalizeResourcePath(value ?? "")]));
  const ignoredPaths = kind === "skills" && project
    ? [...new Set((prefs?.ignoredPaths ?? []).map((path) => normalizeResourcePath(path).replace(/^\/+/, "")).filter(Boolean))].sort()
    : [];
  const prefix = kind === "mcp" ? "mcp-configs" : `agents-${kind}`;
  return [prefix, project ? normalizeResourcePath(scope) : "global", { providers, sourcePaths: normalizedPaths, ignoredPaths }];
}

export function resourceCanLoad(client: QueryClient, key: QueryKey): boolean {
  return client.getQueryState(key)?.status !== "error";
}

export async function refreshResource<T>(
  client: QueryClient,
  key: QueryKey,
  queryFn: () => Promise<T>,
  afterWrite = false,
): Promise<T> {
  const wasFetching = client.getQueryState(key)?.fetchStatus === "fetching";
  const options = { queryKey: key, queryFn, staleTime: 0, gcTime: Infinity, retry: false };
  if (wasFetching && afterWrite) {
    try {
      await client.fetchQuery(options);
    } catch {
      // 先前掃描失敗仍須取得寫入後資料。
    }
  }
  return client.fetchQuery(options);
}

export function resourceKeyId(key: QueryKey): string {
  return hashKey(key);
}

export async function updateResourceAfterWrite<T>(
  client: QueryClient,
  key: QueryKey,
  activeKeys: ReadonlySet<string>,
  dirtyKeys: Set<string>,
  queryFn: () => Promise<T>,
): Promise<void> {
  if (!client.getQueryState(key)) return;
  const id = resourceKeyId(key);
  if (activeKeys.has(id)) {
    await refreshResource(client, key, queryFn, true);
  } else {
    dirtyKeys.add(id);
  }
}

export function writtenSyncKinds(request: SyncRequest, report: SyncReport): ("skills" | "commands")[] {
  if (request.dryRun || report.dryRun) return [];
  const kinds = new Set<"skills" | "commands">();
  for (const action of report.actions) {
    if (!["create", "overwrite", "link-fallback-copy"].includes(action.action)) continue;
    for (const item of request.items) {
      if (normalizeResourcePath(item.target) !== normalizeResourcePath(action.target)) continue;
      if (item.itemKind === "directory") kinds.add("skills");
      else if (/[\\/](?:commands|command|prompts)[\\/]/i.test(item.target)) kinds.add("commands");
    }
  }
  return [...kinds];
}
