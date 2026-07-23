//! `run()` 的 setup 階段邏輯：quota 監控啟動、背景輪詢執行緒、tray icon 建構。
//!
//! Tray 選單為 Windows 原生元件，無法直接複用前端 t() 翻譯機制。
//! 若未來需要多語言 tray menu，需另外設計跨 Rust/前端的語言同步管道
//! （例如啟動時由前端透過 command 傳遞當前語言字串給後端）。
pub(crate) const TRAY_MENU_SHOW_WINDOW: &str = "顯示視窗";
pub(crate) const TRAY_MENU_TOGGLE_OVERLAY: &str = "顯示/隱藏 Quota Overlay";
pub(crate) const TRAY_MENU_TOGGLE_OVERLAY_LOCK: &str = "編輯 / 鎖定 Overlay 位置";
pub(crate) const TRAY_MENU_QUIT: &str = "退出 SessionHub";

use crate::{
    apply_overlay_locked, emit_quota_snapshots_updated, load_settings_internal,
    save_settings_internal, sync_overlay_visibility, toggle_tray_panel, AppSettings, DbState,
    QuotaCache, QUOTA_OVERLAY_LABEL,
};
use tauri::{Emitter, Manager};
use tauri_plugin_autostart::ManagerExt;

pub(crate) fn is_autostart_launch() -> bool {
    std::env::args().any(|arg| arg == "--autostart")
}

fn sync_autostart_registration_with(
    launch_on_startup: bool,
    enable: impl FnOnce() -> Result<(), String>,
    disable: impl FnOnce() -> Result<(), String>,
) -> Result<(), String> {
    if launch_on_startup {
        enable()
    } else {
        disable()
    }
}

pub(crate) fn sync_autostart_registration(
    app: &tauri::AppHandle,
    settings: &AppSettings,
) -> Result<(), String> {
    let autolaunch = app.autolaunch();
    sync_autostart_registration_with(
        settings.launch_on_startup,
        || autolaunch.enable().map_err(|error| error.to_string()),
        || autolaunch.disable().map_err(|error| error.to_string()),
    )
}

pub(crate) fn reconcile_autostart_on_startup(app: &tauri::AppHandle, settings: &AppSettings) {
    let autolaunch = app.autolaunch();
    let result = if settings.launch_on_startup {
        autolaunch.enable()
    } else {
        match autolaunch.is_enabled() {
            Ok(true) => autolaunch.disable(),
            Ok(false) => Ok(()),
            Err(error) => Err(error),
        }
    };

    if let Err(error) = result {
        eprintln!("[autostart] 啟動對帳失敗: {error}");
    }
}

#[cfg(test)]
mod tests {
    use super::sync_autostart_registration_with;

    #[test]
    fn sync_enables_when_requested() {
        let result = sync_autostart_registration_with(true, || Ok(()), || panic!("disable"));
        assert!(result.is_ok());
    }

    #[test]
    fn sync_disables_when_not_requested() {
        let result = sync_autostart_registration_with(false, || panic!("enable"), || Ok(()));
        assert!(result.is_ok());
    }

    #[test]
    fn sync_preserves_failure_for_the_caller_to_report() {
        let result = sync_autostart_registration_with(
            true,
            || Err("registration failed".to_string()),
            || Ok(()),
        );
        assert_eq!(result, Err("registration failed".to_string()));
    }
}

/// Quota monitoring 啟動：讀 DB 快取 + spawn 一次性背景執行緒做首次刷新。
pub(crate) fn setup_quota_monitoring(
    app: &tauri::App,
    settings: &AppSettings,
) -> Result<(), Box<dyn std::error::Error>> {
    if settings.enable_quota_monitoring {
        let db_state = app.state::<DbState>();
        let quota_cache = app.state::<QuotaCache>();
        {
            let conn = db_state.conn.lock().unwrap_or_else(|p| p.into_inner());
            let _ = crate::quota::cache::load_cache_from_db(
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
            // 載入失敗時直接跳過本次背景 quota 監控，維持原本「不啟動監控」的容錯行為，
            // 不落回 AppSettings::default()（其 enable_quota_monitoring 預設為 true，會與此行為衝突）。
            if let Ok(settings) = load_settings_internal() {
                if settings.enable_quota_monitoring {
                    crate::commands::quota::refresh_quota_internal(
                        &db_state,
                        &quota_cache,
                        &settings,
                        None,
                    );
                    emit_quota_snapshots_updated(&app_handle);
                    crate::tray_icon::update_tray_from_cache(&app_handle, &settings);
                }
            }
        });
    }

    Ok(())
}

/// 背景 quota 輪詢執行緒：每 60 秒檢查、每 30 分鐘實際刷新一次。
pub(crate) fn spawn_quota_poller_thread(app_handle: tauri::AppHandle) {
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
            crate::commands::quota::refresh_quota_internal(
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

/// 建立系統匣圖示、選單與事件綁定。
pub(crate) fn build_tray_icon(app: &tauri::App) -> tauri::Result<()> {
    let tray_icon = tauri::image::Image::from_bytes(include_bytes!("../icons/32x32.png")).ok();
    let mut tray_builder = tauri::tray::TrayIconBuilder::with_id(crate::tray_icon::TRAY_ICON_ID);
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

    let show_menu_item = tauri::menu::MenuItemBuilder::new(TRAY_MENU_SHOW_WINDOW)
        .id("show_window")
        .build(app)?;
    let overlay_toggle_item = tauri::menu::MenuItemBuilder::new(TRAY_MENU_TOGGLE_OVERLAY)
        .id("toggle_overlay")
        .build(app)?;
    let overlay_lock_item = tauri::menu::MenuItemBuilder::new(TRAY_MENU_TOGGLE_OVERLAY_LOCK)
        .id("toggle_overlay_lock")
        .build(app)?;
    let quit_menu_item = tauri::menu::MenuItemBuilder::new(TRAY_MENU_QUIT)
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

    Ok(())
}
