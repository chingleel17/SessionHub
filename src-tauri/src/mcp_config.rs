use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use toml_edit::{DocumentMut, Item, Table, Value as TomlValue};

use crate::agents_config::atomic_write_file;
use crate::resource_discovery::{
    discover_cli, normalize_enabled_providers, DiscoveryDiagnostic, DiscoveryScope, ResourceKind,
};
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
    pub(crate) effective: Option<bool>,
    pub(crate) source: Option<String>,
    pub(crate) scope: Option<DiscoveryScope>,
    pub(crate) editable: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct EnsuredMcpConfigFile {
    pub(crate) path: String,
    pub(crate) created: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct McpProviderConfig {
    pub(crate) provider_id: String,
    pub(crate) config_path: String,
    pub(crate) config_exists: bool,
    pub(crate) servers: Vec<McpServerEntry>,
    pub(crate) error: Option<String>,
    pub(crate) enabled: bool,
    pub(crate) diagnostics: Vec<DiscoveryDiagnostic>,
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
    /// opencode 同時存在 `mcp.<name>`（v1）與 `mcp.servers.<name>`（v2）兩種配置。
    opencode_layout: bool,
}

/// opencode v2 存放 server 的巢狀鍵。
const OPENCODE_SERVERS_KEY: &str = "servers";
/// opencode `mcp` 區段內不代表 server 的保留鍵。
const OPENCODE_RESERVED_KEYS: [&str; 2] = [OPENCODE_SERVERS_KEY, "timeout"];
const OPENCODE_JSON_FILE: &str = "opencode.json";
const OPENCODE_JSONC_FILE: &str = "opencode.jsonc";

fn provider_spec(provider: &str) -> Result<ProviderSpec, String> {
    match provider {
        CLAUDE_PROVIDER => Ok(ProviderSpec {
            format: ConfigFormat::Json,
            section_key: "mcpServers",
            native_enabled_flag: false,
            opencode_layout: false,
        }),
        CODEX_PROVIDER => Ok(ProviderSpec {
            format: ConfigFormat::Toml,
            section_key: "mcp_servers",
            native_enabled_flag: true,
            opencode_layout: false,
        }),
        OPENCODE_PROVIDER => Ok(ProviderSpec {
            format: ConfigFormat::Json,
            section_key: "mcp",
            native_enabled_flag: true,
            opencode_layout: true,
        }),
        COPILOT_PROVIDER => Ok(ProviderSpec {
            format: ConfigFormat::Json,
            section_key: "mcpServers",
            native_enabled_flag: false,
            opencode_layout: false,
        }),
        _ => Err(format!("unknown MCP provider: {provider}")),
    }
}

/// opencode 會合併兩種檔名；僅有 `.jsonc` 時使用它，其餘情況維持 `opencode.json`。
fn opencode_config_file(dir: &Path) -> PathBuf {
    let json_path = dir.join(OPENCODE_JSON_FILE);
    let jsonc_path = dir.join(OPENCODE_JSONC_FILE);
    if !json_path.is_file() && jsonc_path.is_file() {
        jsonc_path
    } else {
        json_path
    }
}

/// opencode 不分副檔名都以 JSONC 解析設定檔，因此 `opencode.json` 內的註解與尾逗號也是合法內容。
fn is_jsonc_path(path: &Path) -> bool {
    path.extension()
        .is_some_and(|extension| extension == "jsonc")
        || path
            .file_name()
            .is_some_and(|file_name| file_name == OPENCODE_JSON_FILE)
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
                    let user_profile = crate::settings::user_profile_env()?;
                    Ok(PathBuf::from(user_profile).join(".claude.json"))
                }
                CODEX_PROVIDER => {
                    Ok(resolve_codex_root(Some(settings.codex_root.as_str()))?.join("config.toml"))
                }
                OPENCODE_PROVIDER => Ok(opencode_config_file(&default_opencode_config_root()?)),
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
                OPENCODE_PROVIDER => Ok(opencode_config_file(&project_root)),
                COPILOT_PROVIDER => {
                    // copilot 兩檔並存時只載入 `.mcp.json`，`.github/mcp.json` 整份不生效。
                    let root_path = project_root.join(".mcp.json");
                    if root_path.is_file() {
                        Ok(root_path)
                    } else {
                        Ok(project_root.join(".github").join("mcp.json"))
                    }
                }
                _ => Err(format!("unknown MCP provider: {provider}")),
            }
        }
    }
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
    let value: Value = if is_jsonc_path(path) {
        jsonc_parser::parse_to_serde_value(&content, &Default::default())
            .map_err(|error| format!("failed to parse {}: {error}", path.display()))?
    } else {
        serde_json::from_str(&content)
            .map_err(|error| format!("failed to parse {}: {error}", path.display()))?
    };
    match value {
        Value::Object(map) => Ok(map),
        _ => Err(format!("{} does not contain a JSON object", path.display())),
    }
}

/// 改寫會遺失註解，因此含註解或尾逗號（非嚴格 JSON）的 JSONC 設定檔一律拒絕覆寫。
fn ensure_jsonc_safe_to_write(path: &Path) -> Result<(), String> {
    if !is_jsonc_path(path) || !path.is_file() {
        return Ok(());
    }
    let content = fs::read_to_string(path)
        .map_err(|error| format!("failed to read {}: {error}", path.display()))?;
    if content.trim().is_empty() || serde_json::from_str::<Value>(&content).is_ok() {
        return Ok(());
    }
    Err(format!(
        "{} contains comments or trailing commas; edit it manually to avoid losing them",
        path.display()
    ))
}

