//! 活動快照：單一函式產生 overlay／tray panel／原生 widget 共用的狀態，並節流寫出檔案。
//!
//! 檔案只含平台名稱、用量比例、重置時間、專案名稱與狀態；不含憑證、錯誤訊息、
//! session id、工具名稱或完整路徑（見 `activity-snapshot-feed` spec）。

use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::{Duration, Instant};

use serde::Serialize;
use tauri::{AppHandle, Manager};

use crate::commands::notifications::{merge_activity, OverlayActivity};
use crate::types::{AppSettings, InterventionRegistry, QuotaCache, QuotaSnapshot};

/// 快照檔名
pub(crate) const SNAPSHOT_FILE_NAME: &str = "activity-snapshot.json";

/// macOS App Group 識別碼：與 widget extension 的 entitlement 一致
pub(crate) const APP_GROUP_ID: &str = "group.com.ching.sessionhub";

/// 讀取端判斷過期的有效期限（秒）。寫入端每 `HEARTBEAT_INTERVAL` 刷新 `generatedAt`，
/// App 結束後超過此期限即視為過期。
pub(crate) const SNAPSHOT_VALIDITY_SECS: u64 = 120;

/// 寫入的最小間隔，避免事件連發時頻繁寫檔
const MIN_WRITE_INTERVAL: Duration = Duration::from_secs(2);

/// 沒有任何事件時的心跳間隔，確保 `generatedAt` 持續更新
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(30);

/// 活動快照（後端內部與前端 command 共用）
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ActivitySnapshot {
    pub(crate) quota: Vec<QuotaSnapshot>,
    pub(crate) activity: OverlayActivity,
    pub(crate) generated_at: String,
}

