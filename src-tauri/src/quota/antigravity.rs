use crate::types::{AppSettings, QuotaSnapshot, QuotaWindow, ANTIGRAVITY_PROVIDER};

use super::QuotaAdapter;

pub(crate) struct AntigravityAdapter;

fn current_timestamp() -> String {
    chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string()
}

fn no_auth_snapshot(error_message: impl Into<String>) -> QuotaSnapshot {
    QuotaSnapshot {
        provider: ANTIGRAVITY_PROVIDER.to_string(),
        status: "no_auth".to_string(),
        source: "local_scan".to_string(),
        fetched_at: current_timestamp(),
        error_message: Some(error_message.into()),
        windows: None,
        local_tokens: None,
        extra_credits: None,
        reset_credits: None,
    }
}

fn error_snapshot(error_message: impl Into<String>) -> QuotaSnapshot {
    QuotaSnapshot {
        provider: ANTIGRAVITY_PROVIDER.to_string(),
        status: "error".to_string(),
        source: "local_scan".to_string(),
        fetched_at: current_timestamp(),
        error_message: Some(error_message.into()),
        windows: None,
        local_tokens: None,
        extra_credits: None,
        reset_credits: None,
    }
}

/// 在 Windows 上以 `tasklist` 找出 `language_server.exe` 的 PID，再以 `netstat -ano` 找出其
/// 於 `127.0.0.1` 監聽的所有 port。非 Windows 平台回傳「不支援」錯誤，由呼叫端轉為不可用狀態。
#[cfg(target_os = "windows")]
mod platform {
    use std::os::windows::process::CommandExt;
    use std::process::Command;

    const CREATE_NO_WINDOW: u32 = 0x08000000;

    fn find_language_server_pid() -> Result<String, String> {
        let output = Command::new("tasklist")
            .args(["/FI", "IMAGENAME eq language_server.exe", "/FO", "CSV"])
            .creation_flags(CREATE_NO_WINDOW)
            .output()
            .map_err(|error| format!("執行 tasklist 失敗: {error}"))?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let matched_line = stdout
            .lines()
            .find(|line| line.contains("language_server.exe"))
            .ok_or_else(|| {
                "找不到執行中的 language_server.exe，Antigravity IDE／agy 可能未啟動".to_string()
            })?;

        let pid = matched_line
            .split(',')
            .nth(1)
            .map(|value| value.trim_matches('"').trim().to_string())
            .ok_or_else(|| "無法解析 language_server.exe 的 PID".to_string())?;

        Ok(pid)
    }

    fn find_listening_ports(pid: &str) -> Result<Vec<u16>, String> {
        let output = Command::new("netstat")
            .args(["-ano"])
            .creation_flags(CREATE_NO_WINDOW)
            .output()
            .map_err(|error| format!("執行 netstat 失敗: {error}"))?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut ports = Vec::new();

        for line in stdout.lines() {
            if !line.contains("LISTENING") || !line.contains(pid) {
                continue;
            }
            let parts: Vec<&str> = line.split_whitespace().collect();
            let Some(address) = parts.get(1) else {
                continue;
            };
            let Some(port_str) = address.rsplit(':').next() else {
                continue;
            };
            if let Ok(port) = port_str.parse::<u16>() {
                if !ports.contains(&port) {
                    ports.push(port);
                }
            }
        }

        if ports.is_empty() {
            return Err("找不到 language_server 監聽的 port".to_string());
        }

        // Web port 通常較大，優先嘗試
        ports.sort_unstable_by(|a, b| b.cmp(a));
        Ok(ports)
    }

    /// 找出 Antigravity Language Server 目前監聽的候選 port（可能多個）
    pub(super) fn discover_candidate_ports() -> Result<Vec<u16>, String> {
        let pid = find_language_server_pid()?;
        find_listening_ports(&pid)
    }
}

#[cfg(not(target_os = "windows"))]
mod platform {
    pub(super) fn discover_candidate_ports() -> Result<Vec<u16>, String> {
        Err("Antigravity quota 探測目前僅支援 Windows".to_string())
    }
}

fn fetch_csrf_token(port: u16) -> Result<String, String> {
    let url = format!("http://127.0.0.1:{port}/");
    let response = ureq::get(&url)
        .timeout(std::time::Duration::from_secs(2))
        .call()
        .map_err(|error| format!("GET {url} 失敗: {error}"))?;
    let body = response
        .into_string()
        .map_err(|error| format!("讀取回應內容失敗: {error}"))?;

    let marker = "\"csrfToken\":\"";
    let start = body
        .find(marker)
        .ok_or_else(|| "無法在回應中找到 csrfToken".to_string())?
        + marker.len();
    let end = body[start..]
        .find('"')
        .ok_or_else(|| "csrfToken 格式錯誤".to_string())?
        + start;

    Ok(body[start..end].to_string())
}

