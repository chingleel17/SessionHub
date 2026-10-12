import { useEffect, useState } from "react";
import { useQuery, useQueryClient } from "@tanstack/react-query";
import { invoke } from "@tauri-apps/api/core";
import { emit, listen } from "@tauri-apps/api/event";

import type { AppSettings, InterventionItem, QuotaSnapshot } from "../types";
import { PROVIDER_DISPLAY_ORDER } from "../utils/providerOrder";
import { QuotaOverlay } from "../components/QuotaOverlay";

export function EmbeddedQuotaOverlayApp() {
  const queryClient = useQueryClient();
  const [overlayRevision, setOverlayRevision] = useState(0);
  const settingsQuery = useQuery({
    queryKey: ["embedded_settings", "quota_overlay"],
    queryFn: () => invoke<AppSettings>("get_settings"),
    // Overlay 是獨立動態 webview；輪詢本機設定避免建立時機造成事件遺漏。
    refetchInterval: 1_000,
  });
  const quotaSnapshotQuery = useQuery({
    queryKey: ["embedded_quota_snapshots", "quota_overlay"],
    queryFn: () => invoke<QuotaSnapshot[]>("get_quota_snapshots"),
    staleTime: 60_000,
    refetchInterval: 15_000,
  });
  // 等待處理清單與處理中數量由後端合併 hook 事件與 herdr 偵測狀態後回傳（herdr 認得的 session 以它為準）。
  // 事件只用來觸發立即重新對帳；herdr 狀態沒有事件可訂閱，因此也定期輪詢。
  const activityQuery = useQuery({
    queryKey: ["embedded_overlay_activity", "quota_overlay"],
    queryFn: () => invoke<{ items: InterventionItem[]; activeCount: number }>("get_overlay_activity"),
    staleTime: 1_000,
    refetchInterval: 3_000,
  });

  useEffect(() => {
    let mounted = true;

    const setup = async () => {
      const unlistenSnapshots = await listen("quota-snapshots-updated", () => {
        if (mounted) {
          void queryClient.invalidateQueries({ queryKey: ["embedded_quota_snapshots", "quota_overlay"] });
          void queryClient.refetchQueries({ queryKey: ["embedded_quota_snapshots", "quota_overlay"] });
        }
      });
      const unlistenSettings = await listen<AppSettings>("quota-overlay-settings-changed", (event) => {
        if (mounted) {
          queryClient.setQueryData(["embedded_settings", "quota_overlay"], event.payload);
          // 重新讀取持久化設定，並重新掛載透明 overlay 以確保樣式立即套用。
          void queryClient.invalidateQueries({ queryKey: ["embedded_settings", "quota_overlay"] });
          void queryClient.refetchQueries({ queryKey: ["embedded_settings", "quota_overlay"] });
          setOverlayRevision((revision) => revision + 1);
        }
      });
      const unlistenLock = await listen<boolean>("quota-overlay-locked-changed", (event) => {
        if (mounted) {
          queryClient.setQueryData<AppSettings>(["embedded_settings", "quota_overlay"], (current) =>
            current ? { ...current, quotaOverlayLocked: event.payload } : current,
          );
        }
      });
      const refetchActivity = () => {
        if (mounted) {
          void queryClient.invalidateQueries({ queryKey: ["embedded_overlay_activity", "quota_overlay"] });
        }
      };
      const unlistenInterventionList = await listen("intervention-list-changed", refetchActivity);
      const unlistenActiveCount = await listen("active-session-count-changed", refetchActivity);
      if (mounted) {
        await queryClient.refetchQueries({ queryKey: ["embedded_overlay_activity", "quota_overlay"] });
      }

      return () => {
        unlistenSnapshots();
        unlistenSettings();
        unlistenLock();
        unlistenInterventionList();
        unlistenActiveCount();
      };
    };

    let cleanup: (() => void) | undefined;
    void setup().then((dispose) => {
      cleanup = dispose;
    });

    return () => {
      mounted = false;
      cleanup?.();
    };
  }, [queryClient]);

  const settings = settingsQuery.data;

  return (
    <QuotaOverlay
      key={overlayRevision}
      snapshots={quotaSnapshotQuery.data ?? []}
      enabledProviders={settings?.quotaEnabledProviders ?? [...PROVIDER_DISPLAY_ORDER]}
      selectedProviders={settings?.quotaOverlayProviders ?? []}
      opacity={settings?.quotaOverlayOpacity ?? 0.3}
      locked={settings?.quotaOverlayLocked ?? true}
      theme={settings?.quotaOverlayTheme ?? "dark"}
      styleMode={settings?.quotaOverlayStyle ?? "compact"}
      interventionItems={activityQuery.data?.items ?? []}
      interventionEnabled={settings?.quotaOverlayInterventionEnabled ?? true}
      interventionStyle={settings?.quotaOverlayInterventionStyle ?? "cards"}
      indicatorSide={settings?.quotaOverlayIndicatorSide ?? "left"}
      activeSessionCount={activityQuery.data?.activeCount ?? 0}
      onInterventionCardClick={(sessionId) => {
        void emit("intervention-focus-session", sessionId);
      }}
      onLockToggle={() => {
        if (!settings) return;
        void invoke("save_settings", {
          settings: { ...settings, quotaOverlayLocked: !settings.quotaOverlayLocked },
        }).then(() => {
          void queryClient.invalidateQueries({ queryKey: ["embedded_settings", "quota_overlay"] });
        });
      }}
    />
  );
}