/// 寫入檔案的精簡版本
#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SnapshotFile {
    /// 檔案格式版本，讀取端據此相容
    pub(crate) version: u32,
    pub(crate) generated_at: String,
    pub(crate) validity_secs: u64,
    /// `waiting`（紅）｜`active`（黃）｜`idle`（灰）
    pub(crate) status: &'static str,
    pub(crate) active_count: usize,
    pub(crate) waiting: Vec<SnapshotWaiting>,
    pub(crate) quota: Vec<SnapshotQuota>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SnapshotWaiting {
    pub(crate) project_name: String,
    pub(crate) since: String,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SnapshotQuota {
    pub(crate) provider: String,
    pub(crate) windows: Vec<SnapshotQuotaWindow>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SnapshotQuotaWindow {
    pub(crate) label: String,
    pub(crate) utilization: f64,
    pub(crate) resets_at: Option<String>,
}

/// 由各資料來源組出快照；純函式，狀態判斷沿用 `merge_activity`。
///
/// 只保留列於 `enabled_providers` 的 quota，並依 provider 名稱排序以讓輸出穩定。
pub(crate) fn build_activity_snapshot(
    activity: OverlayActivity,
    quota: Vec<QuotaSnapshot>,
    enabled_providers: &[String],
    generated_at: String,
) -> ActivitySnapshot {
    let mut quota: Vec<QuotaSnapshot> = quota
        .into_iter()
        .filter(|snapshot| enabled_providers.contains(&snapshot.provider))
        .collect();
    quota.sort_by(|a, b| a.provider.cmp(&b.provider));
    ActivitySnapshot {
        quota,
        activity,
        generated_at,
    }
}

/// 轉為寫入檔案的精簡格式；僅 `status == "ok"` 且有 window 的 provider 會輸出。
pub(crate) fn to_snapshot_file(snapshot: &ActivitySnapshot) -> SnapshotFile {
    let status = if !snapshot.activity.items.is_empty() {
        "waiting"
    } else if snapshot.activity.active_count > 0 {
        "active"
    } else {
        "idle"
    };
    SnapshotFile {
        version: 1,
        generated_at: snapshot.generated_at.clone(),
        validity_secs: SNAPSHOT_VALIDITY_SECS,
        status,
        active_count: snapshot.activity.active_count,
        waiting: snapshot
            .activity
            .items
            .iter()
            .map(|item| SnapshotWaiting {
                project_name: item.project_name.clone(),
                since: item.since.clone(),
            })
            .collect(),
        quota: snapshot
            .quota
            .iter()
            .filter(|q| q.status == "ok")
            .filter_map(|q| {
                let windows: Vec<SnapshotQuotaWindow> = q
                    .windows
                    .iter()
                    .flatten()
                    .map(|w| SnapshotQuotaWindow {
                        label: w.label.clone(),
                        utilization: w.utilization,
                        resets_at: w.resets_at.clone(),
                    })
                    .collect();
                if windows.is_empty() {
                    None
                } else {
                    Some(SnapshotQuota {
                        provider: q.provider.clone(),
                        windows,
                    })
                }
            })
            .collect(),
    }
}

/// 合併 hook 事件與 herdr 偵測狀態。herdr 查詢會啟動子程序，呼叫端需在 blocking 執行緒使用。
pub(crate) fn collect_overlay_activity(app: &AppHandle) -> OverlayActivity {
    let registry = app.state::<InterventionRegistry>();
    let herdr_states = crate::sessions::herdr::herdr_agent_states();
    merge_activity(registry.snapshot(), registry.active_ids(), &herdr_states)
}

/// 從 app 狀態收集完整快照（活動狀態 + quota）。
pub(crate) fn collect_activity_snapshot(
    app: &AppHandle,
    settings: &AppSettings,
) -> ActivitySnapshot {
    let activity = collect_overlay_activity(app);
    let quota = app
        .state::<QuotaCache>()
        .snapshots
        .lock()
        .map(|guard| guard.values().cloned().collect())
        .unwrap_or_default();
    let enabled = if settings.enable_quota_monitoring {
        settings.quota_enabled_providers.as_slice()
    } else {
        &[]
    };
    build_activity_snapshot(activity, quota, enabled, chrono::Utc::now().to_rfc3339())
}

/// 決定快照檔所在目錄：App Group 容器存在時使用它，否則退回 App 資料目錄。
///
/// 不主動建立 App Group 容器：它由系統在 entitlement 生效時建立，
/// 手動建立會讓「沒有簽章」的環境誤以為 widget 可用。
pub(crate) fn resolve_snapshot_dir(group_container: Option<&Path>, fallback: PathBuf) -> PathBuf {
    match group_container {
        Some(dir) if dir.is_dir() => dir.to_path_buf(),
        _ => fallback,
    }
}

/// macOS App Group 容器路徑（`~/Library/Group Containers/<id>`）；其他平台為 None。
fn app_group_container() -> Option<PathBuf> {
    if !cfg!(target_os = "macos") {
        return None;
    }
    let home = std::env::var_os("HOME")?;
    Some(
        PathBuf::from(home)
            .join("Library")
            .join("Group Containers")
            .join(APP_GROUP_ID),
    )
}

pub(crate) fn snapshot_file_path() -> Result<PathBuf, String> {
    let fallback = crate::settings::default_app_data_dir()?;
    Ok(resolve_snapshot_dir(app_group_container().as_deref(), fallback).join(SNAPSHOT_FILE_NAME))
}

/// 原子寫入：先寫同目錄暫存檔再 rename，讀取端不會讀到半份資料。
pub(crate) fn write_snapshot_atomic(path: &Path, file: &SnapshotFile) -> Result<(), String> {
    let dir = path
        .parent()
        .ok_or_else(|| "snapshot path has no parent directory".to_string())?;
    std::fs::create_dir_all(dir).map_err(|e| format!("failed to create snapshot dir: {e}"))?;
    let json = serde_json::to_vec_pretty(file)
        .map_err(|e| format!("failed to serialize snapshot: {e}"))?;
    let tmp = path.with_extension("json.tmp");
    std::fs::write(&tmp, json).map_err(|e| format!("failed to write snapshot: {e}"))?;
    std::fs::rename(&tmp, path).map_err(|e| {
        let _ = std::fs::remove_file(&tmp);
        format!("failed to replace snapshot: {e}")
    })
}

/// 節流決策
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum ThrottleDecision {
    /// 立即寫入
    WriteNow,
    /// 間隔內：在指定延遲後補寫一次（尾端寫入，確保最後一次變動不會遺漏）
    ScheduleTrailing(Duration),
    /// 已有排程中的尾端寫入
    Skip,
}

#[derive(Default)]
pub(crate) struct WriteThrottle {
    last_write: Option<Instant>,
    trailing_scheduled: bool,
}

impl WriteThrottle {
    pub(crate) fn request(&mut self, now: Instant, min_interval: Duration) -> ThrottleDecision {
        if self.trailing_scheduled {
            return ThrottleDecision::Skip;
        }
        match self.last_write {
            Some(last) if now.duration_since(last) < min_interval => {
                self.trailing_scheduled = true;
                ThrottleDecision::ScheduleTrailing(min_interval - now.duration_since(last))
            }
            _ => {
                self.last_write = Some(now);
                ThrottleDecision::WriteNow
            }
        }
    }

    /// 尾端寫入執行後呼叫
    pub(crate) fn trailing_done(&mut self, now: Instant) {
        self.trailing_scheduled = false;
        self.last_write = Some(now);
    }
}

fn throttle() -> &'static Mutex<WriteThrottle> {
    static THROTTLE: std::sync::OnceLock<Mutex<WriteThrottle>> = std::sync::OnceLock::new();
    THROTTLE.get_or_init(|| Mutex::new(WriteThrottle::default()))
}

fn write_snapshot_now(app: &AppHandle) {
    let Ok(settings) = crate::load_settings_internal() else {
        return;
    };
    let snapshot = collect_activity_snapshot(app, &settings);
    let result = snapshot_file_path()
        .and_then(|path| write_snapshot_atomic(&path, &to_snapshot_file(&snapshot)));
    if let Err(error) = result {
        eprintln!("[activity-snapshot] {error}");
    }
}

/// 狀態或 quota 變動時呼叫；節流後在背景執行緒寫入，不阻塞呼叫端。
pub(crate) fn request_snapshot_write(app: &AppHandle) {
    let decision = throttle()
        .lock()
        .unwrap_or_else(|p| p.into_inner())
        .request(Instant::now(), MIN_WRITE_INTERVAL);
    let app = app.clone();
    match decision {
        ThrottleDecision::Skip => {}
        ThrottleDecision::WriteNow => {
            std::thread::spawn(move || write_snapshot_now(&app));
        }
        ThrottleDecision::ScheduleTrailing(delay) => {
            std::thread::spawn(move || {
                std::thread::sleep(delay);
                write_snapshot_now(&app);
                throttle()
                    .lock()
                    .unwrap_or_else(|p| p.into_inner())
                    .trailing_done(Instant::now());
            });
        }
    }
}

/// 心跳執行緒：沒有事件時也定期刷新 `generatedAt`，讓讀取端能以時間差判斷 App 是否仍在執行。
pub(crate) fn spawn_snapshot_heartbeat_thread(app: AppHandle) {
    std::thread::spawn(move || loop {
        write_snapshot_now(&app);
        std::thread::sleep(HEARTBEAT_INTERVAL);
    });
}

/// 供前端（overlay／tray panel）取得完整活動快照。
#[tauri::command]
pub(crate) async fn get_activity_snapshot(app: AppHandle) -> Result<ActivitySnapshot, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let settings = crate::load_settings_internal()?;
        Ok(collect_activity_snapshot(&app, &settings))
    })
    .await
    .map_err(|error| error.to_string())?
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{InterventionItem, QuotaWindow};

    fn quota(provider: &str, status: &str, utilization: f64) -> QuotaSnapshot {
        QuotaSnapshot {
            provider: provider.to_string(),
            status: status.to_string(),
            source: "remote_api".to_string(),
            fetched_at: "2026-01-01T00:00:00Z".to_string(),
            error_message: Some("C:\\Users\\secret\\token failure".to_string()),
            windows: Some(vec![QuotaWindow {
                window_key: "w".to_string(),
                label: "5h".to_string(),
                utilization,
                resets_at: Some("2026-01-01T05:00:00Z".to_string()),
                group: None,
            }]),
            extra_credits: None,
            reset_credits: None,
            plan: Some("pro".to_string()),
            ..Default::default()
        }
    }

    fn waiting_item() -> InterventionItem {
        InterventionItem {
            session_id: "sess-secret-id".to_string(),
            project_name: "demo".to_string(),
            tool_label: Some("Bash".to_string()),
            since: "2026-01-01T00:00:00Z".to_string(),
        }
    }

    fn unique_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "sessionhub-snapshot-{name}-{}-{}",
            std::process::id(),
            chrono::Utc::now().timestamp_nanos_opt().unwrap_or_default()
        ));
        std::fs::create_dir_all(&dir).expect("create test dir");
        dir
    }

    #[test]
    fn build_filters_providers_that_are_not_enabled() {
        let activity = OverlayActivity {
            items: vec![],
            active_count: 0,
        };
        let snapshot = build_activity_snapshot(
            activity,
            vec![quota("claude", "ok", 0.4), quota("codex", "ok", 0.1)],
            &["claude".to_string()],
            "t".to_string(),
        );
        assert_eq!(snapshot.quota.len(), 1);
        assert_eq!(snapshot.quota[0].provider, "claude");
    }

    #[test]
    fn file_status_follows_waiting_then_active_then_idle() {
        let make = |items: Vec<InterventionItem>, active_count| {
            to_snapshot_file(&build_activity_snapshot(
                OverlayActivity {
                    items,
                    active_count,
                },
                vec![],
                &[],
                "t".to_string(),
            ))
            .status
        };
        assert_eq!(make(vec![waiting_item()], 3), "waiting");
        assert_eq!(make(vec![], 2), "active");
        assert_eq!(make(vec![], 0), "idle");
    }

    #[test]
    fn file_contains_only_whitelisted_fields() {
        let snapshot = build_activity_snapshot(
            OverlayActivity {
                items: vec![waiting_item()],
                active_count: 1,
            },
            vec![quota("claude", "ok", 0.72), quota("codex", "error", 0.5)],
            &["claude".to_string(), "codex".to_string()],
            "2026-01-01T00:00:00Z".to_string(),
        );
        let file = to_snapshot_file(&snapshot);
        let json = serde_json::to_string(&file).expect("serialize");
        for forbidden in ["secret", "token", "sess-", "Bash", "pro\""] {
            assert!(!json.contains(forbidden), "不應包含 {forbidden}: {json}");
        }
        assert!(json.contains("\"generatedAt\":\"2026-01-01T00:00:00Z\""));
        assert!(json.contains("\"projectName\":\"demo\""));
        assert_eq!(file.quota.len(), 1, "非 ok 的 provider 不輸出");
        assert!((file.quota[0].windows[0].utilization - 0.72).abs() < 1e-9);
    }

    #[test]
    fn atomic_write_replaces_file_and_leaves_no_temp() {
        let dir = unique_dir("atomic");
        let path = dir.join(SNAPSHOT_FILE_NAME);
        let snapshot = |status_count| {
            to_snapshot_file(&build_activity_snapshot(
                OverlayActivity {
                    items: vec![],
                    active_count: status_count,
                },
                vec![],
                &[],
                "t".to_string(),
            ))
        };
        write_snapshot_atomic(&path, &snapshot(1)).expect("first write");
        write_snapshot_atomic(&path, &snapshot(5)).expect("second write");

        let value: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&path).expect("read")).expect("json");
        assert_eq!(value["activeCount"], 5);
        assert!(!path.with_extension("json.tmp").exists());
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn throttle_writes_first_then_schedules_single_trailing() {
        let mut throttle = WriteThrottle::default();
        let interval = Duration::from_secs(2);
        let t0 = Instant::now();

        assert_eq!(throttle.request(t0, interval), ThrottleDecision::WriteNow);
        let second = throttle.request(t0 + Duration::from_millis(500), interval);
        assert_eq!(
            second,
            ThrottleDecision::ScheduleTrailing(Duration::from_millis(1500))
        );
        assert_eq!(
            throttle.request(t0 + Duration::from_millis(700), interval),
            ThrottleDecision::Skip
        );

        throttle.trailing_done(t0 + Duration::from_secs(2));
        assert_eq!(
            throttle.request(t0 + Duration::from_secs(5), interval),
            ThrottleDecision::WriteNow
        );
    }

    #[test]
    fn dir_falls_back_without_app_group_container() {
        let fallback = PathBuf::from("/fallback/SessionHub");
        assert_eq!(resolve_snapshot_dir(None, fallback.clone()), fallback);
        let missing = std::env::temp_dir().join("sessionhub-no-such-group-container");
        assert_eq!(
            resolve_snapshot_dir(Some(&missing), fallback.clone()),
            fallback
        );
    }

    #[test]
    fn dir_prefers_existing_app_group_container() {
        let group = unique_dir("group");
        let resolved = resolve_snapshot_dir(Some(&group), PathBuf::from("/fallback"));
        assert_eq!(resolved, group);
        let _ = std::fs::remove_dir_all(group);
    }
}
