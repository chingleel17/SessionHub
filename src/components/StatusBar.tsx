import { useEffect, useRef, useState } from "react";

import { useI18n } from "../i18n/I18nProvider";
import type { MessageKey } from "../locales/zh-TW";
import type { BridgeEventLogEntry, ProviderQuota, QuotaSnapshot } from "../types";
import { localizedWindowLabel } from "../utils/quotaWindowLabel";

import { QuotaOverview } from "./QuotaOverview";

type Props = {
  lastBridgeEvent: { entry: BridgeEventLogEntry; receivedAt: Date } | null;
  onOpenEventMonitor: () => void;
  activeSessions: number;
  waitingSessions: number;
  idleSessions: number;
  doneSessions: number;
  isLoadingSessions: boolean;
  providerQuotas?: ProviderQuota[];
  quotaSnapshots?: QuotaSnapshot[];
  quotaEnabledProviders?: string[];
  onRefreshQuota?: (provider?: string) => void;
};

const STATUS_COLORS: Record<string, string> = {
  targeted: "var(--color-blue, #58a6ff)",
  fallback: "var(--color-yellow, #d29922)",
  full_refresh: "var(--color-green, #3fb950)",
  skipped_dedup: "var(--color-muted, #8b949e)",
  skipped_rate_limit: "var(--color-muted, #8b949e)",
};

const PROVIDER_ABBR: Record<string, string> = {
  claude: "CC",
  copilot: "GH",
  opencode: "OC",
  codex: "CX",
  antigravity: "AG",
};

const PROVIDER_COLOR: Record<string, string> = {
  claude: "#D97757",
  copilot: "#57ab5a",
  opencode: "#8957e5",
  codex: "#10a37f",
  antigravity: "#4285F4",
};

function statusBarWindowsForSnapshot(snap: QuotaSnapshot) {
  const windows = snap.windows ?? [];
  if (snap.provider !== "antigravity") return windows;
  return windows.filter((w) => (w.group ?? "").toLowerCase().includes("gemini"));
}

function quotaBarColor(pct: number): string {
  if (pct >= 90) return "var(--quota-bar-danger)";
  if (pct >= 70) return "var(--quota-bar-warning)";
  return "var(--quota-bar-ok)";
}

function QuotaRing({ pct }: { pct: number }) {
  const clamped = Math.min(Math.max(pct, 0), 100);
  const size = 14;
  const stroke = 2;
  const radius = (size - stroke) / 2;
  const circumference = 2 * Math.PI * radius;
  const dash = (clamped / 100) * circumference;
  const color = quotaBarColor(clamped);
  return (
    <svg width={size} height={size} viewBox={`0 0 ${size} ${size}`} className="global-status-bar-quota-ring">
      <circle
        cx={size / 2}
        cy={size / 2}
        r={radius}
        fill="none"
        stroke="var(--color-border)"
        strokeWidth={stroke}
      />
      <circle
        cx={size / 2}
        cy={size / 2}
        r={radius}
        fill="none"
        stroke={color}
        strokeWidth={stroke}
        strokeDasharray={`${dash} ${circumference}`}
        strokeLinecap="round"
        transform={`rotate(-90 ${size / 2} ${size / 2})`}
      />
    </svg>
  );
}

function truncateCwd(cwd: string, maxLen = 40): string {
  if (cwd.length <= maxLen) return cwd;
  return "…" + cwd.slice(-(maxLen - 1));
}

function formatTokenCount(n: number): string {
  if (n >= 1_000_000) return `${(n / 1_000_000).toFixed(1)}M`;
  if (n >= 1_000) return `${(n / 1_000).toFixed(1)}K`;
  return String(n);
}

function formatResetDateTime(iso: string | null | undefined, amLabel: string, pmLabel: string): string {
  if (!iso) return "";
  try {
    const d = new Date(iso);
    const MM = String(d.getMonth() + 1).padStart(2, "0");
    const DD = String(d.getDate()).padStart(2, "0");
    const rawHours = d.getHours();
    const period = rawHours < 12 ? amLabel : pmLabel;
    const hours12 = rawHours % 12 === 0 ? 12 : rawHours % 12;
    const hh = String(hours12).padStart(2, "0");
    const mm = String(d.getMinutes()).padStart(2, "0");
    return `${MM}/${DD} ${period}${hh}:${mm}`;
  } catch {
    return iso;
  }
}

function getNearestResetCreditExpiry(snapshot: QuotaSnapshot): string | null {
  const candidates = (snapshot.resetCredits?.credits ?? [])
    .map((credit) => credit.expiresAt)
    .filter((value): value is string => Boolean(value))
    .map((value) => ({ value, time: new Date(value).getTime() }))
    .filter((entry) => !Number.isNaN(entry.time) && entry.time > Date.now())
    .sort((left, right) => left.time - right.time);

  return candidates[0]?.value ?? null;
}

