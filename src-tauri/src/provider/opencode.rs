use std::fs;

use crate::db::ensure_parent_dir;
use crate::settings::resolve_opencode_integration_path;
use crate::types::*;

use super::bridge::read_bridge_diagnostics;
use super::{
    build_install_failure_status, build_provider_integration_status, managed_provider_metadata,
    validate_integration_target, validate_managed_metadata, write_provider_integration_file,
};

fn render_opencode_integration(bridge_path: &std::path::Path) -> Result<String, String> {
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

    Ok(format!(
        concat!(
            "{metadata_prefix}{metadata}\n",
            "import {{ appendFile, mkdir }} from \"node:fs/promises\";\n",
            "import type {{ Plugin }} from \"@opencode-ai/plugin\";\n",
            "import type {{ Event }} from \"@opencode-ai/sdk\";\n\n",
            "const BRIDGE_PATH = {bridge_path};\n",
            "const BRIDGE_DIR = {bridge_dir};\n\n",
            "async function appendRecord(record: object) {{\n",
            "  await mkdir(BRIDGE_DIR, {{ recursive: true }});\n",
            "  await appendFile(BRIDGE_PATH, `${{JSON.stringify(record)}}\\n`, \"utf8\");\n",
            "}}\n\n",
            "export const SessionHubBridge: Plugin = async () => {{\n",
            "  return {{\n",
            "    event: async ({{ event }}: {{ event: Event }}) => {{\n",
            "      if (event.type === \"session.updated\" || event.type === \"session.created\") {{\n",
            "        const session = event.properties.info;\n",
            "        await appendRecord({{\n",
            "          version: {version},\n",
            "          provider: \"{provider}\",\n",
            "          eventType: event.type,\n",
            "          timestamp: new Date(session.time.updated).toISOString(),\n",
            "          sessionId: session.id,\n",
            "          cwd: session.directory,\n",
            "          sourcePath: null,\n",
            "          title: session.title ?? null,\n",
            "          error: null,\n",
            "        }});\n",
            "      }} else if (event.type === \"session.error\") {{\n",
            "        const props = event.properties;\n",
            "        await appendRecord({{\n",
            "          version: {version},\n",
            "          provider: \"{provider}\",\n",
            "          eventType: event.type,\n",
            "          timestamp: new Date().toISOString(),\n",
            "          sessionId: (props as any).sessionID ?? null,\n",
            "          cwd: null,\n",
            "          sourcePath: null,\n",
            "          title: null,\n",
            "          error: ((props as any).error as any)?.message ?? String((props as any).error) ?? null,\n",
            "        }});\n",
            "      }}\n",
            "    }},\n",
            "  }};\n",
            "}};\n"
        ),
        metadata_prefix = OPENCODE_PLUGIN_METADATA_PREFIX,
        metadata = metadata,
        bridge_path = bridge_path_literal,
        bridge_dir = bridge_parent_literal,
        version = PROVIDER_INTEGRATION_VERSION,
        provider = OPENCODE_PROVIDER
    ))
}

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
    match validate_managed_metadata(&metadata, OPENCODE_PROVIDER, &expected_bridge_path) {
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
