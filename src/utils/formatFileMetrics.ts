import type { FileContentMetrics } from "../types";

export function formatCharacterCount(value: number, locale: string): string {
  return new Intl.NumberFormat(locale).format(Math.max(0, Math.round(value)));
}

export function formatEstimatedTokens(value: number, locale: string, tokenLabel: string): string {
  const normalized = Math.max(0, Math.round(value));
  const formatted = normalized < 1000
    ? new Intl.NumberFormat(locale).format(normalized)
    : `${new Intl.NumberFormat(locale, { maximumFractionDigits: 1 }).format(normalized / 1000)}k`;
  return `~${formatted} ${tokenLabel}`;
}

export function formatFileMetrics(metrics: FileContentMetrics, locale: string, characterLabel: string, tokenLabel: string): string {
  return `${formatCharacterCount(metrics.characterCount, locale)} ${characterLabel} · ${formatEstimatedTokens(metrics.estimatedTokens, locale, tokenLabel)}`;
}