fn call_rpc(port: u16, path: &str, csrf_token: &str) -> Result<serde_json::Value, String> {
    let url = format!("http://127.0.0.1:{port}{path}");
    let response = ureq::post(&url)
        .set("Content-Type", "application/json")
        .set("Origin", &format!("http://127.0.0.1:{port}"))
        .set("x-codeium-csrf-token", csrf_token)
        .send_string("{}")
        .map_err(|error| format!("POST {url} 失敗: {error}"))?;

    response
        .into_json::<serde_json::Value>()
        .map_err(|error| format!("解析 {path} 回應失敗: {error}"))
}

struct QuotaBucket {
    window: String,
    remaining_fraction: f64,
    reset_time: Option<String>,
}

struct QuotaGroup {
    display_name: String,
    buckets: Vec<QuotaBucket>,
}

fn parse_quota_groups(response: &serde_json::Value) -> Vec<QuotaGroup> {
    let mut groups = Vec::new();
    let Some(response_obj) = response.get("response") else {
        return groups;
    };
    let Some(raw_groups) = response_obj.get("groups").and_then(|v| v.as_array()) else {
        return groups;
    };

    for raw_group in raw_groups {
        let display_name = raw_group
            .get("displayName")
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown")
            .to_string();

        let mut buckets = Vec::new();
        if let Some(raw_buckets) = raw_group.get("buckets").and_then(|v| v.as_array()) {
            for raw_bucket in raw_buckets {
                let Some(remaining_fraction) =
                    raw_bucket.get("remainingFraction").and_then(|v| v.as_f64())
                else {
                    continue;
                };
                let window = raw_bucket
                    .get("window")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown")
                    .to_string();
                let reset_time = raw_bucket
                    .get("resetTime")
                    .and_then(|v| v.as_str())
                    .map(str::to_string);

                buckets.push(QuotaBucket {
                    window,
                    remaining_fraction,
                    reset_time,
                });
            }
        }

        groups.push(QuotaGroup {
            display_name,
            buckets,
        });
    }

    groups
}

fn window_label(window_key: &str) -> String {
    match window_key {
        "5h" => "5 小時".to_string(),
        "weekly" => "每週".to_string(),
        other => other.to_string(),
    }
}

fn groups_to_windows(groups: &[QuotaGroup]) -> Vec<QuotaWindow> {
    let mut windows = Vec::new();
    for group in groups {
        for bucket in &group.buckets {
            windows.push(QuotaWindow {
                window_key: bucket.window.clone(),
                label: window_label(&bucket.window),
                utilization: (1.0 - bucket.remaining_fraction).clamp(0.0, 1.0),
                resets_at: bucket.reset_time.clone(),
                group: Some(group.display_name.clone()),
            });
        }
    }
    windows
}

fn fetch_quota_via_port(port: u16) -> Result<Vec<QuotaWindow>, String> {
    let csrf_token = fetch_csrf_token(port)?;
    let response = call_rpc(
        port,
        "/exa.language_server_pb.LanguageServerService/RetrieveUserQuotaSummary",
        &csrf_token,
    )?;
    let groups = parse_quota_groups(&response);
    Ok(groups_to_windows(&groups))
}

impl QuotaAdapter for AntigravityAdapter {
    fn provider_key(&self) -> &str {
        ANTIGRAVITY_PROVIDER
    }

