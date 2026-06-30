import { useState } from "react";
import type { QuotaSnapshot, QuotaWindow } from "../types";

// ─── helpers ────────────────────────────────────────────────────────────────

function formatResetCountdown(resetsAt: string | null | undefined): string {
  if (!resetsAt) return "";
  const diffMs = new Date(resetsAt).getTime() - Date.now();
  if (diffMs <= 0) return "已重置";
  const totalMins = Math.floor(diffMs / 60000);
  const days = Math.floor(totalMins / 1440);
  const hours = Math.floor((totalMins % 1440) / 60);
  const mins = totalMins % 60;
  if (days > 0) return `${days}d ${hours}h`;
  if (hours > 0) return `${hours}h ${mins}m`;
  return `${mins}m`;
}

function formatResetDateTime(resetsAt: string | null | undefined): string {
  if (!resetsAt) return "";
  try {
    const d = new Date(resetsAt);
    const MM = String(d.getMonth() + 1).padStart(2, "0");
    const DD = String(d.getDate()).padStart(2, "0");
    const hh = String(d.getHours()).padStart(2, "0");
    const mm = String(d.getMinutes()).padStart(2, "0");
    return `${MM}/${DD} ${hh}:${mm}`;
  } catch {
    return "";
  }
}

function formatAge(fetchedAt: string): string {
  const diffMs = Date.now() - new Date(fetchedAt).getTime();
  const totalMins = Math.floor(diffMs / 60000);
  if (totalMins < 1) return "剛剛";
  if (totalMins < 60) return `${totalMins} 分鐘前`;
  const hours = Math.floor(totalMins / 60);
  if (hours < 24) return `${hours} 小時前`;
  return `${Math.floor(hours / 24)} 天前`;
}

function formatTokens(n: number): string {
  if (n >= 1_000_000_000) return `${(n / 1_000_000_000).toFixed(1)}B`;
  if (n >= 1_000_000) return `${(n / 1_000_000).toFixed(1)}M`;
  if (n >= 1_000) return `${(n / 1_000).toFixed(0)}k`;
  return `${n}`;
}

// utilisation 0.0–1.0 → bar colour token
function barColor(pct: number): string {
  if (pct >= 0.9) return "var(--quota-bar-danger)";
  if (pct >= 0.7) return "var(--quota-bar-warning)";
  return "var(--quota-bar-ok)";
}

// window key → display label
function windowLabel(key: string): string {
  const map: Record<string, string> = {
    five_hour: "Session",
    seven_day: "Weekly",
    seven_day_sonnet: "Weekly · Sonnet",
    seven_day_opus: "Weekly · Opus",
  };
  return map[key] ?? key;
}

// provider display name
const PROVIDER_LABELS: Record<string, string> = {
  claude: "Claude",
  copilot: "Copilot",
  opencode: "OpenCode",
  codex: "Codex",
};

// ─── sub-components ──────────────────────────────────────────────────────────

function UtilisationBar({ pct }: { pct: number }) {
  const clamped = Math.min(Math.max(pct, 0), 1);
  const color = barColor(clamped);
  return (
    <div className="qo-bar-track">
      <div
        className="qo-bar-fill"
        style={{ width: `${clamped * 100}%`, background: color }}
      />
    </div>
  );
}

function WindowRow({ w }: { w: QuotaWindow }) {
  const pct = w.utilization;
  const countdown = formatResetCountdown(w.resetsAt);
  const datetime = formatResetDateTime(w.resetsAt);
  return (
    <div className="qo-window" data-key={w.windowKey}>
      <div className="qo-window-label">{windowLabel(w.windowKey)}</div>
      <UtilisationBar pct={pct} />
      <div className="qo-window-meta">
        <span className="qo-pct">{Math.round(pct * 100)}% used</span>
        {countdown ? (
          <span className="qo-reset" title={datetime || undefined}>
            Resets in {countdown}
            {datetime ? <span className="qo-reset-datetime"> · {datetime}</span> : null}
          </span>
        ) : null}
      </div>
    </div>
  );
}

