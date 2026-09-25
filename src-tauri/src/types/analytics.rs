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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum AnalyticsMetric {
    Tokens,
    EstimatedUsd,
    CopilotPoints,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum AnalyticsTokenField {
    Total,
    Input,
    Output,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum AnalyticsBreakdown {
    Total,
    Provider,
    Model,
    TokenType,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum AnalyticsGroupBy {
    Day,
    Week,
    Month,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AnalyticsQuery {
    pub(crate) start_date: String,
    pub(crate) end_date: String,
    pub(crate) time_zone: String,
    pub(crate) group_by: AnalyticsGroupBy,
    pub(crate) providers: Vec<String>,
    pub(crate) cwd: Option<String>,
    pub(crate) model: Option<String>,
    #[serde(default)]
    pub(crate) cwds: Vec<String>,
    #[serde(default)]
    pub(crate) models: Vec<String>,
    pub(crate) include_archived: bool,
    pub(crate) metric: AnalyticsMetric,
    pub(crate) token_field: AnalyticsTokenField,
    pub(crate) breakdown: AnalyticsBreakdown,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AnalyticsMetricValue {
    pub(crate) known_value: Option<f64>,
    pub(crate) eligible_event_count: u64,
    pub(crate) missing_field_event_count: u64,
    pub(crate) price_versions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AnalyticsValues {
    pub(crate) total_tokens: AnalyticsMetricValue,
    pub(crate) input_tokens: AnalyticsMetricValue,
    pub(crate) output_tokens: AnalyticsMetricValue,
    pub(crate) cache_read_tokens: AnalyticsMetricValue,
    pub(crate) cache_write_tokens: AnalyticsMetricValue,
    pub(crate) reasoning_tokens: AnalyticsMetricValue,
    pub(crate) estimated_usd: AnalyticsMetricValue,
    pub(crate) copilot_points: AnalyticsMetricValue,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AnalyticsSeriesPoint {
    pub(crate) label: String,
    pub(crate) start_at: String,
    pub(crate) end_at: String,
    pub(crate) values: AnalyticsValues,
    pub(crate) interaction_count: u64,
    pub(crate) session_count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum AnalyticsCoverageStatus {
    Complete,
    Partial,
    Pending,
    Unsupported,
    Error,
    SummaryOnly,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AnalyticsMetricCoverage {
    pub(crate) eligible_event_count: u64,
    pub(crate) missing_field_event_count: u64,
    pub(crate) status: AnalyticsCoverageStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AnalyticsProviderCoverage {
    pub(crate) provider: String,
    pub(crate) status: AnalyticsCoverageStatus,
    pub(crate) complete_session_count: u64,
    pub(crate) partial_session_count: u64,
    pub(crate) pending_session_count: u64,
    pub(crate) unsupported_session_count: u64,
    pub(crate) error_session_count: u64,
    pub(crate) summary_only_session_count: u64,
    pub(crate) period_unknown_session_count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AnalyticsCoverage {
    pub(crate) metrics: BTreeMap<String, AnalyticsMetricCoverage>,
    pub(crate) providers: Vec<AnalyticsProviderCoverage>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AnalyticsRankingEntry {
    pub(crate) key: String,
    pub(crate) label: String,
    pub(crate) value: AnalyticsMetricValue,
    pub(crate) session_count: u64,
    pub(crate) share_of_known_value: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AnalyticsSessionSummaryOnly {
    pub(crate) provider: String,
    pub(crate) session_id: String,
    pub(crate) cwd: Option<String>,
    pub(crate) model: Option<String>,
    pub(crate) active_from: Option<String>,
    pub(crate) active_until: Option<String>,
    pub(crate) values: AnalyticsValues,
    pub(crate) status: AnalyticsCoverageStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AnalyticsReport {
    pub(crate) summary: AnalyticsValues,
    pub(crate) previous_summary: Option<AnalyticsValues>,
    pub(crate) comparison: AnalyticsComparison,
    pub(crate) series: Vec<AnalyticsSeriesPoint>,
    pub(crate) project_ranking: Vec<AnalyticsRankingEntry>,
    pub(crate) model_ranking: Vec<AnalyticsRankingEntry>,
    pub(crate) platform_ranking: Vec<AnalyticsRankingEntry>,
    pub(crate) supplier_ranking: Vec<AnalyticsRankingEntry>,
    pub(crate) coverage: AnalyticsCoverage,
    pub(crate) session_summary_only: Vec<AnalyticsSessionSummaryOnly>,
    pub(crate) revision: u64,
    pub(crate) generated_at: String,
    pub(crate) time_zone: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum AnalyticsComparisonStatus {
    Comparable,
    NewUsage,
    NoChange,
    NotComparable,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AnalyticsComparison {
    pub(crate) status: AnalyticsComparisonStatus,
    pub(crate) percent_change: Option<f64>,
    pub(crate) reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum AnalyticsDetailStatus {
    Ready,
    StaleRevision,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum AnalyticsSessionSort {
    EventTimeDesc,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AnalyticsSessionDetail {
    pub(crate) provider: String,
    pub(crate) model_provider_ids: Vec<String>,
    pub(crate) session_id: String,
    pub(crate) cwd: Option<String>,
    pub(crate) model: Option<String>,
    pub(crate) first_event_at: Option<String>,
    pub(crate) last_event_at: Option<String>,
    pub(crate) values: AnalyticsValues,
    pub(crate) session_summary_only: Option<AnalyticsValues>,
    pub(crate) status: AnalyticsCoverageStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AnalyticsSessionPage {
    pub(crate) status: AnalyticsDetailStatus,
    pub(crate) revision: u64,
    pub(crate) page: u32,
    pub(crate) page_size: u32,
    pub(crate) total_count: u64,
    pub(crate) items: Vec<AnalyticsSessionDetail>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct UsageEventRecord {
    pub(crate) provider: String,
    pub(crate) model_provider_id: Option<String>,
    pub(crate) service_tier: Option<String>,
    pub(crate) session_id: String,
    pub(crate) source_event_id: String,
    pub(crate) occurred_at: String,
    pub(crate) cwd: Option<String>,
    pub(crate) model: Option<String>,
    pub(crate) input_tokens: Option<i64>,
    pub(crate) output_tokens: Option<i64>,
    pub(crate) cache_read_tokens: Option<i64>,
    pub(crate) cache_write_tokens: Option<i64>,
    pub(crate) reasoning_tokens: Option<i64>,
    pub(crate) estimated_usd_micros: Option<i64>,
    pub(crate) estimated_usd_price_version: Option<String>,
    pub(crate) cost_points_micros: Option<i64>,
    pub(crate) source_kind: String,
    pub(crate) parser_version: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct UsageSessionSummaryRecord {
    pub(crate) provider: String,
    pub(crate) session_id: String,
    pub(crate) source_identity: String,
    pub(crate) cwd: Option<String>,
    pub(crate) model: Option<String>,
    pub(crate) active_from: Option<String>,
    pub(crate) active_until: Option<String>,
    pub(crate) input_tokens: Option<i64>,
    pub(crate) output_tokens: Option<i64>,
    pub(crate) estimated_usd_micros: Option<i64>,
    pub(crate) cost_points_micros: Option<i64>,
    pub(crate) source_kind: String,
    pub(crate) parser_version: i64,
    pub(crate) integrity_status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ProviderUsageExtraction {
    pub(crate) events: Vec<UsageEventRecord>,
    pub(crate) session_summaries: Vec<UsageSessionSummaryRecord>,
    pub(crate) status: AnalyticsCoverageStatus,
    pub(crate) reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
#[cfg(test)]
pub(crate) struct ProviderUsageCapability {
    pub(crate) provider: String,
    pub(crate) token_status: AnalyticsCoverageStatus,
    pub(crate) estimated_usd_status: AnalyticsCoverageStatus,
    pub(crate) copilot_points_status: AnalyticsCoverageStatus,
    pub(crate) event_usage_supported: bool,
    pub(crate) session_summary_supported: bool,
    pub(crate) reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct UsageIngestionStateRecord {
    pub(crate) provider: String,
    pub(crate) session_id: String,
    pub(crate) source_identity: String,
    pub(crate) source_fingerprint: Option<String>,
    pub(crate) parser_version: i64,
    pub(crate) integrity_status: String,
    pub(crate) ingestion_status: String,
    pub(crate) error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[cfg(test)]
pub(crate) struct UsageIndexProgress {
    pub(crate) total_sessions: u64,
    pub(crate) indexed_sessions: u64,
    pub(crate) pending_sessions: u64,
    pub(crate) unsupported_sessions: u64,
    pub(crate) error_sessions: u64,
    pub(crate) unindexed_sessions: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct ModelPricingCacheRecord {
    pub(crate) provider: String,
    pub(crate) model: String,
    pub(crate) status: String,
    pub(crate) prompt_usd_per_token: Option<f64>,
    pub(crate) completion_usd_per_token: Option<f64>,
    pub(crate) cache_read_usd_per_token: Option<f64>,
    pub(crate) cache_write_usd_per_token: Option<f64>,
    pub(crate) source: String,
    pub(crate) fetched_at: i64,
    pub(crate) expires_at: i64,
    pub(crate) error_kind: Option<String>,
    pub(crate) hidden: bool,
    pub(crate) sort_order: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ModelPricingEntry {
    pub(crate) provider: String,
    pub(crate) model: String,
    pub(crate) status: String,
    pub(crate) prompt_usd_per_million: Option<f64>,
    pub(crate) completion_usd_per_million: Option<f64>,
    pub(crate) cache_read_usd_per_million: Option<f64>,
    pub(crate) cache_write_usd_per_million: Option<f64>,
    pub(crate) source: String,
    pub(crate) fetched_at: i64,
    pub(crate) expires_at: i64,
    pub(crate) error_kind: Option<String>,
    pub(crate) hidden: bool,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ManualModelPricingInput {
    pub(crate) provider: String,
    pub(crate) model: String,
    pub(crate) prompt_usd_per_million: f64,
    pub(crate) completion_usd_per_million: f64,
    pub(crate) cache_read_usd_per_million: Option<f64>,
    pub(crate) cache_write_usd_per_million: Option<f64>,
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
    #[serde(default)]
    pub(crate) total_nano_aiu: Option<f64>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SessionShutdownData {
    #[serde(default)]
    pub(crate) model_metrics: BTreeMap<String, SessionShutdownModelMetric>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SessionUsageCheckpointData {
    pub(crate) total_nano_aiu: f64,
}
