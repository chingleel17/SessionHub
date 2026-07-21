use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ModelMetricsEntry {
    pub(crate) requests_count: f64,
    pub(crate) requests_cost: f64,
    pub(crate) input_tokens: u64,
    pub(crate) output_tokens: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SessionStats {
    pub(crate) output_tokens: u64,
    pub(crate) input_tokens: u64,
    pub(crate) interaction_count: u32,
    pub(crate) tool_call_count: u32,
    pub(crate) duration_minutes: u64,
    pub(crate) models_used: Vec<String>,
    pub(crate) reasoning_count: u32,
    pub(crate) tool_breakdown: BTreeMap<String, u32>,
    pub(crate) model_metrics: BTreeMap<String, ModelMetricsEntry>,
    pub(crate) is_live: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AnalyticsDataPoint {
    pub(crate) label: String,
    pub(crate) output_tokens: u64,
    pub(crate) input_tokens: u64,
    pub(crate) interaction_count: u32,
    pub(crate) cost_points: f64,
    pub(crate) session_count: u32,
    pub(crate) missing_count: u32,
}

impl Default for SessionStats {
    fn default() -> Self {
        Self {
            output_tokens: 0,
            input_tokens: 0,
            interaction_count: 0,
            tool_call_count: 0,
            duration_minutes: 0,
            models_used: Vec::new(),
            reasoning_count: 0,
            tool_breakdown: BTreeMap::new(),
            model_metrics: BTreeMap::new(),
            is_live: false,
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SessionStartData {
    #[serde(default)]
    pub(crate) start_time: Option<String>,
    #[serde(default)]
    pub(crate) selected_model: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SessionModelChangeData {
    #[serde(default)]
    pub(crate) new_model: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct TopLevelFilterData {
    #[serde(default)]
    pub(crate) parent_tool_call_id: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ToolExecutionStartData {
    #[serde(default)]
    pub(crate) parent_tool_call_id: Option<String>,
    #[serde(default)]
    pub(crate) tool_name: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AssistantMessageData {
    #[serde(default)]
    pub(crate) parent_tool_call_id: Option<String>,
    #[serde(default)]
    pub(crate) output_tokens: Option<u64>,
    #[serde(default)]
    pub(crate) reasoning_opaque: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SessionShutdownRequestData {
    #[serde(default)]
    pub(crate) count: Option<f64>,
    #[serde(default)]
    pub(crate) cost: Option<f64>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SessionShutdownUsageData {
    #[serde(default)]
    pub(crate) input_tokens: Option<u64>,
    #[serde(default)]
    pub(crate) output_tokens: Option<u64>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SessionShutdownModelMetric {
    #[serde(default)]
    pub(crate) requests: Option<SessionShutdownRequestData>,
    #[serde(default)]
    pub(crate) usage: Option<SessionShutdownUsageData>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SessionShutdownData {
    #[serde(default)]
    pub(crate) model_metrics: BTreeMap<String, SessionShutdownModelMetric>,
}
