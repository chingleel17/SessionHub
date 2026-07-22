use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub(crate) struct WorkspaceYaml {
    pub(crate) id: String,
    pub(crate) cwd: Option<String>,
    pub(crate) summary: Option<String>,
    pub(crate) summary_count: Option<u32>,
    pub(crate) created_at: Option<String>,
    pub(crate) updated_at: Option<String>,
}

pub(crate) fn default_provider() -> String {
    "copilot".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SessionInfo {
    pub(crate) id: String,
    #[serde(default = "default_provider")]
    pub(crate) provider: String,
    pub(crate) cwd: Option<String>,
    pub(crate) repo_root: Option<String>,
    pub(crate) repo_name: Option<String>,
    pub(crate) git_branch: Option<String>,
    pub(crate) summary: Option<String>,
    pub(crate) summary_count: Option<u32>,
    pub(crate) created_at: Option<String>,
    pub(crate) updated_at: Option<String>,
    pub(crate) session_dir: String,
    pub(crate) parse_error: bool,
    pub(crate) is_archived: bool,
    pub(crate) notes: Option<String>,
    pub(crate) tags: Vec<String>,
    pub(crate) has_plan: bool,
    pub(crate) has_events: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SessionTodo {
    pub(crate) id: String,
    pub(crate) title: String,
    pub(crate) status: String,
    pub(crate) description: Option<String>,
    pub(crate) updated_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SessionActivityStatus {
    pub(crate) session_id: String,
    /// "idle" | "active" | "waiting" | "done"
    pub(crate) status: String,
    /// "thinking" | "tool_call" | "file_op" | "sub_agent" | "working" | "completed"
    pub(crate) detail: Option<String>,
    pub(crate) last_activity_at: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SessionTargetedPayload {
    pub(crate) session_id: String,
    pub(crate) cwd: String,
    pub(crate) event_type: String,
}

/// tool.pre / tool.post / prompt.submitted 等純活動提示事件的輕量 payload，
/// 前端可直接用 cwd 比對更新 activity status，不需要任何 IPC 或檔案掃描。
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ActivityHintPayload {
    pub(crate) cwd: String,
    pub(crate) event_type: String,
    pub(crate) title: Option<String>,
    pub(crate) error: Option<String>,
    /// 後端已計算好的 session 狀態，前端直接 patch activityStatusMap 使用
    pub(crate) session_id: Option<String>,
    /// "active" | "waiting" | "idle"
    pub(crate) status: Option<String>,
    /// "thinking" | "tool_call" | "working"
    pub(crate) detail: Option<String>,
    pub(crate) last_activity_at: Option<String>,
}

/// 每次 provider bridge 收到事件時發送給前端的 debug log 記錄。
/// status 可能值："targeted" | "fallback" | "full_refresh" | "skipped_dedup" | "skipped_rate_limit" | "activity_hint"
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct BridgeEventLogEntry {
    pub(crate) id: String,
    pub(crate) provider: String,
    pub(crate) event_type: String,
    pub(crate) timestamp: String,
    pub(crate) cwd: Option<String>,
    pub(crate) session_id: Option<String>,
    pub(crate) title: Option<String>,
    pub(crate) error: Option<String>,
    /// "targeted" | "fallback" | "full_refresh" | "skipped_dedup" | "skipped_rate_limit"
    pub(crate) status: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SessionMeta {
    pub(crate) notes: Option<String>,
    pub(crate) tags: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct SessionEvent {
    #[serde(rename = "type")]
    pub(crate) event_type: String,
    #[serde(default)]
    pub(crate) timestamp: Option<String>,
    #[serde(default)]
    pub(crate) data: serde_json::Value,
}
