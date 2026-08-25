use std::io::ErrorKind;
use std::process::{Child, Command, Output};
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

use serde_json::Value;

use crate::settings::resolve_herdr_executable;
#[cfg(target_os = "windows")]
use crate::types::CREATE_NEW_CONSOLE;

/// herdr server 就緒輪詢間隔
const SERVER_READY_POLL_INTERVAL: Duration = Duration::from_millis(200);
/// herdr server 就緒等待上限
const SERVER_READY_TIMEOUT: Duration = Duration::from_secs(5);
/// client console spawn 後確認其存活的等待時間
const CLIENT_STARTUP_CHECK_DELAY: Duration = Duration::from_millis(500);
/// herdr 用以偵測巢狀啟動的環境變數。
///
/// 本應用程式若由 herdr session 內啟動會繼承這些變數，herdr client 便以
/// 「nested herdr is disabled by default」拒絕啟動並立即結束，必須先清除。
const HERDR_NESTING_ENV_KEYS: [&str; 6] = [
    "HERDR_ENV",
    "HERDR_PANE_ID",
    "HERDR_SOCKET_PATH",
    "HERDR_STARTUP_CWD",
    "HERDR_TAB_ID",
    "HERDR_WORKSPACE_ID",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct HerdrTab {
    pub(crate) pane_id: String,
    pub(crate) tab_id: String,
}

fn stderr_fragment(output: &Output) -> String {
    let text = String::from_utf8_lossy(&output.stderr).trim().to_string();
    if text.is_empty() {
        "no stderr output".to_string()
    } else {
        text.chars().take(500).collect()
    }
}

fn run_herdr(args: &[&str]) -> Result<Output, String> {
    let executable = resolve_herdr_executable()
        .ok_or_else(|| "herdr 未偵測到，請先安裝 herdr 並確認已加入 PATH".to_string())?;

    let mut command = Command::new(executable);
    command.args(args);
    #[cfg(target_os = "windows")]
    command.creation_flags(crate::types::CREATE_NO_WINDOW);
    command.output().map_err(|error| match error.kind() {
        ErrorKind::NotFound => "herdr 未偵測到，請先安裝 herdr 並確認已加入 PATH".to_string(),
        _ => format!("無法執行 herdr：{error}"),
    })
}

/// 由本應用程式建立的 herdr TUI client console 子程序。
///
/// herdr 為 server（headless）+ client（TUI）雙程序架構，socket API 呼叫只影響 server，
/// 若無 client 程序則使用者看不到任何畫面。此處只追蹤本程式自建的 client，
/// 無法辨識使用者手動開啟的 TUI（herdr 未提供查詢已附著 client 的介面）；
/// 追蹤狀態亦不跨程序保存，因此本程式重啟後會再建立一個 client
/// （實測多個 client 可同時附著於同一 session，不影響運作）。
fn client_console() -> &'static Mutex<Option<Child>> {
    static CLIENT_CONSOLE: OnceLock<Mutex<Option<Child>>> = OnceLock::new();
    CLIENT_CONSOLE.get_or_init(|| Mutex::new(None))
}

/// 判定本應用程式建立的 client console 是否仍存活。
fn herdr_client_console_alive() -> bool {
    let Ok(mut guard) = client_console().lock() else {
        return false;
    };
    let Some(child) = guard.as_mut() else {
        return false;
    };
    match child.try_wait() {
        Ok(None) => true,
        _ => {
            *guard = None;
            false
        }
    }
}

