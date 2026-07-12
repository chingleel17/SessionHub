import { useState } from "react";
import { useI18n } from "../i18n/I18nProvider";
import type { QuotaSnapshot, QuotaWindow } from "../types";
import { localizedWindowLabel } from "../utils/quotaWindowLabel";

// ─── helpers ────────────────────────────────────────────────────────────────

function formatResetCountdown(
  resetsAt: string | null | undefined,
  dayUnit: string,
  hourUnit: string,
  minuteUnit: string,
  resetDoneLabel: string,
): string {
  if (!resetsAt) return "";
  const diffMs = new Date(resetsAt).getTime() - Date.now();
  if (diffMs <= 0) return resetDoneLabel;
  const totalMins = Math.floor(diffMs / 60000);
  const days = Math.floor(totalMins / 1440);
  const hours = Math.floor((totalMins % 1440) / 60);
  const mins = totalMins % 60;
  if (days > 0) return `${days}${dayUnit}${hours}${hourUnit}`;
  if (hours > 0) return `${hours}${hourUnit}${mins}${minuteUnit}`;
  return `${mins}${minuteUnit}`;
}

function formatResetDateTime(resetsAt: string | null | undefined, amLabel: string, pmLabel: string): string {
  if (!resetsAt) return "";
  try {
    const d = new Date(resetsAt);
    const MM = String(d.getMonth() + 1).padStart(2, "0");
    const DD = String(d.getDate()).padStart(2, "0");
    const rawHours = d.getHours();
    const period = rawHours < 12 ? amLabel : pmLabel;
    const hours12 = rawHours % 12 === 0 ? 12 : rawHours % 12;
    const hh = String(hours12).padStart(2, "0");
    const mm = String(d.getMinutes()).padStart(2, "0");
    return `${MM}/${DD} ${period}${hh}:${mm}`;
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

// provider display name
const PROVIDER_LABELS: Record<string, string> = {
  claude: "Claude",
  copilot: "Copilot",
  opencode: "OpenCode",
  codex: "Codex",
  antigravity: "Antigravity",
};

// 依 QuotaWindow.group 分組（Antigravity 用），無 group 資訊的 provider 回傳單一未命名分組
function groupWindows(windows: QuotaWindow[]): Array<{ group: string | null; windows: QuotaWindow[] }> {
  const byGroup = new Map<string | null, QuotaWindow[]>();
  for (const w of windows) {
    const key = w.group ?? null;
    if (!byGroup.has(key)) byGroup.set(key, []);
    byGroup.get(key)!.push(w);
  }
  return Array.from(byGroup.entries()).map(([group, groupWindows]) => ({ group, windows: groupWindows }));
}

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

function WindowRow({ w, provider }: { w: QuotaWindow; provider: string }) {
  const { t } = useI18n();
  const pct = w.utilization;
  const countdown = formatResetCountdown(
    w.resetsAt,
    t("quota.unit.day"),
    t("quota.unit.hour"),
    t("quota.unit.minute"),
    t("quota.resetDone"),
  );
  const datetime = formatResetDateTime(w.resetsAt, t("quota.period.am"), t("quota.period.pm"));
  return (
    <div className="qo-window" data-key={w.windowKey}>
      <div className="qo-window-label">{localizedWindowLabel(provider, w.windowKey, w.label, t)}</div>
      <UtilisationBar pct={pct} />
      <div className="qo-window-meta">
        <span className="qo-pct">{t("quota.usedPct", { pct: Math.round(pct * 100) })}</span>
        {countdown ? (
          <span className="qo-reset" title={datetime || undefined}>
            {t("quota.resetsIn", { countdown })}
            {datetime ? <span className="qo-reset-datetime"> · {datetime}</span> : null}
          </span>
        ) : null}
      </div>
    </div>
  );
}

function ProviderPanel({ snap }: { snap: QuotaSnapshot }) {
  const { t } = useI18n();
  const isOk = snap.status === "ok";
  const windows = snap.windows ?? [];

  return (
    <div className="qo-panel">
      {/* header */}
      <div className="qo-panel-header">
        <div className="qo-panel-title-row">
          <span className="qo-panel-name">{PROVIDER_LABELS[snap.provider] ?? snap.provider}</span>
          <span className={`qo-source-badge qo-source-badge--${snap.source}`}>
            {t(snap.source === "remote_api" ? "quota.monitoring.source.remote_api" : "quota.monitoring.source.local_scan")}
          </span>
        </div>
        <div className="qo-panel-sub">
          {snap.fetchedAt ? t("quota.updated", { age: formatAge(snap.fetchedAt) }) : ""}
        </div>
      </div>

      <div className="qo-divider" />

      {/* windows */}
      {isOk && windows.length > 0 ? (
        <div className="qo-windows">
          {groupWindows(windows).map(({ group, windows: groupedWindows }) => (
            <div className="qo-window-group" key={group ?? "__default"}>
              {group ? <div className="qo-window-group-title">{group}</div> : null}
              {groupedWindows.map((w) => (
                <WindowRow key={`${group ?? "default"}-${w.windowKey}`} w={w} provider={snap.provider} />
              ))}
            </div>
          ))}
        </div>
      ) : null}

      {/* local tokens */}
      {isOk && snap.source === "local_scan" && snap.localTokens ? (
        <div className="qo-local">
          <div className="qo-local-row">
            <span className="qo-local-label">{t("quota.monitoring.localUsage")}</span>
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
            <span className="qo-extra-label">{t("quota.extraUsage")}</span>
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
        <p className="qo-hint">{t("quota.pleaseLogin", { provider: PROVIDER_LABELS[snap.provider] ?? snap.provider })}</p>
      ) : null}
      {snap.status === "unsupported" ? (
        <p className="qo-hint qo-hint--muted">{t("quota.unsupportedHint")}</p>
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
  const { t } = useI18n();
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
          title={t("quota.refreshProvider")}
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
    antigravity: "⟡",
  };
  return icons[provider] ?? "◆";
}
