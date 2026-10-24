use std::fs;

use crate::db::ensure_parent_dir;
use crate::settings::resolve_opencode_integration_path;
use crate::types::*;

use super::bridge::read_bridge_diagnostics;
use super::{
    build_install_failure_status, build_provider_integration_status, managed_provider_metadata,
    validate_integration_target, validate_managed_metadata, write_provider_integration_file,
};

pub(crate) fn render_opencode_integration(bridge_path: &std::path::Path) -> Result<String, String> {
    let metadata =
        serde_json::to_string(&managed_provider_metadata(OPENCODE_PROVIDER, bridge_path)).map_err(
            |error| format!("failed to serialize OpenCode integration metadata: {error}"),
        )?;
    let bridge_path_literal = serde_json::to_string(&bridge_path.to_string_lossy().to_string())
        .map_err(|error| format!("failed to serialize OpenCode bridge path: {error}"))?;
    let bridge_parent_literal = serde_json::to_string(
        &bridge_path
            .parent()
            .unwrap_or(bridge_path)
            .to_string_lossy()
            .to_string(),
    )
    .map_err(|error| format!("failed to serialize OpenCode bridge directory: {error}"))?;

    Ok(OPENCODE_PLUGIN_TEMPLATE
        .replace("__METADATA_PREFIX__", OPENCODE_PLUGIN_METADATA_PREFIX)
        .replace("__METADATA__", &metadata)
        .replace("__BRIDGE_PATH__", &bridge_path_literal)
        .replace("__BRIDGE_DIR__", &bridge_parent_literal)
        .replace("__VERSION__", &PROVIDER_INTEGRATION_VERSION.to_string())
        .replace("__PROVIDER__", OPENCODE_PROVIDER))
}

/// OpenCode plugin 範本。2.x 讀取 default export 的 `id` + `setup`，
/// 1.x 讀取 `server()`；兩者互不影響，因此同一份檔案可相容兩個版本。
const OPENCODE_PLUGIN_TEMPLATE: &str = r#"__METADATA_PREFIX____METADATA__
import { appendFile, mkdir, rmdir } from "node:fs/promises";

const BRIDGE_PATH = __BRIDGE_PATH__;
const BRIDGE_DIR = __BRIDGE_DIR__;

async function appendRecord(record: object) {
  await mkdir(BRIDGE_DIR, { recursive: true });
  const lockPath = `${BRIDGE_PATH}.lock`;
  const deadline = Date.now() + 5000;
  while (true) {
    try {
      await mkdir(lockPath);
      break;
    } catch (error) {
      if ((error as NodeJS.ErrnoException).code !== "EEXIST" || Date.now() >= deadline) throw error;
      await new Promise((resolve) => setTimeout(resolve, 25));
    }
  }
  try {
    await appendFile(BRIDGE_PATH, `${JSON.stringify(record)}\n`, "utf8");
  } finally {
    await rmdir(lockPath);
  }
}

function buildRecord(eventType: string, timestamp: string, fields: Record<string, unknown>) {
  return {
    version: __VERSION__,
    provider: "__PROVIDER__",
    eventType,
    timestamp,
    sessionId: null,
    cwd: null,
    sourcePath: null,
    title: null,
    error: null,
    ...fields,
  };
}

// OpenCode 2.x：事件為 { type, created, location, data }
async function handleV2Event(event: any) {
  const data = event?.data ?? {};
  const timestamp = new Date(event?.created ?? Date.now()).toISOString();

  switch (event?.type) {
    case "session.created":
      await appendRecord(buildRecord("session.created", timestamp, {
        sessionId: data.sessionID ?? null,
        cwd: data.location?.directory ?? event.location?.directory ?? null,
        title: data.title ?? null,
      }));
      break;
    case "session.renamed":
      await appendRecord(buildRecord("session.updated", timestamp, {
        sessionId: data.sessionID ?? null,
        cwd: event.location?.directory ?? null,
        title: data.title ?? null,
      }));
      break;
    case "session.execution.failed":
      await appendRecord(buildRecord("session.error", timestamp, {
        sessionId: data.sessionID ?? null,
        error: data.error?.message ?? null,
      }));
      break;
    case "session.status": {
      const status = data.status?.type ?? data.status;
      if (status === "busy" || status === "retry" || status === "idle") {
        await appendRecord(buildRecord(status === "idle" ? "session.idle" : "session.active", timestamp, {
          sessionId: data.sessionID ?? null,
        }));
      }
      break;
    }
    case "session.idle":
      await appendRecord(buildRecord("session.idle", timestamp, {
        sessionId: data.sessionID ?? null,
      }));
      break;
    case "permission.asked": {
      const permissionType = data.action ?? "permission";
      await appendRecord(buildRecord("permission.asked", timestamp, {
        sessionId: data.sessionID ?? null,
        title: data.id ? `${permissionType} [${data.id}]` : permissionType,
      }));
      break;
    }
    case "permission.replied":
      await appendRecord(buildRecord("permission.replied", timestamp, {
        sessionId: data.sessionID ?? null,
        title: data.requestID ?? null,
      }));
      break;
  }
}

