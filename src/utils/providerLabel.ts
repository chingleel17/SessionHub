/** provider 縮寫代碼，全 App 唯一定義來源。 */
const PROVIDER_ABBR: Record<string, string> = {
  claude: "CC",
  copilot: "CP",
  opencode: "OC",
  codex: "CX",
  antigravity: "AG",
};

/** 執行平台正式顯示名稱，全 App 共用。 */
export const PROVIDER_LABELS: Record<string, string> = {
  claude: "Claude Code",
  copilot: "GitHub Copilot",
  "github-copilot": "GitHub Copilot",
  opencode: "OpenCode",
  codex: "Codex",
  antigravity: "Antigravity",
};

/** 模型供應商正式顯示名稱，全 App 共用。 */
export const MODEL_PROVIDER_LABELS: Record<string, string> = {
  claude: "Anthropic",
  anthropic: "Anthropic",
  openai: "OpenAI",
  opencode: "OpenCode",
  copilot: "GitHub Copilot",
  "github-copilot": "GitHub Copilot",
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
  return PROVIDER_LABELS[provider.trim().toLowerCase()] ?? provider;
}

export function getModelProviderLabel(provider: string): string {
  return MODEL_PROVIDER_LABELS[provider.trim().toLowerCase()] ?? provider;
}