fn spawn_client_console() -> Result<(), String> {
    let executable = resolve_herdr_executable()
        .ok_or_else(|| "herdr 未偵測到，請先安裝 herdr 並確認已加入 PATH".to_string())?;

    // 裸指令即「Launch or attach to the persistent session」，server 已在執行時會附著而非另建 session。
    let mut command = Command::new(executable);
    for key in HERDR_NESTING_ENV_KEYS {
        command.env_remove(key);
    }
    #[cfg(target_os = "windows")]
    command.creation_flags(CREATE_NEW_CONSOLE);
    crate::sessions::configure_msys_stackdump_suppression(&mut command);

    let mut child = command
        .spawn()
        .map_err(|error| format!("無法開啟 herdr 終端視窗：{error}"))?;

    // herdr client 若因故拒絕啟動（例如巢狀偵測）會立即結束，spawn 本身仍算成功，
    // 此處確認其存活以免回報「已開啟」但實際什麼都沒發生。
    std::thread::sleep(CLIENT_STARTUP_CHECK_DELAY);
    if let Ok(Some(status)) = child.try_wait() {
        return Err(format!(
            "herdr 終端視窗啟動後隨即結束（exit code {:?}），請確認 herdr 可正常啟動",
            status.code()
        ));
    }

    *client_console()
        .lock()
        .map_err(|_| "failed to lock herdr client console state".to_string())? = Some(child);
    Ok(())
}

fn wait_for_server_ready() -> Result<(), String> {
    let deadline = Instant::now() + SERVER_READY_TIMEOUT;
    loop {
        if herdr_server_is_running().unwrap_or(false) {
            return Ok(());
        }
        if Instant::now() >= deadline {
            return Err(format!(
                "herdr 服務在 {} 秒內未能啟動，請確認 herdr 可正常執行",
                SERVER_READY_TIMEOUT.as_secs()
            ));
        }
        std::thread::sleep(SERVER_READY_POLL_INTERVAL);
    }
}

/// 確保有 herdr client（TUI）承載，使建立的 tab 對使用者可見。
///
/// 先以 socket API 詢問 server 是否已有 client 附著——此判定涵蓋使用者手動開啟的 TUI
/// 及本應用程式重啟前建立的 client；無法判定時才退回本程序自建 client 的存活狀態。
pub(crate) fn ensure_herdr_client_console() -> Result<(), String> {
    match herdr_client_attached() {
        Some(true) => return Ok(()),
        Some(false) => {}
        None => {
            if herdr_client_console_alive() {
                return Ok(());
            }
        }
    }

    spawn_client_console()?;
    wait_for_server_ready()
}

pub(crate) fn parse_herdr_tab_create_response(stdout: &str) -> Result<HerdrTab, String> {
    let value: Value = serde_json::from_str(stdout.trim())
        .map_err(|error| format!("herdr 回應不是有效 JSON：{error}"))?;
    let pane_id = value
        .get("result")
        .and_then(|result| result.get("root_pane"))
        .and_then(|pane| pane.get("pane_id"))
        .and_then(Value::as_str)
        .filter(|id| !id.trim().is_empty())
        .ok_or_else(|| "herdr 回應缺少 result.root_pane.pane_id".to_string())?;
    let tab_id = value
        .get("result")
        .and_then(|result| result.get("tab"))
        .and_then(|tab| tab.get("tab_id"))
        .and_then(Value::as_str)
        .filter(|id| !id.trim().is_empty())
        .ok_or_else(|| "herdr 回應缺少 result.tab.tab_id".to_string())?;

    Ok(HerdrTab {
        pane_id: pane_id.to_string(),
        tab_id: tab_id.to_string(),
    })
}

pub(crate) fn parse_herdr_server_status(output: &str) -> bool {
    output.lines().any(|line| {
        let normalized = line.trim().to_ascii_lowercase();
        normalized.starts_with("status:") && normalized.contains("running")
    }) || serde_json::from_str::<Value>(output)
        .ok()
        .and_then(|value| value.get("status").and_then(Value::as_str).map(str::to_ascii_lowercase))
        .is_some_and(|status| status == "running")
}

pub(crate) fn herdr_server_is_running() -> Result<bool, String> {
    let output = run_herdr(&["status", "server"])?;
    Ok(output.status.success()
        && (parse_herdr_server_status(&String::from_utf8_lossy(&output.stdout))
            || parse_herdr_server_status(&String::from_utf8_lossy(&output.stderr))))
}

pub(crate) fn parse_herdr_socket_path(output: &str) -> Option<String> {
    output.lines().find_map(|line| {
        let (key, value) = line.split_once(':')?;
        if key.trim().eq_ignore_ascii_case("socket") {
            let value = value.trim();
            (!value.is_empty()).then(|| value.to_string())
        } else {
            None
        }
    })
}

