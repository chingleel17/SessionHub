import { useI18n } from "../i18n/I18nProvider";
import type { BridgeEventLogEntry } from "../types";

type Props = {
  lastBridgeEvent: { entry: BridgeEventLogEntry; receivedAt: Date } | null;
  onOpenEventMonitor: () => void;
  activeSessions: number;
  waitingSessions: number;
  isLoadingSessions: boolean;
};

const STATUS_COLORS: Record<string, string> = {
  targeted: "var(--color-blue, #58a6ff)",
  fallback: "var(--color-yellow, #d29922)",
  full_refresh: "var(--color-green, #3fb950)",
  skipped_dedup: "var(--color-muted, #8b949e)",
  skipped_rate_limit: "var(--color-muted, #8b949e)",
};

function truncateCwd(cwd: string, maxLen = 40): string {
  if (cwd.length <= maxLen) return cwd;
  return "…" + cwd.slice(-(maxLen - 1));
}

export function StatusBar({
  lastBridgeEvent,
  onOpenEventMonitor,
  activeSessions,
  waitingSessions,
  isLoadingSessions,
}: Props) {
  const { t } = useI18n();

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

      <div className="global-status-bar-right">
        <span
          className={`global-status-bar-count${activeSessions === 0 ? " global-status-bar-count--zero" : ""}`}
        >
          ▶ {isLoadingSessions ? "-" : activeSessions} {t("statusBar.active")}
        </span>
        <span
          className={`global-status-bar-count${waitingSessions === 0 ? " global-status-bar-count--zero" : ""}`}
        >
          ⏳ {isLoadingSessions ? "-" : waitingSessions} {t("statusBar.waiting")}
        </span>
      </div>
    </div>
  );
}
