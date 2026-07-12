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