fn write_json_object(path: &Path, root: &Map<String, Value>) -> Result<(), String> {
    ensure_jsonc_safe_to_write(path)?;
    let content = serde_json::to_vec_pretty(root)
        .map_err(|error| format!("failed to serialize {}: {error}", path.display()))?;
    atomic_write_file(path, &content)
}

fn json_section_entries(root: &Map<String, Value>, spec: &ProviderSpec) -> Vec<(String, Value)> {
    let Some(Value::Object(section)) = root.get(spec.section_key) else {
        return Vec::new();
    };
    let mut entries: Vec<(String, Value)> = section
        .iter()
        .filter(|(name, _)| {
            !spec.opencode_layout || !OPENCODE_RESERVED_KEYS.contains(&name.as_str())
        })
        .map(|(name, value)| (name.clone(), value.clone()))
        .collect();
    if spec.opencode_layout {
        if let Some(Value::Object(nested)) = section.get(OPENCODE_SERVERS_KEY) {
            // 同名時以 v2 配置為準，避免清單出現重複名稱。
            entries.retain(|(name, _)| !nested.contains_key(name));
            entries.extend(
                nested
                    .iter()
                    .map(|(name, value)| (name.clone(), value.clone())),
            );
        }
    }
    entries
}

/// server 位於 opencode v2 巢狀配置時回傳 true；新項目沿用檔案既有配置，都沒有時使用 v1。
/// 實測 opencode 1.19.5 與 2.2.5 皆可讀取 v1 配置，因此不需偵測安裝版本。
fn opencode_uses_nested(section: &Map<String, Value>, name: &str) -> bool {
    match section.get(OPENCODE_SERVERS_KEY) {
        Some(Value::Object(nested)) => nested.contains_key(name) || !section.contains_key(name),
        _ => false,
    }
}

/// 取得實際存放指定 server 的 JSON 物件；區段不存在時建立，由呼叫端決定是否寫回。
fn json_servers_map_mut<'a>(
    root: &'a mut Map<String, Value>,
    spec: &ProviderSpec,
    name: &str,
) -> Result<&'a mut Map<String, Value>, String> {
    let section = root
        .entry(spec.section_key.to_string())
        .or_insert_with(|| Value::Object(Map::new()));
    let Value::Object(section_map) = section else {
        return Err(format!("{} is not a JSON object", spec.section_key));
    };
    if spec.opencode_layout && opencode_uses_nested(section_map, name) {
        let Some(Value::Object(nested)) = section_map.get_mut(OPENCODE_SERVERS_KEY) else {
            return Err(format!(
                "{}.{OPENCODE_SERVERS_KEY} is not a JSON object",
                spec.section_key
            ));
        };
        return Ok(nested);
    }
    Ok(section_map)
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

#[cfg(test)]
pub(crate) fn list_mcp_configs_internal(
    scope: &McpScope,
) -> Result<Vec<McpProviderConfig>, String> {
    list_mcp_configs_with_providers_internal(
        scope,
        MCP_PROVIDERS
            .iter()
            .map(|provider| provider.to_string())
            .collect::<Vec<_>>()
            .as_slice(),
        false,
    )
}

pub(crate) fn list_mcp_configs_with_providers(
    scope: &McpScope,
    enabled_providers: &[String],
) -> Result<Vec<McpProviderConfig>, String> {
    list_mcp_configs_with_providers_internal(scope, enabled_providers, true)
}

fn list_mcp_configs_with_providers_internal(
    scope: &McpScope,
    enabled_providers: &[String],
    include_cli: bool,
) -> Result<Vec<McpProviderConfig>, String> {
    let disabled_store = load_disabled_store()?;
    let mut results = Vec::with_capacity(MCP_PROVIDERS.len());
    let (enabled, _) = normalize_enabled_providers(enabled_providers);
    for provider in enabled
        .iter()
        .filter(|provider| MCP_PROVIDERS.contains(&provider.as_str()))
    {
        results.push(list_one_provider(
            provider,
            scope,
            &disabled_store,
            include_cli,
        ));
    }
    Ok(results)
}

fn list_one_provider(
    provider: &str,
    scope: &McpScope,
    disabled_store: &DisabledStore,
    include_cli: bool,
) -> McpProviderConfig {
    match list_one_provider_inner(provider, scope, disabled_store, include_cli) {
        Ok(config) => config,
        Err(error) => {
            // 設定檔解析失敗時仍回傳路徑，讓使用者能開啟檔案修正。
            let path = mcp_config_path(provider, scope).ok();
            McpProviderConfig {
                provider_id: provider.to_string(),
                config_path: path
                    .as_deref()
                    .map(normalize_display_path)
                    .unwrap_or_default(),
                config_exists: path.as_deref().is_some_and(Path::is_file),
                servers: Vec::new(),
                error: Some(error),
                enabled: true,
                diagnostics: Vec::new(),
            }
        }
    }
}

