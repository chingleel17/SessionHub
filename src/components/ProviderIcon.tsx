import { getProviderAbbr } from "../utils/providerLabel";

type ProviderIconProps = {
  provider: string;
  label: string;
  className?: string;
};

export function ProviderIcon({ provider, label, className }: ProviderIconProps) {
  const initials = getProviderAbbr(provider);

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