    fn fetch_snapshot(&self, _settings: &AppSettings) -> QuotaSnapshot {
        let ports = match platform::discover_candidate_ports() {
            Ok(ports) => ports,
            Err(error) => return no_auth_snapshot(error),
        };

        let mut last_error: Option<String> = None;
        for port in ports {
            match fetch_quota_via_port(port) {
                Ok(windows) => {
                    return QuotaSnapshot {
                        provider: ANTIGRAVITY_PROVIDER.to_string(),
                        status: "ok".to_string(),
                        source: "local_scan".to_string(),
                        fetched_at: current_timestamp(),
                        error_message: None,
                        windows: if windows.is_empty() {
                            None
                        } else {
                            Some(windows)
                        },
                        local_tokens: None,
                        extra_credits: None,
                        reset_credits: None,
                    };
                }
                Err(error) => {
                    last_error = Some(error);
                }
            }
        }

        error_snapshot(last_error.unwrap_or_else(|| "所有候選 port 皆連線失敗".to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn antigravity_adapter_degrades_gracefully_regardless_of_language_server_state() {
        let settings = AppSettings {
            claude_root: String::new(),
            copilot_root: String::new(),
            opencode_root: String::new(),
            codex_root: String::new(),
            antigravity_root: String::new(),
            terminal_path: None,
            external_editor_path: None,
            show_archived: false,
            pinned_projects: Vec::new(),
            enabled_providers: vec!["antigravity".to_string()],
            provider_integrations: Vec::new(),
            default_launcher: None,
            enable_intervention_notification: false,
            enable_session_end_notification: false,
            show_status_bar: false,
            analytics_refresh_interval: 30,
            analytics_panel_collapsed: false,
            hook_scripts_path: String::new(),
            claude_quota_reset_day: 1,
            minimize_to_tray: false,
            launch_on_startup: false,
            start_minimized_on_startup: true,
            enable_quota_monitoring: true,
            quota_enabled_providers: crate::types::default_enabled_providers_all(),
            allow_create_project_config_dir: false,
            agents_source_root: String::new(),
            tray_quota_mode: crate::types::TrayQuotaMode::default(),
            tray_quota_primary_provider: None,
            tray_quota_panel_enabled: true,
            quota_overlay_enabled: false,
            quota_overlay_locked: true,
            quota_overlay_opacity: 0.85,
            quota_overlay_providers: Vec::new(),
            quota_overlay_theme: crate::types::OverlayTheme::default(),
            quota_overlay_style: crate::types::OverlayStyle::default(),
        };

        let adapter = AntigravityAdapter;
        let snapshot = adapter.fetch_snapshot(&settings);
        // 不論 Antigravity IDE 是否執行中，adapter 都不應 panic：
        // 未執行 → no_auth；執行中但 RPC 失敗 → error；執行中且成功 → ok
        assert!(["no_auth", "error", "ok"].contains(&snapshot.status.as_str()));
        assert_eq!(snapshot.provider, "antigravity");
    }

    #[test]
    #[ignore = "requires Antigravity IDE/agy running with language_server.exe active; run manually with --ignored"]
    fn manual_smoke_fetch_real_quota_snapshot() {
        let settings = AppSettings {
            claude_root: String::new(),
            copilot_root: String::new(),
            opencode_root: String::new(),
            codex_root: String::new(),
            antigravity_root: String::new(),
            terminal_path: None,
            external_editor_path: None,
            show_archived: false,
            pinned_projects: Vec::new(),
            enabled_providers: vec!["antigravity".to_string()],
            provider_integrations: Vec::new(),
            default_launcher: None,
            enable_intervention_notification: false,
            enable_session_end_notification: false,
            show_status_bar: false,
            analytics_refresh_interval: 30,
            analytics_panel_collapsed: false,
            hook_scripts_path: String::new(),
            claude_quota_reset_day: 1,
            minimize_to_tray: false,
            launch_on_startup: false,
            start_minimized_on_startup: true,
            enable_quota_monitoring: true,
            quota_enabled_providers: crate::types::default_enabled_providers_all(),
            allow_create_project_config_dir: false,
            agents_source_root: String::new(),
            tray_quota_mode: crate::types::TrayQuotaMode::default(),
            tray_quota_primary_provider: None,
            tray_quota_panel_enabled: true,
            quota_overlay_enabled: false,
            quota_overlay_locked: true,
            quota_overlay_opacity: 0.85,
            quota_overlay_providers: Vec::new(),
            quota_overlay_theme: crate::types::OverlayTheme::default(),
            quota_overlay_style: crate::types::OverlayStyle::default(),
        };

        let adapter = AntigravityAdapter;
        let snapshot = adapter.fetch_snapshot(&settings);
        println!("status={} source={}", snapshot.status, snapshot.source);
        println!("error_message={:?}", snapshot.error_message);
        let windows = snapshot.windows.clone().unwrap_or_default();
        for w in &windows {
            println!(
                "group={:?} window_key={} label={} utilization={:.4} resets_at={:?}",
                w.group, w.window_key, w.label, w.utilization, w.resets_at
            );
        }

        assert_eq!(
            snapshot.status, "ok",
            "expected live language_server.exe to answer successfully"
        );
        let groups: std::collections::HashSet<_> =
            windows.iter().filter_map(|w| w.group.clone()).collect();
        println!("distinct groups: {groups:?}");
        assert!(
            groups.len() >= 2,
            "expected at least Gemini + Claude/GPT groups"
        );
        for w in &windows {
            assert!(
                w.utilization >= 0.0 && w.utilization <= 1.0,
                "utilization out of range: {}",
                w.utilization
            );
        }
    }

    #[test]
    fn provider_key_is_antigravity() {
        let adapter = AntigravityAdapter;
        assert_eq!(adapter.provider_key(), "antigravity");
    }

    #[test]
    fn window_label_maps_known_keys() {
        assert_eq!(window_label("5h"), "5 小時");
        assert_eq!(window_label("weekly"), "每週");
        assert_eq!(window_label("other"), "other");
    }

    #[test]
    fn groups_to_windows_maps_utilization_and_group() {
        let groups = vec![QuotaGroup {
            display_name: "Gemini Models".to_string(),
            buckets: vec![QuotaBucket {
                window: "5h".to_string(),
                remaining_fraction: 0.75,
                reset_time: Some("2026-07-13T00:00:00Z".to_string()),
            }],
        }];
        let windows = groups_to_windows(&groups);
        assert_eq!(windows.len(), 1);
        assert_eq!(windows[0].utilization, 0.25);
        assert_eq!(windows[0].group.as_deref(), Some("Gemini Models"));
        assert_eq!(
            windows[0].resets_at.as_deref(),
            Some("2026-07-13T00:00:00Z")
        );
    }
}