function QuotaChip({ quota, noLimitLabel }: { quota: ProviderQuota; noLimitLabel: string }) {
  const totalTokens = quota.inputTokens + quota.outputTokens;
  const abbr = PROVIDER_ABBR[quota.provider] ?? quota.provider.slice(0, 2).toUpperCase();
  const hasCost = quota.costUsd > 0;
  const hasLimit = quota.monthlyLimitTokens != null;

  const tooltipLines = [
    `${quota.provider} — ${quota.billingPeriod}`,
    `Input: ${formatTokenCount(quota.inputTokens)}`,
    `Output: ${formatTokenCount(quota.outputTokens)}`,
    quota.cacheCreationTokens > 0 ? `Cache create: ${formatTokenCount(quota.cacheCreationTokens)}` : null,
    quota.cacheReadTokens > 0 ? `Cache read: ${formatTokenCount(quota.cacheReadTokens)}` : null,
    hasCost ? `Cost: $${quota.costUsd.toFixed(4)}` : null,
    hasLimit
      ? `Limit: ${formatTokenCount(quota.monthlyLimitTokens!)} (${((totalTokens / quota.monthlyLimitTokens!) * 100).toFixed(1)}%)`
      : noLimitLabel,
    `Reset: ${quota.nextResetDate}`,
  ]
    .filter(Boolean)
    .join("\n");

  const limitPct = hasLimit ? Math.min(100, (totalTokens / quota.monthlyLimitTokens!) * 100) : 0;

  return (
    <span className="global-status-bar-quota-chip" title={tooltipLines}>
      <span className="global-status-bar-quota-abbr" style={{ color: PROVIDER_COLOR[quota.provider] }}>
        {abbr}
      </span>
      {hasCost ? <span className="global-status-bar-quota-cost">${quota.costUsd.toFixed(2)}</span> : null}
      {hasLimit ? (
        <>
          <QuotaRing pct={limitPct} />
          <span className="global-status-bar-quota-pct" style={{ color: quotaBarColor(limitPct) }}>
            {Math.round(limitPct)}%
          </span>
        </>
      ) : null}
    </span>
  );
}

function QuotaSnapshotChip({
  snap,
  amLabel,
  pmLabel,
  resetsLabel,
  t,
}: {
  snap: QuotaSnapshot;
  amLabel: string;
  pmLabel: string;
  resetsLabel: string;
  t: (key: MessageKey, params?: Record<string, string | number>) => string;
}) {
  const abbr = PROVIDER_ABBR[snap.provider] ?? snap.provider.slice(0, 2).toUpperCase();
  const windows = statusBarWindowsForSnapshot(snap);
  const topWindow = windows[0];
  const pct = topWindow ? Math.round(topWindow.utilization * 100) : null;
  const nearestExpiry = getNearestResetCreditExpiry(snap);
  const resetCreditsSummary =
    snap.provider === "codex" && snap.resetCredits
      ? t("quota.resetCredits.tooltipSummary", {
          count: snap.resetCredits.availableCount,
          expiresAt: nearestExpiry ? formatResetDateTime(nearestExpiry, amLabel, pmLabel) : t("quota.resetCredits.noExpiry"),
        })
      : null;
  const tooltip = [
    `${snap.provider} · ${snap.source}`,
    ...windows.map((w) => {
      const wPct = Math.round(w.utilization * 100);
      const reset = w.resetsAt ? ` · ${resetsLabel}: ${formatResetDateTime(w.resetsAt, amLabel, pmLabel)}` : "";
      const wLabel = localizedWindowLabel(snap.provider, w.windowKey, w.label, t);
      return `${wLabel}: ${wPct}%${reset}`;
    }),
    resetCreditsSummary,
  ]
    .filter(Boolean)
    .join("\n");

  return (
    <span className="global-status-bar-quota-chip global-status-bar-quota-chip--snapshot" title={tooltip}>
      <span className="global-status-bar-quota-abbr" style={{ color: PROVIDER_COLOR[snap.provider] }}>
        {abbr}
      </span>
      {pct !== null ? (
        <>
          <QuotaRing pct={pct} />
          <span className="global-status-bar-quota-pct" style={{ color: quotaBarColor(pct) }}>
            {pct}%
          </span>
        </>
      ) : null}
    </span>
  );
}

