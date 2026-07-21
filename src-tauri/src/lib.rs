mod activity;
mod agents_config;
mod antigravity_hooks;
mod commands;
mod db;
mod mcp_config;
mod openspec_scan;
mod platform;
mod provider;
mod quota;
mod sessions;
mod settings;
mod sisyphus;
mod stats;
mod tray_icon;
mod types;
mod watcher;

use tauri::{Emitter, Manager};

pub(crate) use commands::*;
pub(crate) use db::*;
pub(crate) use settings::*;
pub(crate) use types::*;
pub(crate) use watcher::*;

/// overlay 視窗 label
pub(crate) const QUOTA_OVERLAY_LABEL: &str = "quota-overlay";
/// mini panel 視窗 label
pub(crate) const TRAY_PANEL_LABEL: &str = "tray-panel";

/// 將設定變更定向通知 overlay webview，避免動態建立視窗遺漏 AppHandle 廣播。
pub(crate) fn emit_overlay_settings_changed(
    app: &tauri::AppHandle,
    settings: &AppSettings,
) {
    if let Some(overlay) = app.get_webview_window(QUOTA_OVERLAY_LABEL) {
        let _ = overlay.emit("quota-overlay-settings-changed", settings);
    }
}

/// 同時廣播並定向通知 overlay，讓動態建立的 webview 不會錯過快照更新。
pub(crate) fn emit_quota_snapshots_updated(app: &tauri::AppHandle) {
    let _ = app.emit("quota-snapshots-updated", ());
    if let Some(overlay) = app.get_webview_window(QUOTA_OVERLAY_LABEL) {
        let _ = overlay.emit("quota-snapshots-updated", ());
    }
}

/// 檢查 `tauri-plugin-window-state` 是否已為指定視窗 label 存過位置狀態。
fn has_saved_window_state(app: &tauri::AppHandle, label: &str) -> bool {
    let Ok(app_data_dir) = app.path().app_data_dir() else {
        return false;
    };
    let state_path = app_data_dir.join(".window-state.json");
    let Ok(contents) = std::fs::read_to_string(state_path) else {
        return false;
    };
    let Ok(value) = serde_json::from_str::<serde_json::Value>(&contents) else {
        return false;
    };
    value.get(label).is_some()
}

/// 將視窗定位到主螢幕右下角（保留少量邊距）。
fn position_window_bottom_right(window: &tauri::WebviewWindow) {
    const MARGIN: i32 = 16;
    let (Ok(monitor), Ok(size)) = (window.current_monitor(), window.outer_size()) else {
        return;
    };
    let Some(monitor) = monitor else { return };
    let screen_pos = monitor.position();
    let screen_size = monitor.size();
    let x = screen_pos.x + screen_size.width as i32 - size.width as i32 - MARGIN;
    let y = screen_pos.y + screen_size.height as i32 - size.height as i32 - MARGIN;
    let _ = window.set_position(tauri::PhysicalPosition::new(x.max(screen_pos.x), y.max(screen_pos.y)));
}

/// 建立（或顯示）常駐 quota overlay 視窗。
///
/// 必須在 Rust 端以 `WebviewWindowBuilder` 建立，`tauri.conf.json` 的 `focus: false`
/// 在 Windows 不生效（tauri#11566 / #7519）。
pub(crate) fn create_quota_overlay(app: &tauri::AppHandle, settings: &AppSettings) {
    if let Some(existing) = app.get_webview_window(QUOTA_OVERLAY_LABEL) {
        // 尺寸由前端依內容量測後自行同步（QuotaOverlay 的 ResizeObserver），這裡不強制固定尺寸
        let _ = existing.show();
        // Windows 上 show() 可能重置 taskbar 樣式（tauri#10422），顯示後重新套用
        let _ = existing.set_skip_taskbar(true);
        apply_overlay_locked(&existing, settings.quota_overlay_locked);
        emit_overlay_settings_changed(app, settings);
        return;
    }

    let builder = tauri::webview::WebviewWindowBuilder::new(
        app,
        QUOTA_OVERLAY_LABEL,
        tauri::WebviewUrl::App("index.html?view=quota-overlay".into()),
    )
    .title("SessionHub Quota Overlay")
    .transparent(true)
    .decorations(false)
    .shadow(false)
    .always_on_top(true)
    .skip_taskbar(true)
    .focused(false)
    .resizable(false)
    .visible(false)
    .inner_size(320.0, 360.0);

    let window = match builder.build() {
        Ok(w) => w,
        Err(err) => {
            eprintln!("[tray-quota] 建立 overlay 失敗: {err}");
            return;
        }
    };

    // 還原上次位置（若有），並套用多螢幕邊界驗證
    use tauri_plugin_window_state::{StateFlags, WindowExt};
    // 舊版 overlay 曾儲存過過小的尺寸，僅還原位置以免內容再次被裁切。
    if has_saved_window_state(app, QUOTA_OVERLAY_LABEL) {
        let _ = window.restore_state(StateFlags::POSITION);
    } else {
        // 首次啟用時尚無已存位置，預設定位到主螢幕右下角
        position_window_bottom_right(&window);
    }

    // Windows transparent 白底 workaround（tauri#4881）：微調 size 觸發重繪後再顯示
    if let Ok(size) = window.inner_size() {
        let _ = window.set_size(tauri::PhysicalSize::new(
            size.width.saturating_add(1),
            size.height,
        ));
        let _ = window.set_size(size);
    }

    apply_overlay_locked(&window, settings.quota_overlay_locked);
    emit_overlay_settings_changed(app, settings);
    let _ = window.show();
    // Windows 上 show() 可能重置 taskbar 樣式（tauri#10422），顯示後重新套用
    let _ = window.set_skip_taskbar(true);
}

/// 依鎖定狀態設定滑鼠穿透（鎖定 = 穿透）
pub(crate) fn apply_overlay_locked(window: &tauri::WebviewWindow, locked: bool) {
    let _ = window.set_ignore_cursor_events(locked);
}

/// 關閉 overlay 視窗（若存在）
pub(crate) fn close_quota_overlay(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window(QUOTA_OVERLAY_LABEL) {
        // 關閉前先把目前位置寫入 window-state 檔，
        // 確保「隱藏後再開啟」（未重啟 app）也能還原到相同位置
        use tauri_plugin_window_state::{AppHandleExt, StateFlags};
        let _ = app.save_window_state(StateFlags::POSITION);
        let _ = window.close();
    }
}

/// 依 `quota_overlay_enabled` 建立或關閉 overlay，並套用鎖定狀態。
pub(crate) fn sync_overlay_visibility(app: &tauri::AppHandle, settings: &AppSettings) {
    if settings.quota_overlay_enabled {
        create_quota_overlay(app, settings);
    } else {
        close_quota_overlay(app);
    }
}

/// 切換 tray mini panel 的顯示狀態（左鍵點擊 tray 時呼叫）。
/// panel 不存在則建立於系統匣附近；已可見則隱藏；隱藏中則顯示並置前。
pub(crate) fn toggle_tray_panel(app: &tauri::AppHandle, tray_rect: Option<(f64, f64)>) {
    if let Some(window) = app.get_webview_window(TRAY_PANEL_LABEL) {
        let visible = window.is_visible().unwrap_or(false);
        if visible {
            let _ = window.hide();
        } else {
            position_panel_near_tray(app, &window, tray_rect);
            let _ = window.show();
            // Windows 上 show() 可能重置 taskbar 樣式（tauri#10422），顯示後重新套用
            let _ = window.set_skip_taskbar(true);
            let _ = window.set_focus();
        }
        return;
    }

    let builder = tauri::webview::WebviewWindowBuilder::new(
        app,
        TRAY_PANEL_LABEL,
        tauri::WebviewUrl::App("index.html?view=tray-panel".into()),
    )
    .title("SessionHub Quota")
    .transparent(true)
    .decorations(false)
    .shadow(false)
    .always_on_top(true)
    .skip_taskbar(true)
    .resizable(false)
    .inner_size(320.0, 480.0)
    .visible(false);

    let window = match builder.build() {
        Ok(w) => w,
        Err(err) => {
            eprintln!("[tray-quota] 建立 mini panel 失敗: {err}");
            return;
        }
    };

    // 定位到系統匣附近（右下角，taskbar 上方）
    position_panel_near_tray(app, &window, tray_rect);
    let _ = window.show();
    // Windows 上 show() 可能重置 taskbar 樣式（tauri#10422），顯示後重新套用
    let _ = window.set_skip_taskbar(true);
    let _ = window.set_focus();
}

