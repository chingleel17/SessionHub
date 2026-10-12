use tauri::State;
use tauri_plugin_notification::NotificationExt;

use crate::types::{InterventionItem, InterventionRegistry};

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
    #[allow(dead_code)] notification_type: Option<String>,
) -> Result<(), String> {
    let ntype = notification_type.as_deref().unwrap_or("waiting");
    send_intervention_notification_internal(&app, &session_id, &project_name, &summary, ntype)
}

/// 供 quota overlay 視窗（或主視窗）初次訂閱時查詢當前 waiting 清單快照，
/// 避免建立時機晚於 `intervention-list-changed` 廣播而永遠顯示空清單。
#[tauri::command]
pub(crate) fn get_intervention_list(
    registry: State<'_, InterventionRegistry>,
) -> Vec<InterventionItem> {
    registry.snapshot()
}

/// 目前處理中（`active`）的 session 數，供 overlay 狀態燈初次查詢與對帳。
#[tauri::command]
pub(crate) fn get_active_session_count(registry: State<'_, InterventionRegistry>) -> usize {
    registry.active_count()
}

/// overlay 狀態燈／卡片使用的合併結果。
#[derive(Debug, Clone, serde::Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct OverlayActivity {
    pub(crate) items: Vec<InterventionItem>,
    pub(crate) active_count: usize,
}

/// 合併 hook 事件狀態與 herdr 偵測狀態。
///
/// herdr 是由終端畫面直接偵測（含中斷、授權提示），比 hook 事件準確，因此對 herdr 認得的
/// session 以 herdr 為準：`blocked` → 等待處理、`working` → 處理中、`idle`/`done` → 兩者皆非；
/// `unknown` 與 herdr 不認得的 session（不在 herdr 內執行）維持 hook 推導的狀態。
pub(crate) fn merge_activity(
    hook_items: Vec<InterventionItem>,
    hook_active_ids: Vec<String>,
    herdr_states: &[crate::sessions::herdr::HerdrAgentState],
) -> OverlayActivity {
    use crate::sessions::herdr::HerdrAgentStatus;
    use std::collections::{HashMap, HashSet};

    let herdr: HashMap<&str, &crate::sessions::herdr::HerdrAgentState> = herdr_states
        .iter()
        .map(|state| (state.session_id.as_str(), state))
        .collect();

    let mut items: Vec<InterventionItem> = hook_items
        .into_iter()
        .filter(|item| {
            !matches!(
                herdr.get(item.session_id.as_str()).map(|s| s.status),
                Some(HerdrAgentStatus::Working | HerdrAgentStatus::Idle | HerdrAgentStatus::Done)
            )
        })
        .collect();
    let known: HashSet<String> = items.iter().map(|item| item.session_id.clone()).collect();
    for state in herdr_states {
        if state.status == HerdrAgentStatus::Blocked && !known.contains(&state.session_id) {
            let project_name = state
                .cwd
                .as_deref()
                .and_then(|cwd| {
                    cwd.trim_end_matches(['\\', '/'])
                        .rsplit(['\\', '/'])
                        .find(|segment| !segment.is_empty())
                })
                .unwrap_or(state.session_id.as_str())
                .to_string();
            items.push(InterventionItem {
                session_id: state.session_id.clone(),
                project_name,
                tool_label: None,
                since: chrono::Utc::now().to_rfc3339(),
            });
        }
    }
    items.sort_by(|a, b| a.session_id.cmp(&b.session_id));

    let waiting: HashSet<&str> = items.iter().map(|i| i.session_id.as_str()).collect();
    let mut active: HashSet<String> = hook_active_ids
        .into_iter()
        .filter(|id| {
            !matches!(
                herdr.get(id.as_str()).map(|s| s.status),
                Some(HerdrAgentStatus::Idle | HerdrAgentStatus::Done | HerdrAgentStatus::Blocked)
            )
        })
        .collect();
    for state in herdr_states {
        if state.status == HerdrAgentStatus::Working {
            active.insert(state.session_id.clone());
        }
    }
    active.retain(|id| !waiting.contains(id.as_str()));

    OverlayActivity {
        items,
        active_count: active.len(),
    }
}

/// overlay 初次查詢與定期對帳用：回傳合併後的「等待處理清單」與「處理中數量」。
/// herdr 查詢會啟動子程序，放到 blocking 執行緒以免卡住主執行緒。
#[tauri::command]
pub(crate) async fn get_overlay_activity(app: tauri::AppHandle) -> Result<OverlayActivity, String> {
    tauri::async_runtime::spawn_blocking(move || {
        crate::commands::activity_snapshot::collect_overlay_activity(&app)
    })
    .await
    .map_err(|error| error.to_string())
}

#[cfg(test)]
mod overlay_activity_tests {
    use super::*;
    use crate::sessions::herdr::{HerdrAgentState, HerdrAgentStatus};

    fn item(id: &str) -> InterventionItem {
        InterventionItem {
            session_id: id.to_string(),
            project_name: "p".to_string(),
            tool_label: None,
            since: "t".to_string(),
        }
    }

    fn state(id: &str, status: HerdrAgentStatus) -> HerdrAgentState {
        HerdrAgentState {
            session_id: id.to_string(),
            status,
            cwd: Some("/work/demo".to_string()),
        }
    }

    #[test]
    fn without_herdr_hook_state_is_used_as_is() {
        let merged = merge_activity(vec![item("a")], vec!["b".to_string()], &[]);
        assert_eq!(merged.items.len(), 1);
        assert_eq!(merged.active_count, 1);
    }

    #[test]
    fn herdr_idle_clears_stale_hook_waiting_and_active() {
        let merged = merge_activity(
            vec![item("a")],
            vec!["b".to_string()],
            &[state("a", HerdrAgentStatus::Idle), state("b", HerdrAgentStatus::Done)],
        );
        assert!(merged.items.is_empty());
        assert_eq!(merged.active_count, 0);
    }

    #[test]
    fn herdr_blocked_adds_waiting_even_without_hook_event() {
        let merged = merge_activity(vec![], vec!["a".to_string()], &[state("a", HerdrAgentStatus::Blocked)]);
        assert_eq!(merged.items.len(), 1);
        assert_eq!(merged.items[0].project_name, "demo");
        assert_eq!(merged.active_count, 0, "waiting session is not counted as active");
    }

    #[test]
    fn herdr_working_counts_as_active_and_unknown_defers_to_hooks() {
        let merged = merge_activity(
            vec![item("w")],
            vec![],
            &[state("x", HerdrAgentStatus::Working), state("w", HerdrAgentStatus::Unknown)],
        );
        assert_eq!(merged.active_count, 1);
        assert_eq!(merged.items.len(), 1);
    }
}
