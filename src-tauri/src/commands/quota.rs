use tauri::State;

use crate::db::{
    billing_period_for, get_provider_quota_from_db, get_provider_quota_settings_from_db,
    next_reset_date_for, set_provider_quota_settings_in_db, upsert_provider_quota,
};
use crate::quota::cache::{
    prune_disabled_provider_quota, read_snapshots_from_cache, write_snapshot_to_cache_and_db,
};
use crate::quota::QuotaManager;
use crate::settings::{load_settings_internal, resolve_claude_root};
use crate::stats::{build_claude_usage_blocks, compute_claude_stats, is_claude_session_file};
use crate::types::*;
use crate::DbState;

fn collect_all_claude_sessions(claude_root: &str) -> Vec<std::path::PathBuf> {
    let projects_root = resolve_claude_root(Some(claude_root))
        .map(|p| p.join("projects"))
        .unwrap_or_default();

    if !projects_root.exists() {
        return Vec::new();
    }

    let mut files = Vec::new();
    let mut stack = vec![projects_root];
    while let Some(dir) = stack.pop() {
        if let Ok(entries) = std::fs::read_dir(&dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    stack.push(path);
                } else if is_claude_session_file(&path) {
                    files.push(path);
                }
            }
        }
    }
    files
}

#[tauri::command]
pub fn get_provider_quota(db_state: State<'_, DbState>) -> Result<Vec<ProviderQuota>, String> {
    let conn = db_state
        .conn
        .lock()
        .map_err(|_| "failed to lock db".to_string())?;

    let today = chrono::Utc::now().date_naive();
    let providers = vec![
        CLAUDE_PROVIDER,
        COPILOT_PROVIDER,
        OPENCODE_PROVIDER,
        CODEX_PROVIDER,
    ];

    let mut results = Vec::new();
    for provider in providers {
        let (limit_tokens, limit_usd, reset_day) =
            get_provider_quota_settings_from_db(&conn, provider).unwrap_or((None, None, 1));

        let billing_period = billing_period_for(reset_day, &today);
        let next_reset = next_reset_date_for(reset_day, &today);

        let (input_tokens, output_tokens, cache_creation_tokens, cache_read_tokens, cost_usd) =
            get_provider_quota_from_db(&conn, provider, &billing_period)
                .unwrap_or(None)
                .unwrap_or((0, 0, 0, 0, 0.0));

        results.push(ProviderQuota {
            provider: provider.to_string(),
            billing_period,
            input_tokens,
            output_tokens,
            cache_creation_tokens,
            cache_read_tokens,
            cost_usd,
            monthly_limit_tokens: limit_tokens,
            monthly_limit_usd: limit_usd,
            reset_day,
            next_reset_date: next_reset,
        });
    }

    Ok(results)
}

#[tauri::command]
pub fn set_provider_quota_settings(
    db_state: State<'_, DbState>,
    provider: String,
    monthly_limit_tokens: Option<u64>,
    monthly_limit_usd: Option<f64>,
    reset_day: u8,
) -> Result<(), String> {
    let conn = db_state
        .conn
        .lock()
        .map_err(|_| "failed to lock db".to_string())?;
    set_provider_quota_settings_in_db(
        &conn,
        &provider,
        monthly_limit_tokens,
        monthly_limit_usd,
        reset_day,
    )
}

#[tauri::command]
pub fn get_claude_usage_blocks(session_dir: String) -> Result<Vec<ClaudeUsageBlock>, String> {
    let path = std::path::PathBuf::from(&session_dir);
    if !path.exists() || !is_claude_session_file(&path) {
        return Err(format!("not a valid Claude session file: {session_dir}"));
    }
    build_claude_usage_blocks(&path)
}

#[tauri::command]
pub fn get_quota_snapshots(
    quota_cache: State<'_, QuotaCache>,
) -> Result<Vec<QuotaSnapshot>, String> {
    read_snapshots_from_cache(&quota_cache)
}

