import { useEffect } from "react";
import { useQuery, useQueryClient } from "@tanstack/react-query";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";

import type { AppSettings, QuotaSnapshot } from "../types";
import { PROVIDER_DISPLAY_ORDER } from "../utils/providerOrder";
import { TrayQuotaPanel } from "../components/TrayQuotaPanel";

const DEFAULT_ENABLED_PROVIDERS = [...PROVIDER_DISPLAY_ORDER];

const SNAPSHOT_QUERY_KEY = ["embedded_quota_snapshots", "tray_panel"];
const SETTINGS_QUERY_KEY = ["embedded_settings", "tray_panel"];

export function EmbeddedTrayPanelApp() {
  const queryClient = useQueryClient();
  const quotaSnapshotQuery = useQuery({
    queryKey: SNAPSHOT_QUERY_KEY,
    queryFn: () => invoke<QuotaSnapshot[]>("get_quota_snapshots"),
    staleTime: 0,
  });
  const settingsQuery = useQuery({
    queryKey: SETTINGS_QUERY_KEY,
    queryFn: () => invoke<AppSettings>("get_settings"),
    staleTime: 0,
  });

  useEffect(() => {
    let mounted = true;

    const setup = async () => {
      const unlistenSnapshots = await listen("quota-snapshots-updated", () => {
        if (mounted) {
          void queryClient.invalidateQueries({ queryKey: SNAPSHOT_QUERY_KEY });
        }
      });
      const unlistenSettings = await listen<AppSettings>("quota-overlay-settings-changed", (event) => {
        if (mounted) {
          queryClient.setQueryData(SETTINGS_QUERY_KEY, event.payload);
        }
      });
      const unlistenFocus = await getCurrentWindow().onFocusChanged(({ payload: focused }) => {
        if (mounted && focused) {
          void queryClient.refetchQueries({ queryKey: SNAPSHOT_QUERY_KEY });
          void queryClient.refetchQueries({ queryKey: SETTINGS_QUERY_KEY });
        }
      });

      return () => {
        unlistenSnapshots();
        unlistenSettings();
        unlistenFocus();
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

  useEffect(() => {
    const handleKeyDown = (event: KeyboardEvent) => {
      if (event.key === "Escape") {
        void getCurrentWindow().hide();
      }
    };

    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, []);

  return (
    <TrayQuotaPanel
      snapshots={quotaSnapshotQuery.data ?? []}
      enabledProviders={settingsQuery.data?.quotaEnabledProviders ?? DEFAULT_ENABLED_PROVIDERS}
      onRefresh={() => {
        void invoke<QuotaSnapshot[]>("refresh_quota", { provider: null }).then(() => {
          void queryClient.invalidateQueries({ queryKey: SNAPSHOT_QUERY_KEY });
        });
      }}
      onOpenSettings={() => {
        void invoke("show_main_window", { view: "settings" }).then(() => getCurrentWindow().close());
      }}
    />
  );
}