// OpenCode 1.x：事件為 { type, properties }
async function handleV1Event(event: any) {
  const props = event?.properties ?? {};

  if (event.type === "session.updated" || event.type === "session.created") {
    const session = props.info;
    await appendRecord(buildRecord(event.type, new Date(session.time.updated).toISOString(), {
      sessionId: session.id,
      cwd: session.directory,
      title: session.title ?? null,
    }));
  } else if (event.type === "session.error") {
    await appendRecord(buildRecord(event.type, new Date().toISOString(), {
      sessionId: props.sessionID ?? null,
      error: props.error?.message ?? String(props.error ?? "") ?? null,
    }));
  } else if (event.type === "session.status") {
    const status = props.status?.type ?? props.status;
    if (status === "busy" || status === "retry" || status === "idle") {
      await appendRecord(buildRecord(status === "idle" ? "session.idle" : "session.active", new Date().toISOString(), {
        sessionId: props.sessionID ?? null,
      }));
    }
  } else if (event.type === "session.idle") {
    await appendRecord(buildRecord("session.idle", new Date().toISOString(), {
      sessionId: props.sessionID ?? null,
    }));
  } else if (event.type === "permission.updated" || event.type === "permission.asked") {
    const permissionType = props.title ?? props.permission ?? props.type ?? props.action ?? "permission";
    const requestId = props.id ?? props.requestID ?? props.permissionID ?? null;
    await appendRecord(buildRecord(event.type, new Date(props.time?.created ?? Date.now()).toISOString(), {
      sessionId: props.sessionID ?? null,
      title: requestId ? `${permissionType} [${requestId}]` : permissionType,
    }));
  } else if (event.type === "permission.replied") {
    await appendRecord(buildRecord(event.type, new Date().toISOString(), {
      sessionId: props.sessionID ?? null,
      title: props.requestID ?? props.permissionID ?? null,
    }));
  }
}

export default {
  id: "sessionhub-provider-event-bridge",
  setup(ctx: any) {
    const controller = new AbortController();

    void (async () => {
      for await (const event of ctx.event.subscribe({ signal: controller.signal })) {
        try {
          await handleV2Event(event);
        } catch {
          // 橋接檔寫入失敗不應中斷事件訂閱
        }
      }
    })().catch(() => {});

    return () => controller.abort();
  },
  async server() {
    return {
      event: async ({ event }: { event: any }) => {
        await handleV1Event(event);
      },
    };
  },
};
"#;

fn parse_opencode_integration_metadata(
    content: &str,
) -> Result<Option<ManagedProviderIntegrationMetadata>, String> {
    for line in content.lines() {
        let trimmed = line.trim();
        if let Some(raw_json) = trimmed.strip_prefix(OPENCODE_PLUGIN_METADATA_PREFIX) {
            return serde_json::from_str::<ManagedProviderIntegrationMetadata>(raw_json.trim())
                .map(Some)
                .map_err(|error| {
                    format!("failed to parse OpenCode integration metadata: {error}")
                });
        }
    }

    Ok(None)
}

pub(crate) fn install_or_update_opencode_integration() -> ProviderIntegrationStatus {
    let diagnostics = read_bridge_diagnostics(OPENCODE_PROVIDER);
    let config_path = match resolve_opencode_integration_path() {
        Ok(path) => path,
        Err(error) => {
            return build_provider_integration_status(
                OPENCODE_PROVIDER,
                ProviderIntegrationState::ManualRequired,
                None,
                diagnostics,
                None,
                Some(error),
            );
        }
    };
    let Some(bridge_path) = diagnostics.bridge_path.clone() else {
        return build_provider_integration_status(
            OPENCODE_PROVIDER,
            ProviderIntegrationState::ManualRequired,
            Some(config_path),
            diagnostics,
            None,
            Some("failed to resolve OpenCode bridge path".to_string()),
        );
    };

    let content = match render_opencode_integration(&bridge_path) {
        Ok(content) => content,
        Err(error) => {
            return build_install_failure_status(
                OPENCODE_PROVIDER,
                Some(config_path),
                diagnostics,
                error,
            );
        }
    };

    if let Err(error) = ensure_parent_dir(&bridge_path)
        .and_then(|_| write_provider_integration_file(&config_path, &content))
    {
        return build_install_failure_status(
            OPENCODE_PROVIDER,
            Some(config_path),
            diagnostics,
            error,
        );
    }

    detect_opencode_integration_status()
}

