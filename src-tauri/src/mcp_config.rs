use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use toml_edit::{DocumentMut, Item, Table, Value as TomlValue};

use crate::agents_config::atomic_write_file;
use crate::settings::{
    default_app_data_dir, default_opencode_config_root, resolve_codex_root, resolve_copilot_root,
};
use crate::types::{
    AppSettings, CLAUDE_PROVIDER, CODEX_PROVIDER, COPILOT_PROVIDER, OPENCODE_PROVIDER,
};

pub(crate) const MCP_PROVIDERS: &[&str] = &[
    CLAUDE_PROVIDER,
    CODEX_PROVIDER,
    OPENCODE_PROVIDER,
    COPILOT_PROVIDER,
];

const MCP_DISABLED_FILE_NAME: &str = "mcp-disabled.json";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub(crate) enum McpScope {
    #[serde(rename_all = "camelCase")]
    Project {
        project_cwd: String,
    },
    Global,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct McpServerEntry {
    pub(crate) name: String,
    pub(crate) enabled: bool,
    pub(crate) config_json: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct McpProviderConfig {
    pub(crate) provider_id: String,
    pub(crate) config_path: String,
    pub(crate) config_exists: bool,
    pub(crate) servers: Vec<McpServerEntry>,
    pub(crate) error: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ConfigFormat {
    Json,
    Toml,
}

/// 每個 provider 讀寫設定檔所需的靜態參數：格式、區段鍵、是否有原生 enabled 旗標。
/// scope 只影響「解析出哪個設定檔路徑」（見 `mcp_config_path`），核心讀寫邏輯與 scope 無關。
struct ProviderSpec {
    format: ConfigFormat,
    section_key: &'static str,
    native_enabled_flag: bool,
}

fn provider_spec(provider: &str) -> Result<ProviderSpec, String> {
    match provider {
        CLAUDE_PROVIDER => Ok(ProviderSpec {
            format: ConfigFormat::Json,
            section_key: "mcpServers",
            native_enabled_flag: false,
        }),
        CODEX_PROVIDER => Ok(ProviderSpec {
            format: ConfigFormat::Toml,
            section_key: "mcp_servers",
            native_enabled_flag: true,
        }),
        OPENCODE_PROVIDER => Ok(ProviderSpec {
            format: ConfigFormat::Json,
            section_key: "mcp",
            native_enabled_flag: true,
        }),
        COPILOT_PROVIDER => Ok(ProviderSpec {
            format: ConfigFormat::Json,
            section_key: "mcpServers",
            native_enabled_flag: false,
        }),
        _ => Err(format!("unknown MCP provider: {provider}")),
    }
}

fn load_mcp_settings() -> Result<AppSettings, String> {
    crate::settings::load_settings_internal().or_else(|_| AppSettings::default())
}

/// 解析 provider + scope 對應的設定檔絕對路徑。global 分支沿用既有 provider root 解析函式；
/// project 分支以 `project_cwd` 為根接上各 provider 的專案層相對路徑。
pub(crate) fn mcp_config_path(provider: &str, scope: &McpScope) -> Result<PathBuf, String> {
    match scope {
        McpScope::Global => {
            let settings = load_mcp_settings()?;
            match provider {
                CLAUDE_PROVIDER => {
                    let user_profile = std::env::var("USERPROFILE")
                        .map_err(|_| "USERPROFILE environment variable is not set".to_string())?;
                    Ok(PathBuf::from(user_profile).join(".claude.json"))
                }
                CODEX_PROVIDER => {
                    Ok(resolve_codex_root(Some(settings.codex_root.as_str()))?.join("config.toml"))
                }
                OPENCODE_PROVIDER => Ok(default_opencode_config_root()?.join("opencode.json")),
                COPILOT_PROVIDER => Ok(resolve_copilot_root(Some(settings.copilot_root.as_str()))?
                    .join("mcp-config.json")),
                _ => Err(format!("unknown MCP provider: {provider}")),
            }
        }
        McpScope::Project { project_cwd } => {
            let project_root = PathBuf::from(project_cwd);
            match provider {
                CLAUDE_PROVIDER => Ok(project_root.join(".mcp.json")),
                CODEX_PROVIDER => Ok(project_root.join(".codex").join("config.toml")),
                OPENCODE_PROVIDER => Ok(project_root.join("opencode.json")),
                COPILOT_PROVIDER => {
                    let github_path = project_root.join(".github").join("mcp.json");
                    let legacy_path = project_root.join(".mcp.json");
                    if !github_path.exists() && legacy_path.exists() {
                        Ok(legacy_path)
                    } else {
                        Ok(github_path)
                    }
                }
                _ => Err(format!("unknown MCP provider: {provider}")),
            }
        }
    }
}

/// copilot 專案層寫入固定使用 `.github/mcp.json`，與讀取時的 fallback 邏輯不同，
/// 因此路徑解析獨立於 `mcp_config_path`（讀取用）。
fn mcp_config_write_path(provider: &str, scope: &McpScope) -> Result<PathBuf, String> {
    if provider == COPILOT_PROVIDER {
        if let McpScope::Project { project_cwd } = scope {
            return Ok(PathBuf::from(project_cwd).join(".github").join("mcp.json"));
        }
    }
    mcp_config_path(provider, scope)
}

fn scope_key(scope: &McpScope) -> String {
    match scope {
        McpScope::Global => "global".to_string(),
        McpScope::Project { project_cwd } => project_cwd.trim().to_lowercase().replace('/', "\\"),
    }
}

fn disabled_store_path() -> Result<PathBuf, String> {
    Ok(default_app_data_dir()?.join(MCP_DISABLED_FILE_NAME))
}

type DisabledStore = BTreeMap<String, BTreeMap<String, Value>>;

fn load_disabled_store() -> Result<DisabledStore, String> {
    let path = disabled_store_path()?;
    if !path.is_file() {
        return Ok(DisabledStore::new());
    }
    let content = fs::read_to_string(&path)
        .map_err(|error| format!("failed to read {}: {error}", path.display()))?;
    if content.trim().is_empty() {
        return Ok(DisabledStore::new());
    }
    serde_json::from_str(&content)
        .map_err(|error| format!("failed to parse {}: {error}", path.display()))
}

fn write_disabled_store(store: &DisabledStore) -> Result<(), String> {
    let path = disabled_store_path()?;
    let content = serde_json::to_vec_pretty(store)
        .map_err(|error| format!("failed to serialize disabled MCP store: {error}"))?;
    atomic_write_file(&path, &content)
}

fn disabled_bucket_key(provider: &str, scope: &McpScope) -> String {
    format!("{provider}::{}", scope_key(scope))
}

// ---------- JSON 讀寫 ----------

fn read_json_object(path: &Path) -> Result<Map<String, Value>, String> {
    if !path.is_file() {
        return Ok(Map::new());
    }
    let content = fs::read_to_string(path)
        .map_err(|error| format!("failed to read {}: {error}", path.display()))?;
    if content.trim().is_empty() {
        return Ok(Map::new());
    }
    let value: Value = serde_json::from_str(&content)
        .map_err(|error| format!("failed to parse {}: {error}", path.display()))?;
    match value {
        Value::Object(map) => Ok(map),
        _ => Err(format!("{} does not contain a JSON object", path.display())),
    }
}

fn write_json_object(path: &Path, root: &Map<String, Value>) -> Result<(), String> {
    let content = serde_json::to_vec_pretty(root)
        .map_err(|error| format!("failed to serialize {}: {error}", path.display()))?;
    atomic_write_file(path, &content)
}

fn json_section_entries(root: &Map<String, Value>, section_key: &str) -> Vec<(String, Value)> {
    match root.get(section_key) {
        Some(Value::Object(map)) => map.iter().map(|(k, v)| (k.clone(), v.clone())).collect(),
        _ => Vec::new(),
    }
}

// ---------- TOML 讀寫 ----------

fn read_toml_document(path: &Path) -> Result<DocumentMut, String> {
    if !path.is_file() {
        return Ok(DocumentMut::new());
    }
    let content = fs::read_to_string(path)
        .map_err(|error| format!("failed to read {}: {error}", path.display()))?;
    content
        .parse::<DocumentMut>()
        .map_err(|error| format!("failed to parse {}: {error}", path.display()))
}

fn write_toml_document(path: &Path, document: &DocumentMut) -> Result<(), String> {
    atomic_write_file(path, document.to_string().as_bytes())
}

fn toml_section_entries(document: &DocumentMut, section_key: &str) -> Vec<(String, Value)> {
    let Some(section) = document.get(section_key).and_then(Item::as_table) else {
        return Vec::new();
    };
    section
        .iter()
        .filter_map(|(name, item)| toml_item_to_json(item).map(|value| (name.to_string(), value)))
        .collect()
}

// ---------- TOML <-> JSON 轉換（D2/D4）----------

fn toml_item_to_json(item: &Item) -> Option<Value> {
    match item {
        Item::None => None,
        Item::Value(value) => Some(toml_value_to_json(value)),
        Item::Table(table) => Some(toml_table_to_json(table)),
        Item::ArrayOfTables(array) => {
            let items = array.iter().map(toml_table_to_json).collect();
            Some(Value::Array(items))
        }
    }
}

fn toml_table_to_json(table: &Table) -> Value {
    let mut map = Map::new();
    for (key, item) in table.iter() {
        if let Some(value) = toml_item_to_json(item) {
            map.insert(key.to_string(), value);
        }
    }
    Value::Object(map)
}

fn toml_value_to_json(value: &TomlValue) -> Value {
    match value {
        TomlValue::String(s) => Value::String(s.value().clone()),
        TomlValue::Integer(i) => Value::Number((*i.value()).into()),
        TomlValue::Float(f) => serde_json::Number::from_f64(*f.value())
            .map(Value::Number)
            .unwrap_or(Value::Null),
        TomlValue::Boolean(b) => Value::Bool(*b.value()),
        // datetime→string：TOML 原生 datetime 無對應 JSON 型別，序列化為字串保留資訊（D2）。
        TomlValue::Datetime(dt) => Value::String(dt.value().to_string()),
        TomlValue::Array(array) => Value::Array(array.iter().map(toml_value_to_json).collect()),
        TomlValue::InlineTable(table) => {
            let mut map = Map::new();
            for (key, value) in table.iter() {
                map.insert(key.to_string(), toml_value_to_json(value));
            }
            Value::Object(map)
        }
    }
}

fn json_to_toml_item(value: &Value) -> Result<Item, String> {
    match value {
        Value::Null => Err("null values are not supported in TOML configs".to_string()),
        Value::Bool(b) => Ok(Item::Value(TomlValue::from(*b))),
        Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Ok(Item::Value(TomlValue::from(i)))
            } else if let Some(f) = n.as_f64() {
                Ok(Item::Value(TomlValue::from(f)))
            } else {
                Err("unsupported numeric value".to_string())
            }
        }
        Value::String(s) => Ok(Item::Value(TomlValue::from(s.clone()))),
        Value::Array(items) => {
            let mut array = toml_edit::Array::new();
            for item in items {
                let toml_item = json_to_toml_item(item)?;
                let Item::Value(toml_value) = toml_item else {
                    return Err("nested tables inside arrays are not supported".to_string());
                };
                array.push(toml_value);
            }
            Ok(Item::Value(TomlValue::Array(array)))
        }
        Value::Object(map) => {
            let mut table = Table::new();
            for (key, item_value) in map {
                table.insert(key, json_to_toml_item(item_value)?);
            }
            Ok(Item::Table(table))
        }
    }
}

// ---------- List ----------

pub(crate) fn list_mcp_configs_internal(
    scope: &McpScope,
) -> Result<Vec<McpProviderConfig>, String> {
    let disabled_store = load_disabled_store()?;
    let mut results = Vec::with_capacity(MCP_PROVIDERS.len());
    for &provider in MCP_PROVIDERS {
        results.push(list_one_provider(provider, scope, &disabled_store));
    }
    Ok(results)
}

fn list_one_provider(
    provider: &str,
    scope: &McpScope,
    disabled_store: &DisabledStore,
) -> McpProviderConfig {
    match list_one_provider_inner(provider, scope, disabled_store) {
        Ok(config) => config,
        Err(error) => McpProviderConfig {
            provider_id: provider.to_string(),
            config_path: String::new(),
            config_exists: false,
            servers: Vec::new(),
            error: Some(error),
        },
    }
}

fn list_one_provider_inner(
    provider: &str,
    scope: &McpScope,
    disabled_store: &DisabledStore,
) -> Result<McpProviderConfig, String> {
    let spec = provider_spec(provider)?;
    let path = mcp_config_path(provider, scope)?;
    let config_exists = path.is_file();

    let raw_entries: Vec<(String, Value)> = match spec.format {
        ConfigFormat::Json => {
            let root = read_json_object(&path)?;
            json_section_entries(&root, spec.section_key)
        }
        ConfigFormat::Toml => {
            let document = read_toml_document(&path)?;
            toml_section_entries(&document, spec.section_key)
        }
    };

    let mut servers = Vec::new();
    for (name, mut value) in raw_entries {
        let enabled = if spec.native_enabled_flag {
            extract_native_enabled(&mut value)
        } else {
            true
        };
        servers.push(McpServerEntry {
            name,
            enabled,
            config_json: serde_json::to_string_pretty(&value)
                .map_err(|error| format!("failed to serialize server config: {error}"))?,
        });
    }

    if !spec.native_enabled_flag {
        let bucket_key = disabled_bucket_key(provider, scope);
        if let Some(bucket) = disabled_store.get(&bucket_key) {
            for (name, value) in bucket {
                if servers.iter().any(|entry| &entry.name == name) {
                    // 設定檔已存在同名項目：以設定檔為準，暫存視為過期（風險列表 D4 說明）。
                    continue;
                }
                servers.push(McpServerEntry {
                    name: name.clone(),
                    enabled: false,
                    config_json: serde_json::to_string_pretty(value)
                        .map_err(|error| format!("failed to serialize server config: {error}"))?,
                });
            }
        }
    }

    servers.sort_by(|a, b| a.name.cmp(&b.name));

    Ok(McpProviderConfig {
        provider_id: provider.to_string(),
        config_path: normalize_display_path(&path),
        config_exists,
        servers,
        error: None,
    })
}

/// 讀取並移除原生 `enabled` 旗標（codex/opencode）；旗標不存在時視為啟用（D4：預設即啟用）。
fn extract_native_enabled(value: &mut Value) -> bool {
    let Value::Object(map) = value else {
        return true;
    };
    match map.remove("enabled") {
        Some(Value::Bool(b)) => b,
        _ => true,
    }
}

// ---------- Upsert ----------

pub(crate) fn upsert_mcp_server_internal(
    scope: &McpScope,
    provider: &str,
    name: &str,
    original_name: Option<&str>,
    config_json: &str,
) -> Result<(), String> {
    let name = name.trim();
    if name.is_empty() {
        return Err("server name must not be empty".to_string());
    }
    let spec = provider_spec(provider)?;
    let parsed: Value = serde_json::from_str(config_json)
        .map_err(|error| format!("invalid JSON config: {error}"))?;
    if !parsed.is_object() {
        return Err("server config must be a JSON object".to_string());
    }

    let rename_from = original_name
        .map(str::trim)
        .filter(|value| !value.is_empty() && *value != name);

    // 若目前在停用暫存中（改名前或原名），先就地更新暫存並保持停用狀態，不寫回設定檔。
    if !spec.native_enabled_flag {
        let disabled_lookup_name = rename_from.unwrap_or(name);
        let mut store = load_disabled_store()?;
        let bucket_key = disabled_bucket_key(provider, scope);
        if let Some(bucket) = store.get_mut(&bucket_key) {
            if bucket.remove(disabled_lookup_name).is_some() {
                bucket.insert(name.to_string(), parsed);
                write_disabled_store(&store)?;
                return Ok(());
            }
        }
    }

    let write_path = mcp_config_write_path(provider, scope)?;
    match spec.format {
        ConfigFormat::Json => {
            let mut root = read_json_object(&write_path)?;
            let section = root
                .entry(spec.section_key.to_string())
                .or_insert_with(|| Value::Object(Map::new()));
            let Value::Object(section_map) = section else {
                return Err(format!("{} is not a JSON object", spec.section_key));
            };
            if let Some(old_name) = rename_from {
                section_map.remove(old_name);
            }
            section_map.insert(name.to_string(), parsed);
            write_json_object(&write_path, &root)?;
        }
        ConfigFormat::Toml => {
            let mut document = read_toml_document(&write_path)?;
            ensure_toml_table(&mut document, spec.section_key);
            let section = document[spec.section_key]
                .as_table_mut()
                .ok_or_else(|| format!("{} is not a TOML table", spec.section_key))?;
            if let Some(old_name) = rename_from {
                section.remove(old_name);
            }
            let item = json_to_toml_item(&parsed)?;
            section.insert(name, item);
            write_toml_document(&write_path, &document)?;
        }
    }

    Ok(())
}

fn ensure_toml_table(document: &mut DocumentMut, section_key: &str) {
    if document.get(section_key).and_then(Item::as_table).is_none() {
        document[section_key] = Item::Table(Table::new());
    }
}

// ---------- Delete ----------

pub(crate) fn delete_mcp_server_internal(
    scope: &McpScope,
    provider: &str,
    name: &str,
) -> Result<(), String> {
    let spec = provider_spec(provider)?;

    if !spec.native_enabled_flag {
        let mut store = load_disabled_store()?;
        let bucket_key = disabled_bucket_key(provider, scope);
        if let Some(bucket) = store.get_mut(&bucket_key) {
            if bucket.remove(name).is_some() {
                if bucket.is_empty() {
                    store.remove(&bucket_key);
                }
                write_disabled_store(&store)?;
            }
        }
    }

    let write_path = mcp_config_write_path(provider, scope)?;
    match spec.format {
        ConfigFormat::Json => {
            let mut root = read_json_object(&write_path)?;
            if let Some(Value::Object(section_map)) = root.get_mut(spec.section_key) {
                if section_map.remove(name).is_some() {
                    write_json_object(&write_path, &root)?;
                }
            }
        }
        ConfigFormat::Toml => {
            let mut document = read_toml_document(&write_path)?;
            if let Some(section) = document
                .get_mut(spec.section_key)
                .and_then(Item::as_table_mut)
            {
                if section.remove(name).is_some() {
                    write_toml_document(&write_path, &document)?;
                }
            }
        }
    }

    Ok(())
}

// ---------- Enable / Disable ----------

pub(crate) fn set_mcp_server_enabled_internal(
    scope: &McpScope,
    provider: &str,
    name: &str,
    enabled: bool,
) -> Result<(), String> {
    let spec = provider_spec(provider)?;
    if spec.native_enabled_flag {
        set_native_enabled(scope, provider, &spec, name, enabled)
    } else {
        set_store_backed_enabled(scope, provider, &spec, name, enabled)
    }
}

fn set_native_enabled(
    scope: &McpScope,
    provider: &str,
    spec: &ProviderSpec,
    name: &str,
    enabled: bool,
) -> Result<(), String> {
    let write_path = mcp_config_write_path(provider, scope)?;
    match spec.format {
        ConfigFormat::Json => {
            let mut root = read_json_object(&write_path)?;
            let Some(Value::Object(section_map)) = root.get_mut(spec.section_key) else {
                return Err(format!("server {name} not found"));
            };
            let Some(Value::Object(entry)) = section_map.get_mut(name) else {
                return Err(format!("server {name} not found"));
            };
            if enabled {
                entry.remove("enabled");
            } else {
                entry.insert("enabled".to_string(), Value::Bool(false));
            }
            write_json_object(&write_path, &root)?;
        }
        ConfigFormat::Toml => {
            let mut document = read_toml_document(&write_path)?;
            let section = document
                .get_mut(spec.section_key)
                .and_then(Item::as_table_mut)
                .ok_or_else(|| format!("server {name} not found"))?;
            let entry = section
                .get_mut(name)
                .and_then(Item::as_table_like_mut)
                .ok_or_else(|| format!("server {name} not found"))?;
            if enabled {
                entry.remove("enabled");
            } else {
                entry.insert("enabled", Item::Value(TomlValue::from(false)));
            }
            write_toml_document(&write_path, &document)?;
        }
    }
    Ok(())
}

fn set_store_backed_enabled(
    scope: &McpScope,
    provider: &str,
    spec: &ProviderSpec,
    name: &str,
    enabled: bool,
) -> Result<(), String> {
    let write_path = mcp_config_write_path(provider, scope)?;
    let bucket_key = disabled_bucket_key(provider, scope);

    if enabled {
        // 啟用：從暫存取回設定值，寫回設定檔，並自暫存移除。
        let mut store = load_disabled_store()?;
        let Some(bucket) = store.get_mut(&bucket_key) else {
            return Err(format!("server {name} is not disabled"));
        };
        let Some(value) = bucket.remove(name) else {
            return Err(format!("server {name} is not disabled"));
        };
        if bucket.is_empty() {
            store.remove(&bucket_key);
        }

        let mut root = read_json_object(&write_path)?;
        let section = root
            .entry(spec.section_key.to_string())
            .or_insert_with(|| Value::Object(Map::new()));
        let Value::Object(section_map) = section else {
            return Err(format!("{} is not a JSON object", spec.section_key));
        };
        section_map.insert(name.to_string(), value);
        write_json_object(&write_path, &root)?;
        write_disabled_store(&store)?;
    } else {
        // 停用：從設定檔移除該 server，搬到暫存檔。
        let mut root = read_json_object(&write_path)?;
        let Some(Value::Object(section_map)) = root.get_mut(spec.section_key) else {
            return Err(format!("server {name} not found"));
        };
        let Some(value) = section_map.remove(name) else {
            return Err(format!("server {name} not found"));
        };
        write_json_object(&write_path, &root)?;

        let mut store = load_disabled_store()?;
        store
            .entry(bucket_key)
            .or_insert_with(BTreeMap::new)
            .insert(name.to_string(), value);
        write_disabled_store(&store)?;
    }

    Ok(())
}

// ---------- Codex trust 偵測（D9）----------

pub(crate) fn is_codex_project_trusted(project_cwd: &str) -> Result<bool, String> {
    let settings = load_mcp_settings()?;
    let codex_config_path =
        resolve_codex_root(Some(settings.codex_root.as_str()))?.join("config.toml");
    if !codex_config_path.is_file() {
        return Ok(false);
    }
    let content = fs::read_to_string(&codex_config_path)
        .map_err(|error| format!("failed to read {}: {error}", codex_config_path.display()))?;
    let document = content
        .parse::<DocumentMut>()
        .map_err(|error| format!("failed to parse {}: {error}", codex_config_path.display()))?;

    let Some(projects) = document.get("projects").and_then(Item::as_table) else {
        return Ok(false);
    };

    let normalized_target = normalize_trust_path(project_cwd);
    for (raw_path, item) in projects.iter() {
        if normalize_trust_path(raw_path) != normalized_target {
            continue;
        }
        let trust_level = item
            .as_table_like()
            .and_then(|table| table.get("trust_level"))
            .and_then(Item::as_str);
        return Ok(trust_level == Some("trusted"));
    }

    Ok(false)
}

fn normalize_trust_path(path: &str) -> String {
    path.trim().to_lowercase().replace('/', "\\")
}

fn normalize_display_path(path: &Path) -> String {
    path.to_string_lossy().replace('/', "\\")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::OsString;

    /// 環境變數守衛：與 lib.rs 測試共用同一把 `shared_env_test_lock`（序列化所有
    /// 會改動行程 env 的測試），並於 Drop 時還原被覆寫的變數原值，避免汙染後續測試。
    struct EnvGuard {
        _lock: std::sync::MutexGuard<'static, ()>,
        temp_dir: PathBuf,
        saved_vars: Vec<(&'static str, Option<OsString>)>,
    }

    impl EnvGuard {
        fn new(test_name: &str) -> Self {
            let lock = crate::shared_env_test_lock()
                .lock()
                .unwrap_or_else(|poison| poison.into_inner());
            let temp_dir = std::env::temp_dir().join(format!(
                "sessionhub-mcp-test-{test_name}-{}",
                std::process::id()
            ));
            let _ = fs::remove_dir_all(&temp_dir);
            fs::create_dir_all(&temp_dir).expect("create temp dir");
            let user_profile = temp_dir.join("home");
            let app_data = temp_dir.join("appdata");
            fs::create_dir_all(&user_profile).expect("create home dir");
            fs::create_dir_all(&app_data).expect("create appdata dir");

            let keys: [&'static str; 3] = [
                "USERPROFILE",
                "APPDATA",
                "COPILOT_SESSION_MANAGER_APPDATA_OVERRIDE",
            ];
            let saved_vars = keys
                .iter()
                .map(|key| (*key, std::env::var_os(key)))
                .collect();

            std::env::set_var("USERPROFILE", &user_profile);
            std::env::set_var("APPDATA", &app_data);
            std::env::set_var(
                "COPILOT_SESSION_MANAGER_APPDATA_OVERRIDE",
                app_data.parent().unwrap(),
            );
            Self {
                _lock: lock,
                temp_dir,
                saved_vars,
            }
        }

        fn home(&self) -> PathBuf {
            self.temp_dir.join("home")
        }

        fn project_dir(&self) -> PathBuf {
            let dir = self.temp_dir.join("project");
            fs::create_dir_all(&dir).expect("create project dir");
            dir
        }
    }

    impl Drop for EnvGuard {
        fn drop(&mut self) {
            for (key, previous) in self.saved_vars.drain(..) {
                match previous {
                    Some(value) => std::env::set_var(key, value),
                    None => std::env::remove_var(key),
                }
            }
            let _ = fs::remove_dir_all(&self.temp_dir);
        }
    }

    #[test]
    fn codex_toml_preserves_comments_and_toggles() {
        let env = EnvGuard::new("codex-comments");
        let codex_root = env.home().join(".codex");
        fs::create_dir_all(&codex_root).unwrap();
        let config_path = codex_root.join("config.toml");
        fs::write(
            &config_path,
            "# top comment\nmodel = \"gpt-5\"\n\n[mcp_servers.existing]\ncommand = \"foo\"\n",
        )
        .unwrap();

        upsert_mcp_server_internal(
            &McpScope::Global,
            CODEX_PROVIDER,
            "new-server",
            None,
            r#"{"command": "bar", "args": ["--flag"]}"#,
        )
        .unwrap();

        let content = fs::read_to_string(&config_path).unwrap();
        assert!(content.contains("# top comment"));
        assert!(content.contains("model = \"gpt-5\""));
        assert!(content.contains("[mcp_servers.existing]"));
        assert!(content.contains("[mcp_servers.new-server]"));

        set_mcp_server_enabled_internal(&McpScope::Global, CODEX_PROVIDER, "new-server", false)
            .unwrap();
        let content = fs::read_to_string(&config_path).unwrap();
        assert!(content.contains("enabled = false"));

        set_mcp_server_enabled_internal(&McpScope::Global, CODEX_PROVIDER, "new-server", true)
            .unwrap();
        let content = fs::read_to_string(&config_path).unwrap();
        assert!(!content.contains("enabled = false"));

        let configs = list_mcp_configs_internal(&McpScope::Global).unwrap();
        let codex = configs
            .iter()
            .find(|c| c.provider_id == CODEX_PROVIDER)
            .unwrap();
        assert_eq!(codex.servers.len(), 2);
    }

    #[test]
    fn claude_disable_moves_to_store_and_restore_writes_back() {
        let env = EnvGuard::new("claude-disable");
        let claude_json = env.home().join(".claude.json");
        fs::write(
            &claude_json,
            r#"{"otherSetting": true, "mcpServers": {"srv": {"command": "foo"}}}"#,
        )
        .unwrap();

        set_mcp_server_enabled_internal(&McpScope::Global, CLAUDE_PROVIDER, "srv", false).unwrap();
        let content = fs::read_to_string(&claude_json).unwrap();
        let value: Value = serde_json::from_str(&content).unwrap();
        assert!(value.get("mcpServers").unwrap().get("srv").is_none());
        assert_eq!(value.get("otherSetting"), Some(&Value::Bool(true)));

        let configs = list_mcp_configs_internal(&McpScope::Global).unwrap();
        let claude = configs
            .iter()
            .find(|c| c.provider_id == CLAUDE_PROVIDER)
            .unwrap();
        let entry = claude.servers.iter().find(|s| s.name == "srv").unwrap();
        assert!(!entry.enabled);

        set_mcp_server_enabled_internal(&McpScope::Global, CLAUDE_PROVIDER, "srv", true).unwrap();
        let content = fs::read_to_string(&claude_json).unwrap();
        let value: Value = serde_json::from_str(&content).unwrap();
        assert!(value.get("mcpServers").unwrap().get("srv").is_some());
        assert_eq!(value.get("otherSetting"), Some(&Value::Bool(true)));
    }

    #[test]
    fn opencode_rename_and_native_toggle() {
        let env = EnvGuard::new("opencode-rename");
        let config_root = env.home().join(".config").join("opencode");
        fs::create_dir_all(&config_root).unwrap();
        let config_path = config_root.join("opencode.json");
        fs::write(
            &config_path,
            r#"{"mcp": {"old-name": {"command": ["run"]}}}"#,
        )
        .unwrap();

        upsert_mcp_server_internal(
            &McpScope::Global,
            OPENCODE_PROVIDER,
            "new-name",
            Some("old-name"),
            r#"{"command": ["run", "--x"]}"#,
        )
        .unwrap();
        let content = fs::read_to_string(&config_path).unwrap();
        let value: Value = serde_json::from_str(&content).unwrap();
        let mcp = value.get("mcp").unwrap().as_object().unwrap();
        assert!(!mcp.contains_key("old-name"));
        assert!(mcp.contains_key("new-name"));

        set_mcp_server_enabled_internal(&McpScope::Global, OPENCODE_PROVIDER, "new-name", false)
            .unwrap();
        let content = fs::read_to_string(&config_path).unwrap();
        let value: Value = serde_json::from_str(&content).unwrap();
        let entry = value.get("mcp").unwrap().get("new-name").unwrap();
        assert_eq!(entry.get("enabled"), Some(&Value::Bool(false)));
    }

    #[test]
    fn copilot_creates_file_and_rejects_invalid_input() {
        let env = EnvGuard::new("copilot-create");
        let project_dir = env.project_dir();
        let scope = McpScope::Project {
            project_cwd: project_dir.to_string_lossy().to_string(),
        };

        let err = upsert_mcp_server_internal(&scope, COPILOT_PROVIDER, "srv", None, "not json");
        assert!(err.is_err());

        let err = upsert_mcp_server_internal(&scope, COPILOT_PROVIDER, "  ", None, "{}");
        assert!(err.is_err());

        upsert_mcp_server_internal(
            &scope,
            COPILOT_PROVIDER,
            "srv",
            None,
            r#"{"url": "http://x"}"#,
        )
        .unwrap();
        let written_path = project_dir.join(".github").join("mcp.json");
        assert!(written_path.is_file());
    }

    #[test]
    fn codex_trust_detection_covers_all_branches() {
        let env = EnvGuard::new("codex-trust");
        let codex_root = env.home().join(".codex");
        fs::create_dir_all(&codex_root).unwrap();
        let config_path = codex_root.join("config.toml");
        let project = env.project_dir();
        let project_str = project.to_string_lossy().replace('/', "\\");

        // 區塊不存在
        fs::write(&config_path, "model = \"gpt-5\"\n").unwrap();
        assert!(!is_codex_project_trusted(&project_str).unwrap());

        // untrusted
        fs::write(
            &config_path,
            format!(
                "[projects.\"{}\"]\ntrust_level = \"untrusted\"\n",
                project_str.replace('\\', "\\\\")
            ),
        )
        .unwrap();
        assert!(!is_codex_project_trusted(&project_str).unwrap());

        // trusted
        fs::write(
            &config_path,
            format!(
                "[projects.\"{}\"]\ntrust_level = \"trusted\"\n",
                project_str.replace('\\', "\\\\")
            ),
        )
        .unwrap();
        assert!(is_codex_project_trusted(&project_str).unwrap());

        // 大小寫差異仍應比對成功
        assert!(is_codex_project_trusted(&project_str.to_uppercase()).unwrap());
    }

    #[test]
    fn project_scope_and_global_disabled_store_are_isolated() {
        let env = EnvGuard::new("scope-isolation");
        let claude_json = env.home().join(".claude.json");
        fs::write(
            &claude_json,
            r#"{"mcpServers": {"srv": {"command": "global"}}}"#,
        )
        .unwrap();

        let project_dir = env.project_dir();
        let project_mcp = project_dir.join(".mcp.json");
        fs::write(
            &project_mcp,
            r#"{"mcpServers": {"srv": {"command": "project"}}}"#,
        )
        .unwrap();
        let scope = McpScope::Project {
            project_cwd: project_dir.to_string_lossy().to_string(),
        };

        set_mcp_server_enabled_internal(&McpScope::Global, CLAUDE_PROVIDER, "srv", false).unwrap();
        set_mcp_server_enabled_internal(&scope, CLAUDE_PROVIDER, "srv", false).unwrap();

        let global_configs = list_mcp_configs_internal(&McpScope::Global).unwrap();
        let global_claude = global_configs
            .iter()
            .find(|c| c.provider_id == CLAUDE_PROVIDER)
            .unwrap();
        assert!(
            !global_claude
                .servers
                .iter()
                .find(|s| s.name == "srv")
                .unwrap()
                .enabled
        );

        let project_configs = list_mcp_configs_internal(&scope).unwrap();
        let project_claude = project_configs
            .iter()
            .find(|c| c.provider_id == CLAUDE_PROVIDER)
            .unwrap();
        assert!(
            !project_claude
                .servers
                .iter()
                .find(|s| s.name == "srv")
                .unwrap()
                .enabled
        );

        set_mcp_server_enabled_internal(&McpScope::Global, CLAUDE_PROVIDER, "srv", true).unwrap();
        let project_configs = list_mcp_configs_internal(&scope).unwrap();
        let project_claude = project_configs
            .iter()
            .find(|c| c.provider_id == CLAUDE_PROVIDER)
            .unwrap();
        assert!(
            !project_claude
                .servers
                .iter()
                .find(|s| s.name == "srv")
                .unwrap()
                .enabled
        );
    }

    #[test]
    fn delete_is_idempotent() {
        let env = EnvGuard::new("delete-idempotent");
        let claude_json = env.home().join(".claude.json");
        fs::write(
            &claude_json,
            r#"{"mcpServers": {"srv": {"command": "foo"}}}"#,
        )
        .unwrap();
        delete_mcp_server_internal(&McpScope::Global, CLAUDE_PROVIDER, "srv").unwrap();
        delete_mcp_server_internal(&McpScope::Global, CLAUDE_PROVIDER, "srv").unwrap();
        let configs = list_mcp_configs_internal(&McpScope::Global).unwrap();
        let claude = configs
            .iter()
            .find(|c| c.provider_id == CLAUDE_PROVIDER)
            .unwrap();
        assert!(claude.servers.is_empty());
    }
}