fn herdr_socket_path() -> Option<String> {
    let output = run_herdr(&["status", "server"]).ok()?;
    parse_herdr_socket_path(&String::from_utf8_lossy(&output.stdout))
}

/// 由 `client.window_title.*` 回應判斷是否有 client 附著於 session。
///
/// herdr 在無 client 時回報 `no_foreground_client`；此處的「foreground」指 herdr
/// 用以呈現畫面的 client，與作業系統視窗焦點無關（實測切換前景視窗後仍回報 `set`）。
pub(crate) fn parse_herdr_client_attached(stdout: &str) -> Option<bool> {
    let value: Value = serde_json::from_str(stdout.trim()).ok()?;
    let reason = value
        .get("result")
        .filter(|result| {
            result
                .get("type")
                .and_then(Value::as_str)
                .is_some_and(|kind| kind == "client_window_title")
        })?
        .get("reason")
        .and_then(Value::as_str)?;
    Some(reason != "no_foreground_client")
}

/// 詢問 herdr server 是否已有 client（TUI）附著。
///
/// 以 `client.window_title.clear` 作為探測：它是冪等操作且不留下副作用
/// （herdr 本身會持續設定該標題），同時回報 client 是否存在。
/// 無法判定時回傳 `None`，由呼叫端決定保守行為。
#[cfg(target_os = "windows")]
pub(crate) fn herdr_client_attached() -> Option<bool> {
    use std::fs::OpenOptions;
    use std::io::{BufRead, BufReader, Write};

    let socket_path = herdr_socket_path()?;
    let pipe_path = format!(r"\\.\pipe\{socket_path}");
    let mut pipe = OpenOptions::new()
        .read(true)
        .write(true)
        .open(&pipe_path)
        .ok()?;
    pipe.write_all(
        b"{\"id\":\"sessionhub:client-probe\",\"method\":\"client.window_title.clear\",\"params\":{}}\n",
    )
    .ok()?;
    pipe.flush().ok()?;

    let mut line = String::new();
    BufReader::new(pipe).read_line(&mut line).ok()?;
    parse_herdr_client_attached(&line)
}

#[cfg(not(target_os = "windows"))]
pub(crate) fn herdr_client_attached() -> Option<bool> {
    None
}

fn normalize_workspace_path(path: &str) -> String {
    path.trim()
        .trim_end_matches(['\\', '/'])
        .replace('/', "\\")
        .to_ascii_lowercase()
}

fn extract_json_string_field(
    input: &str,
    field: &str,
    from: usize,
) -> Option<(String, usize, usize)> {
    let marker = format!("\"{field}\":\"");
    let marker_start = input[from..].find(&marker)? + from;
    let value_start = marker_start + marker.len();
    let bytes = input.as_bytes();
    let mut index = value_start;
    while index < bytes.len() {
        if bytes[index] == b'"' {
            let mut backslashes = 0;
            let mut previous = index;
            while previous > value_start && bytes[previous - 1] == b'\\' {
                backslashes += 1;
                previous -= 1;
            }
            if backslashes % 2 == 0 {
                let encoded = &input[value_start - 1..=index];
                let value = serde_json::from_str::<String>(encoded).ok()?;
                return Some((value, marker_start, index + 1));
            }
        }
        index += 1;
    }
    None
}

fn parse_workspace_pairs_for_cwd(stdout: &str, cwd: &str) -> Option<String> {
    let normalized_cwd = normalize_workspace_path(cwd);
    let mut cursor = 0;
    let mut best_match: Option<(usize, String)> = None;

    while let Some((pane_cwd, _, pane_cwd_end)) = extract_json_string_field(stdout, "cwd", cursor)
    {
        let next_cwd_start = extract_json_string_field(stdout, "cwd", pane_cwd_end)
            .map(|(_, start, _)| start)
            .unwrap_or(stdout.len());
        let workspace_id = extract_json_string_field(
            &stdout[pane_cwd_end..next_cwd_start],
            "workspace_id",
            0,
        )
        .map(|(workspace, _, _)| workspace)?;
        let normalized_pane_cwd = normalize_workspace_path(&pane_cwd);
        let common_path_length = if normalized_pane_cwd == normalized_cwd {
            Some(normalized_cwd.len())
        } else if normalized_cwd.starts_with(&format!("{normalized_pane_cwd}\\")) {
            Some(normalized_pane_cwd.len())
        } else if normalized_pane_cwd.starts_with(&format!("{normalized_cwd}\\")) {
            Some(normalized_cwd.len())
        } else {
            None
        };
        if let Some(common_path_length) = common_path_length {
            if best_match
                .as_ref()
                .is_none_or(|(best_length, _)| common_path_length > *best_length)
            {
                best_match = Some((common_path_length, workspace_id));
            }
        }
        cursor = pane_cwd_end;
    }

    best_match.map(|(_, workspace_id)| workspace_id)
}

