import { useI18n } from "../i18n/I18nProvider";
import type { BridgeEventLogEntry, ProviderQuota, QuotaSnapshot } from "../types";

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
  codex: "DX",
};

function truncateCwd(cwd: string, maxLen = 40): string {
  if (cwd.length <= maxLen) return cwd;
  return "…" + cwd.slice(-(maxLen - 1));
}

function formatTokenCount(n: number): string {
  if (n >= 1_000_000) return `${(n / 1_000_000).toFixed(1)}M`;
  if (n >= 1_000) return `${(n / 1_000).toFixed(1)}K`;
  return String(n);
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

  return (
    <span
      className="global-status-bar-quota-chip"
      title={tooltipLines}
    >
      <span className="global-status-bar-quota-abbr">{abbr}</span>
      {hasCost && (
        <span className="global-status-bar-quota-cost">${quota.costUsd.toFixed(2)}</span>
      )}
      {hasLimit && (
        <span className="global-status-bar-quota-bar">
          <span
            className="global-status-bar-quota-fill"
            style={{
              width: `${Math.min(100, (totalTokens / quota.monthlyLimitTokens!) * 100).toFixed(1)}%`,
            }}
          />
        </span>
      )}
    </span>
  );
}

function formatResetDateTime(iso: string | null | undefined): string {
  if (!iso) return "";
  try {
    const d = new Date(iso);
    const MM = String(d.getMonth() + 1).padStart(2, "0");
    const DD = String(d.getDate()).padStart(2, "0");
    const hh = String(d.getHours()).padStart(2, "0");
    const mm = String(d.getMinutes()).padStart(2, "0");
    return `${MM}/${DD} ${hh}:${mm}`;
  } catch {
    return iso;
  }
}

function QuotaSnapshotChip({ snap }: { snap: QuotaSnapshot }) {
  const abbr = PROVIDER_ABBR[snap.provider] ?? snap.provider.slice(0, 2).toUpperCase();
  const topWindow = snap.windows?.[0];
  const pct = topWindow ? Math.round(topWindow.utilization * 100) : null;
  const tooltip = [
    `${snap.provider} · ${snap.source}`,
    topWindow ? `${topWindow.label}: ${pct}%` : null,
    topWindow?.resetsAt ? `Resets: ${formatResetDateTime(topWindow.resetsAt)}` : null,
  ].filter(Boolean).join("\n");

  return (
    <span
      className="global-status-bar-quota-chip global-status-bar-quota-chip--snapshot"
      title={tooltip}
    >
      <span className="global-status-bar-quota-abbr">{abbr}</span>
      {pct !== null ? (
        <>
          <span className="global-status-bar-quota-bar">
            <span
              className="global-status-bar-quota-fill"
              style={{ width: `${Math.min(pct, 100)}%` }}
            />
          </span>
          <span className="global-status-bar-quota-cost">{pct}%</span>
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
}: Props) {
  const { t } = useI18n();
  const dash = isLoadingSessions ? "-" : undefined;
  const activeQuotas = providerQuotas.filter(
    (q) => q.inputTokens > 0 || q.outputTokens > 0 || q.costUsd > 0,
  );

  return (
    <div className="global-status-bar">
      {/* Left: bridge events */}
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
            {lastBridgeEvent.entry.cwd && (
              <span className="global-status-bar-cwd" title={lastBridgeEvent.entry.cwd}>
                {truncateCwd(lastBridgeEvent.entry.cwd)}
              </span>
            )}
          </>
        ) : (
          <span className="global-status-bar-no-event">{t("statusBar.noEvent")}</span>
        )}
      </button>

      {/* Middle: session counts */}
      <div className="global-status-bar-counts">
        <span
          className={`global-status-bar-count${activeSessions === 0 ? " global-status-bar-count--zero" : ""}`}
        >
          ▶ {dash ?? activeSessions} {t("statusBar.active")}
        </span>
        <span
          className={`global-status-bar-count${waitingSessions === 0 ? " global-status-bar-count--zero" : ""}`}
        >
          ⏳ {dash ?? waitingSessions} {t("statusBar.waiting")}
        </span>
        <span
          className={`global-status-bar-count${idleSessions === 0 ? " global-status-bar-count--zero" : ""}`}
        >
          ◌ {dash ?? idleSessions} {t("statusBar.idle")}
        </span>
        <span
          className={`global-status-bar-count${doneSessions === 0 ? " global-status-bar-count--zero" : ""}`}
        >
          ✓ {dash ?? doneSessions} {t("statusBar.done")}
        </span>
      </div>

      {/* Right: provider quota (local aggregates) */}
      {activeQuotas.length > 0 && (
        <div className="global-status-bar-quota">
          {activeQuotas.map((q) => (
            <QuotaChip key={q.provider} quota={q} noLimitLabel={t("quota.noLimit")} />
          ))}
        </div>
      )}

      {/* Right: remote quota snapshots (utilization bars from API) */}
      {quotaSnapshots.filter((s) => s.status === "ok" && s.source === "remote_api").length > 0 && (
        <div className="global-status-bar-quota global-status-bar-quota--snapshots">
          {quotaSnapshots
            .filter((s) => s.status === "ok" && s.source === "remote_api")
            .map((s) => (
              <QuotaSnapshotChip key={s.provider} snap={s} />
            ))}
        </div>
      )}
    </div>
  );
}
