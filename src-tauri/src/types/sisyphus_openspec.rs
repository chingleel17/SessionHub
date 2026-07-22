use serde::{Deserialize, Serialize};

// ── Sisyphus 型別 ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SisyphusBoulder {
    pub(crate) active_plan: Option<String>,
    pub(crate) plan_name: Option<String>,
    pub(crate) agent: Option<String>,
    pub(crate) session_ids: Vec<String>,
    pub(crate) started_at: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SisyphusPlan {
    pub(crate) name: String,
    pub(crate) path: String,
    pub(crate) title: Option<String>,
    pub(crate) tldr: Option<String>,
    pub(crate) is_active: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SisyphusNotepad {
    pub(crate) name: String,
    pub(crate) has_issues: bool,
    pub(crate) has_learnings: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SisyphusData {
    pub(crate) active_plan: Option<SisyphusBoulder>,
    pub(crate) plans: Vec<SisyphusPlan>,
    pub(crate) notepads: Vec<SisyphusNotepad>,
    pub(crate) evidence_files: Vec<String>,
    pub(crate) draft_files: Vec<String>,
}

// ── OpenSpec 型別 ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct OpenSpecTaskProgress {
    pub(crate) done: usize,
    pub(crate) total: usize,
    pub(crate) status: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct OpenSpecChange {
    pub(crate) name: String,
    pub(crate) has_proposal: bool,
    pub(crate) has_design: bool,
    pub(crate) has_tasks: bool,
    pub(crate) task_progress: Option<OpenSpecTaskProgress>,
    pub(crate) specs_count: usize,
    pub(crate) specs: Vec<OpenSpecSpec>,
    pub(crate) created_at: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct OpenSpecSpec {
    pub(crate) name: String,
    pub(crate) path: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct OpenSpecData {
    pub(crate) schema: Option<String>,
    pub(crate) active_changes: Vec<OpenSpecChange>,
    pub(crate) archived_changes: Vec<OpenSpecChange>,
    pub(crate) specs: Vec<OpenSpecSpec>,
}