pub(crate) fn parse_herdr_workspace_for_cwd(stdout: &str, cwd: &str) -> Option<String> {
    let value = serde_json::from_str::<Value>(stdout.trim()).ok();
    let normalized_cwd = normalize_workspace_path(cwd);
    value
        .as_ref()
        .and_then(|value| {
            value
                .get("snapshot")
                .and_then(|snapshot| snapshot.get("panes"))
                .and_then(Value::as_array)
                .and_then(|panes| {
                    panes
                        .iter()
                        .filter_map(|pane| {
                            let pane_cwd = pane.get("cwd").and_then(Value::as_str)?;
                            let normalized_pane_cwd = normalize_workspace_path(pane_cwd);
                            let common_path_length = if normalized_pane_cwd == normalized_cwd {
                                Some(normalized_cwd.len())
                            } else if normalized_cwd.starts_with(&format!("{normalized_pane_cwd}\\")) {
                                Some(normalized_pane_cwd.len())
                            } else if normalized_pane_cwd.starts_with(&format!("{normalized_cwd}\\")) {
                                Some(normalized_cwd.len())
                            } else {
                                None
                            }?;
                            let workspace_id = pane
                                .get("workspace_id")
                                .and_then(Value::as_str)
                                .filter(|workspace| !workspace.trim().is_empty())
                                .map(str::to_string)?;
                            Some((common_path_length, workspace_id))
                        })
                        .max_by_key(|(common_path_length, _)| *common_path_length)
                        .map(|(_, workspace_id)| workspace_id)
                })
        })
        .or_else(|| parse_workspace_pairs_for_cwd(stdout, cwd))
}

pub(crate) fn herdr_workspace_for_cwd(cwd: &str) -> Result<Option<String>, String> {
    let output = run_herdr(&["api", "snapshot"])?;
    if !output.status.success() {
        return Ok(None);
    }
    Ok(parse_herdr_workspace_for_cwd(
        &String::from_utf8_lossy(&output.stdout),
        cwd,
    ))
}

pub(crate) fn herdr_tab_create(cwd: &str, label: &str, focus: bool) -> Result<HerdrTab, String> {
    let workspace_id = herdr_workspace_for_cwd(cwd)?;
    let mut args = vec!["tab", "create"];
    if let Some(workspace_id) = workspace_id.as_deref() {
        args.extend(["--workspace", workspace_id]);
    }
    args.extend(["--cwd", cwd, "--label", label]);
    if focus {
        args.push("--focus");
    }

    let output = run_herdr(&args)?;
    if !output.status.success() {
        return Err(format!(
            "herdr tab create 失敗（exit code {:?}）：{}",
            output.status.code(),
            stderr_fragment(&output)
        ));
    }

    parse_herdr_tab_create_response(&String::from_utf8_lossy(&output.stdout)).map_err(|error| {
        format!(
            "herdr tab create 回應解析失敗：{error}；stderr：{}",
            stderr_fragment(&output)
        )
    })
}

pub(crate) fn herdr_pane_run(pane_id: &str, command: &str) -> Result<(), String> {
    let output = run_herdr(&["pane", "run", pane_id, command])?;
    if output.status.success() {
        Ok(())
    } else {
        Err(format!(
            "herdr pane run 失敗（exit code {:?}）：{}",
            output.status.code(),
            stderr_fragment(&output)
        ))
    }
}

