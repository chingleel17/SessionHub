use std::collections::HashMap;
use std::sync::Mutex;

use serde::{Deserialize, Serialize};

/// 單筆等待授權（`waiting`）項目；欄位刻意最小化，不含指令、檔案內容或完整路徑。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct InterventionItem {
    pub(crate) session_id: String,
    /// 由 session `cwd` 最後一段路徑推導，取不到時 fallback sessionId 尾段或 provider 名
    pub(crate) project_name: String,
    /// 僅工具類型字串（如 `Bash`/`Read`/`Edit`/`Write`），可能缺失
    pub(crate) tool_label: Option<String>,
    /// 進入 waiting 的時間（ISO 8601 UTC）
    pub(crate) since: String,
}

/// 後端維護的 `waiting` session 清單，作為介入提醒的 single source of truth。
///
/// 更新點位於後端 activity 狀態計算之後（`provider/bridge.rs` 的 Claude / OpenCode
/// 分支），不依賴任何前端視窗的 state；主視窗關閉或最小化時仍正常更新。
///
/// 範圍排除 Copilot／Codex：Copilot CLI 官方 hook 事件集合無等待授權訊號，
/// Codex 的 activity 計算從未產生 `waiting`，詳見 design.md 決策 1「已知限制」。
#[derive(Default)]
pub(crate) struct InterventionRegistry {
    items: Mutex<HashMap<String, InterventionItem>>,
}

impl InterventionRegistry {
    /// 將 session 加入或更新為 waiting 狀態，回傳目前完整清單快照。
    /// 將 session 加入或更新為 waiting；內容有實質變動時回傳新快照，否則回傳 None（避免重複廣播）。
    /// `since` 於已存在且其他欄位相同時沿用舊值，不視為變動。
    pub(crate) fn upsert(
        &self,
        session_id: &str,
        project_name: String,
        tool_label: Option<String>,
        since: String,
    ) -> Option<Vec<InterventionItem>> {
        let mut items = self.items.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(existing) = items.get(session_id) {
            if existing.project_name == project_name && existing.tool_label == tool_label {
                return None;
            }
        }
        items.insert(
            session_id.to_string(),
            InterventionItem {
                session_id: session_id.to_string(),
                project_name,
                tool_label,
                since,
            },
        );
        Some(Self::snapshot_locked(&items))
    }

    /// 將 session 自清單移除（離開 waiting）；確有移除時回傳新快照，否則回傳 None。
    pub(crate) fn remove(&self, session_id: &str) -> Option<Vec<InterventionItem>> {
        let mut items = self.items.lock().unwrap_or_else(|e| e.into_inner());
        if items.remove(session_id).is_some() {
            Some(Self::snapshot_locked(&items))
        } else {
            None
        }
    }

    /// 取得目前完整清單快照，供初次查詢 command 使用。
    pub(crate) fn snapshot(&self) -> Vec<InterventionItem> {
        let items = self.items.lock().unwrap_or_else(|e| e.into_inner());
        Self::snapshot_locked(&items)
    }

    fn snapshot_locked(items: &HashMap<String, InterventionItem>) -> Vec<InterventionItem> {
        let mut list: Vec<InterventionItem> = items.values().cloned().collect();
        list.sort_by(|left, right| left.session_id.cmp(&right.session_id));
        list
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn upsert_adds_new_item_and_returns_snapshot() {
        let registry = InterventionRegistry::default();
        let snapshot = registry
            .upsert(
                "session-1",
                "demo-project".to_string(),
                Some("Bash".to_string()),
                "2026-07-21T00:00:00.000Z".to_string(),
            )
            .expect("new item should report change");

        assert_eq!(snapshot.len(), 1);
        assert_eq!(snapshot[0].session_id, "session-1");
        assert_eq!(snapshot[0].project_name, "demo-project");
        assert_eq!(snapshot[0].tool_label.as_deref(), Some("Bash"));
    }

    #[test]
    fn upsert_overwrites_existing_item_with_same_session_id() {
        let registry = InterventionRegistry::default();
        registry.upsert(
            "session-1",
            "demo-project".to_string(),
            Some("Bash".to_string()),
            "2026-07-21T00:00:00.000Z".to_string(),
        );
        let snapshot = registry
            .upsert(
                "session-1",
                "demo-project".to_string(),
                Some("Edit".to_string()),
                "2026-07-21T00:01:00.000Z".to_string(),
            )
            .expect("changed tool_label should report change");

        assert_eq!(snapshot.len(), 1);
        assert_eq!(snapshot[0].tool_label.as_deref(), Some("Edit"));
    }

    #[test]
    fn upsert_identical_item_reports_no_change() {
        let registry = InterventionRegistry::default();
        registry.upsert(
            "session-1",
            "demo-project".to_string(),
            Some("Bash".to_string()),
            "2026-07-21T00:00:00.000Z".to_string(),
        );
        let result = registry.upsert(
            "session-1",
            "demo-project".to_string(),
            Some("Bash".to_string()),
            "2026-07-21T00:05:00.000Z".to_string(),
        );

        assert!(
            result.is_none(),
            "identical upsert should not report change"
        );
    }

    #[test]
    fn remove_deletes_item_from_snapshot() {
        let registry = InterventionRegistry::default();
        registry.upsert(
            "session-1",
            "demo-project".to_string(),
            None,
            "2026-07-21T00:00:00.000Z".to_string(),
        );
        let snapshot = registry
            .remove("session-1")
            .expect("removing existing session should report change");

        assert!(snapshot.is_empty());
    }

    #[test]
    fn remove_nonexistent_session_is_noop() {
        let registry = InterventionRegistry::default();
        let result = registry.remove("missing-session");

        assert!(
            result.is_none(),
            "removing absent session should not report change"
        );
    }

    #[test]
    fn snapshot_returns_items_sorted_by_session_id() {
        let registry = InterventionRegistry::default();
        registry.upsert("session-b", "proj-b".to_string(), None, "t2".to_string());
        registry.upsert("session-a", "proj-a".to_string(), None, "t1".to_string());

        let snapshot = registry.snapshot();

        assert_eq!(snapshot.len(), 2);
        assert_eq!(snapshot[0].session_id, "session-a");
        assert_eq!(snapshot[1].session_id, "session-b");
    }

    #[test]
    fn snapshot_serializes_with_camel_case_fields() {
        let item = InterventionItem {
            session_id: "session-1".to_string(),
            project_name: "demo".to_string(),
            tool_label: Some("Bash".to_string()),
            since: "2026-07-21T00:00:00.000Z".to_string(),
        };

        let json = serde_json::to_string(&item).expect("serialize");
        assert!(json.contains("\"sessionId\":\"session-1\""));
        assert!(json.contains("\"projectName\":\"demo\""));
        assert!(json.contains("\"toolLabel\":\"Bash\""));
    }
}
