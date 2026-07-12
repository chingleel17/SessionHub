use std::collections::BTreeMap;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::agents_config::atomic_write_file;
use crate::settings::resolve_antigravity_root;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub(crate) enum AntigravityHookScope {
    Global,
    #[serde(rename_all = "camelCase")]
    Project { project_cwd: String },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AntigravityHookEntry {
    #[serde(rename = "type")]
    pub(crate) hook_type: String,
    pub(crate) command: String,
    #[serde(default)]
    pub(crate) timeout: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AntigravityHookMatcher {
    pub(crate) matcher: String,
    pub(crate) hooks: Vec<AntigravityHookEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AntigravityHookGroup {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) enabled: Option<bool>,
    #[serde(flatten)]
    pub(crate) events: BTreeMap<String, Vec<AntigravityHookMatcher>>,
}

pub(crate) type AntigravityHookConfig = BTreeMap<String, AntigravityHookGroup>;

fn global_hooks_path() -> Result<PathBuf, String> {
    Ok(resolve_antigravity_root(None)?.join("config").join("hooks.json"))
}

fn project_hooks_path(project_cwd: &str) -> PathBuf {
    PathBuf::from(project_cwd).join(".agents").join("hooks.json")
}

pub(crate) fn resolve_hooks_path(scope: &AntigravityHookScope) -> Result<PathBuf, String> {
    match scope {
        AntigravityHookScope::Global => global_hooks_path(),
        AntigravityHookScope::Project { project_cwd } => Ok(project_hooks_path(project_cwd)),
    }
}

/// 讀取 hooks.json；檔案不存在時回傳空清單，不視為錯誤
pub(crate) fn read_antigravity_hooks(
    scope: &AntigravityHookScope,
) -> Result<AntigravityHookConfig, String> {
    let path = resolve_hooks_path(scope)?;
    if !path.exists() {
        return Ok(AntigravityHookConfig::new());
    }

    let content = std::fs::read_to_string(&path)
        .map_err(|error| format!("failed to read {}: {error}", path.display()))?;

    if content.trim().is_empty() {
        return Ok(AntigravityHookConfig::new());
    }

    serde_json::from_str::<AntigravityHookConfig>(&content)
        .map_err(|error| format!("failed to parse {}: {error}", path.display()))
}

pub(crate) fn write_antigravity_hooks(
    scope: &AntigravityHookScope,
    config: &AntigravityHookConfig,
) -> Result<(), String> {
    let path = resolve_hooks_path(scope)?;
    let content = serde_json::to_string_pretty(config)
        .map_err(|error| format!("failed to serialize hooks.json: {error}"))?;
    atomic_write_file(&path, content.as_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn unique_test_dir(label: &str) -> PathBuf {
        let mut dir = std::env::temp_dir();
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        dir.push(format!("sessionhub-antigravity-hooks-{label}-{nanos}"));
        dir
    }

    #[test]
    fn read_returns_empty_when_file_missing() {
        let scope = AntigravityHookScope::Project {
            project_cwd: unique_test_dir("missing").to_string_lossy().to_string(),
        };
        let config = read_antigravity_hooks(&scope).expect("read");
        assert!(config.is_empty());
    }

    #[test]
    fn write_then_read_roundtrip() {
        let project_dir = unique_test_dir("roundtrip");
        fs::create_dir_all(&project_dir).expect("create project dir");
        let scope = AntigravityHookScope::Project {
            project_cwd: project_dir.to_string_lossy().to_string(),
        };

        let mut config = AntigravityHookConfig::new();
        let mut group = AntigravityHookGroup::default();
        group.events.insert(
            "PreToolUse".to_string(),
            vec![AntigravityHookMatcher {
                matcher: "run_command".to_string(),
                hooks: vec![AntigravityHookEntry {
                    hook_type: "command".to_string(),
                    command: "C:/x.bat".to_string(),
                    timeout: Some(10),
                }],
            }],
        );
        config.insert("SessionHub".to_string(), group);

        write_antigravity_hooks(&scope, &config).expect("write");
        let read_back = read_antigravity_hooks(&scope).expect("read");
        assert_eq!(read_back, config);

        fs::remove_dir_all(&project_dir).ok();
    }

    #[test]
    fn parse_legacy_schema_sample() {
        let sample = r#"{
            "群組名": {
                "enabled": true,
                "PreToolUse": [
                    { "matcher": "run_command", "hooks": [{ "type": "command", "command": "C:/x.bat", "timeout": 10 }] }
                ]
            }
        }"#;
        let parsed: AntigravityHookConfig = serde_json::from_str(sample).expect("parse sample schema");
        let group = parsed.get("群組名").expect("group present");
        assert_eq!(group.enabled, Some(true));
        let matchers = group.events.get("PreToolUse").expect("event present");
        assert_eq!(matchers.len(), 1);
        assert_eq!(matchers[0].matcher, "run_command");
        assert_eq!(matchers[0].hooks[0].command, "C:/x.bat");
        assert_eq!(matchers[0].hooks[0].timeout, Some(10));
    }
}
