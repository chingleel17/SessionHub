import type { AnalyticsQuery } from "../types";

export function getSystemIanaTimeZone(): string | null {
  try {
    const timeZone = Intl.DateTimeFormat().resolvedOptions().timeZone;
    return timeZone?.trim() ? timeZone : null;
  } catch {
    return null;
  }
}

export function resolveAnalyticsTimeZone(utcFallbackConfirmed: boolean): string | null {
  const systemTimeZone = getSystemIanaTimeZone();
  if (systemTimeZone) return systemTimeZone;
  return utcFallbackConfirmed ? "UTC" : null;
}

export function createDefaultAnalyticsQuery(
  timeZone: string,
  providers: string[],
  now = new Date(),
): AnalyticsQuery {
  const dateParts = new Intl.DateTimeFormat("en-CA", {
    timeZone,
    year: "numeric",
    month: "2-digit",
    day: "2-digit",
  }).formatToParts(now);
  const partValue = (type: Intl.DateTimeFormatPartTypes): number =>
    Number(dateParts.find((part) => part.type === type)?.value ?? 0);
  const today = new Date(Date.UTC(partValue("year"), partValue("month") - 1, partValue("day")));
  const start = new Date(today);
  start.setUTCDate(start.getUTCDate() - 6);
  const formatDate = (value: Date): string =>
    `${value.getUTCFullYear()}-${String(value.getUTCMonth() + 1).padStart(2, "0")}-${String(value.getUTCDate()).padStart(2, "0")}`;

  return {
    startDate: formatDate(start),
    endDate: formatDate(today),
    timeZone,
    groupBy: "day",
    providers: [...providers],
    cwd: null,
    model: null,
    cwds: [],
    models: [],
    includeArchived: false,
    metric: "tokens",
    tokenField: "total",
    breakdown: "total",
  };
}

export function createDashboardAnalyticsQuery(
  timeZone: string,
  providers: string[],
  period: "week" | "month",
  now = new Date(),
): AnalyticsQuery {
  const query = createDefaultAnalyticsQuery(timeZone, providers, now);
  const end = new Date(`${query.endDate}T00:00:00Z`);
  const start = new Date(end);
  if (period === "week") {
    start.setUTCDate(start.getUTCDate() - ((start.getUTCDay() + 6) % 7));
  } else {
    start.setUTCDate(1);
  }
  return { ...query, startDate: formatUtcDate(start) };
}

function formatUtcDate(date: Date): string {
  return `${date.getUTCFullYear()}-${String(date.getUTCMonth() + 1).padStart(2, "0")}-${String(date.getUTCDate()).padStart(2, "0")}`;
}
