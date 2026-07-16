import { useI18n } from "../i18n/I18nProvider";
import type { QuotaSnapshot, QuotaWindow } from "../types";
import { localizedWindowLabel } from "../utils/quotaWindowLabel";
import { RefreshIcon, SettingsIcon } from "./Icons";
import { IconButton } from "./ui/IconButton";

type TrayQuotaPanelProps = {
  snapshots: QuotaSnapshot[];
  onRefresh: () => void;
  onOpenSettings: () => void;
};

const PROVIDER_ORDER = ["claude", "copilot", "codex", "opencode", "antigravity"] as const;

const PROVIDER_LABELS: Record<string, string> = {
  claude: "Claude",
  copilot: "Copilot",
  codex: "Codex",
  opencode: "OpenCode",
  antigravity: "Antigravity",
};

function formatAge(fetchedAt: string, locale: string): string {
  const diffMs = Date.now() - new Date(fetchedAt).getTime();
  const totalMins = Math.max(0, Math.floor(diffMs / 60000));
  if (totalMins < 1) return locale === "zh-TW" ? "剛剛" : "just now";
  if (totalMins < 60) return locale === "zh-TW" ? `${totalMins} 分鐘前` : `${totalMins}m ago`;
  const hours = Math.floor(totalMins / 60);
  if (hours < 24) return locale === "zh-TW" ? `${hours} 小時前` : `${hours}h ago`;
  const days = Math.floor(hours / 24);
  return locale === "zh-TW" ? `${days} 天前` : `${days}d ago`;
}

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

function getBarTone(utilization: number): "ok" | "warn" | "danger" {
  if (utilization >= 0.8) return "danger";
  if (utilization >= 0.5) return "warn";
  return "ok";
}

function sortSnapshots(snapshots: QuotaSnapshot[]): QuotaSnapshot[] {
  return [...snapshots]
    .filter((snapshot) => snapshot.status !== "unsupported")
    .sort((left, right) => {
      const leftIndex = PROVIDER_ORDER.indexOf(left.provider as (typeof PROVIDER_ORDER)[number]);
      const rightIndex = PROVIDER_ORDER.indexOf(right.provider as (typeof PROVIDER_ORDER)[number]);
      const leftRank = leftIndex === -1 ? Number.MAX_SAFE_INTEGER : leftIndex;
      const rightRank = rightIndex === -1 ? Number.MAX_SAFE_INTEGER : rightIndex;
      return leftRank - rightRank || left.provider.localeCompare(right.provider);
    });
}

function WindowRow({ provider, window }: { provider: string; window: QuotaWindow }) {
  const { t } = useI18n();
  const utilization = Math.max(0, Math.min(100, window.utilization * 100));
  const tone = getBarTone(window.utilization);
  const countdown = formatResetCountdown(
    window.resetsAt,
    t("quota.unit.day"),
    t("quota.unit.hour"),
    t("quota.unit.minute"),
    t("quota.resetDone"),
  );

  return (
    <div className="tray-panel-window" data-tone={tone}>
      <div className="tray-panel-window-top">
        <span className="tray-panel-window-label">
          {localizedWindowLabel(provider, window.windowKey, window.label, t)}
        </span>
        <span className="tray-panel-window-value">{Math.round(utilization)}%</span>
      </div>
      <div className="tray-panel-bar">
        <div className="tray-panel-bar-fill" style={{ width: `${utilization}%` }} />
      </div>
      {countdown ? <div className="tray-panel-window-reset">{t("quota.resetsIn", { countdown })}</div> : null}
    </div>
  );
}

export function TrayQuotaPanel({ snapshots, onRefresh, onOpenSettings }: TrayQuotaPanelProps) {
  const { t, locale } = useI18n();
  const visibleSnapshots = sortSnapshots(snapshots);
  const latestFetchedAt = visibleSnapshots
    .map((snapshot) => snapshot.fetchedAt)
    .sort((left, right) => new Date(right).getTime() - new Date(left).getTime())[0];

  return (
    <div className="tray-panel-window-root">
      <section className="tray-panel-root">
        <header className="tray-panel-header">
          <div>
            <h1 className="tray-panel-title">SessionHub Quota</h1>
            <p className="tray-panel-subtitle">{t("quota.monitoring.overview")}</p>
          </div>
          <IconButton label={t("settings.title")} className="tray-panel-icon-button" onClick={onOpenSettings}>
            <SettingsIcon size={14} />
          </IconButton>
        </header>

        <div className="tray-panel-content">
          {visibleSnapshots.map((snapshot) => (
            <article className="tray-panel-provider" key={snapshot.provider}>
              <div className="tray-panel-provider-header">
                <span className="tray-panel-provider-name">{PROVIDER_LABELS[snapshot.provider] ?? snapshot.provider}</span>
                <span className={`tray-panel-provider-source tray-panel-provider-source--${snapshot.source}`}>
                  {t(snapshot.source === "remote_api" ? "quota.monitoring.source.remote_api" : "quota.monitoring.source.local_scan")}
                </span>
              </div>

              {snapshot.status === "ok" || snapshot.status === "rate_limited" ? (
                <>
                  {snapshot.windows?.map((window) => (
                    <WindowRow key={`${snapshot.provider}-${window.windowKey}`} provider={snapshot.provider} window={window} />
                  ))}

                  {snapshot.localTokens ? (
                    <div className="tray-panel-local-usage">
                      <span>{snapshot.localTokens.periodLabel}</span>
                      <strong>
                        {Math.round(snapshot.localTokens.inputTokens / 1000)}k / {Math.round(snapshot.localTokens.outputTokens / 1000)}k
                      </strong>
                    </div>
                  ) : null}

                  {snapshot.status === "rate_limited" ? (
                    <div className="tray-panel-provider-note">{t("quota.overlay.stale")}</div>
                  ) : null}
                </>
              ) : (
                <div className="tray-panel-provider-note tray-panel-provider-note--error">
                  {snapshot.status === "no_auth"
                    ? t("quota.pleaseLogin", { provider: PROVIDER_LABELS[snapshot.provider] ?? snapshot.provider })
                    : snapshot.errorMessage || t("quota.monitoring.status.error")}
                </div>
              )}
            </article>
          ))}

          {visibleSnapshots.length === 0 ? <div className="tray-panel-empty">{t("quota.monitoring.noData")}</div> : null}
        </div>

        <footer className="tray-panel-footer">
          <IconButton label={t("quota.monitoring.manualRefresh")} className="tray-panel-refresh" onClick={onRefresh}>
            <RefreshIcon size={16} />
          </IconButton>
          <span className="tray-panel-last-updated">
            {latestFetchedAt ? t("quota.updated", { age: formatAge(latestFetchedAt, locale) }) : t("quota.monitoring.noData")}
          </span>
        </footer>
      </section>
    </div>
  );
}
