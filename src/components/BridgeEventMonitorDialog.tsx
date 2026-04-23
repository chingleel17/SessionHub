import { useEffect, useRef, useState } from "react";
import { useI18n } from "../i18n/I18nProvider";
import type { BridgeEventLogEntry } from "../types";

type Props = {
  events: BridgeEventLogEntry[];
  onClose: () => void;
  onClear: () => void;
};

function statusClass(status: string): string {
  switch (status) {
    case "targeted": return "event-log-status event-log-status--targeted";
    case "fallback": return "event-log-status event-log-status--fallback";
    case "full_refresh": return "event-log-status event-log-status--full-refresh";
    case "skipped_dedup":
    case "skipped_rate_limit": return "event-log-status event-log-status--skipped";
    default: return "event-log-status";
  }
}

function statusKey(status: string): string {
  switch (status) {
    case "targeted": return "eventMonitor.status.targeted";
    case "fallback": return "eventMonitor.status.fallback";
    case "full_refresh": return "eventMonitor.status.full_refresh";
    case "skipped_dedup": return "eventMonitor.status.skipped_dedup";
    case "skipped_rate_limit": return "eventMonitor.status.skipped_rate_limit";
    default: return "eventMonitor.status.full_refresh";
  }
}

function formatTime(isoOrRaw: string): string {
  try {
    return new Date(isoOrRaw).toLocaleTimeString("zh-TW", { hour12: false });
  } catch {
    return isoOrRaw;
  }
}

export function BridgeEventMonitorDialog({ events, onClose, onClear }: Props) {
  const { t } = useI18n();
  const listRef = useRef<HTMLDivElement>(null);
  const panelRef = useRef<HTMLDivElement>(null);

  // Initial position: centered, slightly upper-middle of viewport
  const [pos, setPos] = useState<{ x: number; y: number }>(() => ({
    x: Math.max(0, window.innerWidth / 2 - 430),
    y: Math.max(0, window.innerHeight / 2 - 300),
  }));

  const dragRef = useRef<{ startX: number; startY: number; origX: number; origY: number } | null>(null);

  // 新事件到達時捲動到頂部（最新事件在頂）
  useEffect(() => {
    if (listRef.current) {
      listRef.current.scrollTop = 0;
    }
  }, [events]);

  function onHeaderMouseDown(e: React.MouseEvent<HTMLDivElement>) {
    // 只回應左鍵，且不在按鈕上觸發
    if (e.button !== 0) return;
    if ((e.target as HTMLElement).closest("button")) return;
    e.preventDefault();
    dragRef.current = { startX: e.clientX, startY: e.clientY, origX: pos.x, origY: pos.y };

    function onMouseMove(ev: MouseEvent) {
      if (!dragRef.current) return;
      const newX = dragRef.current.origX + (ev.clientX - dragRef.current.startX);
      const newY = dragRef.current.origY + (ev.clientY - dragRef.current.startY);
      // 限制不超出視窗範圍
      const panel = panelRef.current;
      const maxX = panel ? window.innerWidth - panel.offsetWidth : newX;
      const maxY = panel ? window.innerHeight - panel.offsetHeight : newY;
      setPos({
        x: Math.max(0, Math.min(newX, maxX)),
        y: Math.max(0, Math.min(newY, maxY)),
      });
    }

    function onMouseUp() {
      dragRef.current = null;
      window.removeEventListener("mousemove", onMouseMove);
      window.removeEventListener("mouseup", onMouseUp);
    }

    window.addEventListener("mousemove", onMouseMove);
    window.addEventListener("mouseup", onMouseUp);
  }

  return (
    <div
      ref={panelRef}
      className="event-monitor-panel"
      style={{ left: pos.x, top: pos.y }}
    >
      <div
        className="event-monitor-header"
        onMouseDown={onHeaderMouseDown}
      >
        <span className="event-monitor-title">{t("eventMonitor.title")}</span>
        <div className="event-monitor-header-actions">
          <button
            type="button"
            className="ghost-button event-monitor-clear-btn"
            onClick={onClear}
          >
            {t("eventMonitor.clearButton")}
          </button>
          <button type="button" className="ghost-button" onClick={onClose}>
            {t("eventMonitor.closeButton")}
          </button>
        </div>
      </div>

      <div className="event-log-list" ref={listRef}>
        {events.length === 0 ? (
          <p className="event-log-empty">{t("eventMonitor.noEvents")}</p>
        ) : (
          [...events].reverse().map((entry) => (
            <div key={entry.id} className={`event-log-entry event-log-entry--${entry.provider}`}>
              <span className="event-log-time">{formatTime(entry.timestamp)}</span>
              <span className="event-log-provider">{entry.provider}</span>
              <span className="event-log-type">{entry.eventType}</span>
              <span className={statusClass(entry.status)}>
                {t(statusKey(entry.status) as Parameters<typeof t>[0])}
              </span>
              {entry.cwd && (
                <span className="event-log-cwd" title={entry.cwd}>
                  {entry.cwd}
                </span>
              )}
              {entry.error && (
                <span className="event-log-error" title={entry.error}>
                  ⚠ {entry.error}
                </span>
              )}
            </div>
          ))
        )}
      </div>
    </div>
  );
}
