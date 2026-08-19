use std::io::ErrorKind;
use std::process::{Command, Output};

use serde_json::Value;

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
    Command::new("herdr")
        .args(args)
        .output()
        .map_err(|error| match error.kind() {
            ErrorKind::NotFound => {
                "herdr 未偵測到，請先安裝 herdr 並確認已加入 PATH".to_string()
            }
            _ => format!("無法執行 herdr：{error}"),
        })
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
