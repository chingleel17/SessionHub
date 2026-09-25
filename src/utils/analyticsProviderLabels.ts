import { getModelProviderLabel, getProviderLabel } from "./providerLabel";

export function formatAnalyticsPlatformLabel(provider: string): string {
  return getProviderLabel(provider);
}

export function formatAnalyticsSupplierLabel(provider: string): string {
  return getModelProviderLabel(provider);
}
