import { Dispatch, SetStateAction, useEffect, useState } from "react";
import { useMutation, useQueryClient, type UseQueryResult } from "@tanstack/react-query";
import { invoke } from "@tauri-apps/api/core";

import type { AppSettings, ProviderIntegrationStatus } from "../types";
import type { MessageKey } from "../locales/zh-TW";
import { DEFAULT_APP_SETTINGS, mergeAppSettings } from "../utils/appSettingsDefaults";
import { resolveErrorMessage } from "../utils/resolveErrorMessage";

export type ProviderIntegrationAction = "install" | "update" | "recheck" | "uninstall";

function getProviderLabel(
  provider: string,
  copilotLabel: string,
  opencodeLabel: string,
  codexLabel: string,
  claudeLabel?: string,
  antigravityLabel?: string,
): string {
  switch (provider) {
    case "copilot":
      return copilotLabel;
    case "opencode":
      return opencodeLabel;
    case "codex":
      return codexLabel;
    case "claude":
      return claudeLabel ?? provider;
    case "antigravity":
      return antigravityLabel ?? provider;
    default:
      return provider;
  }
}

function upsertProviderIntegrationStatus(
  integrations: ProviderIntegrationStatus[] | undefined,
  nextStatus: ProviderIntegrationStatus,
): ProviderIntegrationStatus[] {
  const nextIntegrations = [...(integrations ?? [])];
  const existingIndex = nextIntegrations.findIndex(
    (integration) => integration.provider === nextStatus.provider,
  );

  if (existingIndex === -1) {
    nextIntegrations.push(nextStatus);
  } else {
    nextIntegrations[existingIndex] = nextStatus;
  }

  const providerOrder = ["copilot", "opencode"];
  nextIntegrations.sort((left, right) => {
    const leftIndex = providerOrder.indexOf(left.provider);
    const rightIndex = providerOrder.indexOf(right.provider);
    const normalizedLeft = leftIndex === -1 ? Number.MAX_SAFE_INTEGER : leftIndex;
    const normalizedRight = rightIndex === -1 ? Number.MAX_SAFE_INTEGER : rightIndex;
    return normalizedLeft - normalizedRight || left.provider.localeCompare(right.provider);
  });

  return nextIntegrations;
}

interface UseAppSettingsFormParams {
  settingsQuery: UseQueryResult<AppSettings>;
  pinnedProjects: string[];
  showToast: (message: string) => void;
  t: (key: MessageKey, params?: Record<string, string | number>) => string;
  onSettingsSaved: () => void;
}

