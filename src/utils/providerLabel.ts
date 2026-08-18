/** provider 縮寫代碼，全 App 唯一定義來源。 */
const PROVIDER_ABBR: Record<string, string> = {
  claude: "CC",
  copilot: "CP",
  opencode: "OC",
  codex: "CX",
  antigravity: "AG",
};

/**
 * 取得 provider 的兩碼縮寫，未知 provider 以名稱前兩碼大寫作為後備。
 *
 * @param provider - provider 識別碼，例如 "claude"
 */
export function getProviderAbbr(provider: string): string {
  return PROVIDER_ABBR[provider] ?? provider.trim().slice(0, 2).toUpperCase();
}

export function getProviderLabel(provider: string): string {
  switch (provider) {
    case "copilot":
      return "Copilot";
    case "opencode":
      return "OpenCode";
    case "codex":
      return "Codex";
    case "claude":
      return "Claude Code";
    case "antigravity":
      return "Antigravity";
    default:
      return provider;
  }
}