pub(crate) fn detect_opencode_integration_status() -> ProviderIntegrationStatus {
    let diagnostics = read_bridge_diagnostics(OPENCODE_PROVIDER);
    let config_path = match resolve_opencode_integration_path() {
        Ok(path) => path,
        Err(error) => {
            return build_provider_integration_status(
                OPENCODE_PROVIDER,
                ProviderIntegrationState::ManualRequired,
                None,
                diagnostics,
                None,
                Some(error),
            );
        }
    };

    if let Err(error) = validate_integration_target(&config_path) {
        return build_provider_integration_status(
            OPENCODE_PROVIDER,
            ProviderIntegrationState::ManualRequired,
            Some(config_path),
            diagnostics,
            None,
            Some(error),
        );
    }

    let Some(expected_bridge_path) = diagnostics.bridge_path.clone() else {
        return build_provider_integration_status(
            OPENCODE_PROVIDER,
            ProviderIntegrationState::ManualRequired,
            Some(config_path),
            diagnostics,
            None,
            Some("failed to resolve OpenCode bridge path".to_string()),
        );
    };

    if !config_path.exists() {
        return build_provider_integration_status(
            OPENCODE_PROVIDER,
            ProviderIntegrationState::Missing,
            Some(config_path),
            diagnostics,
            None,
            None,
        );
    }

    let content = match fs::read_to_string(&config_path) {
        Ok(content) => content,
        Err(error) => {
            let error_message = format!(
                "failed to read OpenCode integration file {}: {error}",
                config_path.display()
            );
            return build_provider_integration_status(
                OPENCODE_PROVIDER,
                ProviderIntegrationState::Error,
                Some(config_path),
                diagnostics,
                None,
                Some(error_message),
            );
        }
    };

    let metadata = match parse_opencode_integration_metadata(&content) {
        Ok(Some(metadata)) => metadata,
        Ok(None) => {
            return build_provider_integration_status(
                OPENCODE_PROVIDER,
                ProviderIntegrationState::Outdated,
                Some(config_path),
                diagnostics,
                None,
                Some("missing SessionHub integration metadata".to_string()),
            );
        }
        Err(error) => {
            return build_provider_integration_status(
                OPENCODE_PROVIDER,
                ProviderIntegrationState::Error,
                Some(config_path),
                diagnostics,
                None,
                Some(error),
            );
        }
    };

    let installed_version = Some(metadata.integration_version);
    let integration_status =
        validate_managed_metadata(&metadata, OPENCODE_PROVIDER, &expected_bridge_path).and_then(
            |_| {
                if content.contains("case \"session.status\"")
                    && content.contains("event.type === \"session.status\"")
                {
                    Ok(())
                } else {
                    Err("OpenCode bridge 缺少 session 狀態事件，請更新整合。".to_string())
                }
            },
        );
    match integration_status {
        Ok(()) => build_provider_integration_status(
            OPENCODE_PROVIDER,
            ProviderIntegrationState::Installed,
            Some(config_path),
            diagnostics,
            installed_version,
            None,
        ),
        Err(error) => build_provider_integration_status(
            OPENCODE_PROVIDER,
            ProviderIntegrationState::Outdated,
            Some(config_path),
            diagnostics,
            installed_version,
            Some(error),
        ),
    }
}

pub(crate) fn uninstall_opencode_integration() -> ProviderIntegrationStatus {
    let diagnostics = read_bridge_diagnostics(OPENCODE_PROVIDER);
    let config_path = match resolve_opencode_integration_path() {
        Ok(path) => path,
        Err(error) => {
            return build_provider_integration_status(
                OPENCODE_PROVIDER,
                ProviderIntegrationState::ManualRequired,
                None,
                diagnostics,
                None,
                Some(error),
            );
        }
    };

    if config_path.exists() {
        if let Err(error) = fs::remove_file(&config_path) {
            return build_install_failure_status(
                OPENCODE_PROVIDER,
                Some(config_path),
                diagnostics,
                format!("failed to remove OpenCode plugin file: {error}"),
            );
        }
    }

    build_provider_integration_status(
        OPENCODE_PROVIDER,
        ProviderIntegrationState::Missing,
        Some(config_path),
        diagnostics,
        None,
        None,
    )
}
