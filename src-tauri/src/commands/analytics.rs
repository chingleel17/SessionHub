use tauri::{Manager, State};

use crate::db::{read_analytics_revision_internal, DbState};
use crate::types::{
    AnalyticsQuery, AnalyticsReport, AnalyticsSessionPage, AnalyticsSessionSort,
    ManualModelPricingInput, ModelPricingEntry,
};

#[tauri::command]
pub fn get_analytics_report(
    query: AnalyticsQuery,
    db: State<'_, DbState>,
) -> Result<AnalyticsReport, String> {
    let connection = db
        .conn
        .lock()
        .map_err(|error| format!("db lock poisoned: {error}"))?;
    crate::stats::get_analytics_report_internal(&connection, &query)
}

#[tauri::command]
pub fn get_analytics_session_page(
    query: AnalyticsQuery,
    requested_revision: u64,
    page: u32,
    page_size: u32,
    sort: AnalyticsSessionSort,
    db: State<'_, DbState>,
) -> Result<AnalyticsSessionPage, String> {
    let connection = db
        .conn
        .lock()
        .map_err(|error| format!("db lock poisoned: {error}"))?;
    crate::stats::get_analytics_session_page_internal(
        &connection,
        &query,
        requested_revision,
        page,
        page_size,
        sort,
    )
}

#[tauri::command]
pub fn get_analytics_revision(db: State<'_, DbState>) -> Result<u64, String> {
    let connection = db
        .conn
        .lock()
        .map_err(|error| format!("db lock poisoned: {error}"))?;
    read_analytics_revision_internal(&connection)
}

#[tauri::command]
pub fn list_model_pricing(db: State<'_, DbState>) -> Result<Vec<ModelPricingEntry>, String> {
    let connection = db
        .conn
        .lock()
        .map_err(|error| format!("db lock poisoned: {error}"))?;
    crate::stats::list_model_pricing_internal(&connection)
}

#[tauri::command]
pub async fn sync_model_pricing(app: tauri::AppHandle) -> Result<Vec<ModelPricingEntry>, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let db = app.state::<DbState>();
        let connection = db
            .conn
            .lock()
            .map_err(|error| format!("db lock poisoned: {error}"))?;
        crate::stats::sync_model_pricing_internal(&connection)
    })
    .await
    .map_err(|error| format!("pricing sync task failed: {error}"))?
}

#[tauri::command]
pub fn save_manual_model_pricing(
    input: ManualModelPricingInput,
    db: State<'_, DbState>,
) -> Result<Vec<ModelPricingEntry>, String> {
    let connection = db
        .conn
        .lock()
        .map_err(|error| format!("db lock poisoned: {error}"))?;
    crate::stats::save_manual_model_pricing_internal(&connection, input)
}

#[tauri::command]
pub fn delete_manual_model_pricing(
    provider: String,
    model: String,
    db: State<'_, DbState>,
) -> Result<Vec<ModelPricingEntry>, String> {
    let connection = db
        .conn
        .lock()
        .map_err(|error| format!("db lock poisoned: {error}"))?;
    crate::stats::delete_manual_model_pricing_internal(&connection, &provider, &model)
}

#[tauri::command]
pub fn set_model_pricing_visibility(
    provider: String,
    model: String,
    hidden: bool,
    db: State<'_, DbState>,
) -> Result<Vec<ModelPricingEntry>, String> {
    let connection = db
        .conn
        .lock()
        .map_err(|error| format!("db lock poisoned: {error}"))?;
    crate::stats::set_model_pricing_hidden_internal(&connection, &provider, &model, hidden)
}
