use std::collections::HashMap;
use std::sync::Mutex;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct QuotaWindow {
    pub(crate) window_key: String,
    pub(crate) label: String,
    pub(crate) utilization: f64,
    pub(crate) resets_at: Option<String>,
    /// 模型群組名稱（如 "Gemini Models" / "Claude and GPT models"），僅 Antigravity 使用；
    /// 其餘 provider 維持 None，供前端依顯示情境過濾群組。
    #[serde(default)]
    pub(crate) group: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct LocalTokenUsage {
    pub(crate) input_tokens: u64,
    pub(crate) output_tokens: u64,
    pub(crate) period_label: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ExtraCredits {
    pub(crate) is_enabled: bool,
    pub(crate) monthly_limit: Option<u64>,
    pub(crate) used_credits: f64,
    pub(crate) utilization: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ResetCreditEntry {
    pub(crate) granted_at: Option<String>,
    pub(crate) expires_at: Option<String>,
    pub(crate) status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ResetCredits {
    pub(crate) available_count: u32,
    pub(crate) credits: Vec<ResetCreditEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct QuotaSnapshot {
    pub(crate) provider: String,
    /// "ok" | "error" | "unsupported" | "no_auth"
    pub(crate) status: String,
    /// "remote_api" | "local_scan"
    pub(crate) source: String,
    pub(crate) fetched_at: String,
    pub(crate) error_message: Option<String>,
    pub(crate) windows: Option<Vec<QuotaWindow>>,
    pub(crate) local_tokens: Option<LocalTokenUsage>,
    pub(crate) extra_credits: Option<ExtraCredits>,
    #[serde(default)]
    pub(crate) reset_credits: Option<ResetCredits>,
}

pub(crate) struct QuotaCache {
    pub(crate) snapshots: Mutex<HashMap<String, QuotaSnapshot>>,
}

impl Default for QuotaCache {
    fn default() -> Self {
        QuotaCache {
            snapshots: Mutex::new(HashMap::new()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ClaudeUsageBlock {
    pub(crate) start_time: String,
    pub(crate) end_time: String,
    pub(crate) is_active: bool,
    pub(crate) input_tokens: u64,
    pub(crate) output_tokens: u64,
    pub(crate) cache_creation_tokens: u64,
    pub(crate) cache_read_tokens: u64,
    pub(crate) cost_usd: f64,
    pub(crate) usage_limit_reset_time: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ProviderQuota {
    pub(crate) provider: String,
    pub(crate) billing_period: String,
    pub(crate) input_tokens: u64,
    pub(crate) output_tokens: u64,
    pub(crate) cache_creation_tokens: u64,
    pub(crate) cache_read_tokens: u64,
    pub(crate) cost_usd: f64,
    pub(crate) monthly_limit_tokens: Option<u64>,
    pub(crate) monthly_limit_usd: Option<f64>,
    pub(crate) reset_day: u8,
    pub(crate) next_reset_date: String,
}
