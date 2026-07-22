use serde::Deserialize;

// ── Claude JSONL 解析型別 ─────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
pub(crate) struct ClaudeEntry {
    #[serde(rename = "type")]
    pub(crate) entry_type: String,
    pub(crate) uuid: Option<String>,
    #[serde(default)]
    pub(crate) is_sidechain: bool,
    pub(crate) message: Option<ClaudeMessage>,
    pub(crate) request_id: Option<String>,
    pub(crate) timestamp: Option<String>,
    #[serde(default)]
    pub(crate) cwd: Option<String>,
    pub(crate) session_id: Option<String>,
    #[serde(default)]
    pub(crate) is_api_error_message: Option<bool>,
    #[serde(default)]
    pub(crate) git_branch: Option<String>,
    /// user entry 中由系統自動產生的 meta 訊息（local-command-caveat 等），不算互動次數
    #[serde(default)]
    pub(crate) is_meta: Option<bool>,
    /// ai-title entry 中的 AI 生成標題
    #[serde(rename = "aiTitle")]
    pub(crate) ai_title: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ClaudeMessage {
    pub(crate) id: Option<String>,
    #[allow(dead_code)]
    pub(crate) role: Option<String>,
    pub(crate) model: Option<String>,
    pub(crate) usage: Option<ClaudeUsage>,
}

#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ClaudeUsage {
    #[serde(default)]
    pub(crate) input_tokens: u64,
    #[serde(default)]
    pub(crate) output_tokens: u64,
    #[serde(default)]
    pub(crate) cache_creation_input_tokens: u64,
    #[serde(default)]
    pub(crate) cache_read_input_tokens: u64,
    pub(crate) speed: Option<String>,
    pub(crate) service_tier: Option<String>,
    pub(crate) cache_creation: Option<ClaudeCacheCreation>,
}

#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ClaudeCacheCreation {
    #[serde(default)]
    pub(crate) ephemeral_1h_input_tokens: u64,
    #[serde(default)]
    pub(crate) ephemeral_5m_input_tokens: u64,
}
