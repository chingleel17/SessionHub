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
  quotaEnabledProviders?: string[];
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

// provider 品牌代表色（用於縮寫文字上色，不使用商標圖形）
const PROVIDER_COLOR: Record<string, string> = {
  claude: "#D97757",
  copilot: "#57ab5a",
  opencode: "#8957e5",
  codex: "#10a37f",
  antigravity: "#4285F4",
};

// 底部狀態列只顯示 Antigravity 的 Gemini 模型群組視窗，Claude/GPT 群組留給 Dashboard
function statusBarWindowsForSnapshot(snap: QuotaSnapshot) {
  const windows = snap.windows ?? [];
  if (snap.provider !== "antigravity") return windows;
  return windows.filter((w) => (w.group ?? "").toLowerCase().includes("gemini"));
}

// utilisation 0-100 → bar colour token（與 QuotaOverview 的 barColor 門檻/色票一致）
function quotaBarColor(pct: number): string {
  if (pct >= 90) return "var(--quota-bar-danger)";
  if (pct >= 70) return "var(--quota-bar-warning)";
  return "var(--quota-bar-ok)";
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
    <span
      className="global-status-bar-quota-chip"
      title={tooltipLines}
    >
      <span className="global-status-bar-quota-abbr" style={{ color: PROVIDER_COLOR[quota.provider] }}>
        {abbr}
      </span>
      {hasCost && (
        <span className="global-status-bar-quota-cost">${quota.costUsd.toFixed(2)}</span>
      )}
      {hasLimit && (
        <span className="global-status-bar-quota-bar">
          <span
            className="global-status-bar-quota-fill"
            style={{
              width: `${limitPct.toFixed(1)}%`,
              background: quotaBarColor(limitPct),
            }}
          />
        </span>
      )}
    </span>
  );
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

function QuotaSnapshotChip({ snap, amLabel, pmLabel, resetsLabel }: { snap: QuotaSnapshot; amLabel: string; pmLabel: string; resetsLabel: string }) {
  const abbr = PROVIDER_ABBR[snap.provider] ?? snap.provider.slice(0, 2).toUpperCase();
  const windows = statusBarWindowsForSnapshot(snap);
  const topWindow = windows[0];
  const pct = topWindow ? Math.round(topWindow.utilization * 100) : null;
  const tooltip = [
    `${snap.provider} · ${snap.source}`,
    ...windows.map((w) => {
      const wPct = Math.round(w.utilization * 100);
      const reset = w.resetsAt ? ` · ${resetsLabel}: ${formatResetDateTime(w.resetsAt, amLabel, pmLabel)}` : "";
      return `${w.label}: ${wPct}%${reset}`;
    }),
  ].filter(Boolean).join("\n");

  return (
    <span
      className="global-status-bar-quota-chip global-status-bar-quota-chip--snapshot"
      title={tooltip}
    >
      <span className="global-status-bar-quota-abbr" style={{ color: PROVIDER_COLOR[snap.provider] }}>
        {abbr}
      </span>
      {pct !== null ? (
        <>
          <span className="global-status-bar-quota-bar">
            <span
              className="global-status-bar-quota-fill"
              style={{ width: `${Math.min(pct, 100)}%`, background: quotaBarColor(pct) }}
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
  quotaEnabledProviders = [],
}: Props) {
  const { t } = useI18n();
  const dash = isLoadingSessions ? "-" : undefined;
  const activeQuotas = providerQuotas.filter(
    (q) =>
      quotaEnabledProviders.includes(q.provider) &&
      (q.inputTokens > 0 || q.outputTokens > 0 || q.costUsd > 0),
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
      {quotaSnapshots
        .filter((s) => s.status === "ok" && (s.source === "remote_api" || s.provider === "antigravity") && quotaEnabledProviders.includes(s.provider))
        .length > 0 && (
        <div className="global-status-bar-quota global-status-bar-quota--snapshots">
          {quotaSnapshots
            .filter((s) => s.status === "ok" && (s.source === "remote_api" || s.provider === "antigravity") && quotaEnabledProviders.includes(s.provider))
            .map((s) => (
              <QuotaSnapshotChip
                key={s.provider}
                snap={s}
                amLabel={t("quota.period.am")}
                pmLabel={t("quota.period.pm")}
                resetsLabel={t("quota.resetsLabel")}
              />
            ))}
        </div>
      )}
    </div>
  );
}
