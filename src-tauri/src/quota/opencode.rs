use crate::types::{AppSettings, QuotaSnapshot, OPENCODE_PROVIDER};

use super::QuotaAdapter;

pub(crate) struct OpenCodeAdapter;

fn current_timestamp() -> String {
    chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string()
}

impl QuotaAdapter for OpenCodeAdapter {
    fn provider_key(&self) -> &str {
        OPENCODE_PROVIDER
    }

    /// OpenCode 無帳號層級的額度 API，snapshot 不含額度數值，
    /// 由前端依「無可顯示額度資料」的通用規則呈現說明文字。
    fn fetch_snapshot(&self, _settings: &AppSettings) -> QuotaSnapshot {
        QuotaSnapshot {
            provider: OPENCODE_PROVIDER.to_string(),
            status: "ok".to_string(),
            source: "local_scan".to_string(),
            fetched_at: current_timestamp(),
            error_message: None,
            windows: None,
            extra_credits: None,
            reset_credits: None,
        }
    }
}