export function useAppSettingsForm({
  settingsQuery,
  pinnedProjects,
  showToast,
  t,
  onSettingsSaved,
}: UseAppSettingsFormParams) {
  const queryClient = useQueryClient();

  const [settingsForm, setSettingsForm] = useState<AppSettings>({
    copilotRoot: "",
    opencodeRoot: "",
    codexRoot: "",
    showArchived: false,
    enabledProviders: ["copilot", "opencode", "codex"],
    ...DEFAULT_APP_SETTINGS,
  });

  useEffect(() => {
    if (settingsQuery.data) {
      setSettingsForm({
        copilotRoot: settingsQuery.data.copilotRoot,
        opencodeRoot: settingsQuery.data.opencodeRoot ?? "",
        codexRoot: settingsQuery.data.codexRoot ?? "",
        showArchived: settingsQuery.data.showArchived,
        enabledProviders: settingsQuery.data.enabledProviders ?? ["copilot", "opencode", "codex"],
        ...mergeAppSettings(settingsQuery.data),
      });
    }
  }, [settingsQuery.data]);

  const buildSettingsPayload = (overrides: Partial<AppSettings> = {}): AppSettings => {
    const merged = mergeAppSettings({ ...settingsForm, ...overrides });
    return {
      copilotRoot: (overrides.copilotRoot ?? settingsForm.copilotRoot).trim(),
      opencodeRoot: (overrides.opencodeRoot ?? settingsForm.opencodeRoot).trim(),
      codexRoot: (overrides.codexRoot ?? settingsForm.codexRoot).trim(),
      showArchived: overrides.showArchived ?? settingsForm.showArchived,
      enabledProviders: overrides.enabledProviders ?? settingsForm.enabledProviders,
      ...merged,
      terminalPath: merged.terminalPath?.trim() || null,
      externalEditorPath: merged.externalEditorPath?.trim() || null,
      pinnedProjects: overrides.pinnedProjects ?? pinnedProjects,
      antigravityRoot: merged.antigravityRoot.trim(),
      hookScriptsPath: merged.hookScriptsPath.trim(),
      agentsSourceRoot: merged.agentsSourceRoot.trim(),
    };
  };

  const persistSettingsSilently = async (next: AppSettings) => {
    await invoke("save_settings", { settings: next });
    await queryClient.invalidateQueries({ queryKey: ["settings"] });
  };

  const settingsMutation = useMutation({
    mutationFn: (next: AppSettings) => invoke("save_settings", { settings: next }),
    onSuccess: async () => {
      showToast(t("toast.settingsSaved"));
      await queryClient.invalidateQueries({ queryKey: ["settings"] });
      await queryClient.invalidateQueries({ queryKey: ["sessions"] });
      onSettingsSaved();
    },
  });

  const detectTerminalMutation = useMutation({
    mutationFn: () => invoke<string | null>("detect_terminal"),
    onSuccess: (terminalPath) => {
      if (terminalPath) {
        setSettingsForm((v) => ({ ...v, terminalPath }));
        showToast(t("toast.terminalDetected"));
      } else {
        showToast(t("toast.terminalMissing"));
      }
    },
  });

  const detectVscodeMutation = useMutation({
    mutationFn: () => invoke<string | null>("detect_vscode"),
    onSuccess: (editorPath) => {
      if (editorPath) {
        setSettingsForm((v) => ({ ...v, externalEditorPath: editorPath }));
        showToast(t("toast.editorDetected"));
      } else {
        showToast(t("toast.editorMissing"));
      }
    },
  });

  const providerIntegrationMutation = useMutation({
    mutationFn: ({ provider, action }: { provider: string; action: ProviderIntegrationAction }) => {
      const command =
        action === "install"
          ? "install_provider_integration"
          : action === "update"
            ? "update_provider_integration"
            : action === "uninstall"
              ? "uninstall_provider_integration"
              : "recheck_provider_integration";

      return invoke<ProviderIntegrationStatus>(command, {
        provider,
        copilotRoot: settingsForm.copilotRoot.trim() || null,
        codexRoot: settingsForm.codexRoot.trim() || null,
        hookScriptsPath: (settingsForm.hookScriptsPath ?? "").trim() || null,
      });
    },
    onSuccess: (status, variables) => {
      const providerLabel = getProviderLabel(
        status.provider,
        t("settings.fields.providerCopilot"),
        t("settings.fields.providerOpencode"),
        t("settings.fields.providerCodex"),
        t("settings.fields.providerClaude"),
        t("settings.fields.providerAntigravity"),
      );
      setSettingsForm((current) => ({
        ...current,
        providerIntegrations: upsertProviderIntegrationStatus(
          current.providerIntegrations,
          status,
        ),
      }));

      if (
        (variables.action === "install" || variables.action === "update") &&
        status.status !== "installed"
      ) {
        showToast(
          status.lastError ||
            t("toast.providerActionIncomplete").replace("{provider}", providerLabel),
        );
        return;
      }

      const toastMessage =
        variables.action === "install"
          ? t("toast.providerInstalled")
          : variables.action === "update"
            ? t("toast.providerUpdated")
            : variables.action === "uninstall"
              ? t("toast.providerUninstalled")
              : t("toast.providerRechecked");
      showToast(toastMessage.replace("{provider}", providerLabel));
    },
    onError: (error, variables) => {
      const providerLabel = getProviderLabel(
        variables.provider,
        t("settings.fields.providerCopilot"),
        t("settings.fields.providerOpencode"),
        t("settings.fields.providerCodex"),
        t("settings.fields.providerClaude"),
        t("settings.fields.providerAntigravity"),
      );
      showToast(
        resolveErrorMessage(
          error,
          t("toast.providerActionFailed").replace("{provider}", providerLabel),
        ),
      );
    },
  });

  return {
    settingsForm,
    setSettingsForm: setSettingsForm as Dispatch<SetStateAction<AppSettings>>,
    buildSettingsPayload,
    persistSettingsSilently,
    settingsMutation,
    detectTerminalMutation,
    detectVscodeMutation,
    providerIntegrationMutation,
  };
}