/// 將 panel 定位到系統匣附近；取不到 tray 座標時退回螢幕右下角。
fn position_panel_near_tray(
    app: &tauri::AppHandle,
    window: &tauri::WebviewWindow,
    tray_rect: Option<(f64, f64)>,
) {
    let panel_w = 320.0_f64;
    let panel_h = 480.0_f64;
    let margin = 12.0_f64;

    // 使用 tray 所在螢幕的實體座標與原點，支援副螢幕不在 (0, 0) 的情況。
    let monitor = tray_rect.and_then(|(tray_x, tray_y)| {
        app.available_monitors().ok().and_then(|monitors| {
            monitors.into_iter().find(|monitor| {
                let position = monitor.position();
                let size = monitor.size();
                let right = position.x as f64 + size.width as f64;
                let bottom = position.y as f64 + size.height as f64;
                tray_x >= position.x as f64
                    && tray_x <= right
                    && tray_y >= position.y as f64
                    && tray_y <= bottom
            })
        })
    }).or_else(|| window.current_monitor().ok().flatten());

    let (screen_x, screen_y, screen_w, screen_h, scale) = monitor
        .map(|monitor| {
            let position = monitor.position();
            let size = monitor.size();
            (
                position.x as f64,
                position.y as f64,
                size.width as f64,
                size.height as f64,
                monitor.scale_factor(),
            )
        })
        .unwrap_or((0.0, 0.0, 1920.0, 1080.0, 1.0));

    let panel_w_phys = panel_w * scale;
    let panel_h_phys = panel_h * scale;
    let margin_phys = margin * scale;
    // 預留 taskbar 高度（估 48 邏輯像素）
    let taskbar_phys = 48.0 * scale;

    let (x, y) = match tray_rect {
        Some((tray_x, tray_y)) => {
            let x = (tray_x - panel_w_phys / 2.0)
                .min(screen_x + screen_w - panel_w_phys - margin_phys)
                .max(screen_x + margin_phys);
            let y = (tray_y - panel_h_phys - margin_phys).max(screen_y + margin_phys);
            (x, y)
        }
        None => (
            screen_x + screen_w - panel_w_phys - margin_phys,
            screen_y + screen_h - panel_h_phys - taskbar_phys - margin_phys,
        ),
    };

    let _ = window.set_position(tauri::PhysicalPosition::new(x, y));
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _argv, _cwd| {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.unminimize();
                let _ = window.set_focus();
            }
        }))
        .manage(WatcherState::default())
        .manage(std::sync::Arc::new(ScanCache::default()))
        .manage(DbState::new().expect("failed to init metadata db"))
        .manage(QuotaCache::default())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_window_state::Builder::default().build())
        .setup(|app| {
            let settings = load_settings_internal().unwrap_or(AppSettings::default()?);
            ensure_logs_dir();
            provider::ensure_claude_hook_scripts_installed(Some(
                settings.hook_scripts_path.as_str(),
            ))?;
            provider::ensure_codex_hook_scripts_installed()?;
            provider::ensure_copilot_hook_scripts_installed()?;
            let watcher_state = app.state::<WatcherState>();
            restart_session_watcher_internal(
                app.handle(),
                &watcher_state,
                Some(&settings.copilot_root),
                Some(&settings.opencode_root),
                Some(&settings.codex_root),
                Some(&settings.claude_root),
                &settings.enabled_providers,
            )?;

            // Quota monitoring startup: load from DB then do background refresh
            if settings.enable_quota_monitoring {
                let db_state = app.state::<DbState>();
                let quota_cache = app.state::<QuotaCache>();
                {
                    let conn = db_state.conn.lock().unwrap_or_else(|p| p.into_inner());
                    let _ = quota::cache::load_cache_from_db(
                        &conn,
                        &quota_cache,
                        &settings.quota_enabled_providers,
                    );
                }

                let app_handle = app.handle().clone();
                std::thread::spawn(move || {
                    // Small delay to let the app fully start before first refresh
                    std::thread::sleep(std::time::Duration::from_secs(3));
                    let db_state = app_handle.state::<DbState>();
                    let quota_cache = app_handle.state::<QuotaCache>();
                    let settings = load_settings_internal().unwrap_or_else(|_| AppSettings {
                        enable_quota_monitoring: false,
                        copilot_root: String::new(),
                        opencode_root: String::new(),
                        codex_root: String::new(),
                        claude_root: String::new(),
                        antigravity_root: String::new(),
                        hook_scripts_path: String::new(),
                        claude_quota_reset_day: 1,
                        minimize_to_tray: false,
                        terminal_path: None,
                        external_editor_path: None,
                        show_archived: false,
                        pinned_projects: Vec::new(),
                        enabled_providers: Vec::new(),
                        provider_integrations: Vec::new(),
                        default_launcher: None,
                        enable_intervention_notification: false,
                        enable_session_end_notification: false,
                        show_status_bar: true,
                        analytics_refresh_interval: 30,
                        analytics_panel_collapsed: false,
                        quota_enabled_providers: Vec::new(),
                        allow_create_project_config_dir: false,
                        agents_source_root: String::new(),
                        tray_quota_mode: TrayQuotaMode::default(),
                        tray_quota_primary_provider: None,
                        tray_quota_panel_enabled: true,
                        quota_overlay_enabled: false,
                        quota_overlay_locked: true,
                        quota_overlay_opacity: crate::types::default_quota_overlay_opacity(),
                        quota_overlay_providers: Vec::new(),
                        quota_overlay_theme: OverlayTheme::default(),
                        quota_overlay_style: OverlayStyle::default(),
                    });
                    if settings.enable_quota_monitoring {
                        commands::quota::refresh_quota_internal(
                            &db_state,
                            &quota_cache,
                            &settings,
                            None,
                        );
                        emit_quota_snapshots_updated(&app_handle);
                        crate::tray_icon::update_tray_from_cache(&app_handle, &settings);
                    }
                });
            }

            // Background quota polling thread
            {
                let app_handle = app.handle().clone();
                std::thread::spawn(move || {
                    let mut last_refresh = std::time::Instant::now()
                        .checked_sub(std::time::Duration::from_secs(3600))
                        .unwrap_or(std::time::Instant::now());
                    loop {
                        std::thread::sleep(std::time::Duration::from_secs(60));
                        let settings = match load_settings_internal() {
                            Ok(s) => s,
                            Err(_) => continue,
                        };
                        if !settings.enable_quota_monitoring {
                            continue;
                        }
                        // 固定每 30 分鐘刷新一次，避免與 Claude Code CLI 等工具同時打 API 撞到 429 限流
                        const QUOTA_REFRESH_INTERVAL_SECS: u64 = 30 * 60;
                        if last_refresh.elapsed().as_secs() < QUOTA_REFRESH_INTERVAL_SECS {
                            continue;
                        }
                        last_refresh = std::time::Instant::now();
                        let db_state = app_handle.state::<DbState>();
                        let quota_cache = app_handle.state::<QuotaCache>();
                        commands::quota::refresh_quota_internal(
                            &db_state,
                            &quota_cache,
                            &settings,
                            None,
                        );
                        emit_quota_snapshots_updated(&app_handle);
                        crate::tray_icon::update_tray_from_cache(&app_handle, &settings);
                    }
                });
            }

            // Build system tray icon with menu
            let tray_icon =
                tauri::image::Image::from_bytes(include_bytes!("../icons/32x32.png")).ok();
            let mut tray_builder =
                tauri::tray::TrayIconBuilder::with_id(crate::tray_icon::TRAY_ICON_ID);
            if let Some(icon) = tray_icon {
                tray_builder = tray_builder.icon(icon);
            }
            let show_item = tray_builder.on_tray_icon_event(|tray, event| {
                if let tauri::tray::TrayIconEvent::Click {
                    button: tauri::tray::MouseButton::Left,
                    button_state: tauri::tray::MouseButtonState::Up,
                    rect,
                    ..
                } = event
                {
                    let app = tray.app_handle();
                    let panel_enabled = load_settings_internal()
                        .map(|s| s.tray_quota_panel_enabled)
                        .unwrap_or(true);
                    if panel_enabled {
                        // 點擊系統匣圖示彈出 mini panel（rect.position 為 physical/logical enum）
                        let tray_pos = match rect.position {
                            tauri::Position::Physical(p) => (p.x as f64, p.y as f64),
                            tauri::Position::Logical(p) => (p.x, p.y),
                        };
                        toggle_tray_panel(app, Some(tray_pos));
                    } else if let Some(window) = app.get_webview_window("main") {
                        // panel 停用時回復原本開啟主視窗行為
                        let _ = window.show();
                        let _ = window.set_focus();
                    }
                }
            });

            let show_menu_item = tauri::menu::MenuItemBuilder::new("顯示視窗")
                .id("show_window")
                .build(app)?;
            let overlay_toggle_item = tauri::menu::MenuItemBuilder::new("顯示/隱藏 Quota Overlay")
                .id("toggle_overlay")
                .build(app)?;
            let overlay_lock_item = tauri::menu::MenuItemBuilder::new("編輯 / 鎖定 Overlay 位置")
                .id("toggle_overlay_lock")
                .build(app)?;
            let quit_menu_item = tauri::menu::MenuItemBuilder::new("退出 SessionHub")
                .id("quit")
                .build(app)?;
            let menu = tauri::menu::MenuBuilder::new(app)
                .item(&show_menu_item)
                .separator()
                .item(&overlay_toggle_item)
                .item(&overlay_lock_item)
                .separator()
                .item(&quit_menu_item)
                .build()?;

            show_item
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_menu_event(|app, event| match event.id().as_ref() {
                    "show_window" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                    "toggle_overlay" => {
                        // toggle quota_overlay_enabled 並持久化，再建立/關閉視窗
                        if let Ok(mut settings) = load_settings_internal() {
                            settings.quota_overlay_enabled = !settings.quota_overlay_enabled;
                            let _ = save_settings_internal(&settings);
                            sync_overlay_visibility(app, &settings);
                        }
                    }
                    "toggle_overlay_lock" => {
                        // toggle 鎖定狀態，即時套用滑鼠穿透並通知前端切換編輯視覺
                        if let Ok(mut settings) = load_settings_internal() {
                            settings.quota_overlay_locked = !settings.quota_overlay_locked;
                            let _ = save_settings_internal(&settings);
                            if let Some(window) = app.get_webview_window(QUOTA_OVERLAY_LABEL) {
                                apply_overlay_locked(&window, settings.quota_overlay_locked);
                            }
                            let _ = app.emit(
                                "quota-overlay-locked-changed",
                                settings.quota_overlay_locked,
                            );
                        }
                    }
                    "quit" => {
                        app.exit(0);
                    }
                    _ => {}
                })
                .build(app)?;

            // 啟動時依設定初始化 tray icon 與 overlay
            {
                let app_handle = app.handle().clone();
                let startup_settings = settings.clone();
                crate::tray_icon::update_tray_from_cache(&app_handle, &startup_settings);
                if startup_settings.quota_overlay_enabled {
                    create_quota_overlay(&app_handle, &startup_settings);
                }
            }

            Ok(())
        })
        .on_window_event(|window, event| {
            // mini panel 失焦時自動隱藏（僅 panel，overlay 不受此影響）
            if window.label() == TRAY_PANEL_LABEL {
                if let tauri::WindowEvent::Focused(false) = event {
                    let _ = window.hide();
                }
                return;
            }
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                let minimize = load_settings_internal()
                    .map(|s| s.minimize_to_tray)
                    .unwrap_or(false);
                if minimize {
                    api.prevent_close();
                    let _ = window.hide();
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            get_sessions,
            get_sessions_cached,
            get_settings,
            save_settings,
            install_provider_integration,
            update_provider_integration,
            recheck_provider_integration,
            uninstall_provider_integration,
            detect_terminal,
            detect_vscode,
            restart_session_watcher,
            watch_plan_file,
            stop_plan_watch,
            validate_terminal_path,
            archive_session,
            unarchive_session,
            delete_session,
            delete_empty_sessions,
            get_session_stats,
            get_all_session_stats,
            trigger_stats_backfill,
            read_session_todos,
            get_analytics_data,
            open_terminal,
            check_directory_exists,
            read_plan,
            write_plan,
            open_plan_external,
            upsert_session_meta,
            delete_session_meta,
            get_session_by_cwd,
            get_project_plans,
            get_project_specs,
            read_plan_content,
            get_session_activity_statuses,
            open_in_tool,
            resume_session_in_terminal,
            show_main_window,
            focus_terminal_window,
            read_openspec_file,
            write_openspec_file,
            watch_project_files,
            stop_project_watch,
            check_tool_availability,
            check_jq_available,
            send_intervention_notification,
            get_provider_quota,
            set_provider_quota_settings,
            get_claude_usage_blocks,
            refresh_claude_quota,
            get_quota_snapshots,
            refresh_quota,
            scan_agents_md,
            scan_global_agents_md,
            scan_agents_skills,
            scan_agents_commands,
            sync_agents_items,
            read_agents_file,
            write_agents_file,
            load_project_agents_prefs,
            save_project_agents_prefs,
            check_agents_root_link,
            link_agents_root,
            list_mcp_configs,
            upsert_mcp_server,
            delete_mcp_server,
            set_mcp_server_enabled,
            check_codex_project_trust
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

/// 跨測試模組共用的環境變數鎖：任何會改動行程層級 env（USERPROFILE / APPDATA /
/// COPILOT_SESSION_MANAGER_APPDATA_OVERRIDE）的測試都必須先取得此鎖，
/// 否則平行執行的測試會互相汙染環境而間歇性失敗。
#[cfg(test)]
pub(crate) fn shared_env_test_lock() -> &'static std::sync::Mutex<()> {
    static LOCK: std::sync::OnceLock<std::sync::Mutex<()>> = std::sync::OnceLock::new();
    LOCK.get_or_init(|| std::sync::Mutex::new(()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::openspec_scan::{
        read_openspec_file_internal, scan_openspec_internal, write_openspec_file_internal,
    };
    use crate::provider::bridge::{
        build_opencode_watch_snapshot, derive_activity_status, provider_refresh_event_name,
        resolve_copilot_integration_path, should_emit_opencode_refresh,
        should_emit_provider_refresh_at,
    };
    use crate::provider::codex::{
        detect_codex_integration_status, install_or_update_codex_integration,
        resolve_codex_integration_path,
    };
    use crate::provider::copilot::{
        detect_copilot_integration_status, install_or_update_copilot_integration,
    };
    use crate::provider::opencode::{
        detect_opencode_integration_status, install_or_update_opencode_integration,
    };
    use crate::provider::register_provider_bridge_record;
    use crate::sessions::copilot::{
        delete_empty_sessions_internal, dir_mtime_secs, scan_copilot_incremental_internal,
        scan_session_dir, should_full_scan,
    };
    use crate::sessions::get_sessions_internal;
    use crate::sessions::opencode::{
        scan_opencode_incremental_internal, scan_opencode_sessions_internal,
    };
    use crate::sessions::{configure_msys_stackdump_suppression, merge_msys_options};
    use crate::sisyphus::scan_sisyphus_internal;
    use crate::stats::{
        calculate_opencode_session_stats, get_session_stats_internal, parse_session_stats_internal,
        upsert_session_stats_cache,
    };
    use rusqlite::Connection;
    use std::collections::{BTreeMap, HashMap};
    use std::env;
    use std::ffi::OsString;
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::sync::{Arc, Mutex};
    use std::time::{Duration, Instant};

    fn with_appdata<T>(appdata_dir: &Path, callback: impl FnOnce() -> T) -> T {
        unsafe {
            env::set_var("COPILOT_SESSION_MANAGER_APPDATA_OVERRIDE", appdata_dir);
        }
        let result = callback();
        unsafe {
            env::remove_var("COPILOT_SESSION_MANAGER_APPDATA_OVERRIDE");
        }
        result
    }

    fn with_env_var<T>(key: &str, value: &Path, callback: impl FnOnce() -> T) -> T {
        let previous: Option<OsString> = env::var_os(key);
        unsafe {
            env::set_var(key, value);
        }
        let result = callback();
        unsafe {
            match previous {
                Some(previous) => env::set_var(key, previous),
                None => env::remove_var(key),
            }
        }
        result
    }

    fn without_env_var<T>(key: &str, callback: impl FnOnce() -> T) -> T {
        let previous: Option<OsString> = env::var_os(key);
        unsafe {
            env::remove_var(key);
        }
        let result = callback();
        unsafe {
            if let Some(previous) = previous {
                env::set_var(key, previous);
            }
        }
        result
    }

    fn test_lock() -> &'static Mutex<()> {
        crate::shared_env_test_lock()
    }

    fn unique_test_dir(name: &str) -> PathBuf {
        let suffix = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("time went backwards")
            .as_nanos();
        std::env::temp_dir().join(format!("session-hub-{name}-{suffix}"))
    }

    fn bridge_record_json(
        provider: &str,
        event_type: &str,
        timestamp: &str,
        error: Option<&str>,
    ) -> String {
        serde_json::to_string(&ProviderBridgeRecord {
            version: PROVIDER_INTEGRATION_VERSION,
            provider: provider.to_string(),
            event_type: event_type.to_string(),
            timestamp: timestamp.to_string(),
            session_id: Some("session-001".to_string()),
            cwd: Some("D:\\repo".to_string()),
            source_path: None,
            title: Some("Test".to_string()),
            error: error.map(|value| value.to_string()),
        })
        .expect("serialize bridge record")
    }

    #[test]
    fn provider_refresh_dedup_suppresses_duplicate_refreshes_within_window() {
        let refresh_state = Arc::new(Mutex::new(HashMap::new()));
        let now = Instant::now();

        assert!(
            should_emit_provider_refresh_at(&refresh_state, COPILOT_PROVIDER, now)
                .expect("first refresh should emit")
        );
        assert!(!should_emit_provider_refresh_at(
            &refresh_state,
            COPILOT_PROVIDER,
            now + Duration::from_millis(PROVIDER_REFRESH_DEDUP_MS - 1)
        )
        .expect("duplicate refresh should dedupe"));
        assert!(should_emit_provider_refresh_at(
            &refresh_state,
            COPILOT_PROVIDER,
            now + Duration::from_millis(PROVIDER_REFRESH_DEDUP_MS + 1)
        )
        .expect("refresh after window should emit"));
    }

    #[test]
    fn opencode_refresh_snapshot_skips_unchanged_state_and_emits_after_change() {
        let _guard = test_lock().lock().expect("failed to lock test mutex");
        let oc_dir = unique_test_dir("oc-refresh-snapshot");
        fs::create_dir_all(&oc_dir).expect("create opencode dir");

        let snapshot_state = Arc::new(Mutex::new(build_opencode_watch_snapshot(&oc_dir)));
        assert!(!should_emit_opencode_refresh(&oc_dir, &snapshot_state)
            .expect("unchanged snapshot should not emit"));

        create_opencode_json_sessions(&oc_dir, &[("oc-refresh", "Refresh", 1000, 2000, None)]);

        assert!(should_emit_opencode_refresh(&oc_dir, &snapshot_state)
            .expect("db change should emit once"));
        assert!(!should_emit_opencode_refresh(&oc_dir, &snapshot_state)
            .expect("unchanged snapshot after refresh should not emit"));

        fs::remove_dir_all(&oc_dir).expect("cleanup opencode dir");
    }

    #[test]
    fn opencode_refresh_snapshot_emits_after_db_and_wal_change() {
        let _guard = test_lock().lock().expect("failed to lock test mutex");
        let oc_dir = unique_test_dir("oc-refresh-db-snapshot");
        fs::create_dir_all(&oc_dir).expect("create opencode dir");

        let snapshot_state = Arc::new(Mutex::new(build_opencode_watch_snapshot(&oc_dir)));
        assert!(!should_emit_opencode_refresh(&oc_dir, &snapshot_state)
            .expect("unchanged snapshot should not emit"));

        create_opencode_db_sessions(
            &oc_dir,
            "project-db-snap",
            "D:\\repo\\snapshot",
            &[("ses_db_snap", "Snapshot", 1000, 2000, None)],
        );
        assert!(should_emit_opencode_refresh(&oc_dir, &snapshot_state)
            .expect("db creation should emit"));

        fs::write(oc_dir.join("opencode.db-wal"), "wal-change").expect("write wal file");
        assert!(
            should_emit_opencode_refresh(&oc_dir, &snapshot_state).expect("wal change should emit")
        );
        assert!(!should_emit_opencode_refresh(&oc_dir, &snapshot_state)
            .expect("unchanged snapshot after wal should not emit"));

        fs::remove_dir_all(&oc_dir).expect("cleanup opencode dir");
    }

    #[test]
    fn register_provider_bridge_record_skips_duplicate_last_record() {
        let bridge_records = Arc::new(Mutex::new(HashMap::new()));
        let record = ProviderBridgeRecord {
            version: PROVIDER_INTEGRATION_VERSION,
            provider: OPENCODE_PROVIDER.to_string(),
            event_type: "session.updated".to_string(),
            timestamp: "2026-04-01T09:00:00Z".to_string(),
            session_id: Some("session-001".to_string()),
            cwd: Some("D:\\repo".to_string()),
            source_path: None,
            title: Some("Demo".to_string()),
            error: None,
        };

        assert!(
            register_provider_bridge_record(&bridge_records, OPENCODE_PROVIDER, &record)
                .expect("first record should register")
        );
        assert!(
            !register_provider_bridge_record(&bridge_records, OPENCODE_PROVIDER, &record)
                .expect("duplicate record should skip")
        );
    }

    #[test]
    fn register_provider_bridge_record_treats_error_change_as_distinct() {
        let bridge_records = Arc::new(Mutex::new(HashMap::new()));
        let mut record = ProviderBridgeRecord {
            version: PROVIDER_INTEGRATION_VERSION,
            provider: OPENCODE_PROVIDER.to_string(),
            event_type: "session.updated".to_string(),
            timestamp: "2026-04-01T09:00:00Z".to_string(),
            session_id: Some("session-001".to_string()),
            cwd: Some("D:\\repo".to_string()),
            source_path: None,
            title: Some("Demo".to_string()),
            error: None,
        };

        assert!(
            register_provider_bridge_record(&bridge_records, OPENCODE_PROVIDER, &record)
                .expect("first record should register")
        );

        record.error = Some("refresh failed".to_string());

        assert!(
            register_provider_bridge_record(&bridge_records, OPENCODE_PROVIDER, &record)
                .expect("record with different error should not be deduplicated")
        );
    }

    #[test]
    fn merge_msys_options_adds_error_start_when_missing() {
        assert_eq!(merge_msys_options(None), "error_start:");
        assert_eq!(
            merge_msys_options(Some("winsymlinks:nativestrict")),
            "winsymlinks:nativestrict error_start:"
        );
    }

    #[test]
    fn merge_msys_options_preserves_existing_error_start_case_insensitively() {
        assert_eq!(
            merge_msys_options(Some("winsymlinks:nativestrict ERROR_START:debugger")),
            "winsymlinks:nativestrict ERROR_START:debugger"
        );
        assert_eq!(
            merge_msys_options(Some("error_start=debugger")),
            "error_start=debugger"
        );
    }

    #[test]
    fn merge_msys_options_handles_empty_values_and_is_idempotent() {
        assert_eq!(merge_msys_options(Some("")), "error_start:");

        let configured = merge_msys_options(Some("winsymlinks:nativestrict"));
        assert_eq!(merge_msys_options(Some(&configured)), configured);
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn msys_stackdump_suppression_is_inherited_by_child_process() {
        let _guard = test_lock().lock().expect("lock");
        let previous = env::var_os("MSYS");
        unsafe {
            env::set_var("MSYS", "winsymlinks:nativestrict");
        }

        let mut command = std::process::Command::new("cmd");
        command.args(["/C", "echo %MSYS%"]);
        configure_msys_stackdump_suppression(&mut command);
        let output = command.output().expect("start child process");

        unsafe {
            match previous {
                Some(previous) => env::set_var("MSYS", previous),
                None => env::remove_var("MSYS"),
            }
        }

        assert!(output.status.success());
        assert_eq!(
            String::from_utf8_lossy(&output.stdout).trim(),
            "winsymlinks:nativestrict error_start:"
        );
    }

    #[test]
    fn opencode_permission_records_with_distinct_timestamps_are_not_deduplicated() {
        let bridge_records = Arc::new(Mutex::new(HashMap::new()));
        let mut record = ProviderBridgeRecord {
            version: PROVIDER_INTEGRATION_VERSION,
            provider: OPENCODE_PROVIDER.to_string(),
            event_type: "permission.updated".to_string(),
            timestamp: "2026-04-01T09:00:00.001Z".to_string(),
            session_id: Some("session-001".to_string()),
            cwd: None,
            source_path: None,
            title: Some("Access external directory".to_string()),
            error: None,
        };

        assert!(
            register_provider_bridge_record(&bridge_records, OPENCODE_PROVIDER, &record)
                .expect("first permission record should register")
        );

        record.timestamp = "2026-04-01T09:00:01.001Z".to_string();
        assert!(
            register_provider_bridge_record(&bridge_records, OPENCODE_PROVIDER, &record)
                .expect("new permission record should not be deduplicated")
        );
    }

    #[test]
    fn opencode_v2_permission_records_with_distinct_request_ids_are_not_deduplicated() {
        let bridge_records = Arc::new(Mutex::new(HashMap::new()));
        let mut record = ProviderBridgeRecord {
            version: PROVIDER_INTEGRATION_VERSION,
            provider: OPENCODE_PROVIDER.to_string(),
            event_type: "permission.v2.asked".to_string(),
            timestamp: "2026-07-20T09:00:00.001Z".to_string(),
            session_id: Some("session-001".to_string()),
            cwd: None,
            source_path: None,
            title: Some("bash [permission-001]".to_string()),
            error: None,
        };

        assert!(
            register_provider_bridge_record(&bridge_records, OPENCODE_PROVIDER, &record)
                .expect("first v2 permission record should register")
        );

        record.title = Some("bash [permission-002]".to_string());
        assert!(
            register_provider_bridge_record(&bridge_records, OPENCODE_PROVIDER, &record)
                .expect("new v2 request id should not be deduplicated")
        );
    }

    #[test]
    fn derive_activity_status_maps_opencode_permission_events() {
        assert_eq!(
            derive_activity_status("permission.updated", None),
            ("waiting".to_string(), None)
        );
        assert_eq!(
            derive_activity_status("permission.asked", None),
            ("waiting".to_string(), None)
        );
        assert_eq!(
            derive_activity_status("permission.replied", None),
            ("active".to_string(), None)
        );
        assert_eq!(
            derive_activity_status("permission.v2.asked", None),
            ("waiting".to_string(), None)
        );
        assert_eq!(
            derive_activity_status("permission.v2.replied", None),
            ("active".to_string(), None)
        );
    }

    #[test]
    fn scan_sessions_reads_workspace_yaml_and_plan_flag() {
        let _guard = test_lock().lock().expect("failed to lock test mutex");
        let root_dir = unique_test_dir("scan");
        let appdata_dir = unique_test_dir("appdata");
        let session_dir = root_dir.join("session-state").join("session-001");

        fs::create_dir_all(&session_dir).expect("failed to create session dir");
        fs::write(
            session_dir.join("workspace.yaml"),
            "id: session-001\ncwd: D:\\\\repo\\\\demo\nsummary: Test Session\nsummary_count: 3\ncreated_at: 2025-01-01T00:00:00Z\nupdated_at: 2025-01-02T00:00:00Z\n",
        )
        .expect("failed to write workspace yaml");
        fs::write(session_dir.join("plan.md"), "# Plan").expect("failed to write plan");

        // 在 with_appdata 閉包內開啟 DB，閉包結束後 connection 自動 drop，
        // 確保 SQLite 檔案在 remove_dir_all 前已被釋放
        let sessions = with_appdata(&appdata_dir, || {
            let connection = open_db_connection().expect("open db");
            init_db(&connection).expect("init db");
            scan_session_dir(&root_dir.join("session-state"), false, &connection)
                .expect("scan should succeed")
        });

        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].id, "session-001");
        assert_eq!(sessions[0].summary.as_deref(), Some("Test Session"));
        assert_eq!(sessions[0].summary_count, Some(3));
        assert!(sessions[0].has_plan);
        assert!(!sessions[0].parse_error);

        fs::remove_dir_all(&root_dir).expect("failed to cleanup root dir");
        fs::remove_dir_all(&appdata_dir).expect("failed to cleanup appdata dir");
    }

    #[test]
    fn validate_terminal_path_returns_true_for_existing_file() {
        let test_dir = unique_test_dir("terminal");
        fs::create_dir_all(&test_dir).expect("failed to create terminal test dir");
        let terminal_path = test_dir.join("pwsh.exe");
        fs::write(&terminal_path, "").expect("failed to create fake terminal");

        assert!(validate_terminal_path_internal(
            terminal_path.to_string_lossy().as_ref()
        ));
        assert!(!validate_terminal_path_internal(
            test_dir.join("missing.exe").to_string_lossy().as_ref()
        ));

        fs::remove_dir_all(&test_dir).expect("failed to cleanup terminal test dir");
    }

    #[test]
    fn delete_empty_sessions_returns_zero_when_no_sessions() {
        let _guard = test_lock().lock().expect("failed to lock test mutex");
        let root_dir = unique_test_dir("empty-del-none");
        let appdata_dir = unique_test_dir("appdata");
        fs::create_dir_all(root_dir.join("session-state")).expect("failed to create session-state");
        fs::create_dir_all(&appdata_dir).expect("failed to create appdata dir");

        unsafe {
            env::set_var("COPILOT_SESSION_MANAGER_APPDATA_OVERRIDE", &appdata_dir);
        }
        let conn = open_db_connection().expect("open db");
        init_db(&conn).expect("init db");
        let result = delete_empty_sessions_internal(&root_dir.to_string_lossy(), &conn);
        drop(conn);
        unsafe {
            env::remove_var("COPILOT_SESSION_MANAGER_APPDATA_OVERRIDE");
        }

        assert_eq!(result.expect("should succeed"), 0);

        fs::remove_dir_all(&root_dir).expect("cleanup root");
        fs::remove_dir_all(&appdata_dir).expect("cleanup appdata");
    }

    #[test]
    fn delete_empty_sessions_deletes_sessions_without_events() {
        let _guard = test_lock().lock().expect("failed to lock test mutex");
        let root_dir = unique_test_dir("empty-del-some");
        let appdata_dir = unique_test_dir("appdata");

        // session without events.jsonl (should be deleted)
        let empty_session = root_dir.join("session-state").join("session-empty");
        fs::create_dir_all(&empty_session).expect("create empty session dir");
        fs::write(empty_session.join("workspace.yaml"), "id: session-empty\n")
            .expect("write workspace.yaml");

        // session with non-empty events.jsonl (should be kept)
        let active_session = root_dir.join("session-state").join("session-active");
        fs::create_dir_all(&active_session).expect("create active session dir");
        fs::write(
            active_session.join("workspace.yaml"),
            "id: session-active\n",
        )
        .expect("write workspace.yaml");
        fs::write(
            active_session.join("events.jsonl"),
            "{\"type\":\"session_start\"}\n",
        )
        .expect("write events.jsonl");

        // session with empty events.jsonl (should be deleted)
        let no_count_session = root_dir.join("session-state").join("session-no-count");
        fs::create_dir_all(&no_count_session).expect("create no-count session dir");
        fs::write(
            no_count_session.join("workspace.yaml"),
            "id: session-no-count\n",
        )
        .expect("write workspace.yaml");
        fs::write(no_count_session.join("events.jsonl"), "").expect("write empty events.jsonl");

        unsafe {
            env::set_var("COPILOT_SESSION_MANAGER_APPDATA_OVERRIDE", &appdata_dir);
        }
        let conn = open_db_connection().expect("open db");
        init_db(&conn).expect("init db");
        let result = delete_empty_sessions_internal(&root_dir.to_string_lossy(), &conn);
        drop(conn);
        unsafe {
            env::remove_var("COPILOT_SESSION_MANAGER_APPDATA_OVERRIDE");
        }

        assert_eq!(result.expect("should succeed"), 2);
        assert!(!empty_session.exists(), "empty session should be deleted");
        assert!(active_session.exists(), "active session should remain");
        assert!(
            !no_count_session.exists(),
            "no-count session should be deleted"
        );

        fs::remove_dir_all(&root_dir).expect("cleanup root");
        fs::remove_dir_all(&appdata_dir).expect("cleanup appdata");
    }

    #[test]
    fn delete_empty_sessions_returns_zero_when_no_session_state_dir() {
        let _guard = test_lock().lock().expect("failed to lock test mutex");
        let root_dir = unique_test_dir("empty-del-nodir");
        let appdata_dir = unique_test_dir("appdata");
        fs::create_dir_all(&root_dir).expect("create root dir");
        fs::create_dir_all(&appdata_dir).expect("create appdata dir");

        unsafe {
            env::set_var("COPILOT_SESSION_MANAGER_APPDATA_OVERRIDE", &appdata_dir);
        }
        let conn = open_db_connection().expect("open db");
        init_db(&conn).expect("init db");
        let result = delete_empty_sessions_internal(&root_dir.to_string_lossy(), &conn);
        drop(conn);
        unsafe {
            env::remove_var("COPILOT_SESSION_MANAGER_APPDATA_OVERRIDE");
        }

        assert_eq!(result.expect("should succeed"), 0);

        fs::remove_dir_all(&root_dir).expect("cleanup root");
        fs::remove_dir_all(&appdata_dir).expect("cleanup appdata");
    }

    #[test]
    fn parse_stats_empty_dir_returns_zero_values() {
        let _guard = test_lock().lock().expect("failed to lock test mutex");
        let session_dir = unique_test_dir("stats-empty");
        fs::create_dir_all(&session_dir).expect("create session dir");

        let stats = parse_session_stats_internal(&session_dir).expect("stats should parse");

        assert_eq!(stats, SessionStats::default());

        fs::remove_dir_all(&session_dir).expect("cleanup session dir");
    }

    #[test]
    fn parse_stats_basic_reads_top_level_counts() {
        let _guard = test_lock().lock().expect("failed to lock test mutex");
        let root_dir = unique_test_dir("stats-basic");
        let appdata_dir = unique_test_dir("appdata");
        let session_dir = root_dir.join("session-state").join("session-basic");
        fs::create_dir_all(&session_dir).expect("create session dir");
        fs::create_dir_all(&appdata_dir).expect("create appdata dir");
        fs::write(
            session_dir.join("events.jsonl"),
            concat!(
                "{\"type\":\"session.start\",\"data\":{\"startTime\":\"2026-03-31T10:00:00Z\",\"selectedModel\":\"gpt-4.1\"},\"timestamp\":\"2026-03-31T10:00:00Z\"}\n",
                "{\"type\":\"user.message\",\"data\":{},\"timestamp\":\"2026-03-31T10:01:00Z\"}\n",
                "{\"type\":\"tool.execution_start\",\"data\":{\"toolName\":\"grep\"},\"timestamp\":\"2026-03-31T10:02:00Z\"}\n",
                "{\"type\":\"assistant.message\",\"data\":{\"outputTokens\":120,\"reasoningOpaque\":\"opaque\"},\"timestamp\":\"2026-03-31T10:05:00Z\"}\n",
                "{\"type\":\"session.model_change\",\"data\":{\"newModel\":\"gpt-5.4\"},\"timestamp\":\"2026-03-31T10:06:00Z\"}\n"
            ),
        )
        .expect("write events");

        let stats = with_appdata(&appdata_dir, || {
            let connection = open_db_connection().expect("open db");
            init_db(&connection).expect("init db");
            get_session_stats_internal(&connection, &session_dir.to_string_lossy())
                .expect("stats should parse")
        });

        assert_eq!(stats.output_tokens, 120);
        assert_eq!(stats.interaction_count, 1);
        assert_eq!(stats.tool_call_count, 1);
        assert_eq!(stats.duration_minutes, 6);
        assert_eq!(stats.reasoning_count, 1);
        assert_eq!(
            stats.models_used,
            vec!["gpt-4.1".to_string(), "gpt-5.4".to_string()]
        );
        assert_eq!(stats.tool_breakdown.get("grep"), Some(&1));

        fs::remove_dir_all(&root_dir).expect("cleanup root");
        fs::remove_dir_all(&appdata_dir).expect("cleanup appdata");
    }

    #[test]
    fn parse_stats_skips_subagent_events() {
        let _guard = test_lock().lock().expect("failed to lock test mutex");
        let root_dir = unique_test_dir("stats-subagent");
        let appdata_dir = unique_test_dir("appdata");
        let session_dir = root_dir.join("session-state").join("session-subagent");
        fs::create_dir_all(&session_dir).expect("create session dir");
        fs::create_dir_all(&appdata_dir).expect("create appdata dir");
        fs::write(
            session_dir.join("events.jsonl"),
            concat!(
                "{\"type\":\"session.start\",\"data\":{\"startTime\":\"2026-03-31T10:00:00Z\"},\"timestamp\":\"2026-03-31T10:00:00Z\"}\n",
                "{\"type\":\"user.message\",\"data\":{},\"timestamp\":\"2026-03-31T10:01:00Z\"}\n",
                "{\"type\":\"tool.execution_start\",\"data\":{\"toolName\":\"grep\",\"parentToolCallId\":\"call-1\"},\"timestamp\":\"2026-03-31T10:02:00Z\"}\n",
                "{\"type\":\"assistant.message\",\"data\":{\"parentToolCallId\":\"call-1\",\"outputTokens\":200,\"reasoningOpaque\":\"opaque\"},\"timestamp\":\"2026-03-31T10:03:00Z\"}\n"
            ),
        )
        .expect("write events");

        let stats = with_appdata(&appdata_dir, || {
            let connection = open_db_connection().expect("open db");
            init_db(&connection).expect("init db");
            get_session_stats_internal(&connection, &session_dir.to_string_lossy())
                .expect("stats should parse")
        });

        assert_eq!(stats.interaction_count, 1);
        assert_eq!(stats.tool_call_count, 0);
        assert_eq!(stats.output_tokens, 0);
        assert_eq!(stats.reasoning_count, 0);

        fs::remove_dir_all(&root_dir).expect("cleanup root");
        fs::remove_dir_all(&appdata_dir).expect("cleanup appdata");
    }

    // ──────────────────────────────────────────────────────────────────────────
    // should_full_scan
    // ──────────────────────────────────────────────────────────────────────────

    #[test]
    fn should_full_scan_returns_true_when_cache_is_none() {
        // 快取為 None（首次啟動）→ 必須執行全掃
        assert!(should_full_scan(&None, false));
    }

    #[test]
    fn should_full_scan_returns_true_when_force_full_is_set() {
        // force_full = true，無論快取狀態都必須全掃
        let cache = Some(ProviderCache {
            sessions: Vec::new(),
            session_mtimes: HashMap::new(),
            last_full_scan_at: Instant::now(),
            last_cursor: 0,
        });
        assert!(should_full_scan(&cache, true));
    }

    #[test]
    fn should_full_scan_returns_false_when_cache_is_fresh() {
        // 快取剛建立（elapsed ≈ 0），不需全掃
        let cache = Some(ProviderCache {
            sessions: Vec::new(),
            session_mtimes: HashMap::new(),
            last_full_scan_at: Instant::now(),
            last_cursor: 0,
        });
        assert!(!should_full_scan(&cache, false));
    }

    // ──────────────────────────────────────────────────────────────────────────
    // dir_mtime_secs
    // ──────────────────────────────────────────────────────────────────────────

    #[test]
    fn dir_mtime_secs_returns_zero_for_missing_path() {
        let missing = std::env::temp_dir().join("session-hub-nonexistent-dir-xyz");
        assert_eq!(dir_mtime_secs(&missing), 0);
    }

    #[test]
    fn dir_mtime_secs_returns_positive_for_existing_dir() {
        let dir = unique_test_dir("mtime");
        fs::create_dir_all(&dir).expect("create dir");

        let mtime = dir_mtime_secs(&dir);
        assert!(mtime > 0, "mtime should be a positive unix timestamp");

        fs::remove_dir_all(&dir).expect("cleanup");
    }

    // ──────────────────────────────────────────────────────────────────────────
    // scan_copilot_incremental_internal
    // ──────────────────────────────────────────────────────────────────────────

    /// 建立最小測試用 ProviderCache（空快取）
    fn empty_copilot_cache() -> ProviderCache {
        ProviderCache {
            sessions: Vec::new(),
            session_mtimes: HashMap::new(),
            last_full_scan_at: Instant::now(),
            last_cursor: 0,
        }
    }

    #[test]
    fn incremental_copilot_picks_up_new_session() {
        // 對一個空快取執行增量掃描，應偵測到新建的 session 目錄
        let _guard = test_lock().lock().expect("lock");
        let root_dir = unique_test_dir("inc-new");
        let appdata_dir = unique_test_dir("appdata");
        let session_dir = root_dir.join("session-state").join("session-inc-001");
        fs::create_dir_all(&session_dir).expect("create session dir");
        fs::write(
            session_dir.join("workspace.yaml"),
            "id: session-inc-001\ncwd: D:\\repo\\demo\nsummary: Inc Test\nsummary_count: 1\ncreated_at: 2025-01-01T00:00:00Z\nupdated_at: 2025-01-02T00:00:00Z\n",
        )
        .expect("write workspace.yaml");

        let sessions = with_appdata(&appdata_dir, || {
            let conn = open_db_connection().expect("open db");
            init_db(&conn).expect("init db");
            let mut cache = empty_copilot_cache();
            scan_copilot_incremental_internal(
                &root_dir.join("session-state"),
                false,
                &conn,
                &mut cache,
            )
            .expect("incremental scan");
            cache.sessions
        });

        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].id, "session-inc-001");
        assert_eq!(sessions[0].summary.as_deref(), Some("Inc Test"));
        assert!(!sessions[0].is_archived);

        fs::remove_dir_all(&root_dir).expect("cleanup root");
        fs::remove_dir_all(&appdata_dir).expect("cleanup appdata");
    }

    #[test]
    fn incremental_copilot_skips_unchanged_session() {
        // mtime 未變化的 session 不應重新解析（快取命中）
        let _guard = test_lock().lock().expect("lock");
        let root_dir = unique_test_dir("inc-skip");
        let appdata_dir = unique_test_dir("appdata");
        let session_dir = root_dir.join("session-state").join("session-skip-001");
        fs::create_dir_all(&session_dir).expect("create session dir");
        fs::write(
            session_dir.join("workspace.yaml"),
            "id: session-skip-001\ncwd: D:\\repo\nsummary: Original\nsummary_count: 1\ncreated_at: 2025-01-01T00:00:00Z\nupdated_at: 2025-01-01T00:00:00Z\n",
        )
        .expect("write workspace.yaml");

        with_appdata(&appdata_dir, || {
            let conn = open_db_connection().expect("open db");
            init_db(&conn).expect("init db");

            // 第一次掃描：讀入快取並記錄 mtime
            let mut cache = empty_copilot_cache();
            scan_copilot_incremental_internal(
                &root_dir.join("session-state"),
                false,
                &conn,
                &mut cache,
            )
            .expect("first scan");
            assert_eq!(cache.sessions.len(), 1);

            // 手動竄改快取中的 summary，模擬「已有舊資料」
            cache.sessions[0].summary = Some("Cached Value".to_string());

            // 第二次掃描：mtime 未變，不應覆蓋快取內容
            scan_copilot_incremental_internal(
                &root_dir.join("session-state"),
                false,
                &conn,
                &mut cache,
            )
            .expect("second scan");

            // 快取命中，summary 仍是手動設定的值
            assert_eq!(cache.sessions[0].summary.as_deref(), Some("Cached Value"));
        });

        fs::remove_dir_all(&root_dir).expect("cleanup root");
        fs::remove_dir_all(&appdata_dir).expect("cleanup appdata");
    }

    #[test]
    fn incremental_copilot_removes_deleted_session() {
        // 目錄被刪除後，再次增量掃描應從快取中移除對應 session
        let _guard = test_lock().lock().expect("lock");
        let root_dir = unique_test_dir("inc-del");
        let appdata_dir = unique_test_dir("appdata");
        let session_state = root_dir.join("session-state");
        let session_dir = session_state.join("session-del-001");
        fs::create_dir_all(&session_dir).expect("create session dir");
        fs::write(
            session_dir.join("workspace.yaml"),
            "id: session-del-001\ncwd: D:\\repo\nsummary: To Delete\nsummary_count: 1\ncreated_at: 2025-01-01T00:00:00Z\nupdated_at: 2025-01-01T00:00:00Z\n",
        )
        .expect("write workspace.yaml");

        with_appdata(&appdata_dir, || {
            let conn = open_db_connection().expect("open db");
            init_db(&conn).expect("init db");

            let mut cache = empty_copilot_cache();

            // 第一次掃描：快取 1 個 session
            scan_copilot_incremental_internal(&session_state, false, &conn, &mut cache)
                .expect("first scan");
            assert_eq!(cache.sessions.len(), 1);

            // 刪除目錄
            fs::remove_dir_all(&session_dir).expect("remove session dir");

            // 第二次掃描：session 應從快取消失
            scan_copilot_incremental_internal(&session_state, false, &conn, &mut cache)
                .expect("second scan");
            assert!(
                cache.sessions.is_empty(),
                "deleted session should be removed from cache"
            );
            assert!(
                cache.session_mtimes.is_empty(),
                "mtime entry should be removed"
            );
        });

        fs::remove_dir_all(&root_dir).expect("cleanup root");
        fs::remove_dir_all(&appdata_dir).expect("cleanup appdata");
    }

    #[test]
    fn incremental_copilot_clears_cache_when_dir_missing() {
        // session-state 目錄本身不存在時，對應 bucket 的快取應被清空
        let _guard = test_lock().lock().expect("lock");
        let appdata_dir = unique_test_dir("appdata");
        let missing_dir = unique_test_dir("inc-missing").join("session-state");

        // 預先塞入一筆假資料到快取
        let mut cache = empty_copilot_cache();
        cache.sessions.push(SessionInfo {
            id: "ghost-session".to_string(),
            provider: "copilot".to_string(),
            cwd: None,
            repo_root: None,
            repo_name: None,
            git_branch: None,
            summary: None,
            summary_count: None,
            created_at: None,
            updated_at: None,
            session_dir: String::new(),
            parse_error: false,
            is_archived: false,
            notes: None,
            tags: Vec::new(),
            has_plan: false,
            has_events: false,
        });

        with_appdata(&appdata_dir, || {
            let conn = open_db_connection().expect("open db");
            init_db(&conn).expect("init db");

            scan_copilot_incremental_internal(&missing_dir, false, &conn, &mut cache)
                .expect("scan on missing dir");

            assert!(
                cache.sessions.is_empty(),
                "cache should be cleared when dir is missing"
            );
        });

        fs::remove_dir_all(&appdata_dir).expect("cleanup appdata");
    }

    #[test]
    fn incremental_copilot_preserves_other_bucket_on_dir_missing() {
        // session-state 消失時，只清除 is_archived=false 的 bucket，
        // is_archived=true 的 session 應保留
        let _guard = test_lock().lock().expect("lock");
        let appdata_dir = unique_test_dir("appdata");
        let missing_dir = unique_test_dir("inc-bucket").join("session-state");

        let mut cache = empty_copilot_cache();
        // 塞入 active session（is_archived=false）
        cache.sessions.push(SessionInfo {
            id: "active-session".to_string(),
            provider: "copilot".to_string(),
            cwd: None,
            repo_root: None,
            repo_name: None,
            git_branch: None,
            summary: None,
            summary_count: None,
            created_at: None,
            updated_at: None,
            session_dir: String::new(),
            parse_error: false,
            is_archived: false,
            notes: None,
            tags: Vec::new(),
            has_plan: false,
            has_events: false,
        });
        // 塞入 archived session（is_archived=true）
        cache.sessions.push(SessionInfo {
            id: "archived-session".to_string(),
            provider: "copilot".to_string(),
            cwd: None,
            repo_root: None,
            repo_name: None,
            git_branch: None,
            summary: None,
            summary_count: None,
            created_at: None,
            updated_at: None,
            session_dir: String::new(),
            parse_error: false,
            is_archived: true,
            notes: None,
            tags: Vec::new(),
            has_plan: false,
            has_events: false,
        });

        with_appdata(&appdata_dir, || {
            let conn = open_db_connection().expect("open db");
            init_db(&conn).expect("init db");

            // 掃描 is_archived=false 的目錄（不存在）→ 只清除 active bucket
            scan_copilot_incremental_internal(&missing_dir, false, &conn, &mut cache)
                .expect("scan");

            assert_eq!(
                cache.sessions.len(),
                1,
                "only archived session should remain"
            );
            assert_eq!(cache.sessions[0].id, "archived-session");
        });

        fs::remove_dir_all(&appdata_dir).expect("cleanup appdata");
    }

    // ──────────────────────────────────────────────────────────────────────────
    // scan_opencode_incremental_internal
    // ──────────────────────────────────────────────────────────────────────────

    /// 建立 JSON-based OpenCode session storage（符合新版 JSON 儲存格式）
    /// sessions: (id, title, time_created, time_updated, time_archived)
    fn create_opencode_json_sessions(dir: &Path, sessions: &[(&str, &str, i64, i64, Option<i64>)]) {
        let project_id = "project-001";
        let project_dir = dir.join("storage").join("project");
        fs::create_dir_all(&project_dir).expect("create storage/project dir");
        fs::write(
            project_dir.join(format!("{project_id}.json")),
            format!(r#"{{"id":"{project_id}","worktree":null}}"#),
        )
        .expect("write project json");

        let session_dir = dir.join("storage").join("session").join(project_id);
        fs::create_dir_all(&session_dir).expect("create storage/session dir");

        for (id, title, time_created, time_updated, time_archived) in sessions {
            let archived_json = match time_archived {
                Some(ts) => ts.to_string(),
                None => "null".to_string(),
            };
            let json = format!(
                r#"{{"id":"{id}","title":"{title}","directory":null,"time":{{"created":{time_created},"updated":{time_updated},"archived":{archived_json}}}}}"#
            );
            fs::write(session_dir.join(format!("{id}.json")), json).expect("write session json");
        }
    }

    fn create_opencode_db_sessions(
        dir: &Path,
        project_id: &str,
        worktree: &str,
        sessions: &[(&str, &str, i64, i64, Option<i64>)],
    ) {
        fs::create_dir_all(dir).expect("create opencode root dir");
        let db_path = dir.join("opencode.db");
        let connection = Connection::open(&db_path).expect("open opencode db");
        connection
            .execute_batch(
                "
                CREATE TABLE project (
                  id TEXT PRIMARY KEY,
                  worktree TEXT NOT NULL,
                  name TEXT,
                  time_created INTEGER NOT NULL,
                  time_updated INTEGER NOT NULL,
                  sandboxes TEXT NOT NULL
                );
                CREATE TABLE session (
                  id TEXT PRIMARY KEY,
                  project_id TEXT NOT NULL,
                  parent_id TEXT,
                  slug TEXT NOT NULL,
                  directory TEXT NOT NULL,
                  title TEXT NOT NULL,
                  version TEXT NOT NULL,
                  share_url TEXT,
                  summary_additions INTEGER,
                  summary_deletions INTEGER,
                  summary_files INTEGER,
                  summary_diffs TEXT,
                  revert TEXT,
                  permission TEXT,
                  time_created INTEGER NOT NULL,
                  time_updated INTEGER NOT NULL,
                  time_compacting INTEGER,
                  time_archived INTEGER,
                  workspace_id TEXT,
                  path TEXT,
                  agent TEXT,
                  model TEXT,
                  cost REAL DEFAULT 0 NOT NULL,
                  tokens_input INTEGER DEFAULT 0 NOT NULL,
                  tokens_output INTEGER DEFAULT 0 NOT NULL,
                  tokens_reasoning INTEGER DEFAULT 0 NOT NULL,
                  tokens_cache_read INTEGER DEFAULT 0 NOT NULL,
                  tokens_cache_write INTEGER DEFAULT 0 NOT NULL
                );
                CREATE TABLE message (
                  id TEXT PRIMARY KEY,
                  session_id TEXT NOT NULL,
                  time_created INTEGER NOT NULL,
                  time_updated INTEGER NOT NULL,
                  data TEXT NOT NULL
                );
                CREATE TABLE part (
                  id TEXT PRIMARY KEY,
                  message_id TEXT NOT NULL,
                  session_id TEXT NOT NULL,
                  time_created INTEGER NOT NULL,
                  time_updated INTEGER NOT NULL,
                  data TEXT NOT NULL
                );
                ",
            )
            .expect("create opencode schema");
        connection
            .execute(
                "INSERT INTO project (id, worktree, name, time_created, time_updated, sandboxes) VALUES (?1, ?2, NULL, 1, 1, '[]')",
                rusqlite::params![project_id, worktree],
            )
            .expect("insert project");

        for (id, title, time_created, time_updated, time_archived) in sessions {
            connection
                .execute(
                    "
                    INSERT INTO session (
                      id, project_id, parent_id, slug, directory, title, version,
                      share_url, summary_additions, summary_deletions, summary_files, summary_diffs,
                      revert, permission, time_created, time_updated, time_compacting, time_archived,
                      workspace_id, path, agent, model, cost, tokens_input, tokens_output,
                      tokens_reasoning, tokens_cache_read, tokens_cache_write
                    ) VALUES (
                      ?1, ?2, NULL, 'slug', ?3, ?4, '1.1.60',
                      NULL, 2, 3, 4, NULL,
                      NULL, NULL, ?5, ?6, NULL, ?7,
                      NULL, NULL, NULL, NULL, 0, 0, 0,
                      0, 0, 0
                    )",
                    rusqlite::params![id, project_id, worktree, title, time_created, time_updated, time_archived],
                )
                .expect("insert session");
        }
    }

    fn insert_opencode_db_message(
        dir: &Path,
        session_id: &str,
        message_id: &str,
        time_created: i64,
        time_updated: i64,
        data: &str,
    ) {
        let connection = Connection::open(dir.join("opencode.db")).expect("open opencode db");
        connection
            .execute(
                "INSERT INTO message (id, session_id, time_created, time_updated, data) VALUES (?1, ?2, ?3, ?4, ?5)",
                rusqlite::params![message_id, session_id, time_created, time_updated, data],
            )
            .expect("insert message");
    }

    fn insert_opencode_db_part(
        dir: &Path,
        session_id: &str,
        message_id: &str,
        part_id: &str,
        time_created: i64,
        time_updated: i64,
        data: &str,
    ) {
        let connection = Connection::open(dir.join("opencode.db")).expect("open opencode db");
        connection
            .execute(
                "INSERT INTO part (id, message_id, session_id, time_created, time_updated, data) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                rusqlite::params![part_id, message_id, session_id, time_created, time_updated, data],
            )
            .expect("insert part");
    }

    #[test]
    fn scan_opencode_sessions_reads_json_files_and_maps_metadata() {
        let _guard = test_lock().lock().expect("lock");
        let oc_dir = unique_test_dir("oc-full-scan");
        fs::create_dir_all(&oc_dir).expect("create oc dir");

        // 建立 project JSON
        let project_dir = oc_dir.join("storage").join("project");
        fs::create_dir_all(&project_dir).expect("create project dir");
        fs::write(
            project_dir.join("project-001.json"),
            r#"{"id":"project-001","worktree":"D:\\repo\\demo"}"#,
        )
        .expect("write project json");

        // 建立 session JSON（summary_count: 12+3+5=20）
        let session_dir = oc_dir.join("storage").join("session").join("project-001");
        fs::create_dir_all(&session_dir).expect("create session dir");
        fs::write(
            session_dir.join("oc-session-001.json"),
            r#"{"id":"oc-session-001","title":"OpenCode Title","directory":null,"time":{"created":1710000000000,"updated":1710000300000,"archived":null},"summary":{"additions":12,"deletions":3,"files":5}}"#,
        ).expect("write session json");

        let metadata_conn = Connection::open_in_memory().expect("open metadata db");
        init_db(&metadata_conn).expect("init metadata db");
        upsert_session_meta_internal(
            &metadata_conn,
            "oc-session-001",
            Some("同步備註".to_string()),
            vec!["research".to_string(), "multi-platform".to_string()],
        )
        .expect("insert metadata");

        let sessions =
            scan_opencode_sessions_internal(&oc_dir, false, &metadata_conn).expect("scan sessions");

        assert_eq!(sessions.len(), 1);
        let session = &sessions[0];
        assert_eq!(session.id, "oc-session-001");
        assert_eq!(session.provider, OPENCODE_PROVIDER);
        assert_eq!(session.cwd.as_deref(), Some("D:\\repo\\demo"));
        assert_eq!(session.summary.as_deref(), Some("OpenCode Title"));
        assert_eq!(session.summary_count, Some(20));
        assert_eq!(session.created_at.as_deref(), Some("2024-03-09T16:00:00Z"));
        assert_eq!(session.updated_at.as_deref(), Some("2024-03-09T16:05:00Z"));
        assert_eq!(
            session.session_dir,
            oc_dir
                .join("storage")
                .join("message")
                .join("oc-session-001")
                .to_string_lossy()
                .to_string()
        );
        assert_eq!(session.notes.as_deref(), Some("同步備註"));
        assert_eq!(
            session.tags,
            vec!["research".to_string(), "multi-platform".to_string()]
        );
        assert!(!session.is_archived);
        assert!(!session.parse_error);

        fs::remove_dir_all(&oc_dir).expect("cleanup oc dir");
    }

    #[test]
    fn scan_opencode_sessions_returns_empty_when_storage_missing() {
        let _guard = test_lock().lock().expect("lock");
        let oc_dir = unique_test_dir("oc-missing-storage");
        fs::create_dir_all(&oc_dir).expect("create oc dir");
        // 故意不建立 storage/project/ 或 storage/session/

        let metadata_conn = Connection::open_in_memory().expect("open metadata db");
        init_db(&metadata_conn).expect("init metadata db");

        let sessions =
            scan_opencode_sessions_internal(&oc_dir, false, &metadata_conn).expect("scan sessions");

        assert!(sessions.is_empty(), "missing storage dir should be ignored");

        fs::remove_dir_all(&oc_dir).expect("cleanup oc dir");
    }

    #[test]
    fn scan_opencode_sessions_reads_db_rows_and_maps_metadata() {
        let _guard = test_lock().lock().expect("lock");
        let oc_dir = unique_test_dir("oc-db-full-scan");
        create_opencode_db_sessions(
            &oc_dir,
            "project-db-001",
            "D:\\repo\\db-demo",
            &[(
                "ses_db_001",
                "DB Session",
                1710000000000,
                1710000300000,
                None,
            )],
        );
        insert_opencode_db_message(
            &oc_dir,
            "ses_db_001",
            "msg_db_001",
            1710000000000,
            1710000300000,
            r#"{"role":"assistant","time":{"created":1710000000000,"completed":1710000300000},"modelID":"gpt-5","tokens":{"input":10,"output":20,"reasoning":2,"cache":{"write":0,"read":0}}}"#,
        );

        let metadata_conn = Connection::open_in_memory().expect("open metadata db");
        init_db(&metadata_conn).expect("init metadata db");
        upsert_session_meta_internal(
            &metadata_conn,
            "ses_db_001",
            Some("DB 備註".to_string()),
            vec!["db".to_string()],
        )
        .expect("insert metadata");

        let sessions = scan_opencode_sessions_internal(&oc_dir, false, &metadata_conn)
            .expect("scan db sessions");

        assert_eq!(sessions.len(), 1);
        let session = &sessions[0];
        assert_eq!(session.id, "ses_db_001");
        assert_eq!(session.provider, OPENCODE_PROVIDER);
        assert_eq!(session.cwd.as_deref(), Some("D:\\repo\\db-demo"));
        assert_eq!(session.summary.as_deref(), Some("DB Session"));
        assert_eq!(session.summary_count, Some(9));
        assert_eq!(session.created_at.as_deref(), Some("2024-03-09T16:00:00Z"));
        assert_eq!(session.updated_at.as_deref(), Some("2024-03-09T16:05:00Z"));
        assert_eq!(session.notes.as_deref(), Some("DB 備註"));
        assert_eq!(session.tags, vec!["db".to_string()]);
        assert!(session.has_events);

        fs::remove_dir_all(&oc_dir).expect("cleanup oc dir");
    }

    #[test]
    fn get_sessions_filters_by_enabled_providers() {
        let _guard = test_lock().lock().expect("lock");
        let appdata_dir = unique_test_dir("providers-appdata");
        let copilot_root = unique_test_dir("providers-copilot");
        let opencode_root = unique_test_dir("providers-opencode");
        let copilot_session_dir = copilot_root.join("session-state").join("cp-session-001");

        fs::create_dir_all(&copilot_session_dir).expect("create copilot session dir");
        fs::create_dir_all(&opencode_root).expect("create opencode dir");
        fs::write(
            copilot_session_dir.join("workspace.yaml"),
            concat!(
                "id: cp-session-001\n",
                "cwd: D:\\repo\\copilot\n",
                "summary: Copilot Session\n",
                "updated_at: 2025-01-02T00:00:00Z\n"
            ),
        )
        .expect("write workspace yaml");

        create_opencode_json_sessions(
            &opencode_root,
            &[(
                "oc-session-001",
                "OpenCode Session",
                1_735_689_600_000,
                1_735_693_200_000,
                None,
            )],
        );

        let scan_cache = ScanCache::default();

        with_appdata(&appdata_dir, || {
            let conn = open_db_connection().expect("open db");
            init_db(&conn).expect("init db");
            let copilot_only = get_sessions_internal(
                Some(copilot_root.to_string_lossy().to_string()),
                Some(opencode_root.to_string_lossy().to_string()),
                Some(String::new()),
                Some(String::new()),
                Some(String::new()),
                Some(false),
                Some(vec![COPILOT_PROVIDER.to_string()]),
                Some(true),
                &scan_cache,
                &conn,
            )
            .expect("scan copilot only");
            assert_eq!(copilot_only.len(), 1);
            assert_eq!(copilot_only[0].provider, COPILOT_PROVIDER);

            let opencode_only = get_sessions_internal(
                Some(copilot_root.to_string_lossy().to_string()),
                Some(opencode_root.to_string_lossy().to_string()),
                Some(String::new()),
                Some(String::new()),
                Some(String::new()),
                Some(false),
                Some(vec![OPENCODE_PROVIDER.to_string()]),
                Some(true),
                &scan_cache,
                &conn,
            )
            .expect("scan opencode only");
            assert_eq!(opencode_only.len(), 1);
            assert_eq!(opencode_only[0].provider, OPENCODE_PROVIDER);

            let all_providers = get_sessions_internal(
                Some(copilot_root.to_string_lossy().to_string()),
                Some(opencode_root.to_string_lossy().to_string()),
                Some(String::new()),
                Some(String::new()),
                Some(String::new()),
                Some(false),
                Some(vec![
                    COPILOT_PROVIDER.to_string(),
                    OPENCODE_PROVIDER.to_string(),
                ]),
                Some(true),
                &scan_cache,
                &conn,
            )
            .expect("scan all providers");
            assert_eq!(all_providers.len(), 2);
            assert!(all_providers
                .iter()
                .any(|session| session.provider == COPILOT_PROVIDER));
            assert!(all_providers
                .iter()
                .any(|session| session.provider == OPENCODE_PROVIDER));
        });

        fs::remove_dir_all(&appdata_dir).expect("cleanup appdata");
        fs::remove_dir_all(&copilot_root).expect("cleanup copilot root");
        fs::remove_dir_all(&opencode_root).expect("cleanup opencode root");
    }

    #[test]
    fn scan_sisyphus_reads_project_metadata() {
        let _guard = test_lock().lock().expect("lock");
        let project_dir = unique_test_dir("sisyphus-project");
        let sisyphus_dir = project_dir.join(".sisyphus");
        fs::create_dir_all(sisyphus_dir.join("plans")).expect("create plans dir");
        fs::create_dir_all(sisyphus_dir.join("notepads").join("alpha"))
            .expect("create alpha notepad");
        fs::create_dir_all(sisyphus_dir.join("notepads").join("beta"))
            .expect("create beta notepad");
        fs::create_dir_all(sisyphus_dir.join("evidence")).expect("create evidence dir");
        fs::create_dir_all(sisyphus_dir.join("drafts")).expect("create drafts dir");

        fs::write(
            sisyphus_dir.join("boulder.json"),
            r#"{
                "activePlan": "plans/alpha.md",
                "planName": "Alpha Plan",
                "agent": "copilot",
                "sessionIds": ["session-001", "session-002"],
                "startedAt": "2026-04-01T09:00:00Z"
            }"#,
        )
        .expect("write boulder.json");
        fs::write(
            sisyphus_dir.join("plans").join("alpha.md"),
            "# Alpha Title\n\n## TL;DR\n第一行摘要\n第二行摘要\n\n## Details\n內容\n",
        )
        .expect("write alpha plan");
        fs::write(
            sisyphus_dir.join("plans").join("beta.md"),
            "# Beta Title\n\n一般內容\n",
        )
        .expect("write beta plan");
        fs::write(
            sisyphus_dir
                .join("notepads")
                .join("alpha")
                .join("issues.md"),
            "- issue",
        )
        .expect("write alpha issues");
        fs::write(
            sisyphus_dir
                .join("notepads")
                .join("alpha")
                .join("learnings.md"),
            "- learning",
        )
        .expect("write alpha learnings");
        fs::write(
            sisyphus_dir.join("notepads").join("beta").join("issues.md"),
            "- beta issue",
        )
        .expect("write beta issues");
        fs::write(sisyphus_dir.join("evidence").join("b.txt"), "b").expect("write evidence b");
        fs::write(sisyphus_dir.join("evidence").join("a.txt"), "a").expect("write evidence a");
        fs::write(sisyphus_dir.join("drafts").join("draft-b.md"), "# Draft B")
            .expect("write draft b");
        fs::write(sisyphus_dir.join("drafts").join("draft-a.md"), "# Draft A")
            .expect("write draft a");

        let data = scan_sisyphus_internal(&project_dir);

        assert_eq!(
            data.active_plan
                .as_ref()
                .and_then(|plan| plan.plan_name.as_deref()),
            Some("Alpha Plan")
        );
        assert_eq!(
            data.active_plan
                .as_ref()
                .and_then(|plan| plan.agent.as_deref()),
            Some("copilot")
        );
        assert_eq!(
            data.active_plan
                .as_ref()
                .map(|plan| plan.session_ids.clone())
                .unwrap_or_default(),
            vec!["session-001".to_string(), "session-002".to_string()]
        );
        assert_eq!(data.plans.len(), 2);
        assert_eq!(data.plans[0].name, "alpha");
        assert_eq!(data.plans[0].title.as_deref(), Some("Alpha Title"));
        assert_eq!(
            data.plans[0].tldr.as_deref(),
            Some("第一行摘要\n第二行摘要")
        );
        assert!(data.plans[0].is_active);
        assert_eq!(data.plans[1].name, "beta");
        assert!(!data.plans[1].is_active);
        assert_eq!(data.notepads.len(), 2);
        assert_eq!(data.notepads[0].name, "alpha");
        assert!(data.notepads[0].has_issues);
        assert!(data.notepads[0].has_learnings);
        assert_eq!(data.notepads[1].name, "beta");
        assert!(data.notepads[1].has_issues);
        assert!(!data.notepads[1].has_learnings);
        assert_eq!(
            data.evidence_files,
            vec!["a.txt".to_string(), "b.txt".to_string()]
        );
        assert_eq!(
            data.draft_files,
            vec!["draft-a.md".to_string(), "draft-b.md".to_string()]
        );

        fs::remove_dir_all(&project_dir).expect("cleanup project dir");
    }

    #[test]
    fn scan_openspec_reads_project_metadata() {
        let _guard = test_lock().lock().expect("lock");
        let project_dir = unique_test_dir("openspec-project");
        let openspec_dir = project_dir.join("openspec");
        fs::create_dir_all(
            openspec_dir
                .join("changes")
                .join("feature-b")
                .join("specs")
                .join("auth"),
        )
        .expect("create feature-b specs dir");
        fs::create_dir_all(
            openspec_dir
                .join("changes")
                .join("archive")
                .join("legacy-a"),
        )
        .expect("create archive dir");
        fs::create_dir_all(openspec_dir.join("specs").join("api")).expect("create api spec dir");
        fs::create_dir_all(openspec_dir.join("specs").join("workflow"))
            .expect("create workflow spec dir");

        fs::write(openspec_dir.join("config.yaml"), "schema: v2\n").expect("write config");
        fs::write(
            openspec_dir
                .join("changes")
                .join("feature-b")
                .join(".openspec.yaml"),
            "created: 2026-07-02\n",
        )
        .expect("write change yaml");
        fs::write(
            openspec_dir
                .join("changes")
                .join("feature-b")
                .join("proposal.md"),
            "# Proposal",
        )
        .expect("write proposal");
        fs::write(
            openspec_dir
                .join("changes")
                .join("feature-b")
                .join("tasks.md"),
            "- [ ] task",
        )
        .expect("write tasks");
        fs::write(
            openspec_dir
                .join("changes")
                .join("feature-b")
                .join("specs")
                .join("auth")
                .join("spec.md"),
            "# Auth Spec",
        )
        .expect("write change spec");
        fs::write(
            openspec_dir
                .join("changes")
                .join("archive")
                .join("legacy-a")
                .join("design.md"),
            "# Design",
        )
        .expect("write archive design");
        fs::write(
            openspec_dir.join("specs").join("api").join("spec.md"),
            "# API Spec",
        )
        .expect("write api spec");

        let data = scan_openspec_internal(&project_dir);

        assert_eq!(data.schema.as_deref(), Some("v2"));
        assert_eq!(data.active_changes.len(), 1);
        assert_eq!(data.active_changes[0].name, "feature-b");
        assert!(data.active_changes[0].has_proposal);
        assert!(!data.active_changes[0].has_design);
        assert!(data.active_changes[0].has_tasks);
        assert_eq!(
            data.active_changes[0].created_at.as_deref(),
            Some("2026-07-02")
        );
        assert_eq!(
            data.active_changes[0]
                .task_progress
                .as_ref()
                .map(|p| p.done),
            Some(0)
        );
        assert_eq!(
            data.active_changes[0]
                .task_progress
                .as_ref()
                .map(|p| p.total),
            Some(1)
        );
        assert_eq!(data.active_changes[0].specs_count, 1);
        assert_eq!(data.active_changes[0].specs.len(), 1);
        assert_eq!(data.active_changes[0].specs[0].name, "auth");
        assert_eq!(
            data.active_changes[0].specs[0].path,
            openspec_dir
                .join("changes")
                .join("feature-b")
                .join("specs")
                .join("auth")
                .join("spec.md")
                .to_string_lossy()
                .to_string()
        );
        assert_eq!(data.archived_changes.len(), 1);
        assert_eq!(data.archived_changes[0].name, "legacy-a");
        assert!(!data.archived_changes[0].has_proposal);
        assert!(data.archived_changes[0].has_design);
        assert!(!data.archived_changes[0].has_tasks);
        assert!(data.archived_changes[0].task_progress.is_none());
        assert!(data.archived_changes[0].specs.is_empty());
        assert_eq!(data.specs.len(), 2);
        assert_eq!(data.specs[0].name, "api");
        assert_eq!(
            data.specs[0].path,
            openspec_dir
                .join("specs")
                .join("api")
                .join("spec.md")
                .to_string_lossy()
                .to_string()
        );
        assert_eq!(data.specs[1].name, "workflow");
        assert_eq!(
            data.specs[1].path,
            openspec_dir
                .join("specs")
                .join("workflow")
                .to_string_lossy()
                .to_string()
        );

        fs::remove_dir_all(&project_dir).expect("cleanup project dir");
    }

    #[test]
    fn scan_openspec_falls_back_to_filesystem_created_time_when_yaml_missing() {
        let _guard = test_lock().lock().expect("lock");
        let project_dir = unique_test_dir("openspec-project-fallback-created-at");
        let change_dir = project_dir
            .join("openspec")
            .join("changes")
            .join("feature-fallback");

        fs::create_dir_all(&change_dir).expect("create change dir");

        let data = scan_openspec_internal(&project_dir);

        assert_eq!(data.active_changes.len(), 1);
        assert_eq!(data.active_changes[0].name, "feature-fallback");
        assert!(data.active_changes[0].created_at.is_some());

        fs::remove_dir_all(&project_dir).expect("cleanup project dir");
    }

    #[test]
    fn scan_project_metadata_returns_empty_structures_when_dirs_missing() {
        let _guard = test_lock().lock().expect("lock");
        let project_dir = unique_test_dir("project-empty-metadata");
        fs::create_dir_all(&project_dir).expect("create project dir");

        let sisyphus_data = scan_sisyphus_internal(&project_dir);
        assert!(sisyphus_data.active_plan.is_none());
        assert!(sisyphus_data.plans.is_empty());
        assert!(sisyphus_data.notepads.is_empty());
        assert!(sisyphus_data.evidence_files.is_empty());
        assert!(sisyphus_data.draft_files.is_empty());

        let openspec_data = scan_openspec_internal(&project_dir);
        assert!(openspec_data.schema.is_none());
        assert!(openspec_data.active_changes.is_empty());
        assert!(openspec_data.archived_changes.is_empty());
        assert!(openspec_data.specs.is_empty());

        fs::remove_dir_all(&project_dir).expect("cleanup project dir");
    }

    #[test]
    fn openspec_file_roundtrip_enforces_scope() {
        let _guard = test_lock().lock().expect("lock");
        let project_dir = unique_test_dir("openspec-file-roundtrip");
        let openspec_dir = project_dir.join("openspec");
        let tasks_path = openspec_dir
            .join("changes")
            .join("feature-a")
            .join("tasks.md");
        fs::create_dir_all(tasks_path.parent().expect("tasks parent"))
            .expect("create tasks parent");
        fs::write(&tasks_path, "- [ ] first task\n").expect("write tasks");

        let original = read_openspec_file_internal(
            &project_dir.to_string_lossy(),
            "changes/feature-a/tasks.md",
        )
        .expect("read openspec file");
        assert_eq!(original, "- [ ] first task\n");

        write_openspec_file_internal(
            &project_dir.to_string_lossy(),
            "changes/feature-a/tasks.md",
            "- [x] first task\n",
        )
        .expect("write openspec file");

        let updated = read_openspec_file_internal(
            &project_dir.to_string_lossy(),
            "changes/feature-a/tasks.md",
        )
        .expect("read updated openspec file");
        assert_eq!(updated, "- [x] first task\n");

        let traversal_error =
            write_openspec_file_internal(&project_dir.to_string_lossy(), "../README.md", "denied")
                .expect_err("path traversal should fail");
        assert!(
            traversal_error.contains("File not found") || traversal_error.contains("Access denied")
        );

        fs::remove_dir_all(&project_dir).expect("cleanup project dir");
    }

    #[test]
    fn incremental_opencode_picks_up_new_session() {
        // cursor = 0 時應掃到所有 session
        let _guard = test_lock().lock().expect("lock");
        let oc_dir = unique_test_dir("oc-new");
        let appdata_dir = unique_test_dir("appdata");
        fs::create_dir_all(&oc_dir).expect("create oc dir");

        create_opencode_json_sessions(&oc_dir, &[("oc-session-001", "OC Title", 1000, 2000, None)]);

        with_appdata(&appdata_dir, || {
            let metadata_conn = open_db_connection().expect("open metadata db");
            init_db(&metadata_conn).expect("init db");

            let mut cache = empty_copilot_cache(); // cursor = 0
            scan_opencode_incremental_internal(&oc_dir, false, &metadata_conn, &mut cache)
                .expect("incremental scan");

            assert_eq!(cache.sessions.len(), 1);
            assert_eq!(cache.sessions[0].id, "oc-session-001");
            assert_eq!(cache.sessions[0].provider, "opencode");
            assert_eq!(cache.sessions[0].summary.as_deref(), Some("OC Title"));
            assert_eq!(
                cache.last_cursor, 2000,
                "cursor should advance to max time_updated"
            );
        });

        fs::remove_dir_all(&oc_dir).expect("cleanup oc dir");
        fs::remove_dir_all(&appdata_dir).expect("cleanup appdata");
    }

    #[test]
    fn incremental_opencode_cursor_advances_after_scan() {
        // 掃描後 cursor 應更新為最大 time_updated
        let _guard = test_lock().lock().expect("lock");
        let oc_dir = unique_test_dir("oc-cursor");
        let appdata_dir = unique_test_dir("appdata");
        fs::create_dir_all(&oc_dir).expect("create oc dir");

        create_opencode_json_sessions(
            &oc_dir,
            &[
                ("oc-a", "A", 1000, 3000, None),
                ("oc-b", "B", 1000, 5000, None),
                ("oc-c", "C", 1000, 4000, None),
            ],
        );

        with_appdata(&appdata_dir, || {
            let metadata_conn = open_db_connection().expect("open metadata db");
            init_db(&metadata_conn).expect("init db");

            let mut cache = empty_copilot_cache();
            scan_opencode_incremental_internal(&oc_dir, false, &metadata_conn, &mut cache)
                .expect("scan");

            assert_eq!(cache.sessions.len(), 3);
            assert_eq!(cache.last_cursor, 5000, "cursor should be max time_updated");
        });

        fs::remove_dir_all(&oc_dir).expect("cleanup oc dir");
        fs::remove_dir_all(&appdata_dir).expect("cleanup appdata");
    }

    #[test]
    fn incremental_opencode_skips_sessions_before_cursor() {
        // cursor 設為 3000，time_updated <= 3000 的 session 不應被撈到
        let _guard = test_lock().lock().expect("lock");
        let oc_dir = unique_test_dir("oc-skip");
        let appdata_dir = unique_test_dir("appdata");
        fs::create_dir_all(&oc_dir).expect("create oc dir");

        create_opencode_json_sessions(
            &oc_dir,
            &[
                ("oc-old", "Old", 1000, 2000, None),
                ("oc-new", "New", 1000, 5000, None),
            ],
        );

        with_appdata(&appdata_dir, || {
            let metadata_conn = open_db_connection().expect("open metadata db");
            init_db(&metadata_conn).expect("init db");

            let mut cache = ProviderCache {
                sessions: Vec::new(),
                session_mtimes: HashMap::new(),
                last_full_scan_at: Instant::now(),
                last_cursor: 3000, // 只掃 time_updated > 3000
            };
            scan_opencode_incremental_internal(&oc_dir, false, &metadata_conn, &mut cache)
                .expect("scan");

            assert_eq!(
                cache.sessions.len(),
                1,
                "only new session should be picked up"
            );
            assert_eq!(cache.sessions[0].id, "oc-new");
            assert_eq!(cache.last_cursor, 5000);
        });

        fs::remove_dir_all(&oc_dir).expect("cleanup oc dir");
        fs::remove_dir_all(&appdata_dir).expect("cleanup appdata");
    }

    #[test]
    fn incremental_opencode_upserts_existing_session() {
        // cursor 推進後，若同一 session time_updated 再次超過 cursor 應 upsert 而非 duplicate
        let _guard = test_lock().lock().expect("lock");
        let oc_dir = unique_test_dir("oc-upsert");
        let appdata_dir = unique_test_dir("appdata");
        fs::create_dir_all(&oc_dir).expect("create oc dir");

        create_opencode_json_sessions(&oc_dir, &[("oc-x", "Title v1", 1000, 2000, None)]);

        with_appdata(&appdata_dir, || {
            let metadata_conn = open_db_connection().expect("open metadata db");
            init_db(&metadata_conn).expect("init db");

            let mut cache = empty_copilot_cache();

            // 第一次掃描
            scan_opencode_incremental_internal(&oc_dir, false, &metadata_conn, &mut cache)
                .expect("first scan");
            assert_eq!(cache.sessions.len(), 1);
            assert_eq!(cache.last_cursor, 2000);

            // 手動更新 JSON 檔案模擬 session 被修改（time_updated 推進）
            let session_json_path = oc_dir
                .join("storage")
                .join("session")
                .join("project-001")
                .join("oc-x.json");
            fs::write(
                &session_json_path,
                r#"{"id":"oc-x","title":"Title v2","directory":null,"time":{"created":1000,"updated":4000,"archived":null}}"#,
            ).expect("update session json");

            // 第二次增量掃描：只撈 time_updated > 2000
            scan_opencode_incremental_internal(&oc_dir, false, &metadata_conn, &mut cache)
                .expect("second scan");

            assert_eq!(cache.sessions.len(), 1, "should upsert, not duplicate");
            assert_eq!(
                cache.sessions[0].summary.as_deref(),
                Some("Title v2"),
                "summary should be updated"
            );
            assert_eq!(cache.last_cursor, 4000);
        });

        fs::remove_dir_all(&oc_dir).expect("cleanup oc dir");
        fs::remove_dir_all(&appdata_dir).expect("cleanup appdata");
    }

    #[test]
    fn incremental_opencode_excludes_archived_when_show_archived_false() {
        // show_archived=false 時，已封存的 session 不應出現在結果中
        let _guard = test_lock().lock().expect("lock");
        let oc_dir = unique_test_dir("oc-arch");
        let appdata_dir = unique_test_dir("appdata");
        fs::create_dir_all(&oc_dir).expect("create oc dir");

        create_opencode_json_sessions(
            &oc_dir,
            &[
                ("oc-active", "Active", 1000, 2000, None),
                ("oc-archived", "Archived", 1000, 3000, Some(9000)),
            ],
        );

        with_appdata(&appdata_dir, || {
            let metadata_conn = open_db_connection().expect("open metadata db");
            init_db(&metadata_conn).expect("init db");

            let mut cache = empty_copilot_cache();
            scan_opencode_incremental_internal(&oc_dir, false, &metadata_conn, &mut cache)
                .expect("scan");

            assert_eq!(cache.sessions.len(), 1);
            assert_eq!(cache.sessions[0].id, "oc-active");
        });

        fs::remove_dir_all(&oc_dir).expect("cleanup oc dir");
        fs::remove_dir_all(&appdata_dir).expect("cleanup appdata");
    }

    #[test]
    fn incremental_opencode_includes_archived_when_show_archived_true() {
        // show_archived=true 時，封存 session 也應出現
        let _guard = test_lock().lock().expect("lock");
        let oc_dir = unique_test_dir("oc-arch-all");
        let appdata_dir = unique_test_dir("appdata");
        fs::create_dir_all(&oc_dir).expect("create oc dir");

        create_opencode_json_sessions(
            &oc_dir,
            &[
                ("oc-active", "Active", 1000, 2000, None),
                ("oc-archived", "Archived", 1000, 3000, Some(9000)),
            ],
        );

        with_appdata(&appdata_dir, || {
            let metadata_conn = open_db_connection().expect("open metadata db");
            init_db(&metadata_conn).expect("init db");

            let mut cache = empty_copilot_cache();
            scan_opencode_incremental_internal(&oc_dir, true, &metadata_conn, &mut cache)
                .expect("scan");

            assert_eq!(cache.sessions.len(), 2);
        });

        fs::remove_dir_all(&oc_dir).expect("cleanup oc dir");
        fs::remove_dir_all(&appdata_dir).expect("cleanup appdata");
    }

    #[test]
    fn incremental_opencode_noop_when_storage_missing() {
        // storage/session/ 不存在時應靜默回傳 Ok，不修改快取
        let _guard = test_lock().lock().expect("lock");
        let oc_dir = unique_test_dir("oc-no-storage");
        let appdata_dir = unique_test_dir("appdata");
        fs::create_dir_all(&oc_dir).expect("create oc dir");
        // 故意不建立 storage/ 目錄

        with_appdata(&appdata_dir, || {
            let metadata_conn = open_db_connection().expect("open metadata db");
            init_db(&metadata_conn).expect("init db");

            let mut cache = empty_copilot_cache();
            let result =
                scan_opencode_incremental_internal(&oc_dir, false, &metadata_conn, &mut cache);

            assert!(result.is_ok(), "should not error when storage is missing");
            assert!(cache.sessions.is_empty(), "cache should remain empty");
        });

        fs::remove_dir_all(&oc_dir).expect("cleanup oc dir");
        fs::remove_dir_all(&appdata_dir).expect("cleanup appdata");
    }

    #[test]
    fn incremental_opencode_cursor_unchanged_when_no_new_rows() {
        // 沒有新 row 時 cursor 不應改變
        let _guard = test_lock().lock().expect("lock");
        let oc_dir = unique_test_dir("oc-no-new");
        let appdata_dir = unique_test_dir("appdata");
        fs::create_dir_all(&oc_dir).expect("create oc dir");

        create_opencode_json_sessions(&oc_dir, &[("oc-z", "Z", 1000, 2000, None)]);

        with_appdata(&appdata_dir, || {
            let metadata_conn = open_db_connection().expect("open metadata db");
            init_db(&metadata_conn).expect("init db");

            let mut cache = ProviderCache {
                sessions: Vec::new(),
                session_mtimes: HashMap::new(),
                last_full_scan_at: Instant::now(),
                last_cursor: 9999, // cursor 已超過所有 time_updated
            };
            scan_opencode_incremental_internal(&oc_dir, false, &metadata_conn, &mut cache)
                .expect("scan");

            assert!(cache.sessions.is_empty(), "no new sessions");
            assert_eq!(cache.last_cursor, 9999, "cursor should not change");
        });

        fs::remove_dir_all(&oc_dir).expect("cleanup oc dir");
        fs::remove_dir_all(&appdata_dir).expect("cleanup appdata");
    }

    #[test]
    fn incremental_opencode_picks_up_db_only_session() {
        let _guard = test_lock().lock().expect("lock");
        let oc_dir = unique_test_dir("oc-db-incremental");
        let appdata_dir = unique_test_dir("appdata");
        create_opencode_db_sessions(
            &oc_dir,
            "project-db-002",
            "D:\\repo\\db-only",
            &[("ses_db_new", "DB New", 1000, 5000, None)],
        );

        with_appdata(&appdata_dir, || {
            let metadata_conn = open_db_connection().expect("open metadata db");
            init_db(&metadata_conn).expect("init db");

            let mut cache = empty_copilot_cache();
            scan_opencode_incremental_internal(&oc_dir, false, &metadata_conn, &mut cache)
                .expect("incremental db scan");

            assert_eq!(cache.sessions.len(), 1);
            assert_eq!(cache.sessions[0].id, "ses_db_new");
            assert_eq!(cache.last_cursor, 5000);
        });

        fs::remove_dir_all(&oc_dir).expect("cleanup oc dir");
        fs::remove_dir_all(&appdata_dir).expect("cleanup appdata");
    }

    #[test]
    fn provider_bridge_paths_use_appdata_override() {
        let _guard = test_lock().lock().expect("lock");
        let appdata_dir = unique_test_dir("provider-bridge-appdata");
        fs::create_dir_all(&appdata_dir).expect("create appdata dir");

        with_appdata(&appdata_dir, || {
            let copilot_path =
                resolve_provider_bridge_path(COPILOT_PROVIDER).expect("resolve copilot bridge");
            let opencode_path =
                resolve_provider_bridge_path(OPENCODE_PROVIDER).expect("resolve opencode bridge");
            let codex_path =
                resolve_provider_bridge_path(CODEX_PROVIDER).expect("resolve codex bridge");

            assert_eq!(
                copilot_path,
                appdata_dir
                    .join("SessionHub")
                    .join("provider-bridge")
                    .join("copilot.jsonl")
            );
            assert_eq!(
                opencode_path,
                appdata_dir
                    .join("SessionHub")
                    .join("provider-bridge")
                    .join("opencode.jsonl")
            );
            assert_eq!(
                codex_path,
                appdata_dir
                    .join("SessionHub")
                    .join("provider-bridge")
                    .join("codex.jsonl")
            );
        });

        fs::remove_dir_all(&appdata_dir).expect("cleanup appdata");
    }

    #[test]
    fn detect_copilot_integration_status_reads_installed_state_and_last_error() {
        let _guard = test_lock().lock().expect("lock");
        let appdata_dir = unique_test_dir("copilot-provider-appdata");
        let copilot_root = unique_test_dir("copilot-provider-root");
        let config_path = resolve_copilot_integration_path(&copilot_root);

        fs::create_dir_all(config_path.parent().expect("config parent")).expect("create hooks dir");
        fs::create_dir_all(&appdata_dir).expect("create appdata dir");

        let status = with_appdata(&appdata_dir, || {
            let bridge_path =
                resolve_provider_bridge_path(COPILOT_PROVIDER).expect("resolve copilot bridge");
            ensure_parent_dir(&bridge_path).expect("create bridge dir");
            fs::write(
                &bridge_path,
                format!(
                    "{}\n",
                    bridge_record_json(
                        COPILOT_PROVIDER,
                        "session.error",
                        "2026-04-01T12:00:00Z",
                        Some("hook failed")
                    )
                ),
            )
            .expect("write bridge record");

            let integration = serde_json::json!({
                "version": 1,
                "sessionHub": {
                    "provider": COPILOT_PROVIDER,
                    "bridgePath": bridge_path.to_string_lossy().to_string(),
                    "integrationVersion": PROVIDER_INTEGRATION_VERSION
                },
                "hooks": {
                    "sessionStart": [{ "type": "command", "command": "node on-session-start.cjs" }],
                    "sessionEnd": [{ "type": "command", "command": "node on-session-end.cjs" }],
                    "userPromptSubmitted": [{ "type": "command", "command": "node on-user-prompt-submitted.cjs" }],
                    "preToolUse": [{ "type": "command", "command": "node on-pre-tool-use.cjs" }],
                    "postToolUse": [{ "type": "command", "command": "node on-post-tool-use.cjs" }],
                    "errorOccurred": [{ "type": "command", "command": "node on-error-occurred.cjs" }]
                }
            });
            fs::write(
                &config_path,
                serde_json::to_string_pretty(&integration).expect("serialize integration"),
            )
            .expect("write copilot integration");

            let root_string = copilot_root.to_string_lossy().to_string();
            detect_copilot_integration_status(Some(root_string.as_str()))
        });

        assert_eq!(status.status, ProviderIntegrationState::Installed);
        assert_eq!(
            status.last_event_at.as_deref(),
            Some("2026-04-01T12:00:00Z")
        );
        assert_eq!(status.last_error.as_deref(), Some("hook failed"));
        assert_eq!(
            status.config_path.as_deref(),
            Some(config_path.to_string_lossy().as_ref())
        );
        assert_eq!(status.installed_version, Some(PROVIDER_INTEGRATION_VERSION));

        fs::remove_dir_all(&copilot_root).expect("cleanup copilot root");
        fs::remove_dir_all(&appdata_dir).expect("cleanup appdata");
    }

    #[test]
    fn detect_copilot_integration_status_marks_outdated_when_version_mismatches() {
        let _guard = test_lock().lock().expect("lock");
        let appdata_dir = unique_test_dir("copilot-provider-outdated-appdata");
        let copilot_root = unique_test_dir("copilot-provider-outdated-root");
        let config_path = resolve_copilot_integration_path(&copilot_root);

        fs::create_dir_all(config_path.parent().expect("config parent")).expect("create hooks dir");
        fs::create_dir_all(&appdata_dir).expect("create appdata dir");

        let status = with_appdata(&appdata_dir, || {
            let bridge_path =
                resolve_provider_bridge_path(COPILOT_PROVIDER).expect("resolve copilot bridge");
            let integration = serde_json::json!({
                "version": 1,
                "sessionHub": {
                    "provider": COPILOT_PROVIDER,
                    "bridgePath": bridge_path.to_string_lossy().to_string(),
                    "integrationVersion": PROVIDER_INTEGRATION_VERSION - 1
                },
                "hooks": {
                    "sessionStart": [{ "type": "command", "command": "node on-session-start.cjs" }],
                    "sessionEnd": [{ "type": "command", "command": "node on-session-end.cjs" }],
                    "userPromptSubmitted": [{ "type": "command", "command": "node on-user-prompt-submitted.cjs" }],
                    "preToolUse": [{ "type": "command", "command": "node on-pre-tool-use.cjs" }],
                    "postToolUse": [{ "type": "command", "command": "node on-post-tool-use.cjs" }],
                    "errorOccurred": [{ "type": "command", "command": "node on-error-occurred.cjs" }]
                }
            });
            fs::write(
                &config_path,
                serde_json::to_string_pretty(&integration).expect("serialize integration"),
            )
            .expect("write copilot integration");

            let root_string = copilot_root.to_string_lossy().to_string();
            detect_copilot_integration_status(Some(root_string.as_str()))
        });

        assert_eq!(status.status, ProviderIntegrationState::Outdated);
        assert!(status
            .last_error
            .as_deref()
            .is_some_and(|error| error.contains("outdated")));
        assert_eq!(
            status.installed_version,
            Some(PROVIDER_INTEGRATION_VERSION - 1)
        );

        fs::remove_dir_all(&copilot_root).expect("cleanup copilot root");
        fs::remove_dir_all(&appdata_dir).expect("cleanup appdata");
    }

    #[test]
    fn install_copilot_integration_writes_managed_hook_file() {
        let _guard = test_lock().lock().expect("lock");
        let appdata_dir = unique_test_dir("copilot-provider-install-appdata");
        let copilot_root = unique_test_dir("copilot-provider-install-root");

        fs::create_dir_all(&appdata_dir).expect("create appdata dir");
        fs::create_dir_all(&copilot_root).expect("create copilot root");

        let status = with_appdata(&appdata_dir, || {
            let root_string = copilot_root.to_string_lossy().to_string();
            install_or_update_copilot_integration(Some(root_string.as_str()))
        });
        let config_path = resolve_copilot_integration_path(&copilot_root);
        let content = fs::read_to_string(&config_path).expect("read copilot hook file");

        assert_eq!(status.status, ProviderIntegrationState::Installed);
        assert!(content.contains("\"sessionHub\""));
        assert!(content.contains("\"sessionEnd\""));
        assert!(content.contains("\"sessionStart\""));
        assert!(content.contains("\"userPromptSubmitted\""));
        assert!(content.contains("\"preToolUse\""));
        assert!(content.contains("\"postToolUse\""));
        assert!(content.contains("\"errorOccurred\""));
        assert!(content.contains("node "));
        assert!(content.contains("on-session-start.cjs"));
        assert!(content.contains("\"command\""));
        assert!(!content.contains("\"powershell\""));

        fs::remove_dir_all(&copilot_root).expect("cleanup copilot root");
        fs::remove_dir_all(&appdata_dir).expect("cleanup appdata");
    }

    #[test]
    fn detect_opencode_integration_status_reads_installed_state_from_plugin_metadata() {
        let _guard = test_lock().lock().expect("lock");
        let appdata_dir = unique_test_dir("opencode-provider-appdata");
        let user_profile = unique_test_dir("opencode-provider-user");

        fs::create_dir_all(&appdata_dir).expect("create appdata dir");
        fs::create_dir_all(&user_profile).expect("create user profile dir");

        let status = with_appdata(&appdata_dir, || {
            with_env_var("USERPROFILE", &user_profile, || {
                let config_path =
                    resolve_opencode_integration_path().expect("resolve opencode integration path");
                fs::create_dir_all(config_path.parent().expect("plugin parent"))
                    .expect("create plugin dir");

                let bridge_path =
                    resolve_provider_bridge_path(OPENCODE_PROVIDER).expect("resolve bridge path");
                ensure_parent_dir(&bridge_path).expect("create bridge dir");
                fs::write(
                    &bridge_path,
                    format!(
                        "{}\n",
                        bridge_record_json(
                            OPENCODE_PROVIDER,
                            "session.updated",
                            "2026-04-02T08:30:00Z",
                            None
                        )
                    ),
                )
                .expect("write bridge record");

                let metadata = serde_json::json!({
                    "provider": OPENCODE_PROVIDER,
                    "bridgePath": bridge_path.to_string_lossy().to_string(),
                    "integrationVersion": PROVIDER_INTEGRATION_VERSION
                });
                fs::write(
                    &config_path,
                    format!(
                        "{OPENCODE_PLUGIN_METADATA_PREFIX}{}\nexport const SessionHubBridge = () => ({{}});\n",
                        serde_json::to_string(&metadata).expect("serialize metadata")
                    ),
                )
                .expect("write plugin file");

                detect_opencode_integration_status()
            })
        });

        assert_eq!(status.status, ProviderIntegrationState::Installed);
        assert_eq!(
            status.last_event_at.as_deref(),
            Some("2026-04-02T08:30:00Z")
        );
        assert!(status.last_error.is_none());
        assert!(status
            .config_path
            .as_deref()
            .is_some_and(|path| path.ends_with(OPENCODE_PLUGIN_FILE_NAME)));

        fs::remove_dir_all(&user_profile).expect("cleanup user profile");
        fs::remove_dir_all(&appdata_dir).expect("cleanup appdata");
    }

    #[test]
    fn detect_opencode_integration_status_marks_missing_and_manual_required() {
        let _guard = test_lock().lock().expect("lock");
        let appdata_dir = unique_test_dir("opencode-provider-missing-appdata");
        let user_profile = unique_test_dir("opencode-provider-missing-user");

        fs::create_dir_all(&appdata_dir).expect("create appdata dir");
        fs::create_dir_all(&user_profile).expect("create user profile dir");

        let missing_status = with_appdata(&appdata_dir, || {
            with_env_var("USERPROFILE", &user_profile, || {
                let config_path =
                    resolve_opencode_integration_path().expect("resolve opencode integration path");
                fs::create_dir_all(config_path.parent().expect("plugin parent"))
                    .expect("create plugin dir");
                detect_opencode_integration_status()
            })
        });

        assert_eq!(missing_status.status, ProviderIntegrationState::Missing);
        assert!(missing_status.last_error.is_none());
        assert!(missing_status
            .config_path
            .as_deref()
            .is_some_and(|path| path.ends_with(OPENCODE_PLUGIN_FILE_NAME)));

        let manual_required_status = with_appdata(&appdata_dir, || {
            without_env_var("USERPROFILE", detect_opencode_integration_status)
        });

        assert_eq!(
            manual_required_status.status,
            ProviderIntegrationState::ManualRequired
        );
        assert!(manual_required_status.config_path.is_none());
        assert!(manual_required_status
            .last_error
            .as_deref()
            .is_some_and(|error| error.contains("USERPROFILE")));

        fs::remove_dir_all(&user_profile).expect("cleanup user profile");
        fs::remove_dir_all(&appdata_dir).expect("cleanup appdata");
    }

    #[test]
    fn install_opencode_integration_writes_managed_plugin_file() {
        let _guard = test_lock().lock().expect("lock");
        let appdata_dir = unique_test_dir("opencode-provider-install-appdata");
        let user_profile = unique_test_dir("opencode-provider-install-user");

        fs::create_dir_all(&appdata_dir).expect("create appdata dir");
        fs::create_dir_all(&user_profile).expect("create user profile dir");

        let status = with_appdata(&appdata_dir, || {
            with_env_var(
                "USERPROFILE",
                &user_profile,
                install_or_update_opencode_integration,
            )
        });
        let config_path = with_env_var("USERPROFILE", &user_profile, || {
            resolve_opencode_integration_path().expect("resolve OpenCode plugin path")
        });
        let content = fs::read_to_string(&config_path).expect("read OpenCode plugin file");

        assert_eq!(status.status, ProviderIntegrationState::Installed);
        assert!(content.contains(OPENCODE_PLUGIN_METADATA_PREFIX));
        assert!(content.contains("\"session.updated\""));
        assert!(content.contains("\"permission.updated\""));
        assert!(content.contains("\"permission.asked\""));
        assert!(content.contains("\"permission.replied\""));
        assert!(content.contains("\"permission.v2.asked\""));
        assert!(content.contains("\"permission.v2.replied\""));
        assert!(content.contains(
            "props.title ?? props.permission ?? props.type ?? props.action ?? \"permission\""
        ));
        assert!(content.contains("props.id ?? props.requestID ?? props.permissionID ?? null"));
        assert!(!content.contains("props.resources"));
        assert!(content.contains("appendFile"));

        fs::remove_dir_all(&user_profile).expect("cleanup user profile");
        fs::remove_dir_all(&appdata_dir).expect("cleanup appdata");
    }

    #[test]
    fn detect_codex_integration_status_reads_installed_state_and_last_error() {
        let _guard = test_lock().lock().expect("lock");
        let appdata_dir = unique_test_dir("codex-provider-appdata");
        let codex_root = unique_test_dir("codex-provider-root");
        let config_path = resolve_codex_integration_path(&codex_root);

        fs::create_dir_all(&codex_root).expect("create codex root");
        fs::create_dir_all(&appdata_dir).expect("create appdata dir");

        let status = with_appdata(&appdata_dir, || {
            let bridge_path =
                resolve_provider_bridge_path(CODEX_PROVIDER).expect("resolve codex bridge");
            ensure_parent_dir(&bridge_path).expect("create bridge dir");
            fs::write(
                &bridge_path,
                format!(
                    "{}\n",
                    bridge_record_json(
                        CODEX_PROVIDER,
                        "session.updated",
                        "2026-04-03T09:15:00Z",
                        Some("codex hook failed")
                    )
                ),
            )
            .expect("write bridge record");

            let integration = serde_json::json!({
                "sessionHub": {
                    "provider": CODEX_PROVIDER,
                    "bridgePath": bridge_path.to_string_lossy().to_string(),
                    "integrationVersion": PROVIDER_INTEGRATION_VERSION
                },
                "hooks": {
                    "SessionStart": [{
                        "matcher": "startup|resume|clear|compact",
                        "hooks": [{
                            "type": "command",
                            "command": format!(
                                "node '/tmp/on-session-start.cjs' --bridge-path '{}' --provider codex # sessionhub-provider-event-bridge",
                                bridge_path.to_string_lossy()
                            )
                        }]
                    }],
                    "PreToolUse": [{
                        "hooks": [{
                            "type": "command",
                            "command": format!(
                                "node '/tmp/on-pre-tool-use.cjs' --bridge-path '{}' --provider codex # sessionhub-provider-event-bridge",
                                bridge_path.to_string_lossy()
                            )
                        }]
                    }],
                    "PostToolUse": [{
                        "hooks": [{
                            "type": "command",
                            "command": format!(
                                "node '/tmp/on-post-tool-use.cjs' --bridge-path '{}' --provider codex # sessionhub-provider-event-bridge",
                                bridge_path.to_string_lossy()
                            )
                        }]
                    }],
                    "UserPromptSubmit": [{
                        "hooks": [{
                            "type": "command",
                            "command": format!(
                                "node '/tmp/on-user-prompt-submit.cjs' --bridge-path '{}' --provider codex # sessionhub-provider-event-bridge",
                                bridge_path.to_string_lossy()
                            )
                        }]
                    }],
                    "Stop": [{
                        "hooks": [{
                            "type": "command",
                            "command": format!(
                                "node '/tmp/on-stop.cjs' --bridge-path '{}' --provider codex # sessionhub-provider-event-bridge",
                                bridge_path.to_string_lossy()
                            )
                        }]
                    }]
                }
            });
            fs::write(
                &config_path,
                serde_json::to_string_pretty(&integration).expect("serialize codex integration"),
            )
            .expect("write codex integration");

            let root_string = codex_root.to_string_lossy().to_string();
            detect_codex_integration_status(Some(root_string.as_str()))
        });

        assert_eq!(status.status, ProviderIntegrationState::Installed);
        assert_eq!(
            status.last_event_at.as_deref(),
            Some("2026-04-03T09:15:00Z")
        );
        assert_eq!(status.last_error.as_deref(), Some("codex hook failed"));
        assert_eq!(
            status.config_path.as_deref(),
            Some(config_path.to_string_lossy().as_ref())
        );
        assert_eq!(status.installed_version, None);

        fs::remove_dir_all(&codex_root).expect("cleanup codex root");
        fs::remove_dir_all(&appdata_dir).expect("cleanup appdata");
    }

    #[test]
    fn detect_codex_integration_status_marks_missing_and_manual_required() {
        let _guard = test_lock().lock().expect("lock");
        let appdata_dir = unique_test_dir("codex-provider-missing-appdata");
        let codex_root = unique_test_dir("codex-provider-missing-root");

        fs::create_dir_all(&appdata_dir).expect("create appdata dir");
        fs::create_dir_all(&codex_root).expect("create codex root");

        let missing_status = with_appdata(&appdata_dir, || {
            let root_string = codex_root.to_string_lossy().to_string();
            detect_codex_integration_status(Some(root_string.as_str()))
        });
        assert_eq!(missing_status.status, ProviderIntegrationState::Missing);
        assert!(missing_status.last_error.is_none());
        assert!(missing_status
            .config_path
            .as_deref()
            .is_some_and(|path| path.ends_with(CODEX_HOOK_FILE_NAME)));

        let manual_required_status = with_appdata(&appdata_dir, || {
            without_env_var("USERPROFILE", || detect_codex_integration_status(None))
        });
        assert_eq!(
            manual_required_status.status,
            ProviderIntegrationState::ManualRequired
        );
        assert!(manual_required_status.config_path.is_none());
        assert!(manual_required_status
            .last_error
            .as_deref()
            .is_some_and(|error| error.contains("USERPROFILE")));

        fs::remove_dir_all(&codex_root).expect("cleanup codex root");
        fs::remove_dir_all(&appdata_dir).expect("cleanup appdata");
    }

    #[test]
    fn install_codex_integration_writes_managed_hook_file() {
        let _guard = test_lock().lock().expect("lock");
        let appdata_dir = unique_test_dir("codex-provider-install-appdata");
        let codex_root = unique_test_dir("codex-provider-install-root");

        fs::create_dir_all(&appdata_dir).expect("create appdata dir");
        fs::create_dir_all(&codex_root).expect("create codex root");

        let status = with_appdata(&appdata_dir, || {
            let root_string = codex_root.to_string_lossy().to_string();
            install_or_update_codex_integration(Some(root_string.as_str()))
        });
        let config_path = resolve_codex_integration_path(&codex_root);
        let content = fs::read_to_string(&config_path).expect("read codex hook file");

        assert_eq!(status.status, ProviderIntegrationState::Installed);
        assert!(content.contains("\"SessionStart\""));
        assert!(content.contains("\"PreToolUse\""));
        assert!(content.contains("\"PostToolUse\""));
        assert!(content.contains("\"UserPromptSubmit\""));
        assert!(content.contains("\"Stop\""));
        assert!(content.contains("node "));
        assert!(content.contains("on-session-start.cjs"));
        assert!(content.contains("sessionhub-provider-event-bridge"));
        assert!(!content.contains("\"commandWindows\""));

        fs::remove_dir_all(&codex_root).expect("cleanup codex root");
        fs::remove_dir_all(&appdata_dir).expect("cleanup appdata");
    }

    #[test]
    fn provider_refresh_event_name_supports_codex() {
        assert_eq!(
            provider_refresh_event_name(CODEX_PROVIDER).expect("resolve codex refresh event"),
            "codex-sessions-updated"
        );
    }

    // ──────────────────────────────────────────────────────────────────────────
    // session-cache-sqlite-migration：測試 10.1-10.3
    // ──────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_sessions_cache_roundtrip() {
        let _guard = test_lock().lock().expect("failed to lock test mutex");
        let appdata_dir = unique_test_dir("appdata-cache");
        fs::create_dir_all(&appdata_dir).expect("create appdata dir");

        with_appdata(&appdata_dir, || {
            let connection = open_db_connection().expect("open db");
            init_db(&connection).expect("init db");

            let sessions = vec![
                SessionInfo {
                    id: "ses-aaa".to_string(),
                    provider: "copilot".to_string(),
                    cwd: Some("C:/proj/a".to_string()),
                    repo_root: Some("C:/proj".to_string()),
                    repo_name: Some("proj".to_string()),
                    git_branch: Some("main".to_string()),
                    summary: Some("initial session".to_string()),
                    summary_count: Some(3),
                    created_at: Some("2026-01-01T00:00:00Z".to_string()),
                    updated_at: Some("2026-01-01T01:00:00Z".to_string()),
                    session_dir: "C:/proj/a/.copilot/session-state/ses-aaa".to_string(),
                    parse_error: false,
                    is_archived: false,
                    notes: None,
                    tags: Vec::new(),
                    has_plan: true,
                    has_events: true,
                },
                SessionInfo {
                    id: "ses-bbb".to_string(),
                    provider: "copilot".to_string(),
                    cwd: Some("C:/proj/b".to_string()),
                    repo_root: Some("C:/proj".to_string()),
                    repo_name: Some("proj".to_string()),
                    git_branch: Some("feature/test".to_string()),
                    summary: None,
                    summary_count: None,
                    created_at: None,
                    updated_at: None,
                    session_dir: "C:/proj/b/.copilot/session-state/ses-bbb".to_string(),
                    parse_error: true,
                    is_archived: true,
                    notes: None,
                    tags: Vec::new(),
                    has_plan: false,
                    has_events: false,
                },
            ];

            save_sessions_cache_to_db(&connection, &["copilot".to_string()], &sessions)
                .expect("save sessions cache");

            let loaded = load_sessions_cache_from_db(&connection, Some("copilot"))
                .expect("load sessions cache");

            assert_eq!(loaded.len(), 2);

            let a = loaded
                .iter()
                .find(|s| s.id == "ses-aaa")
                .expect("ses-aaa not found");
            assert_eq!(a.provider, "copilot");
            assert_eq!(a.cwd.as_deref(), Some("C:/proj/a"));
            assert_eq!(a.repo_root.as_deref(), Some("C:/proj"));
            assert_eq!(a.repo_name.as_deref(), Some("proj"));
            assert_eq!(a.git_branch.as_deref(), Some("main"));
            assert_eq!(a.summary.as_deref(), Some("initial session"));
            assert_eq!(a.summary_count, Some(3));
            assert!(!a.parse_error);
            assert!(!a.is_archived);
            assert!(a.has_plan);
            assert!(a.has_events);

            let b = loaded
                .iter()
                .find(|s| s.id == "ses-bbb")
                .expect("ses-bbb not found");
            assert!(b.parse_error);
            assert!(b.is_archived);
            assert!(!b.has_plan);
            assert!(!b.has_events);
        });

        fs::remove_dir_all(&appdata_dir).expect("cleanup appdata");
    }

    #[test]
    fn test_scan_state_roundtrip() {
        let _guard = test_lock().lock().expect("failed to lock test mutex");
        let appdata_dir = unique_test_dir("appdata-scan-state");
        fs::create_dir_all(&appdata_dir).expect("create appdata dir");

        with_appdata(&appdata_dir, || {
            let connection = open_db_connection().expect("open db");
            init_db(&connection).expect("init db");

            // 首次讀取應回傳 (0, 0)
            let (ts0, cursor0) =
                load_scan_state_from_db(&connection, "copilot").expect("load scan_state (empty)");
            assert_eq!(ts0, 0);
            assert_eq!(cursor0, 0);

            save_scan_state_to_db(&connection, "copilot", 12345, 67890).expect("save scan_state");
            let (ts, cursor) =
                load_scan_state_from_db(&connection, "copilot").expect("load scan_state");
            assert_eq!(ts, 12345);
            assert_eq!(cursor, 67890);

            // 覆寫後應讀到新值
            save_scan_state_to_db(&connection, "copilot", 99999, 11111)
                .expect("overwrite scan_state");
            let (ts2, cursor2) = load_scan_state_from_db(&connection, "copilot")
                .expect("load scan_state after overwrite");
            assert_eq!(ts2, 99999);
            assert_eq!(cursor2, 11111);
        });

        fs::remove_dir_all(&appdata_dir).expect("cleanup appdata");
    }

    #[test]
    fn test_session_mtimes_roundtrip() {
        let _guard = test_lock().lock().expect("failed to lock test mutex");
        let appdata_dir = unique_test_dir("appdata-mtimes");
        fs::create_dir_all(&appdata_dir).expect("create appdata dir");

        with_appdata(&appdata_dir, || {
            let connection = open_db_connection().expect("open db");
            init_db(&connection).expect("init db");

            let mut mtimes = HashMap::new();
            mtimes.insert("ses-aaa".to_string(), 1000_i64);
            mtimes.insert("ses-bbb".to_string(), 2000_i64);
            mtimes.insert("ses-ccc".to_string(), 3000_i64);

            save_session_mtimes_to_db(&connection, "copilot", &mtimes)
                .expect("save session_mtimes");

            let loaded =
                load_session_mtimes_from_db(&connection, "copilot").expect("load session_mtimes");

            assert_eq!(loaded.len(), 3);
            assert_eq!(loaded.get("ses-aaa"), Some(&1000));
            assert_eq!(loaded.get("ses-bbb"), Some(&2000));
            assert_eq!(loaded.get("ses-ccc"), Some(&3000));

            // 覆寫：縮減為 1 個，舊的應清除
            let mut mtimes2 = HashMap::new();
            mtimes2.insert("ses-aaa".to_string(), 5555_i64);
            save_session_mtimes_to_db(&connection, "copilot", &mtimes2)
                .expect("overwrite session_mtimes");

            let loaded2 = load_session_mtimes_from_db(&connection, "copilot")
                .expect("load session_mtimes after overwrite");
            assert_eq!(loaded2.len(), 1);
            assert_eq!(loaded2.get("ses-aaa"), Some(&5555));
        });

        fs::remove_dir_all(&appdata_dir).expect("cleanup appdata");
    }

    #[test]
    fn test_get_analytics_data_internal_groups_by_day_week_month() {
        let _guard = test_lock().lock().expect("failed to lock test mutex");
        let appdata_dir = unique_test_dir("appdata-analytics-group");
        fs::create_dir_all(&appdata_dir).expect("create appdata dir");

        with_appdata(&appdata_dir, || {
            let connection = open_db_connection().expect("open db");
            init_db(&connection).expect("init db");

            save_sessions_cache_to_db(
                &connection,
                &["copilot".to_string()],
                &[
                    SessionInfo {
                        id: "analytics-a".to_string(),
                        provider: "copilot".to_string(),
                        cwd: Some("D:\\repo\\demo".to_string()),
                        repo_root: Some("D:\\repo\\demo".to_string()),
                        repo_name: Some("demo".to_string()),
                        git_branch: Some("main".to_string()),
                        summary: Some("A".to_string()),
                        summary_count: Some(1),
                        created_at: Some("2026-04-01T00:00:00Z".to_string()),
                        updated_at: Some("2026-04-01T08:00:00Z".to_string()),
                        session_dir: "D:\\repo\\demo\\a".to_string(),
                        parse_error: false,
                        is_archived: false,
                        notes: None,
                        tags: Vec::new(),
                        has_plan: false,
                        has_events: true,
                    },
                    SessionInfo {
                        id: "analytics-b".to_string(),
                        provider: "copilot".to_string(),
                        cwd: Some("D:\\repo\\demo".to_string()),
                        repo_root: Some("D:\\repo\\demo".to_string()),
                        repo_name: Some("demo".to_string()),
                        git_branch: Some("main".to_string()),
                        summary: Some("B".to_string()),
                        summary_count: Some(1),
                        created_at: Some("2026-04-02T00:00:00Z".to_string()),
                        updated_at: Some("2026-04-08T09:00:00Z".to_string()),
                        session_dir: "D:\\repo\\demo\\b".to_string(),
                        parse_error: false,
                        is_archived: false,
                        notes: None,
                        tags: Vec::new(),
                        has_plan: false,
                        has_events: true,
                    },
                ],
            )
            .expect("save sessions cache");

            upsert_session_stats_cache(
                &connection,
                "analytics-a",
                100,
                &SessionStats {
                    output_tokens: 120,
                    input_tokens: 80,
                    interaction_count: 4,
                    model_metrics: BTreeMap::from([(
                        "gpt-5.4".to_string(),
                        ModelMetricsEntry {
                            requests_count: 2.0,
                            requests_cost: 1.5,
                            input_tokens: 80,
                            output_tokens: 120,
                        },
                    )]),
                    ..SessionStats::default()
                },
            )
            .expect("upsert stats a");

            upsert_session_stats_cache(
                &connection,
                "analytics-b",
                101,
                &SessionStats {
                    output_tokens: 60,
                    input_tokens: 30,
                    interaction_count: 2,
                    model_metrics: BTreeMap::from([(
                        "gpt-5.4".to_string(),
                        ModelMetricsEntry {
                            requests_count: 1.0,
                            requests_cost: 0.5,
                            input_tokens: 30,
                            output_tokens: 60,
                        },
                    )]),
                    ..SessionStats::default()
                },
            )
            .expect("upsert stats b");

            let by_day = get_analytics_data_internal(
                &connection,
                Some("D:\\repo\\demo"),
                "2026-04-01",
                "2026-04-30",
                "day",
            )
            .expect("analytics by day");
            assert_eq!(by_day.len(), 2);
            assert_eq!(by_day[0].label, "2026-04-01");
            assert_eq!(by_day[0].output_tokens, 120);
            assert_eq!(by_day[0].cost_points, 1.5);
            assert_eq!(by_day[1].label, "2026-04-08");

            let by_week = get_analytics_data_internal(
                &connection,
                Some("D:\\repo\\demo"),
                "2026-04-01",
                "2026-04-30",
                "week",
            )
            .expect("analytics by week");
            assert_eq!(by_week.len(), 2);
            assert_eq!(by_week[0].label, "2026-W14");
            assert_eq!(by_week[1].label, "2026-W15");

            let by_month = get_analytics_data_internal(
                &connection,
                Some("D:\\repo\\demo"),
                "2026-04-01",
                "2026-04-30",
                "month",
            )
            .expect("analytics by month");
            assert_eq!(by_month.len(), 1);
            assert_eq!(by_month[0].label, "2026-04");
            assert_eq!(by_month[0].output_tokens, 180);
            assert_eq!(by_month[0].interaction_count, 6);
            assert_eq!(by_month[0].cost_points, 2.0);
        });

        fs::remove_dir_all(&appdata_dir).expect("cleanup appdata");
    }

    #[test]
    fn test_get_analytics_data_internal_returns_empty_for_no_matches() {
        let _guard = test_lock().lock().expect("failed to lock test mutex");
        let appdata_dir = unique_test_dir("appdata-analytics-empty");
        fs::create_dir_all(&appdata_dir).expect("create appdata dir");

        with_appdata(&appdata_dir, || {
            let connection = open_db_connection().expect("open db");
            init_db(&connection).expect("init db");

            let points = get_analytics_data_internal(
                &connection,
                Some("D:\\repo\\demo"),
                "2026-04-01",
                "2026-04-30",
                "day",
            )
            .expect("empty analytics");
            assert!(points.is_empty());
        });

        fs::remove_dir_all(&appdata_dir).expect("cleanup appdata");
    }

    #[test]
    fn test_get_analytics_data_internal_validates_inputs() {
        let _guard = test_lock().lock().expect("failed to lock test mutex");
        let appdata_dir = unique_test_dir("appdata-analytics-errors");
        fs::create_dir_all(&appdata_dir).expect("create appdata dir");

        with_appdata(&appdata_dir, || {
            let connection = open_db_connection().expect("open db");
            init_db(&connection).expect("init db");

            let invalid_start =
                get_analytics_data_internal(&connection, None, "2026/04/01", "2026-04-30", "day");
            assert_eq!(invalid_start.unwrap_err(), "invalid date format: startDate");

            let reversed =
                get_analytics_data_internal(&connection, None, "2026-04-30", "2026-04-01", "day");
            assert_eq!(reversed.unwrap_err(), "startDate must be before endDate");

            let invalid_group = get_analytics_data_internal(
                &connection,
                None,
                "2026-04-01",
                "2026-04-30",
                "quarter",
            );
            assert_eq!(invalid_group.unwrap_err(), "invalid groupBy value");
        });

        fs::remove_dir_all(&appdata_dir).expect("cleanup appdata");
    }

    // ──────────────────────────────────────────────────────────────────────────
    // session-stats-panel-opencode-redesign：測試 7.1
    // ──────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_opencode_session_stats_session_dir() {
        let _guard = test_lock().lock().expect("failed to lock test mutex");
        let root_dir = unique_test_dir("oc-stats");
        let storage_root = root_dir.join("storage");
        let message_dir = storage_root.join("message").join("ses_test001");
        fs::create_dir_all(&message_dir).expect("create message dir");

        // 建立一個 assistant message JSON 帶 output tokens
        fs::write(
            message_dir.join("msg1.json"),
            r#"{"id":"msg1","role":"assistant","metadata":{"assistant":{"modelId":"claude-3.5","tokens":{"output":150}}}}"#,
        )
        .expect("write msg1.json");

        let stats = calculate_opencode_session_stats(&message_dir).expect("stats should calculate");

        assert_eq!(stats.output_tokens, 150);
        assert!(stats.interaction_count == 0); // assistant message 不計入

        // 加一個 user message
        fs::write(
            message_dir.join("msg2.json"),
            r#"{"id":"msg2","role":"user","metadata":{}}"#,
        )
        .expect("write msg2.json");

        let stats2 =
            calculate_opencode_session_stats(&message_dir).expect("stats2 should calculate");

        assert_eq!(stats2.output_tokens, 150);
        assert_eq!(stats2.interaction_count, 1);

        fs::remove_dir_all(&root_dir).expect("cleanup root");
    }

    #[test]
    fn test_opencode_session_stats_db_only_message_source() {
        let _guard = test_lock().lock().expect("failed to lock test mutex");
        let root_dir = unique_test_dir("oc-stats-db");
        let storage_root = root_dir.join("storage");
        let message_dir = storage_root.join("message").join("ses_db_stats");

        fs::create_dir_all(&storage_root).expect("create storage root");
        create_opencode_db_sessions(
            &root_dir,
            "project-db-stats",
            "D:\\repo\\stats",
            &[("ses_db_stats", "Stats Session", 1000, 4000, None)],
        );
        insert_opencode_db_message(
            &root_dir,
            "ses_db_stats",
            "msg_db_user",
            1000,
            1000,
            r#"{"role":"user","time":{"created":1000}}"#,
        );
        insert_opencode_db_message(
            &root_dir,
            "ses_db_stats",
            "msg_db_assistant",
            1001,
            4000,
            r#"{"role":"assistant","time":{"created":1001,"completed":4000},"modelID":"gpt-5","tokens":{"input":80,"output":150,"reasoning":10,"cache":{"write":0,"read":0}},"finish":"tool-calls"}"#,
        );
        insert_opencode_db_part(
            &root_dir,
            "ses_db_stats",
            "msg_db_assistant",
            "prt_db_tool",
            1002,
            1002,
            r#"{"type":"tool","tool":"bash"}"#,
        );

        let stats =
            calculate_opencode_session_stats(&message_dir).expect("db stats should calculate");

        assert_eq!(stats.output_tokens, 150);
        assert_eq!(stats.input_tokens, 80);
        assert_eq!(stats.reasoning_count, 1);
        assert_eq!(stats.interaction_count, 1);
        assert_eq!(stats.tool_call_count, 1);
        assert_eq!(stats.tool_breakdown.get("bash"), Some(&1));

        fs::remove_dir_all(&root_dir).expect("cleanup root");
    }
}