#[tauri::command]
pub fn refresh_quota(
    app: tauri::AppHandle,
    db_state: State<'_, DbState>,
    quota_cache: State<'_, QuotaCache>,
    provider: Option<String>,
) -> Result<Vec<QuotaSnapshot>, String> {
    let settings = load_settings_internal().map_err(|e| format!("failed to load settings: {e}"))?;

    if !settings.enable_quota_monitoring {
        return read_snapshots_from_cache(&quota_cache);
    }

    let conn = db_state
        .conn
        .lock()
        .map_err(|_| "failed to lock db".to_string())?;

    let manager = QuotaManager::new();
    let snapshots = match provider.as_deref() {
        Some(key) => manager
            .refresh_one(key, &settings)
            .map(|s| vec![s])
            .unwrap_or_default(),
        None => manager.refresh_all(&settings),
    };

    // Remove snapshots for providers that are no longer enabled for quota monitoring
    if provider.is_none() {
        let _ =
            prune_disabled_provider_quota(&conn, &quota_cache, &settings.quota_enabled_providers);
    }

    for snapshot in &snapshots {
        // 429 限流時保留快取中的舊 snapshot，不用這次的暫時性錯誤覆蓋畫面
        if snapshot.status == "rate_limited" {
            continue;
        }
        let _ = write_snapshot_to_cache_and_db(&conn, &quota_cache, snapshot);
    }

    crate::emit_quota_snapshots_updated(&app);

    // 回傳值以快取為底，但疊加本次遇到的 rate_limited 狀態，讓前端能提示使用者
    let mut result = read_snapshots_from_cache(&quota_cache)?;
    for snapshot in &snapshots {
        if snapshot.status == "rate_limited" {
            if let Some(existing) = result.iter_mut().find(|s| s.provider == snapshot.provider) {
                existing.status = "rate_limited".to_string();
                existing.error_message = snapshot.error_message.clone();
            } else {
                result.push(snapshot.clone());
            }
        }
    }
    Ok(result)
}

pub(crate) fn refresh_quota_internal(
    db_state: &DbState,
    quota_cache: &QuotaCache,
    settings: &AppSettings,
    provider: Option<&str>,
) -> Vec<QuotaSnapshot> {
    let conn = match db_state.conn.lock() {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };

    let manager = QuotaManager::new();
    let snapshots = match provider {
        Some(key) => manager
            .refresh_one(key, settings)
            .map(|s| vec![s])
            .unwrap_or_default(),
        None => manager.refresh_all(settings),
    };

    for snapshot in &snapshots {
        if snapshot.status == "rate_limited" {
            continue;
        }
        let _ = write_snapshot_to_cache_and_db(&conn, quota_cache, snapshot);
    }

    snapshots
}

#[tauri::command]
pub fn refresh_claude_quota(
    db_state: State<'_, DbState>,
    claude_root: String,
) -> Result<(), String> {
    let conn = db_state
        .conn
        .lock()
        .map_err(|_| "failed to lock db".to_string())?;

    let today = chrono::Utc::now().date_naive();
    let (_, _, reset_day) =
        get_provider_quota_settings_from_db(&conn, CLAUDE_PROVIDER).unwrap_or((None, None, 1));
    let billing_period = billing_period_for(reset_day, &today);

    let session_files = collect_all_claude_sessions(&claude_root);

    let mut total_input: u64 = 0;
    let mut total_output: u64 = 0;
    let total_cache_creation: u64 = 0;
    let total_cache_read: u64 = 0;
    let mut total_cost: f64 = 0.0;

    for path in session_files {
        if let Ok(stats) = compute_claude_stats(&path) {
            total_input += stats.input_tokens;
            total_output += stats.output_tokens;
            // cache tokens aren't in SessionStats directly; use model_metrics cost as proxy
            total_cost += stats
                .model_metrics
                .values()
                .map(|m| m.requests_cost)
                .sum::<f64>();
        }
    }

    upsert_provider_quota(
        &conn,
        CLAUDE_PROVIDER,
        &billing_period,
        total_input,
        total_output,
        total_cache_creation,
        total_cache_read,
        total_cost,
    )
}