/// 確保設定檔存在：僅在檔案不存在時建立空設定，絕不覆寫既有內容。
pub(crate) fn ensure_mcp_config_file_internal(
    scope: &McpScope,
    provider: &str,
) -> Result<EnsuredMcpConfigFile, String> {
    let spec = provider_spec(provider)?;
    let path = mcp_config_path(provider, scope)?;
    let created = !path.exists();
    if created {
        let initial_content: &[u8] = match spec.format {
            ConfigFormat::Json => b"{}\n",
            ConfigFormat::Toml => b"",
        };
        atomic_write_file(&path, initial_content)?;
    }
    Ok(EnsuredMcpConfigFile {
        path: normalize_display_path(&path),
        created,
    })
}

/// 回傳檔案總管要定位的路徑：設定檔不存在時退回最近的既有上層目錄，不產生任何檔案。
/// 顯示位置不應改動磁碟，例如建立專案層 `.mcp.json` 會讓 copilot 不再載入 `.github/mcp.json`。
pub(crate) fn resolve_mcp_reveal_path_internal(
    scope: &McpScope,
    provider: &str,
) -> Result<String, String> {
    let path = mcp_config_path(provider, scope)?;
    path.ancestors()
        .find(|candidate| candidate.exists())
        .map(normalize_display_path)
        .ok_or_else(|| format!("no existing directory found for {}", path.display()))
}

fn list_one_provider_inner(
    provider: &str,
    scope: &McpScope,
    disabled_store: &DisabledStore,
    include_cli: bool,
) -> Result<McpProviderConfig, String> {
    let spec = provider_spec(provider)?;
    let path = mcp_config_path(provider, scope)?;
    let config_exists = path.is_file();

    let raw_entries: Vec<(String, Value)> = match spec.format {
        ConfigFormat::Json => {
            let root = read_json_object(&path)?;
            json_section_entries(&root, &spec)
        }
        ConfigFormat::Toml => {
            let document = read_toml_document(&path)?;
            toml_section_entries(&document, spec.section_key)
        }
    };

    let mut servers = Vec::new();
    for (name, mut value) in raw_entries {
        let enabled = if spec.native_enabled_flag {
            extract_native_enabled(&mut value, spec.opencode_layout)
        } else {
            true
        };
        servers.push(McpServerEntry {
            name,
            enabled,
            config_json: serde_json::to_string_pretty(&value)
                .map_err(|error| format!("failed to serialize server config: {error}"))?,
            effective: None,
            source: None,
            scope: None,
            editable: true,
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
                    effective: None,
                    source: None,
                    scope: None,
                    editable: true,
                });
            }
        }
    }

    servers.sort_by(|a, b| a.name.cmp(&b.name));

    let cli_scope = match scope {
        McpScope::Global => DiscoveryScope::Global,
        McpScope::Project { .. } => DiscoveryScope::Project,
    };
    let project_cwd = match scope {
        McpScope::Project { project_cwd } => Some(project_cwd.as_str()),
        McpScope::Global => None,
    };
    let codex_trusted = if provider == CODEX_PROVIDER {
        project_cwd
            .map(is_codex_project_trusted)
            .transpose()?
            .unwrap_or(true)
    } else {
        true
    };
    let (mut diagnostics, cli_entries) = match (include_cli, provider, codex_trusted) {
        (true, COPILOT_PROVIDER, _) | (true, CODEX_PROVIDER, true) => {
            match discover_cli(provider, ResourceKind::Mcp, cli_scope, project_cwd) {
                Ok(entries) => (Vec::new(), entries),
                Err(error) => (vec![error], Vec::new()),
            }
        }
        _ => (Vec::new(), Vec::new()),
    };
    for server in &mut servers {
        if let Some(cli) = cli_entries
            .iter()
            .find(|entry| entry.name.eq_ignore_ascii_case(&server.name))
        {
            server.effective = Some(true);
            server.source = cli.source.clone();
            server.scope = Some(cli_scope);
        }
    }
    let configured_names = servers
        .iter()
        .map(|server| server.name.to_lowercase())
        .collect::<std::collections::HashSet<_>>();
    for cli in cli_entries
        .iter()
        .filter(|entry| !configured_names.contains(&entry.name.to_lowercase()))
    {
        servers.push(McpServerEntry {
            name: cli.name.clone(),
            enabled: cli.enabled.unwrap_or(true),
            config_json: "{}".to_string(),
            effective: Some(true),
            source: cli.source.clone(),
            scope: Some(cli_scope),
            editable: false,
        });
    }
    Ok(McpProviderConfig {
        provider_id: provider.to_string(),
        config_path: normalize_display_path(&path),
        config_exists,
        servers,
        error: None,
        enabled: true,
        diagnostics: {
            diagnostics.shrink_to_fit();
            diagnostics
        },
    })
}

/// 讀取並移除原生 `enabled`（codex/opencode v1）與 `disabled`（opencode v2）旗標；
/// 旗標不存在時視為啟用（D4：預設即啟用）。
fn extract_native_enabled(value: &mut Value, opencode_layout: bool) -> bool {
    let Value::Object(map) = value else {
        return true;
    };
    let enabled = !matches!(map.remove("enabled"), Some(Value::Bool(false)));
    let disabled = opencode_layout && matches!(map.remove("disabled"), Some(Value::Bool(true)));
    enabled && !disabled
}

