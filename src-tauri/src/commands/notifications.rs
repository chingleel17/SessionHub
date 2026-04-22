use tauri_plugin_notification::NotificationExt;

fn send_intervention_notification_internal(
    app: &tauri::AppHandle,
    session_id: &str,
    project_name: &str,
    summary: &str,
    notification_type: &str,
) -> Result<(), String> {
    let summary_truncated = summary
        .char_indices()
        .nth(60)
        .map(|(i, _)| &summary[..i])
        .unwrap_or(summary);

    let body = if project_name.is_empty() {
        summary_truncated.to_string()
    } else if summary_truncated.is_empty() {
        project_name.to_string()
    } else {
        format!("{}: {}", project_name, summary_truncated)
    };

    let title = match notification_type {
        "session_end" => "SessionHub — Session 已完成",
        _ => "SessionHub — 需要您介入",
    };

    app.notification()
        .builder()
        .title(title)
        .body(&body)
        .show()
        .map_err(|e| format!("failed to send notification for session {session_id}: {e}"))?;

    Ok(())
}

#[tauri::command]
pub(crate) fn send_intervention_notification(
    app: tauri::AppHandle,
    session_id: String,
    project_name: String,
    summary: String,
    #[allow(dead_code)]
    notification_type: Option<String>,
) -> Result<(), String> {
    let ntype = notification_type.as_deref().unwrap_or("waiting");
    send_intervention_notification_internal(&app, &session_id, &project_name, &summary, ntype)
}
