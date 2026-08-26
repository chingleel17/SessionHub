use std::collections::{BTreeMap, BTreeSet};
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::settings::{default_app_data_dir, default_agents_root, default_opencode_config_root,
    resolve_antigravity_root, resolve_claude_root,
    resolve_codex_root, resolve_copilot_root};
use crate::types::{AppSettings, ANTIGRAVITY_PROVIDER, CLAUDE_PROVIDER, CODEX_PROVIDER,
    COPILOT_PROVIDER, CREATE_NO_WINDOW, OPENCODE_PROVIDER};

pub(crate) const MAX_CLI_OUTPUT_BYTES: usize = 4 * 1024 * 1024;
pub(crate) const CLI_TIMEOUT: Duration = Duration::from_secs(8);
const MAX_DIAGNOSTIC_BYTES: usize = 512;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum ResourceKind {
    Skill,
    Command,
    Mcp,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum DiscoveryScope {
    Global,
    Project,
    Effective,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum DiscoverySource {
    Cli,
    File,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum ResourceState {
    Available,
    Configured,
    Disabled,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ResourceLocation {
    pub(crate) provider_id: String,
    pub(crate) scope: DiscoveryScope,
    pub(crate) root: String,
    pub(crate) path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ResourceDiscovery {
    pub(crate) provider_id: String,
    pub(crate) kind: ResourceKind,
    pub(crate) scope: DiscoveryScope,
    pub(crate) locations: Vec<ResourceLocation>,
    pub(crate) effective_path: Option<String>,
    pub(crate) source: DiscoverySource,
    pub(crate) state: ResourceState,
    pub(crate) editable: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DiscoveryDiagnostic {
    pub(crate) provider_id: String,
    pub(crate) kind: ResourceKind,
    pub(crate) scope: DiscoveryScope,
    pub(crate) message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CliResource {
    pub(crate) name: String,
    pub(crate) effective_path: Option<String>,
    pub(crate) source: Option<String>,
    pub(crate) enabled: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CliOutput {
    pub(crate) stdout: String,
    pub(crate) stderr: String,
    pub(crate) exit_code: Option<i32>,
    pub(crate) timed_out: bool,
    pub(crate) truncated: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CliCapability {
    OpenCodeSkill,
    OpenCodeCommand,
    CopilotMcp,
    CodexMcp,
}

fn capability(provider: &str, kind: ResourceKind) -> Option<CliCapability> {
    match (provider, kind) {
        (OPENCODE_PROVIDER, ResourceKind::Skill) => Some(CliCapability::OpenCodeSkill),
        (OPENCODE_PROVIDER, ResourceKind::Command) => Some(CliCapability::OpenCodeCommand),
        (COPILOT_PROVIDER, ResourceKind::Mcp) => Some(CliCapability::CopilotMcp),
        (CODEX_PROVIDER, ResourceKind::Mcp) => Some(CliCapability::CodexMcp),
        _ => None,
    }
}

fn capability_command(capability: CliCapability) -> (&'static str, &'static [&'static str]) {
    match capability {
        CliCapability::OpenCodeSkill => ("opencode", &["debug", "skill"]),
        CliCapability::OpenCodeCommand => ("opencode", &["debug", "config"]),
        CliCapability::CopilotMcp => ("copilot", &["mcp", "list", "--json"]),
        CliCapability::CodexMcp => ("codex", &["mcp", "list", "--json"]),
    }
}

pub(crate) fn known_provider(provider: &str) -> bool {
    matches!(
        provider,
        CLAUDE_PROVIDER | CODEX_PROVIDER | OPENCODE_PROVIDER | COPILOT_PROVIDER | ANTIGRAVITY_PROVIDER
    )
}

pub(crate) fn normalize_enabled_providers(enabled: &[String]) -> (Vec<String>, Vec<String>) {
    let mut seen = BTreeSet::new();
    let mut providers = Vec::new();
    let mut unknown = Vec::new();
    for provider in enabled {
        let provider = provider.trim().to_lowercase();
        if provider.is_empty() || !known_provider(&provider) {
            if !provider.is_empty() && !unknown.contains(&provider) {
                unknown.push(provider);
            }
            continue;
        }
        if seen.insert(provider.clone()) {
            providers.push(provider);
        }
    }
    (providers, unknown)
}

pub(crate) fn global_probe_dir() -> Result<PathBuf, String> {
    let path = default_app_data_dir()?.join("cli-probe");
    std::fs::create_dir_all(&path)
        .map_err(|error| format!("failed to create CLI probe directory: {error}"))?;
    Ok(path)
}

pub(crate) fn probe_cwd(scope: DiscoveryScope, project_cwd: Option<&str>) -> Result<PathBuf, String> {
    match scope {
        DiscoveryScope::Project => project_cwd
            .map(PathBuf::from)
            .ok_or_else(|| "project scope requires a project working directory".to_string()),
        DiscoveryScope::Global | DiscoveryScope::Effective => global_probe_dir(),
    }
}

fn allowed_command(executable: &str, args: &[&str]) -> bool {
    let name = Path::new(executable)
        .file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or(executable)
        .to_lowercase();
    matches!(
        (name.as_str(), args),
        ("opencode", ["debug", "skill"])
            | ("opencode", ["debug", "config"])
            | ("copilot", ["mcp", "list", "--json"])
            | ("codex", ["mcp", "list", "--json"])
    )
}

fn spawn_reader<R: Read + Send + 'static>(mut reader: R) -> thread::JoinHandle<(Vec<u8>, bool)> {
    thread::spawn(move || {
        let mut output = Vec::with_capacity(MAX_CLI_OUTPUT_BYTES.min(8192));
        let mut buffer = [0u8; 8192];
        let mut truncated = false;
        loop {
            match reader.read(&mut buffer) {
                Ok(0) => break,
                Ok(bytes) => {
                    let remaining = MAX_CLI_OUTPUT_BYTES.saturating_sub(output.len());
                    output.extend_from_slice(&buffer[..bytes.min(remaining)]);
                    if bytes > remaining {
                        truncated = true;
                    }
                }
                Err(_) => break,
            }
        }
        (output, truncated)
    })
}

fn terminate_child(child: &mut Child) {
    let _ = child.kill();
    let _ = child.wait();
}

fn run_process_internal(
    executable: &str,
    args: &[&str],
    cwd: &Path,
    timeout: Duration,
    enforce_allowlist: bool,
) -> Result<CliOutput, String> {
    if enforce_allowlist && !allowed_command(executable, args) {
        return Err("CLI command is not in the read-only capability allowlist".to_string());
    }
    let mut command = Command::new(executable);
    command
        .args(args)
        .current_dir(cwd)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .env("CI", "1")
        .env("NO_COLOR", "1")
        .env_remove("OPENAI_API_KEY")
        .env_remove("ANTHROPIC_API_KEY")
        .env_remove("GITHUB_TOKEN")
        .env_remove("GH_TOKEN")
        .env_remove("COPILOT_TOKEN");

    #[cfg(target_os = "windows")]
    command.creation_flags(CREATE_NO_WINDOW);

    let mut child = command
        .spawn()
        .map_err(|error| format!("failed to start {executable}: {error}"))?;
    let stdout = child.stdout.take().ok_or_else(|| "CLI stdout pipe unavailable".to_string())?;
    let stderr = child.stderr.take().ok_or_else(|| "CLI stderr pipe unavailable".to_string())?;
    let stdout_reader = spawn_reader(stdout);
    let stderr_reader = spawn_reader(stderr);
    let started = Instant::now();
    let (status, timed_out) = loop {
        let status = child
            .try_wait()
            .map_err(|error| format!("failed to poll CLI: {error}"))?;
        if status.is_some() {
            break (status, false);
        }
        if started.elapsed() >= timeout {
            terminate_child(&mut child);
            break (child.try_wait().ok().flatten(), true);
        }
        thread::sleep(Duration::from_millis(20));
    };
    let (stdout, stdout_truncated) = stdout_reader.join().unwrap_or_default();
    let (stderr, stderr_truncated) = stderr_reader.join().unwrap_or_default();
    Ok(CliOutput {
        stdout: clean_text(&stdout),
        stderr: clean_text(&stderr),
        exit_code: status.and_then(|value| value.code()),
        timed_out,
        truncated: stdout_truncated || stderr_truncated,
    })
}

fn run_process(executable: &str, args: &[&str], cwd: &Path, timeout: Duration) -> Result<CliOutput, String> {
    run_process_internal(executable, args, cwd, timeout, true)
}

fn clean_text(bytes: &[u8]) -> String {
    let text = String::from_utf8_lossy(bytes);
    let mut result = String::with_capacity(text.len());
    let mut in_escape = false;
    let mut csi_escape = false;
    for character in text.chars() {
        if in_escape {
            if !csi_escape && character == '[' {
                csi_escape = true;
                continue;
            }
            if csi_escape {
                if ('@'..='~').contains(&character) {
                    in_escape = false;
                    csi_escape = false;
                }
            } else if ('@'..='~').contains(&character) {
                in_escape = false;
            }
            continue;
        }
        if character == '\u{1b}' {
            in_escape = true;
            continue;
        }
        if !character.is_control() || character == '\n' || character == '\r' || character == '\t' {
            result.push(character);
        }
    }
    result
}

fn sanitize_diagnostic(message: &str) -> String {
    let mut sanitized = message.to_string();
    for key in [
        "Authorization", "authorization", "Bearer", "token", "Token", "api_key", "API_KEY",
        "password", "PASSWORD", "secret", "SECRET", "env", "headers",
    ] {
        let mut cursor = 0;
        while let Some(relative) = sanitized[cursor..].find(key) {
            let start = cursor + relative;
            let end = sanitized[start..]
                .find(['\n', '\r', ','])
                .map(|offset| start + offset)
                .unwrap_or(sanitized.len());
            sanitized.replace_range(start..end, "[redacted]");
            cursor = start + "[redacted]".len();
            if cursor >= sanitized.len() { break; }
        }
    }
    sanitized
}

fn diagnostic(provider: &str, kind: ResourceKind, scope: DiscoveryScope, message: &str) -> DiscoveryDiagnostic {
    let mut message = sanitize_diagnostic(&clean_text(message.as_bytes()));
    if message.len() > MAX_DIAGNOSTIC_BYTES {
        message.truncate(MAX_DIAGNOSTIC_BYTES);
        message.push_str("…");
    }
    DiscoveryDiagnostic { provider_id: provider.to_string(), kind, scope, message }
}

fn json_root(output: &CliOutput) -> Result<Value, String> {
    if output.timed_out {
        return Err("CLI query timed out".to_string());
    }
    if output.truncated {
        return Err("CLI output exceeded the safety limit".to_string());
    }
    if output.exit_code != Some(0) {
        let error = if output.stderr.trim().is_empty() { "CLI exited with a non-zero status" } else { &output.stderr };
        return Err(error.to_string());
    }
    serde_json::from_str(output.stdout.trim()).map_err(|error| format!("invalid CLI JSON: {error}"))
}

fn string_field(value: &Value, keys: &[&str]) -> Option<String> {
    keys.iter().find_map(|key| value.get(*key).and_then(Value::as_str).map(ToString::to_string))
}

fn collect_json_resources(value: &Value, output: &mut Vec<CliResource>) {
    match value {
        Value::Array(items) => items.iter().for_each(|item| collect_json_resources(item, output)),
        Value::Object(map) => {
            if let Some(name) = string_field(value, &["name", "id", "key"]) {
                output.push(CliResource {
                    name,
                    effective_path: string_field(value, &["path", "file", "filePath", "location"]),
                    source: string_field(value, &["source", "scope", "origin"]),
                    enabled: value.get("enabled").and_then(Value::as_bool),
                });
            }
            for (key, item) in map {
                if matches!(key.as_str(), "servers" | "mcpServers" | "skills" | "commands" | "command" | "items" | "resolved" | "config") {
                    collect_json_resources(item, output);
                } else if matches!(value, Value::Object(_)) && item.is_object() {
                    let has_identity = string_field(item, &["name", "id", "key"]).is_some();
                    if !has_identity {
                        let mut named = item.clone();
                        if let Value::Object(named_map) = &mut named {
                            named_map.insert("name".to_string(), Value::String(key.clone()));
                        }
                        collect_json_resources(&named, output);
                    }
                }
            }
        }
        _ => {}
    }
}

fn parse_fixture_resources(raw: &str) -> Result<Vec<CliResource>, String> {
    let value: Value = serde_json::from_str(raw).map_err(|error| format!("invalid fixture JSON: {error}"))?;
    let mut resources = Vec::new();
    collect_json_resources(&value, &mut resources);
    let mut unique = BTreeMap::new();
    for resource in resources {
        if !resource.name.trim().is_empty() {
            unique.entry(resource.name.to_lowercase()).or_insert(resource);
        }
    }
    Ok(unique.into_values().collect())
}

fn parse_named_resources(value: &Value) -> Vec<CliResource> {
    let mut resources = Vec::new();
    match value {
        Value::Array(items) => {
            for item in items {
                if let Value::Object(map) = item {
                    if let Some(name) = string_field(item, &["name", "id", "key"]) {
                        resources.push(CliResource {
                            name,
                            effective_path: string_field(item, &["path", "file", "filePath", "location"]),
                            source: string_field(item, &["source", "scope", "origin"]),
                            enabled: item.get("enabled").and_then(Value::as_bool),
                        });
                    } else if map.len() == 1 {
                        if let Some((name, item)) = map.iter().next() {
                            resources.push(CliResource {
                                name: name.clone(),
                                effective_path: string_field(item, &["path", "file", "filePath", "location"]),
                                source: string_field(item, &["source", "scope", "origin"]),
                                enabled: item.get("enabled").and_then(Value::as_bool),
                            });
                        }
                    }
                }
            }
        }
        Value::Object(map) => {
            for (name, item) in map {
                if let Value::Object(item_map) = item {
                    resources.push(CliResource {
                        name: name.clone(),
                        effective_path: string_field(item, &["path", "file", "filePath", "location"]),
                        source: string_field(item, &["source", "scope", "origin"]),
                        enabled: item_map.get("enabled").and_then(Value::as_bool),
                    });
                }
            }
        }
        _ => {}
    }
    resources
}

fn parse_cli_resources(provider: &str, kind: ResourceKind, root: &Value) -> Vec<CliResource> {
    let resources = match (provider, kind) {
        (OPENCODE_PROVIDER, ResourceKind::Skill) => root
            .get("skills")
            .map(parse_named_resources)
            .unwrap_or_else(|| parse_named_resources(root)),
        (OPENCODE_PROVIDER, ResourceKind::Command) => root
            .get("command")
            .map(parse_named_resources)
            .unwrap_or_default(),
        (_, ResourceKind::Mcp) => ["servers", "mcpServers", "mcp"]
            .iter()
            .find_map(|key| root.get(*key).map(parse_named_resources))
            .unwrap_or_else(|| parse_named_resources(root)),
        _ => Vec::new(),
    };
    let mut unique = BTreeMap::new();
    for resource in resources {
        if !resource.name.trim().is_empty() {
            unique.entry(resource.name.to_lowercase()).or_insert(resource);
        }
    }
    unique.into_values().collect()
}

pub(crate) fn discover_cli(
    provider: &str,
    kind: ResourceKind,
    scope: DiscoveryScope,
    project_cwd: Option<&str>,
) -> Result<Vec<CliResource>, DiscoveryDiagnostic> {
    let Some(capability) = capability(provider, kind) else {
        return Ok(Vec::new());
    };
    let (executable, args) = capability_command(capability);
    let cwd = probe_cwd(scope, project_cwd)
        .map_err(|error| diagnostic(provider, kind, scope, &error))?;
    let output = run_process(executable, args, &cwd, CLI_TIMEOUT)
        .map_err(|error| diagnostic(provider, kind, scope, &error))?;
    let root = json_root(&output).map_err(|error| diagnostic(provider, kind, scope, &error))?;
    Ok(parse_cli_resources(provider, kind, &root))
}

pub(crate) fn skill_roots(
    scope: DiscoveryScope,
    project_cwd: Option<&Path>,
    settings: &AppSettings,
    enabled_providers: &[String],
) -> Result<Vec<(String, PathBuf)>, String> {
    let project = match scope {
        DiscoveryScope::Project => project_cwd.map(Path::to_path_buf),
        _ => None,
    };
    let root_for = |provider: &str| -> Result<Vec<PathBuf>, String> {
        let roots = match provider {
            CLAUDE_PROVIDER => vec![project.clone().map(|root| root.join(".claude/skills")).unwrap_or(resolve_claude_root(Some(&settings.claude_root))?.join("skills"))],
            CODEX_PROVIDER => vec![project.clone().map(|root| root.join(".codex/skills")).unwrap_or(resolve_codex_root(Some(&settings.codex_root))?.join("skills")), project.clone().map(|root| root.join(".agents/skills")).unwrap_or(default_agents_root()?.join("skills"))],
            OPENCODE_PROVIDER => vec![project.clone().map(|root| root.join(".opencode/skill")).unwrap_or(default_opencode_config_root()?.join("skill")), project.clone().map(|root| root.join(".opencode/skills")).unwrap_or(default_opencode_config_root()?.join("skills")), project.clone().map(|root| root.join(".claude/skills")).unwrap_or(resolve_claude_root(Some(&settings.claude_root))?.join("skills")), project.clone().map(|root| root.join(".agents/skills")).unwrap_or(default_agents_root()?.join("skills"))],
            COPILOT_PROVIDER => vec![project.clone().map(|root| root.join(".github/skills")).unwrap_or(resolve_copilot_root(Some(&settings.copilot_root))?.join("skills")), project.clone().map(|root| root.join(".agents/skills")).unwrap_or(default_agents_root()?.join("skills")), project.clone().map(|root| root.join(".claude/skills")).unwrap_or(resolve_claude_root(Some(&settings.claude_root))?.join("skills"))],
            ANTIGRAVITY_PROVIDER => vec![project.clone().map(|root| root.join(".gemini/skills")).unwrap_or(resolve_antigravity_root(Some(&settings.antigravity_root))?.join("skills")), project.clone().map(|root| root.join(".agents/skills")).unwrap_or(default_agents_root()?.join("skills"))],
            _ => Vec::new(),
        };
        Ok(roots)
    };
    let mut result = Vec::new();
    let (enabled, _) = normalize_enabled_providers(enabled_providers);
    for provider in enabled {
        for root in root_for(&provider)? {
            result.push((provider.clone(), root));
        }
    }
    Ok(result)
}

pub(crate) fn command_roots(
    scope: DiscoveryScope,
    project_cwd: Option<&Path>,
    settings: &AppSettings,
    enabled_providers: &[String],
) -> Result<Vec<(String, PathBuf)>, String> {
    let project = match scope {
        DiscoveryScope::Project => project_cwd.map(Path::to_path_buf),
        _ => None,
    };
    let roots = |provider: &str| -> Result<Vec<PathBuf>, String> {
        Ok(match provider {
            CLAUDE_PROVIDER => vec![project.clone().map(|root| root.join(".claude/commands")).unwrap_or(resolve_claude_root(Some(&settings.claude_root))?.join("commands"))],
            CODEX_PROVIDER => vec![project.clone().map(|root| root.join(".codex/prompts")).unwrap_or(resolve_codex_root(Some(&settings.codex_root))?.join("prompts"))],
            OPENCODE_PROVIDER => vec![project.clone().map(|root| root.join(".opencode/command")).unwrap_or(default_opencode_config_root()?.join("command")), project.clone().map(|root| root.join(".opencode/commands")).unwrap_or(default_opencode_config_root()?.join("commands"))],
            COPILOT_PROVIDER => vec![project.clone().map(|root| root.join(".github/prompts")).unwrap_or(resolve_copilot_root(Some(&settings.copilot_root))?.join("prompts")), project.clone().map(|root| root.join(".copilot/prompts")).unwrap_or(resolve_copilot_root(Some(&settings.copilot_root))?.join("prompts"))],
            ANTIGRAVITY_PROVIDER => vec![project.clone().map(|root| root.join(".gemini/commands")).unwrap_or(resolve_antigravity_root(Some(&settings.antigravity_root))?.join("commands"))],
            _ => Vec::new(),
        })
    };
    let mut result = Vec::new();
    let (enabled, _) = normalize_enabled_providers(enabled_providers);
    for provider in enabled {
        for root in roots(&provider)? {
            result.push((provider.clone(), root));
        }
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allowlist_rejects_non_read_only_commands() {
        assert!(!allowed_command("powershell", &["-Command", "whoami"]));
        assert!(allowed_command("opencode", &["debug", "skill"]));
    }

    #[test]
    fn ansi_and_control_characters_are_removed() {
        assert_eq!(clean_text(b"\x1b[31msecret\x1b[0m\n"), "secret\n");
    }

    #[test]
    fn diagnostics_are_truncated() {
        let diagnostic = diagnostic("copilot", ResourceKind::Mcp, DiscoveryScope::Global, &"x".repeat(2048));
        assert!(diagnostic.message.len() <= MAX_DIAGNOSTIC_BYTES + 3);
    }

    #[test]
    fn diagnostics_redact_sensitive_fields() {
        let diagnostic = diagnostic(
            "copilot",
            ResourceKind::Mcp,
            DiscoveryScope::Global,
            "Authorization: Bearer secret-value headers={\"x\":\"secret\"}",
        );
        assert!(!diagnostic.message.contains("secret-value"));
        assert!(!diagnostic.message.contains("x"));
    }

    #[test]
    fn runner_reports_success_and_non_zero_exit() {
        let cwd = std::env::temp_dir();
        let success = run_process_internal(
            "cmd",
            &["/C", "echo", "ok"],
            &cwd,
            Duration::from_secs(2),
            false,
        )
        .expect("success");
        assert_eq!(success.exit_code, Some(0));
        assert!(success.stdout.contains("ok"));

        let failure = run_process_internal(
            "cmd",
            &["/C", "exit", "7"],
            &cwd,
            Duration::from_secs(2),
            false,
        )
        .expect("failure output");
        assert_eq!(failure.exit_code, Some(7));
    }

    #[test]
    fn runner_reports_timeout_and_truncated_output() {
        let cwd = std::env::temp_dir();
        let timeout = run_process_internal(
            "cmd",
            &["/C", "ping", "127.0.0.1", "-n", "3"],
            &cwd,
            Duration::from_millis(20),
            false,
        )
        .expect("timeout output");
        assert!(timeout.timed_out);

        let truncated = run_process_internal(
            "cmd",
            &["/C", "for /L %i in (1,1,500000) do @echo 1234567890"],
            &cwd,
            Duration::from_secs(30),
            false,
        )
        .expect("large output");
        assert!(truncated.truncated);
        assert!(truncated.stdout.len() <= MAX_CLI_OUTPUT_BYTES);
    }

    #[test]
    fn provider_order_is_deduplicated_and_unknowns_are_reported() {
        let (providers, unknown) = normalize_enabled_providers(&[
            "opencode".to_string(), "opencode".to_string(), "unknown".to_string(),
        ]);
        assert_eq!(providers, vec!["opencode"]);
        assert_eq!(unknown, vec!["unknown"]);
    }

    #[test]
    fn fixture_adapters_parse_opencode_skill_and_command_shapes() {
        let skills = parse_fixture_resources(
            r#"{"skills":[{"name":"review","path":"D:/skills/review/SKILL.md"}]}"#,
        )
        .expect("skills fixture");
        assert_eq!(skills[0].name, "review");
        assert_eq!(skills[0].effective_path.as_deref(), Some("D:/skills/review/SKILL.md"));

        let commands = parse_fixture_resources(
            r#"{"config":{"commands":[{"name":"deploy","file":"D:/commands/deploy.md"}]}}"#,
        )
        .expect("commands fixture");
        assert_eq!(commands[0].name, "deploy");

        let keyed = parse_fixture_resources(
            r#"{"commands":{"review":{"file":"D:/commands/review.md"}}}"#,
        )
        .expect("keyed commands fixture");
        assert_eq!(keyed[0].name, "review");

        let config: Value = serde_json::from_str(
            r#"{"command":{"review":{"description":"Review"}},"provider":{"review":{"name":"not-a-command"}}}"#,
        ).expect("scoped config fixture");
        let commands = parse_cli_resources(OPENCODE_PROVIDER, ResourceKind::Command, &config);
        assert_eq!(commands.len(), 1);
        assert_eq!(commands[0].name, "review");
    }

    #[test]
    fn fixture_adapters_parse_mcp_sources_and_enabled_state() {
        let resources = parse_fixture_resources(
            r#"{"servers":[{"name":"remote","source":"project","enabled":false}]}"#,
        )
        .expect("MCP fixture");
        assert_eq!(resources[0].source.as_deref(), Some("project"));
        assert_eq!(resources[0].enabled, Some(false));
    }

    #[test]
    fn fixture_adapters_reject_invalid_json_and_missing_names() {
        assert!(parse_fixture_resources("not-json").is_err());
        let resources = parse_fixture_resources(r#"{"skills":[{"path":"missing-name"}]}"#)
            .expect("missing name fixture");
        assert!(resources.is_empty());
    }

    #[test]
    fn registry_only_returns_enabled_provider_roots() {
        let mut settings = AppSettings::default().expect("settings");
        settings.claude_root = "D:/home/.claude".to_string();
        settings.codex_root = "D:/home/.codex".to_string();
        let project = PathBuf::from("D:/workspace");
        let skill_roots = skill_roots(
            DiscoveryScope::Project,
            Some(&project),
            &settings,
            &[CODEX_PROVIDER.to_string()],
        )
        .expect("skill roots");
        assert!(skill_roots.iter().all(|(provider, _)| provider == CODEX_PROVIDER));
        assert_eq!(skill_roots.len(), 2);

        let command_roots = command_roots(
            DiscoveryScope::Project,
            Some(&project),
            &settings,
            &[ANTIGRAVITY_PROVIDER.to_string()],
        )
        .expect("command roots");
        assert_eq!(command_roots.len(), 1);
        assert!(command_roots[0].1.ends_with(Path::new(".gemini").join("commands")));
    }

    #[test]
    fn unsupported_capability_does_not_start_a_process() {
        let result = discover_cli("claude", ResourceKind::Skill, DiscoveryScope::Project, None)
            .expect("unsupported capability should be skipped");
        assert!(result.is_empty());
    }

    #[test]
    fn project_and_global_probe_cwds_are_distinct() {
        let _guard = crate::shared_env_test_lock()
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let project = PathBuf::from("D:/project");
        assert_eq!(probe_cwd(DiscoveryScope::Project, Some("D:/project")).unwrap(), project);

        let appdata = std::env::temp_dir().join(format!("sessionhub-probe-test-{}", std::process::id()));
        let previous = std::env::var_os("COPILOT_SESSION_MANAGER_APPDATA_OVERRIDE");
        unsafe {
            std::env::set_var("COPILOT_SESSION_MANAGER_APPDATA_OVERRIDE", &appdata);
        }
        let global = probe_cwd(DiscoveryScope::Global, None).expect("global probe");
        assert!(global.ends_with(Path::new("SessionHub").join("cli-probe")));
        assert_ne!(global, project);
        let _ = std::fs::remove_dir_all(appdata);
        unsafe {
            match previous {
                Some(value) => std::env::set_var("COPILOT_SESSION_MANAGER_APPDATA_OVERRIDE", value),
                None => std::env::remove_var("COPILOT_SESSION_MANAGER_APPDATA_OVERRIDE"),
            }
        }
    }
}