function ProviderPanel({ snap }: { snap: QuotaSnapshot }) {
  const isOk = snap.status === "ok";
  const windows = snap.windows ?? [];

  return (
    <div className="qo-panel">
      {/* header */}
      <div className="qo-panel-header">
        <div className="qo-panel-title-row">
          <span className="qo-panel-name">{PROVIDER_LABELS[snap.provider] ?? snap.provider}</span>
          <span className={`qo-source-badge qo-source-badge--${snap.source}`}>
            {snap.source === "remote_api" ? "API" : "本地估算"}
          </span>
        </div>
        <div className="qo-panel-sub">
          {snap.fetchedAt ? `Updated ${formatAge(snap.fetchedAt)}` : ""}
        </div>
      </div>

      <div className="qo-divider" />

      {/* windows */}
      {isOk && windows.length > 0 ? (
        <div className="qo-windows">
          {windows.map((w) => (
            <WindowRow key={w.windowKey} w={w} />
          ))}
        </div>
      ) : null}

      {/* local tokens */}
      {isOk && snap.source === "local_scan" && snap.localTokens ? (
        <div className="qo-local">
          <div className="qo-local-row">
            <span className="qo-local-label">本月用量（估算）</span>
            <span className="qo-local-value">
              {formatTokens(snap.localTokens.inputTokens + snap.localTokens.outputTokens)} tok
            </span>
          </div>
          <div className="qo-local-period">{snap.localTokens.periodLabel}</div>
        </div>
      ) : null}

      {/* extra credits */}
      {isOk && snap.extraCredits?.isEnabled ? (
        <div className="qo-extra">
          <div className="qo-extra-row">
            <span className="qo-extra-label">超額用量</span>
            <span className="qo-extra-value">
              ${snap.extraCredits.usedCredits.toFixed(2)}
              {snap.extraCredits.monthlyLimit
                ? ` / $${(snap.extraCredits.monthlyLimit / 100).toFixed(0)}`
                : ""}
            </span>
          </div>
        </div>
      ) : null}

      {/* error / auth states */}
      {snap.status === "no_auth" ? (
        <p className="qo-hint">請登入 {PROVIDER_LABELS[snap.provider] ?? snap.provider}</p>
      ) : null}
      {snap.status === "unsupported" ? (
        <p className="qo-hint qo-hint--muted">不支援 quota 查詢</p>
      ) : null}
      {snap.status === "error" && snap.errorMessage ? (
        <p className="qo-hint qo-hint--error" title={snap.errorMessage}>
          {snap.errorMessage.slice(0, 120)}
        </p>
      ) : null}
    </div>
  );
}

// ─── main export ─────────────────────────────────────────────────────────────

interface Props {
  snapshots: QuotaSnapshot[];
  onRefresh?: () => void;
  onRefreshProvider?: (provider: string) => void;
}

const STORAGE_KEY = "quota-overview-active-provider";

export function QuotaOverview({ snapshots, onRefresh, onRefreshProvider }: Props) {
  const visible = snapshots.filter(
    (s) => s.status !== "unsupported" || s.source !== "remote_api"
  );

  const [activeProvider, setActiveProvider] = useState<string>(() => {
    const saved = localStorage.getItem(STORAGE_KEY);
    if (saved && visible.some((s) => s.provider === saved)) return saved;
    return visible[0]?.provider ?? "";
  });

  // Keep activeProvider valid when visible changes (e.g. snapshots first load)
  const resolvedProvider =
    visible.some((s) => s.provider === activeProvider)
      ? activeProvider
      : (visible[0]?.provider ?? "");

  const handleTabClick = (provider: string) => {
    setActiveProvider(provider);
    localStorage.setItem(STORAGE_KEY, provider);
  };

  const active = visible.find((s) => s.provider === resolvedProvider) ?? visible[0];

  if (!active) return null;

  return (
    <div className="qo-root">
      {/* provider tabs */}
      {visible.length > 1 ? (
        <div className="qo-tabs">
          {visible.map((snap) => (
            <button
              key={snap.provider}
              type="button"
              className={`qo-tab${snap.provider === resolvedProvider ? " qo-tab--active" : ""} qo-tab--${snap.provider}`}
              onClick={() => handleTabClick(snap.provider)}
            >
              <span className="qo-tab-icon">{providerIcon(snap.provider)}</span>
              <span className="qo-tab-label">{PROVIDER_LABELS[snap.provider] ?? snap.provider}</span>
              {snap.status === "ok" && snap.windows && snap.windows[0] ? (
                <span className="qo-tab-pct">
                  {Math.round(snap.windows[0].utilization * 100)}%
                </span>
              ) : null}
            </button>
          ))}
        </div>
      ) : null}

      {/* active panel */}
      <ProviderPanel snap={active} />

      {/* footer actions */}
      <div className="qo-footer">
        <button
          type="button"
          className="qo-refresh-btn"
          onClick={() => {
            onRefreshProvider?.(active.provider);
            onRefresh?.();
          }}
          title="重新整理此 provider"
        >
          ↻
        </button>
      </div>
    </div>
  );
}

function providerIcon(provider: string): string {
  const icons: Record<string, string> = {
    claude: "✳",
    copilot: "◎",
    opencode: "◈",
    codex: "◇",
  };
  return icons[provider] ?? "◆";
}