pub(crate) fn herdr_tab_focus(tab_id: &str) -> Result<(), String> {
    let output = run_herdr(&["tab", "focus", tab_id])?;
    if output.status.success() {
        Ok(())
    } else {
        Err(format!(
            "herdr tab focus 失敗，tab 可能已關閉（exit code {:?}）：{}",
            output.status.code(),
            stderr_fragment(&output)
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_tab_create_ids() {
        let tab = parse_herdr_tab_create_response(
            r#"{"result":{"root_pane":{"pane_id":"w1:pQ"},"tab":{"tab_id":"w1:t7"}}}"#,
        )
        .expect("tab response");
        assert_eq!(tab.pane_id, "w1:pQ");
        assert_eq!(tab.tab_id, "w1:t7");
    }

    #[test]
    fn rejects_tab_create_response_without_pane_id() {
        let result = parse_herdr_tab_create_response(r#"{"result":{"tab":{"tab_id":"w1:t7"}}}"#);
        assert!(result.unwrap_err().contains("pane_id"));
    }

    #[test]
    fn parses_socket_path_from_status() {
        let status = "status: running\nprotocol: 19\nsocket: C:\\Users\\User\\AppData\\Roaming\\herdr\\herdr.sock";
        assert_eq!(
            parse_herdr_socket_path(status),
            Some("C:\\Users\\User\\AppData\\Roaming\\herdr\\herdr.sock".to_string())
        );
    }

    #[test]
    fn ignores_status_without_socket_line() {
        assert_eq!(parse_herdr_socket_path("status: running"), None);
    }

    #[test]
    fn detects_attached_client() {
        let response =
            r#"{"id":"x","result":{"type":"client_window_title","changed":true,"reason":"cleared"}}"#;
        assert_eq!(parse_herdr_client_attached(response), Some(true));
    }

    #[test]
    fn detects_missing_client() {
        let response = r#"{"id":"x","result":{"type":"client_window_title","changed":false,"reason":"no_foreground_client"}}"#;
        assert_eq!(parse_herdr_client_attached(response), Some(false));
    }

    #[test]
    fn rejects_unrelated_client_probe_response() {
        let error = r#"{"id":"x","error":{"code":"invalid_request","message":"nope"}}"#;
        assert_eq!(parse_herdr_client_attached(error), None);
        let other = r#"{"id":"x","result":{"type":"notification_show","reason":"set"}}"#;
        assert_eq!(parse_herdr_client_attached(other), None);
    }

    #[test]
    fn parses_server_status() {
        assert!(parse_herdr_server_status("status: running\ncompatible: yes"));
        assert!(!parse_herdr_server_status("status: stopped"));
    }

    #[test]
    fn finds_workspace_for_matching_cwd() {
        let snapshot = r#"{
            "snapshot": {
                "panes": [
                    {"cwd":"D:\\Projects\\SessionHub\\","workspace_id":"wB"}
                ]
            }
        }"#;
        assert_eq!(
            parse_herdr_workspace_for_cwd(snapshot, "d:/projects/sessionhub"),
            Some("wB".to_string())
        );
    }

    #[test]
    fn ignores_workspace_when_cwd_does_not_match() {
        let snapshot = r#"{"snapshot":{"panes":[{"cwd":"D:\\other","workspace_id":"wA"}]}}"#;
        assert_eq!(parse_herdr_workspace_for_cwd(snapshot, "D:/session_hub"), None);
    }

    #[test]
    fn finds_workspace_when_pane_cwd_is_inside_project() {
        let snapshot = r#"{"snapshot":{"panes":[{"cwd":"D:\\Projects\\SessionHub\\src","workspace_id":"wB"}]}}"#;
        assert_eq!(
            parse_herdr_workspace_for_cwd(snapshot, "D:/Projects/SessionHub"),
            Some("wB".to_string())
        );
    }

    #[test]
    fn tolerates_invalid_terminal_title_when_finding_workspace() {
        let snapshot = r#"{"panes":[{"cwd":"D:\\Projects\\SessionHub","terminal_title":"broken "title","workspace_id":"wB"}]}"#;
        assert_eq!(
            parse_herdr_workspace_for_cwd(snapshot, "D:/Projects/SessionHub"),
            Some("wB".to_string())
        );
    }
}
