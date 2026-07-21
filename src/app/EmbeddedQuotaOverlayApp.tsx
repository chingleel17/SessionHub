import { useEffect, useState } from "react";
import { useQuery, useQueryClient } from "@tanstack/react-query";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

import type { AppSettings, QuotaSnapshot } from "../types";
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

      return () => {
        unlistenSnapshots();
        unlistenSettings();
        unlistenLock();
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
      enabledProviders={settings?.quotaEnabledProviders ?? ["claude", "copilot", "opencode", "codex", "antigravity"]}
      selectedProviders={settings?.quotaOverlayProviders ?? []}
      opacity={settings?.quotaOverlayOpacity ?? 0.3}
      locked={settings?.quotaOverlayLocked ?? true}
      theme={settings?.quotaOverlayTheme ?? "dark"}
      styleMode={settings?.quotaOverlayStyle ?? "compact"}
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
