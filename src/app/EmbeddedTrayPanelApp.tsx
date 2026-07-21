import { useEffect } from "react";
import { useQuery, useQueryClient } from "@tanstack/react-query";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";

import type { QuotaSnapshot } from "../types";
import { TrayQuotaPanel } from "../components/TrayQuotaPanel";

export function EmbeddedTrayPanelApp() {
  const queryClient = useQueryClient();
  const quotaSnapshotQuery = useQuery({
    queryKey: ["embedded_quota_snapshots", "tray_panel"],
    queryFn: () => invoke<QuotaSnapshot[]>("get_quota_snapshots"),
    staleTime: 60_000,
  });

  useEffect(() => {
    let mounted = true;

    const setup = async () => {
      const unlistenSnapshots = await listen("quota-snapshots-updated", () => {
        if (mounted) {
          void queryClient.invalidateQueries({ queryKey: ["embedded_quota_snapshots", "tray_panel"] });
        }
      });

      return () => {
        unlistenSnapshots();
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
      onRefresh={() => {
        void invoke<QuotaSnapshot[]>("refresh_quota", { provider: null }).then(() => {
          void queryClient.invalidateQueries({ queryKey: ["embedded_quota_snapshots", "tray_panel"] });
        });
      }}
      onOpenSettings={() => {
        void invoke("show_main_window", { view: "settings" }).then(() => getCurrentWindow().close());
      }}
    />
  );
}
