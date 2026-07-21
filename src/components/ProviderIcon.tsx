type ProviderIconProps = {
  provider: string;
  label: string;
  className?: string;
};

export function ProviderIcon({ provider, label, className }: ProviderIconProps) {
  const initialsByProvider: Record<string, string> = {
    copilot: "CP",
    opencode: "OC",
    codex: "CX",
    claude: "CL",
    antigravity: "AG",
  };
  const initials = initialsByProvider[provider] ?? label.trim().slice(0, 2).toUpperCase();

  return (
    <span
      className={`provider-icon provider-icon--${provider}${className ? ` ${className}` : ""}`}
      aria-label={label}
      title={label}
    >
      {initials}
    </span>
  );
}
