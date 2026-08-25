export const PROVIDER_DISPLAY_ORDER = ["claude", "opencode", "codex", "copilot", "antigravity"] as const;

const PROVIDER_RANK = new Map<string, number>(
  PROVIDER_DISPLAY_ORDER.map((provider, index) => [provider, index]),
);

export function compareProviders(left: string, right: string): number {
  const leftRank = PROVIDER_RANK.get(left) ?? Number.MAX_SAFE_INTEGER;
  const rightRank = PROVIDER_RANK.get(right) ?? Number.MAX_SAFE_INTEGER;
  return leftRank - rightRank || left.localeCompare(right);
}