/// 清單會移除原生啟用旗標，編輯內容因此不含它們；儲存時沿用原項目的旗標，避免編輯改變啟用狀態。
/// 編輯內容自行指定的旗標優先。
fn carry_native_enabled_flags(previous: &Value, next: &mut Value, opencode_layout: bool) {
    let (Value::Object(previous), Value::Object(next)) = (previous, next) else {
        return;
    };
    let flag_keys: &[&str] = if opencode_layout {
        &["enabled", "disabled"]
    } else {
        &["enabled"]
    };
    for key in flag_keys {
        if let (Some(flag), false) = (previous.get(*key), next.contains_key(*key)) {
            next.insert((*key).to_string(), flag.clone());
        }
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
    if spec.opencode_layout && OPENCODE_RESERVED_KEYS.contains(&name) {
        return Err(format!(
            "{name} is a reserved name in the OpenCode MCP config"
        ));
    }
    let mut parsed: Value = serde_json::from_str(config_json)
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

    let write_path = mcp_config_path(provider, scope)?;
    match spec.format {
        ConfigFormat::Json => {
            let mut root = read_json_object(&write_path)?;
            // 改名時留在舊名稱所在的配置，v1/v2 項目不會因編輯而搬移。
            let servers = json_servers_map_mut(&mut root, &spec, rename_from.unwrap_or(name))?;
            if spec.native_enabled_flag {
                if let Some(previous) = servers.get(rename_from.unwrap_or(name)) {
                    carry_native_enabled_flags(previous, &mut parsed, spec.opencode_layout);
                }
            }
            if let Some(old_name) = rename_from {
                servers.remove(old_name);
            }
            servers.insert(name.to_string(), parsed);
            write_json_object(&write_path, &root)?;
        }
        ConfigFormat::Toml => {
            let mut document = read_toml_document(&write_path)?;
            ensure_toml_table(&mut document, spec.section_key);
            let section = document[spec.section_key]
                .as_table_mut()
                .ok_or_else(|| format!("{} is not a TOML table", spec.section_key))?;
            if let Some(previous) = section
                .get(rename_from.unwrap_or(name))
                .and_then(toml_item_to_json)
            {
                carry_native_enabled_flags(&previous, &mut parsed, spec.opencode_layout);
            }
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

    let write_path = mcp_config_path(provider, scope)?;
    match spec.format {
        ConfigFormat::Json => {
            let mut root = read_json_object(&write_path)?;
            let removed = json_servers_map_mut(&mut root, &spec, name)?
                .remove(name)
                .is_some();
            if removed {
                write_json_object(&write_path, &root)?;
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
    let write_path = mcp_config_path(provider, scope)?;
    match spec.format {
        ConfigFormat::Json => {
            let mut root = read_json_object(&write_path)?;
            // opencode v2 會忽略 mcp.servers 內的 enabled（實測 2.2.5 仍會連線），必須寫 disabled。
            let uses_disabled_flag = spec.opencode_layout
                && matches!(
                    root.get(spec.section_key),
                    Some(Value::Object(section)) if opencode_uses_nested(section, name)
                );
            let servers = json_servers_map_mut(&mut root, spec, name)?;
            let Some(Value::Object(entry)) = servers.get_mut(name) else {
                return Err(format!("server {name} not found"));
            };
            entry.remove("enabled");
            entry.remove("disabled");
            if !enabled {
                if uses_disabled_flag {
                    entry.insert("disabled".to_string(), Value::Bool(true));
                } else {
                    entry.insert("enabled".to_string(), Value::Bool(false));
                }
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
    let write_path = mcp_config_path(provider, scope)?;
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

// ---------- HTTP / SSE 連線測試 ----------

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub(crate) enum McpConnectionTestResult {
    /// 成功建立連線並取得 MCP initialize 回應
    Ok,
    /// 認證失敗（401 / 403）
    Unauthorized,
    /// 連線建立但回應非預期（非 JSON-RPC 或狀態碼異常）
    UnexpectedResponse { status: u16 },
    /// 無法建立連線（DNS、TCP、TLS 等傳輸層錯誤）
    ConnectionFailed { message: String },
}

const MCP_TEST_TIMEOUT_SECS: u64 = 5;

/// 對 HTTP / SSE 類型的 MCP server 送出 JSON-RPC `initialize` 請求，驗證連線是否可用。
pub(crate) fn test_mcp_http_connection(
    url: &str,
    headers: &BTreeMap<String, String>,
) -> McpConnectionTestResult {
    let agent = ureq::Agent::config_builder()
        .http_status_as_error(false)
        .timeout_global(Some(std::time::Duration::from_secs(MCP_TEST_TIMEOUT_SECS)))
        .build()
        .new_agent();

    let body = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "protocolVersion": "2025-06-18",
            "capabilities": {},
            "clientInfo": { "name": "session-hub", "version": env!("CARGO_PKG_VERSION") }
        }
    });

    let mut request = agent
        .post(url)
        .header("Content-Type", "application/json")
        .header("Accept", "application/json, text/event-stream");
    for (key, value) in headers {
        request = request.header(key.as_str(), value.as_str());
    }

    match request.send_json(&body) {
        Ok(mut response) => {
            let status = response.status().as_u16();
            match status {
                401 | 403 => McpConnectionTestResult::Unauthorized,
                200..=299 => {
                    // 僅有 2xx 不足以判定成功：誤填為一般網頁端點時同樣回 2xx。
                    // 依 spec 需確認回應確實為 MCP 的 JSON-RPC 結果。
                    let payload = response.body_mut().read_to_string().unwrap_or_default();
                    if is_json_rpc_response(&payload) {
                        McpConnectionTestResult::Ok
                    } else {
                        McpConnectionTestResult::UnexpectedResponse { status }
                    }
                }
                other => McpConnectionTestResult::UnexpectedResponse { status: other },
            }
        }
        Err(error) => McpConnectionTestResult::ConnectionFailed {
            message: error.to_string(),
        },
    }
}

/// 判斷回應內容是否為合法的 JSON-RPC 回應。
///
/// MCP 的 Streamable HTTP 傳輸允許兩種回應格式，故兩者皆需支援：
/// - `application/json`：整個 body 即為 JSON-RPC 物件
/// - `text/event-stream`：SSE 事件串流，JSON-RPC 物件位於 `data:` 欄位
fn is_json_rpc_response(payload: &str) -> bool {
    if payload.trim().is_empty() {
        return false;
    }

    if is_json_rpc_object(payload) {
        return true;
    }

    // SSE：逐行取出 `data:` 欄位內容判斷。單一事件的 data 可跨多行，
    // 但 MCP 的 JSON-RPC 回應實務上為單行，逐行檢查已足夠。
    payload
        .lines()
        .filter_map(|line| line.strip_prefix("data:"))
        .any(is_json_rpc_object)
}

/// 判斷單段文字是否為帶 `jsonrpc` 欄位、且含 `result` 或 `error` 的 JSON-RPC 物件。
fn is_json_rpc_object(text: &str) -> bool {
    let Ok(value) = serde_json::from_str::<serde_json::Value>(text.trim()) else {
        return false;
    };
    value.get("jsonrpc").and_then(|v| v.as_str()) == Some("2.0")
        && (value.get("result").is_some() || value.get("error").is_some())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::OsString;

    use crate::types::ANTIGRAVITY_PROVIDER;

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

    fn opencode_project(env: &EnvGuard, content: &str) -> (McpScope, PathBuf) {
        let project_dir = env.project_dir();
        let path = project_dir.join(OPENCODE_JSON_FILE);
        fs::write(&path, content).unwrap();
        let scope = McpScope::Project {
            project_cwd: project_dir.to_string_lossy().to_string(),
        };
        (scope, path)
    }

    fn list_opencode(scope: &McpScope) -> McpProviderConfig {
        list_mcp_configs_internal(scope)
            .unwrap()
            .into_iter()
            .find(|entry| entry.provider_id == OPENCODE_PROVIDER)
            .unwrap()
    }

    fn read_value(path: &Path) -> Value {
        serde_json::from_str(&fs::read_to_string(path).unwrap()).unwrap()
    }

    #[test]
    fn opencode_lists_v1_and_v2_servers_without_reserved_keys() {
        let env = EnvGuard::new("opencode-layouts");
        let (scope, _) = opencode_project(
            &env,
            r#"{"mcp": {
                "timeout": {"startup": 1000},
                "old": {"type": "local", "command": ["a"], "enabled": false},
                "servers": {"new": {"type": "remote", "url": "http://x", "disabled": true}}
            }}"#,
        );

        let listed = list_opencode(&scope);
        let names: Vec<&str> = listed.servers.iter().map(|s| s.name.as_str()).collect();
        assert_eq!(names, ["new", "old"]);
        assert!(listed.servers.iter().all(|server| !server.enabled));
        let new_server = &listed.servers[0];
        assert!(!new_server.config_json.contains("disabled"));
    }

    #[test]
    fn opencode_writes_follow_existing_layout() {
        let env = EnvGuard::new("opencode-v2-write");
        let (scope, path) = opencode_project(
            &env,
            r#"{"mcp": {"old": {"type": "local", "command": ["a"]},
                "servers": {"existing": {"type": "local", "command": ["b"]}}}}"#,
        );
        let config = r#"{"type": "remote", "url": "http://x"}"#;

        upsert_mcp_server_internal(&scope, OPENCODE_PROVIDER, "added", None, config).unwrap();
        upsert_mcp_server_internal(&scope, OPENCODE_PROVIDER, "renamed", Some("old"), config)
            .unwrap();
        let value = read_value(&path);
        assert!(value["mcp"]["servers"]["added"].is_object());
        assert!(value["mcp"]["renamed"].is_object());
        assert!(value["mcp"].get("old").is_none());

        set_mcp_server_enabled_internal(&scope, OPENCODE_PROVIDER, "added", false).unwrap();
        set_mcp_server_enabled_internal(&scope, OPENCODE_PROVIDER, "renamed", false).unwrap();
        let value = read_value(&path);
        assert_eq!(
            value["mcp"]["servers"]["added"]["disabled"],
            Value::Bool(true)
        );
        assert!(value["mcp"]["servers"]["added"].get("enabled").is_none());
        assert_eq!(value["mcp"]["renamed"]["enabled"], Value::Bool(false));

        set_mcp_server_enabled_internal(&scope, OPENCODE_PROVIDER, "added", true).unwrap();
        let value = read_value(&path);
        assert!(value["mcp"]["servers"]["added"].get("disabled").is_none());

        delete_mcp_server_internal(&scope, OPENCODE_PROVIDER, "added").unwrap();
        delete_mcp_server_internal(&scope, OPENCODE_PROVIDER, "renamed").unwrap();
        let value = read_value(&path);
        assert!(value["mcp"]["servers"].get("added").is_none());
        assert!(value["mcp"].get("renamed").is_none());
    }

    #[test]
    fn opencode_new_file_uses_v1_layout_and_rejects_reserved_names() {
        let env = EnvGuard::new("opencode-new-file");
        let project_dir = env.project_dir();
        let scope = McpScope::Project {
            project_cwd: project_dir.to_string_lossy().to_string(),
        };
        let config = r#"{"type": "remote", "url": "http://x"}"#;

        upsert_mcp_server_internal(&scope, OPENCODE_PROVIDER, "srv", None, config).unwrap();
        let value = read_value(&project_dir.join(OPENCODE_JSON_FILE));
        assert!(value["mcp"]["srv"].is_object());

        for reserved in OPENCODE_RESERVED_KEYS {
            assert!(
                upsert_mcp_server_internal(&scope, OPENCODE_PROVIDER, reserved, None, config)
                    .is_err()
            );
        }
    }

    #[test]
    fn opencode_jsonc_is_readable_and_comments_block_rewrite() {
        let env = EnvGuard::new("opencode-jsonc");
        let project_dir = env.project_dir();
        let scope = McpScope::Project {
            project_cwd: project_dir.to_string_lossy().to_string(),
        };
        let path = project_dir.join(OPENCODE_JSONC_FILE);
        let commented = "// note\n{\"mcp\": {\"servers\": {\"srv\": {\"type\": \"remote\", \"url\": \"http://x\",},},},}";
        fs::write(&path, commented).unwrap();
        let config = r#"{"type": "remote", "url": "http://y"}"#;

        let listed = list_opencode(&scope);
        assert_eq!(listed.servers[0].name, "srv");
        assert!(listed.config_path.ends_with(OPENCODE_JSONC_FILE));
        assert!(
            upsert_mcp_server_internal(&scope, OPENCODE_PROVIDER, "new", None, config).is_err()
        );
        assert_eq!(fs::read_to_string(&path).unwrap(), commented);

        fs::write(&path, r#"{"$schema": "https://opencode.ai/config.json"}"#).unwrap();
        upsert_mcp_server_internal(&scope, OPENCODE_PROVIDER, "new", None, config).unwrap();
        assert!(read_value(&path)["mcp"]["new"].is_object());
        assert!(!project_dir.join(OPENCODE_JSON_FILE).exists());
    }

    #[test]
    fn opencode_json_with_comments_is_readable_and_blocks_rewrite() {
        let env = EnvGuard::new("opencode-json-comments");
        let commented = "{\n  // note\n  \"mcp\": {\"srv\": {\"type\": \"remote\", \"url\": \"http://x\",},},\n}";
        let (scope, path) = opencode_project(&env, commented);

        let listed = list_opencode(&scope);
        assert_eq!(listed.error, None);
        assert_eq!(listed.servers[0].name, "srv");

        let config = r#"{"type": "remote", "url": "http://y"}"#;
        assert!(
            upsert_mcp_server_internal(&scope, OPENCODE_PROVIDER, "new", None, config).is_err()
        );
        assert_eq!(fs::read_to_string(&path).unwrap(), commented);
    }

    #[test]
    fn editing_disabled_opencode_server_keeps_it_disabled() {
        let env = EnvGuard::new("opencode-edit-disabled");
        let (scope, path) = opencode_project(
            &env,
            r#"{"mcp": {
                "v1": {"type": "remote", "url": "http://a", "enabled": false},
                "servers": {"v2": {"type": "remote", "url": "http://b", "disabled": true}}
            }}"#,
        );
        let edited = r#"{"type": "remote", "url": "http://edited"}"#;

        upsert_mcp_server_internal(&scope, OPENCODE_PROVIDER, "v1", Some("v1"), edited).unwrap();
        upsert_mcp_server_internal(&scope, OPENCODE_PROVIDER, "v2-new", Some("v2"), edited)
            .unwrap();

        let value = read_value(&path);
        assert_eq!(value["mcp"]["v1"]["url"], "http://edited");
        assert_eq!(value["mcp"]["v1"]["enabled"], false);
        assert_eq!(value["mcp"]["servers"]["v2-new"]["url"], "http://edited");
        assert_eq!(value["mcp"]["servers"]["v2-new"]["disabled"], true);
        assert!(list_opencode(&scope).servers.iter().all(|s| !s.enabled));

        set_mcp_server_enabled_internal(&scope, OPENCODE_PROVIDER, "v1", true).unwrap();
        upsert_mcp_server_internal(&scope, OPENCODE_PROVIDER, "v1", Some("v1"), edited).unwrap();
        assert!(read_value(&path)["mcp"]["v1"].get("enabled").is_none());

        // 自訂 JSON 明確指定的旗標優先於原項目的旗標。
        upsert_mcp_server_internal(
            &scope,
            OPENCODE_PROVIDER,
            "v2-new",
            Some("v2-new"),
            r#"{"type": "remote", "url": "http://edited", "disabled": false}"#,
        )
        .unwrap();
        assert_eq!(
            read_value(&path)["mcp"]["servers"]["v2-new"]["disabled"],
            false
        );
    }

    #[test]
    fn editing_disabled_codex_server_keeps_it_disabled() {
        let env = EnvGuard::new("codex-edit-disabled");
        let project_dir = env.project_dir();
        let scope = McpScope::Project {
            project_cwd: project_dir.to_string_lossy().to_string(),
        };
        let path = project_dir.join(".codex").join("config.toml");
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(
            &path,
            "[mcp_servers.srv]\nurl = \"http://a\"\nenabled = false\n",
        )
        .unwrap();

        upsert_mcp_server_internal(
            &scope,
            CODEX_PROVIDER,
            "renamed",
            Some("srv"),
            r#"{"url": "http://edited"}"#,
        )
        .unwrap();

        let document = read_toml_document(&path).unwrap();
        let entry = &document["mcp_servers"]["renamed"];
        assert_eq!(entry["url"].as_str(), Some("http://edited"));
        assert_eq!(entry["enabled"].as_bool(), Some(false));
    }

    #[test]
    fn unparsable_config_still_reports_path() {
        let env = EnvGuard::new("mcp-broken-config");
        let (scope, _) = opencode_project(&env, "{ not valid");

        let listed = list_opencode(&scope);
        assert!(listed.error.is_some());
        assert!(listed.config_path.ends_with(OPENCODE_JSON_FILE));
        assert!(listed.config_exists);
    }

    #[test]
    fn ensure_config_file_creates_only_when_missing() {
        let env = EnvGuard::new("mcp-ensure-config");
        let project_dir = env.project_dir();
        let scope = McpScope::Project {
            project_cwd: project_dir.to_string_lossy().to_string(),
        };

        let created = ensure_mcp_config_file_internal(&scope, COPILOT_PROVIDER).unwrap();
        assert!(created.created);
        let copilot_path = project_dir.join(".github").join("mcp.json");
        assert_eq!(read_value(&copilot_path), serde_json::json!({}));

        let codex = ensure_mcp_config_file_internal(&scope, CODEX_PROVIDER).unwrap();
        assert!(codex.created);
        let codex_path = project_dir.join(".codex").join("config.toml");
        assert_eq!(fs::read_to_string(&codex_path).unwrap(), "");

        let existing = r#"{"mcpServers": {"srv": {"command": "run"}}}"#;
        fs::write(&copilot_path, existing).unwrap();
        let again = ensure_mcp_config_file_internal(&scope, COPILOT_PROVIDER).unwrap();
        assert!(!again.created);
        assert_eq!(fs::read_to_string(&copilot_path).unwrap(), existing);
    }

    #[test]
    fn reveal_path_falls_back_to_existing_ancestor_without_creating() {
        let env = EnvGuard::new("mcp-reveal-path");
        let project_dir = env.project_dir();
        let scope = McpScope::Project {
            project_cwd: project_dir.to_string_lossy().to_string(),
        };

        let missing = resolve_mcp_reveal_path_internal(&scope, CODEX_PROVIDER).unwrap();
        assert_eq!(missing, normalize_display_path(&project_dir));
        assert!(!project_dir.join(".codex").exists());

        let config_path = ensure_mcp_config_file_internal(&scope, CODEX_PROVIDER)
            .unwrap()
            .path;
        assert_eq!(
            resolve_mcp_reveal_path_internal(&scope, CODEX_PROVIDER).unwrap(),
            config_path
        );
    }

    /// 複製到其他 provider 只能寫入目標設定檔，來源設定檔與來源清單都必須維持不變。
    #[test]
    fn copy_to_other_provider_keeps_source_server() {
        let env = EnvGuard::new("copy-keeps-source");
        let project_dir = env.project_dir();
        let scope = McpScope::Project {
            project_cwd: project_dir.to_string_lossy().to_string(),
        };
        let config = r#"{"type": "http", "url": "http://x"}"#;
        upsert_mcp_server_internal(&scope, CLAUDE_PROVIDER, "產題系統mcp", None, config).unwrap();
        let source_path = project_dir.join(".mcp.json");
        let source_before = fs::read_to_string(&source_path).unwrap();

        upsert_mcp_server_internal(
            &scope,
            OPENCODE_PROVIDER,
            "產題系統mcp",
            None,
            r#"{"type": "remote", "url": "http://x"}"#,
        )
        .unwrap();

        assert_eq!(fs::read_to_string(&source_path).unwrap(), source_before);
        let providers = list_mcp_configs_internal(&scope).unwrap();
        for provider in [CLAUDE_PROVIDER, OPENCODE_PROVIDER] {
            let listed = providers
                .iter()
                .find(|entry| entry.provider_id == provider)
                .unwrap();
            assert!(
                listed
                    .servers
                    .iter()
                    .any(|server| server.name == "產題系統mcp"),
                "{provider} should still list the server"
            );
        }
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

    fn copilot_project_server_names(scope: &McpScope) -> Vec<String> {
        list_mcp_configs_internal(scope)
            .unwrap()
            .into_iter()
            .find(|config| config.provider_id == COPILOT_PROVIDER)
            .unwrap()
            .servers
            .into_iter()
            .map(|server| server.name)
            .collect()
    }

    #[test]
    fn copilot_project_uses_root_file_when_both_exist() {
        let env = EnvGuard::new("copilot-both-files");
        let project_dir = env.project_dir();
        let scope = McpScope::Project {
            project_cwd: project_dir.to_string_lossy().to_string(),
        };
        let root_path = project_dir.join(".mcp.json");
        let github_path = project_dir.join(".github").join("mcp.json");
        let github_content = r#"{"mcpServers": {"from-github": {"command": "g"}}}"#;
        fs::create_dir_all(github_path.parent().unwrap()).unwrap();
        fs::write(&github_path, github_content).unwrap();
        fs::write(
            &root_path,
            r#"{"mcpServers": {"from-root": {"command": "r"}}}"#,
        )
        .unwrap();

        // Copilot CLI 1.0.105 實測：兩檔並存時只載入 .mcp.json，不依名稱合併。
        assert_eq!(copilot_project_server_names(&scope), vec!["from-root"]);

        upsert_mcp_server_internal(
            &scope,
            COPILOT_PROVIDER,
            "added",
            None,
            r#"{"command": "a"}"#,
        )
        .unwrap();

        assert!(read_value(&root_path)["mcpServers"]["added"].is_object());
        assert_eq!(fs::read_to_string(&github_path).unwrap(), github_content);
    }

    #[test]
    fn copilot_project_keeps_listing_root_file_after_writes() {
        let env = EnvGuard::new("copilot-root-only");
        let project_dir = env.project_dir();
        let scope = McpScope::Project {
            project_cwd: project_dir.to_string_lossy().to_string(),
        };
        let root_path = project_dir.join(".mcp.json");
        fs::write(
            &root_path,
            r#"{"mcpServers": {"existing": {"command": "e"}}}"#,
        )
        .unwrap();

        upsert_mcp_server_internal(
            &scope,
            COPILOT_PROVIDER,
            "added",
            None,
            r#"{"command": "a"}"#,
        )
        .unwrap();

        assert!(!project_dir.join(".github").join("mcp.json").exists());
        assert_eq!(
            copilot_project_server_names(&scope),
            vec!["added", "existing"]
        );

        delete_mcp_server_internal(&scope, COPILOT_PROVIDER, "existing").unwrap();
        assert!(read_value(&root_path)["mcpServers"]
            .get("existing")
            .is_none());

        set_mcp_server_enabled_internal(&scope, COPILOT_PROVIDER, "added", false).unwrap();
        assert!(read_value(&root_path)["mcpServers"].get("added").is_none());
        set_mcp_server_enabled_internal(&scope, COPILOT_PROVIDER, "added", true).unwrap();
        assert!(read_value(&root_path)["mcpServers"]["added"].is_object());
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

    #[test]
    fn list_respects_enabled_provider_intersection() {
        let env = EnvGuard::new("enabled-provider-filter");
        fs::write(
            env.home().join(".claude.json"),
            r#"{"mcpServers":{"claude-server":{"command":"claude"}}}"#,
        )
        .unwrap();
        let result = list_mcp_configs_with_providers(
            &McpScope::Global,
            &[
                OPENCODE_PROVIDER.to_string(),
                ANTIGRAVITY_PROVIDER.to_string(),
            ],
        )
        .expect("list filtered providers");
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].provider_id, OPENCODE_PROVIDER);
        assert!(result[0].servers.is_empty());
    }

    #[test]
    fn unsupported_mcp_provider_does_not_create_empty_config() {
        let _env = EnvGuard::new("antigravity-mcp-filter");
        let result =
            list_mcp_configs_with_providers(&McpScope::Global, &[ANTIGRAVITY_PROVIDER.to_string()])
                .expect("list unsupported provider");
        assert!(result.is_empty());
    }

    #[test]
    fn json_rpc_response_accepts_result_and_error_payloads() {
        assert!(is_json_rpc_response(
            r#"{"jsonrpc":"2.0","id":1,"result":{"protocolVersion":"2025-06-18"}}"#
        ));
        assert!(is_json_rpc_response(
            r#"{"jsonrpc":"2.0","id":1,"error":{"code":-32601}}"#
        ));
    }

    #[test]
    fn json_rpc_response_accepts_sse_event_stream() {
        let payload = "event: message\ndata: {\"jsonrpc\":\"2.0\",\"id\":1,\"result\":{}}\n\n";
        assert!(is_json_rpc_response(payload));
    }

    #[test]
    fn json_rpc_response_rejects_non_mcp_payloads() {
        // 誤填為一般網頁端點：回 2xx 但內容為 HTML
        assert!(!is_json_rpc_response(
            "<!doctype html><html><body>hi</body></html>"
        ));
        // 合法 JSON 但非 JSON-RPC
        assert!(!is_json_rpc_response(r#"{"status":"ok"}"#));
        // 有 jsonrpc 欄位但缺少 result / error
        assert!(!is_json_rpc_response(r#"{"jsonrpc":"2.0","id":1}"#));
        // 空回應
        assert!(!is_json_rpc_response("   "));
    }
}
