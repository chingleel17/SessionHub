import { useEffect, useRef } from "react";
import type { CSSProperties } from "react";

import { LogicalSize, PhysicalSize } from "@tauri-apps/api/dpi";
import { getCurrentWindow } from "@tauri-apps/api/window";

import { useI18n } from "../i18n/I18nProvider";
import type { OverlayStyle, QuotaSnapshot } from "../types";
import { localizedWindowLabel } from "../utils/quotaWindowLabel";
import { LockIcon, MoveIcon } from "./Icons";
import { IconButton } from "./ui/IconButton";

type QuotaOverlayProps = {
  snapshots: QuotaSnapshot[];
  enabledProviders: string[];
  selectedProviders: string[];
  opacity: number;
  locked: boolean;
  theme: "dark" | "light";
  styleMode: OverlayStyle;
  onLockToggle?: () => void;
};

const PROVIDER_ORDER = ["claude", "copilot", "codex", "opencode", "antigravity"] as const;

const PROVIDER_LABELS: Record<string, string> = {
  claude: "Claude",
  copilot: "Copilot",
  codex: "Codex",
  opencode: "OpenCode",
  antigravity: "Antigravity",
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

function toneColor(tone: "ok" | "warn" | "danger"): string {
  if (tone === "danger") return "var(--quota-bar-danger)";
  if (tone === "warn") return "var(--quota-bar-warning)";
  return "var(--quota-bar-ok)";
}

function getVisibleRows(
  snapshots: QuotaSnapshot[],
  enabledProviders: string[],
  selectedProviders: string[],
): QuotaSnapshot[] {
  const activeProviders = selectedProviders.length > 0 ? selectedProviders : enabledProviders;
  return snapshots
    .filter((snapshot) => activeProviders.includes(snapshot.provider) && snapshot.status !== "unsupported")
    .sort((left, right) => {
      const leftIndex = PROVIDER_ORDER.indexOf(left.provider as (typeof PROVIDER_ORDER)[number]);
      const rightIndex = PROVIDER_ORDER.indexOf(right.provider as (typeof PROVIDER_ORDER)[number]);
      const leftRank = leftIndex === -1 ? Number.MAX_SAFE_INTEGER : leftIndex;
      const rightRank = rightIndex === -1 ? Number.MAX_SAFE_INTEGER : rightIndex;
      return leftRank - rightRank || left.provider.localeCompare(right.provider);
    });
}

/** 取 snapshot 的主要 window 用量（windows[0]，與 StatusBar / QuotaOverview 一致），無資料回 null */
function getPrimaryUtilization(snapshot: QuotaSnapshot): number | null {
  const windows = snapshot.windows ?? [];
  if (windows.length === 0) return null;
  return windows[0].utilization;
}

/** 精簡模式的迷你圓環（與狀態列 QuotaRing 同款視覺） */
function OverlayRing({ utilization }: { utilization: number }) {
  const clamped = Math.min(Math.max(utilization, 0), 1);
  const size = 14;
  const stroke = 2;
  const radius = (size - stroke) / 2;
  const circumference = 2 * Math.PI * radius;
  const dash = clamped * circumference;
  const color = toneColor(getBarTone(clamped));
  return (
    <svg width={size} height={size} viewBox={`0 0 ${size} ${size}`} className="quota-overlay-ring" aria-hidden="true">
      <circle cx={size / 2} cy={size / 2} r={radius} fill="none" stroke="currentColor" strokeOpacity="0.22" strokeWidth={stroke} />
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

export function QuotaOverlay({
  snapshots,
  enabledProviders,
  selectedProviders,
  opacity,
  locked,
  theme,
  styleMode,
  onLockToggle,
}: QuotaOverlayProps) {
  const { t } = useI18n();
  const visibleSnapshots = getVisibleRows(snapshots, enabledProviders, selectedProviders);
  const wrapperRef = useRef<HTMLDivElement | null>(null);

  // 視窗尺寸貼合內容：量測 wrapper 實際尺寸並同步到原生視窗，避免出現滾動條
  useEffect(() => {
    const wrapper = wrapperRef.current;
    if (!wrapper) return undefined;

    let lastWidth = 0;
    let lastHeight = 0;

    const syncWindowSize = () => {
      const width = Math.ceil(wrapper.scrollWidth);
      const height = Math.ceil(wrapper.scrollHeight);
      if (width === 0 || height === 0) return;
      if (width === lastWidth && height === lastHeight) return;
      lastWidth = width;
      lastHeight = height;
      void getCurrentWindow().setSize(new LogicalSize(width, height));
    };

    syncWindowSize();
    const observer = new ResizeObserver(syncWindowSize);
    observer.observe(wrapper);
    const mutationObserver = new MutationObserver(syncWindowSize);
    mutationObserver.observe(wrapper, { childList: true, subtree: true, characterData: true });
    return () => {
      observer.disconnect();
      mutationObserver.disconnect();
    };
  }, [styleMode, locked, visibleSnapshots.length]);

  useEffect(() => {
    const refreshTransparentWebview = async () => {
      const currentWindow = getCurrentWindow();
      const size = await currentWindow.innerSize();
      await currentWindow.setSize(new PhysicalSize(size.width + 1, size.height));
      await currentWindow.setSize(size);
    };

    void requestAnimationFrame(() => {
      void refreshTransparentWebview();
    });
  }, [opacity, theme, styleMode, locked, selectedProviders]);

  const isCompact = styleMode === "compact";

  return (
    <div className="quota-overlay-window" ref={wrapperRef}>
      <section
        className={`quota-overlay-root quota-overlay-root--${theme}${locked ? "" : " quota-overlay-editing"}${isCompact ? " quota-overlay-root--compact" : ""}`}
        style={{ "--quota-overlay-opacity": Math.min(Math.max(opacity, 0.3), 1) } as CSSProperties}
        data-tauri-drag-region={!locked ? true : undefined}
        onMouseDown={(event) => {
          if (locked || event.button !== 0 || (event.target as HTMLElement).closest("button")) return;
          void getCurrentWindow().startDragging();
        }}
      >
        {!locked ? (
          <div className="quota-overlay-edit-controls">
            <span className="quota-overlay-move-icon" aria-label={t("quota.overlay.editMode")} title={t("quota.overlay.editMode")}>
              <MoveIcon size={14} />
            </span>
            <IconButton label={t("quota.overlay.lock")} className="quota-overlay-lock-button quota-overlay-no-drag" onClick={onLockToggle}>
              <LockIcon size={15} />
            </IconButton>
          </div>
        ) : null}

        {isCompact ? (
          <div className="quota-overlay-compact-list">
            {visibleSnapshots.map((snapshot) => {
              const abbr = PROVIDER_ABBR[snapshot.provider] ?? snapshot.provider.slice(0, 2).toUpperCase();
              const primaryUtilization = getPrimaryUtilization(snapshot);
              const hasData = snapshot.status === "ok" || snapshot.status === "rate_limited";
              const tooltip = [
                PROVIDER_LABELS[snapshot.provider] ?? snapshot.provider,
                ...(snapshot.windows ?? []).map((window) => {
                  const label = localizedWindowLabel(snapshot.provider, window.windowKey, window.label, t);
                  return `${label}: ${Math.round(window.utilization * 100)}%`;
                }),
              ].join("\n");

              return (
                <span className="quota-overlay-chip" key={snapshot.provider} title={tooltip}>
                  <span className="quota-overlay-chip-abbr" style={{ color: PROVIDER_COLOR[snapshot.provider] }}>
                    {abbr}
                  </span>
                  {hasData && primaryUtilization !== null ? (
                    <>
                      <OverlayRing utilization={primaryUtilization} />
                      <span className="quota-overlay-chip-pct" style={{ color: toneColor(getBarTone(primaryUtilization)) }}>
                        {Math.round(primaryUtilization * 100)}%
                      </span>
                    </>
                  ) : (
                    <span className="quota-overlay-chip-pct quota-overlay-meta--muted">–</span>
                  )}
                </span>
              );
            })}
            {visibleSnapshots.length === 0 ? <div className="quota-overlay-empty">{t("quota.monitoring.noData")}</div> : null}
          </div>
        ) : (
          <div className="quota-overlay-list">
            {visibleSnapshots.map((snapshot) => {
              const providerLabel = PROVIDER_LABELS[snapshot.provider] ?? snapshot.provider;
              const windows = snapshot.windows ?? [];

              return (
                <article className="quota-overlay-provider" key={snapshot.provider}>
                  <div className="quota-overlay-provider-header">
                    <span className="quota-overlay-provider-name">{providerLabel}</span>
                    <span className="quota-overlay-provider-source">
                      {t(snapshot.source === "remote_api" ? "quota.monitoring.source.remote_api" : "quota.monitoring.source.local_scan")}
                    </span>
                  </div>

                  {snapshot.status === "ok" || snapshot.status === "rate_limited" ? (
                    windows.length > 0 ? (
                      windows.map((window) => {
                        const utilization = window.utilization;
                        const tone = getBarTone(utilization);
                        const countdown = formatResetCountdown(
                          window.resetsAt,
                          t("quota.unit.day"),
                          t("quota.unit.hour"),
                          t("quota.unit.minute"),
                          t("quota.resetDone"),
                        );
                        const metaText = [countdown, snapshot.status === "rate_limited" ? t("quota.overlay.stale") : ""]
                          .filter(Boolean)
                          .join(" · ");
                        return (
                          <div className="quota-overlay-row" key={`${snapshot.provider}-${window.windowKey}`} data-tone={tone}>
                            <div className="quota-overlay-labels">
                              <span className="quota-overlay-name">
                                {localizedWindowLabel(snapshot.provider, window.windowKey, window.label, t)}
                              </span>
                              {metaText ? <span className="quota-overlay-meta">{metaText}</span> : null}
                            </div>

                            <div className="quota-overlay-bar">
                              <div className="quota-overlay-bar-fill" style={{ width: `${Math.max(0, Math.min(100, utilization * 100))}%` }} />
                            </div>
                            <span className="quota-overlay-value">{Math.round(utilization * 100)}%</span>
                          </div>
                        );
                      })
                    ) : (
                      <span className="quota-overlay-local-summary">
                        {snapshot.localTokens
                          ? `${snapshot.localTokens.periodLabel} · ${Math.round((snapshot.localTokens.inputTokens + snapshot.localTokens.outputTokens) / 1000)}k`
                          : t("quota.monitoring.source.local_scan")}
                      </span>
                    )
                  ) : (
                    <div className="quota-overlay-provider-note quota-overlay-meta--muted">
                      {snapshot.status === "no_auth"
                        ? t("quota.monitoring.status.no_auth")
                        : snapshot.errorMessage || t("quota.monitoring.status.error")}
                    </div>
                  )}
                </article>
              );
            })}

            {visibleSnapshots.length === 0 ? <div className="quota-overlay-empty">{t("quota.monitoring.noData")}</div> : null}
          </div>
        )}
      </section>
    </div>
  );
}