export function StatusBar({
  lastBridgeEvent,
  onOpenEventMonitor,
  activeSessions,
  waitingSessions,
  idleSessions,
  doneSessions,
  isLoadingSessions,
  providerQuotas = [],
  quotaSnapshots = [],
  quotaEnabledProviders = [],
  onRefreshQuota,
}: Props) {
  const { t } = useI18n();
  const [isQuotaPopupOpen, setIsQuotaPopupOpen] = useState(false);
  const quotaPopupRef = useRef<HTMLDivElement | null>(null);
  const dash = isLoadingSessions ? "-" : undefined;
  const activeQuotas = providerQuotas.filter(
    (q) =>
      quotaEnabledProviders.includes(q.provider) &&
      (q.inputTokens > 0 || q.outputTokens > 0 || q.costUsd > 0),
  );
  const visibleSnapshots = quotaSnapshots.filter(
    (s) =>
      s.status === "ok" &&
      (s.source === "remote_api" || s.provider === "antigravity") &&
      quotaEnabledProviders.includes(s.provider),
  );
  const hasQuotaContent = activeQuotas.length > 0 || visibleSnapshots.length > 0;

  useEffect(() => {
    if (!isQuotaPopupOpen) return undefined;

    const handleMouseDown = (event: MouseEvent) => {
      if (!quotaPopupRef.current?.contains(event.target as Node)) {
        setIsQuotaPopupOpen(false);
      }
    };

    const handleKeyDown = (event: KeyboardEvent) => {
      if (event.key === "Escape") {
        setIsQuotaPopupOpen(false);
      }
    };

    document.addEventListener("mousedown", handleMouseDown);
    document.addEventListener("keydown", handleKeyDown);
    return () => {
      document.removeEventListener("mousedown", handleMouseDown);
      document.removeEventListener("keydown", handleKeyDown);
    };
  }, [isQuotaPopupOpen]);

  return (
    <div className="global-status-bar">
      <button
        type="button"
        className="global-status-bar-event"
        onClick={onOpenEventMonitor}
        title={t("eventMonitor.openButton")}
      >
        {lastBridgeEvent ? (
          <>
            <span className="global-status-bar-time">
              {lastBridgeEvent.receivedAt.toLocaleTimeString("zh-TW", { hour12: false })}
            </span>
            <span className="global-status-bar-provider">{lastBridgeEvent.entry.provider}</span>
            <span className="global-status-bar-type">{lastBridgeEvent.entry.eventType}</span>
            <span
              className="global-status-bar-status-dot"
              style={{ color: STATUS_COLORS[lastBridgeEvent.entry.status] ?? "inherit" }}
            >
              ●
            </span>
            <span className="global-status-bar-status-label">{lastBridgeEvent.entry.status}</span>
            {lastBridgeEvent.entry.cwd ? (
              <span className="global-status-bar-cwd" title={lastBridgeEvent.entry.cwd}>
                {truncateCwd(lastBridgeEvent.entry.cwd)}
              </span>
            ) : null}
          </>
        ) : (
          <span className="global-status-bar-no-event">{t("statusBar.noEvent")}</span>
        )}
      </button>

      <div className="global-status-bar-counts">
        <span className={`global-status-bar-count${activeSessions === 0 ? " global-status-bar-count--zero" : ""}`}>
          ▶ {dash ?? activeSessions} {t("statusBar.active")}
        </span>
        <span className={`global-status-bar-count${waitingSessions === 0 ? " global-status-bar-count--zero" : ""}`}>
          ⏳ {dash ?? waitingSessions} {t("statusBar.waiting")}
        </span>
        <span className={`global-status-bar-count${idleSessions === 0 ? " global-status-bar-count--zero" : ""}`}>
          ◌ {dash ?? idleSessions} {t("statusBar.idle")}
        </span>
        <span className={`global-status-bar-count${doneSessions === 0 ? " global-status-bar-count--zero" : ""}`}>
          ✓ {dash ?? doneSessions} {t("statusBar.done")}
        </span>
      </div>

      {hasQuotaContent ? (
        <div className="global-status-bar-quota-anchor" ref={quotaPopupRef}>
          <button
            type="button"
            className={`global-status-bar-quota-trigger${isQuotaPopupOpen ? " global-status-bar-quota-trigger--active" : ""}`}
            onClick={() => setIsQuotaPopupOpen((open) => !open)}
          >
            {activeQuotas.length > 0 ? (
              <div className="global-status-bar-quota">
                {activeQuotas.map((q) => (
                  <QuotaChip key={q.provider} quota={q} noLimitLabel={t("quota.noLimit")} />
                ))}
              </div>
            ) : null}

            {visibleSnapshots.length > 0 ? (
              <div className="global-status-bar-quota global-status-bar-quota--snapshots">
                {visibleSnapshots.map((s) => (
                  <QuotaSnapshotChip
                    key={s.provider}
                    snap={s}
                    amLabel={t("quota.period.am")}
                    pmLabel={t("quota.period.pm")}
                    resetsLabel={t("quota.resetsLabel")}
                    t={t}
                  />
                ))}
              </div>
            ) : null}
          </button>

          {isQuotaPopupOpen ? (
            <div className="global-status-bar-quota-popup" role="dialog" aria-label={t("quota.monitoring.overview")}>
              <QuotaOverview
                snapshots={quotaSnapshots.filter((snapshot) => quotaEnabledProviders.includes(snapshot.provider))}
                onRefresh={() => onRefreshQuota?.()}
                onRefreshProvider={(provider) => onRefreshQuota?.(provider)}
                storageKey="quota-popup-active-provider"
              />
            </div>
          ) : null}
        </div>
      ) : null}
    </div>
  );
}
