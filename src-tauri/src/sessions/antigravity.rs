use std::collections::{HashMap, HashSet};
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

use rusqlite::Connection;

use crate::db::read_session_meta;
use crate::sessions::dir_mtime_secs;
use crate::types::*;

/// Antigravity 的三個形態子目錄（皆位於 `<antigravity_root>` 之下）
const ANTIGRAVITY_FLAVORS: &[&str] = &["antigravity", "antigravity-cli", "antigravity-ide"];

fn transcript_path(conversation_dir: &Path) -> PathBuf {
    conversation_dir
        .join(".system_generated")
        .join("logs")
        .join("transcript.jsonl")
}

fn summaries_pb_path(flavor_root: &Path) -> PathBuf {
    flavor_root.join("agyhub_summaries_proto.pb")
}

/// 單一 conversation 的標題／workspace 查表結果
#[derive(Debug, Default, Clone)]
struct SummaryEntry {
    title: Option<String>,
    workspace: Option<String>,
}

/// 讀 varint（protobuf 變長整數）
fn read_varint(data: &[u8], pos: usize) -> Option<(u64, usize)> {
    let mut result: u64 = 0;
    let mut shift = 0u32;
    let mut cursor = pos;
    loop {
        let byte = *data.get(cursor)?;
        result |= ((byte & 0x7f) as u64) << shift;
        cursor += 1;
        if byte & 0x80 == 0 {
            break;
        }
        shift += 7;
        if shift >= 64 {
            return None;
        }
    }
    Some((result, cursor))
}

/// 解析單層 protobuf message，回傳 (field_number, wire_type, payload) 清單。
/// payload 對 wire_type=2（length-delimited）是原始 bytes；其餘 wire type 直接略過細節，只用來跳過欄位。
fn parse_fields(data: &[u8]) -> Vec<(u64, &[u8])> {
    let mut fields = Vec::new();
    let mut pos = 0usize;
    while pos < data.len() {
        let Some((tag, next)) = read_varint(data, pos) else {
            break;
        };
        let field_number = tag >> 3;
        let wire_type = tag & 0x7;
        pos = next;
        match wire_type {
            0 => {
                // varint：跳過
                let Some((_, next)) = read_varint(data, pos) else {
                    break;
                };
                pos = next;
            }
            1 => {
                // fixed64
                if pos + 8 > data.len() {
                    break;
                }
                pos += 8;
            }
            2 => {
                // length-delimited：字串／子訊息／bytes
                let Some((len, next)) = read_varint(data, pos) else {
                    break;
                };
                let len = len as usize;
                if next + len > data.len() {
                    break;
                }
                fields.push((field_number, &data[next..next + len]));
                pos = next + len;
            }
            5 => {
                // fixed32
                if pos + 4 > data.len() {
                    break;
                }
                pos += 4;
            }
            _ => break,
        }
    }
    fields
}

/// 從 workspace 詳情子訊息（field 9）擷取第一個 `file:///` 開頭的字串作為 workspace 路徑
fn extract_workspace_from_detail_field9(payload: &[u8]) -> Option<String> {
    for (_, bytes) in parse_fields(payload) {
        if let Ok(text) = std::str::from_utf8(bytes) {
            if text.starts_with("file:///") {
                return Some(text.to_string());
            }
        }
    }
    None
}

/// URL-decode 一個以 `file:///` 開頭的路徑字串（只處理 `%XX` escape，不含 scheme 轉換）
fn url_decode(input: &str) -> String {
    let bytes = input.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let Ok(hex) = std::str::from_utf8(&bytes[i + 1..i + 3]) {
                if let Ok(value) = u8::from_str_radix(hex, 16) {
                    out.push(value);
                    i += 3;
                    continue;
                }
            }
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).to_string()
}

/// 將 `file:///h:/Code/...` 形式的 workspace URL 轉為一般路徑字串
fn workspace_url_to_path(url: &str) -> String {
    let decoded = url_decode(url);
    decoded
        .strip_prefix("file:///")
        .map(|rest| rest.to_string())
        .unwrap_or(decoded)
}

/// 輕量解析 `agyhub_summaries_proto.pb`：頂層 repeated field 1 為單一 entry 訊息，
/// entry 內 field 1 = conversation UUID 字串，field 2 = 詳情子訊息（field 1 = 標題字串，field 9 = workspace 詳情子訊息）。
/// 欄位規律經實測樣本確認，屬輕量查表用途，不追求完整 schema 還原。
fn parse_summaries_proto(path: &Path) -> HashMap<String, SummaryEntry> {
    let mut map = HashMap::new();
    let Ok(data) = fs::read(path) else {
        return map;
    };

    for (field_number, entry_bytes) in parse_fields(&data) {
        if field_number != 1 {
            continue;
        }
        let entry_fields = parse_fields(entry_bytes);
        let uuid = entry_fields
            .iter()
            .find(|(n, _)| *n == 1)
            .and_then(|(_, bytes)| std::str::from_utf8(bytes).ok())
            .map(|s| s.to_string());
        let Some(uuid) = uuid else {
            continue;
        };

        let detail_bytes = entry_fields.iter().find(|(n, _)| *n == 2).map(|(_, b)| *b);
        let mut summary = SummaryEntry::default();
        if let Some(detail_bytes) = detail_bytes {
            let detail_fields = parse_fields(detail_bytes);
            summary.title = detail_fields
                .iter()
                .find(|(n, _)| *n == 1)
                .and_then(|(_, bytes)| std::str::from_utf8(bytes).ok())
                .map(|s| s.to_string());
            summary.workspace = detail_fields
                .iter()
                .find(|(n, _)| *n == 9)
                .and_then(|(_, bytes)| extract_workspace_from_detail_field9(bytes))
                .map(|url| workspace_url_to_path(&url));
        }

        map.insert(uuid, summary);
    }

    map
}

/// transcript.jsonl 中一行的最小欄位子集
#[derive(Debug, serde::Deserialize)]
struct TranscriptLine {
    #[serde(default)]
    r#type: Option<String>,
    #[serde(default)]
    created_at: Option<String>,
    #[serde(default)]
    content: Option<String>,
}

/// 從 transcript 首則 USER_INPUT 的 `<USER_REQUEST>` 內文擷取前段文字作為標題 fallback
fn extract_user_request_title(content: &str) -> Option<String> {
    let start_tag = "<USER_REQUEST>";
    let end_tag = "</USER_REQUEST>";
    let start = content.find(start_tag)? + start_tag.len();
    let end = content[start..].find(end_tag)? + start;
    let text = content[start..end].trim();
    if text.is_empty() {
        return None;
    }
    const MAX_LEN: usize = 100;
    let truncated: String = text.chars().take(MAX_LEN).collect();
    Some(truncated)
}

struct TranscriptSummary {
    first_request_title: Option<String>,
    created_at: Option<String>,
    updated_at: Option<String>,
    has_events: bool,
}

fn scan_transcript(path: &Path) -> TranscriptSummary {
    let mut summary = TranscriptSummary {
        first_request_title: None,
        created_at: None,
        updated_at: None,
        has_events: false,
    };

    let Ok(file) = fs::File::open(path) else {
        return summary;
    };
    let reader = BufReader::new(file);

    for line in reader.lines().map_while(Result::ok) {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        summary.has_events = true;

        let Ok(entry) = serde_json::from_str::<TranscriptLine>(trimmed) else {
            continue;
        };

        if summary.created_at.is_none() {
            summary.created_at = entry.created_at.clone();
        }
        if entry.created_at.is_some() {
            summary.updated_at = entry.created_at.clone();
        }

        if summary.first_request_title.is_none() && entry.r#type.as_deref() == Some("USER_INPUT") {
            if let Some(content) = &entry.content {
                summary.first_request_title = extract_user_request_title(content);
            }
        }
    }

    summary
}

fn conversation_id_from_dir(dir: &Path) -> String {
    dir.file_name()
        .map(|value| value.to_string_lossy().to_string())
        .unwrap_or_else(|| dir.to_string_lossy().to_string())
}

fn fallback_dir_mtime_rfc3339(dir: &Path) -> Option<String> {
    let secs = dir_mtime_secs(dir);
    if secs <= 0 {
        return None;
    }
    chrono::DateTime::<chrono::Utc>::from_timestamp(secs, 0)
        .map(|dt| dt.to_rfc3339_opts(chrono::SecondsFormat::Secs, true))
}

fn parse_conversation(
    conversation_dir: &Path,
    is_cli_flavor: bool,
    summaries: &HashMap<String, SummaryEntry>,
    meta: SessionMeta,
) -> SessionInfo {
    let conversation_id = conversation_id_from_dir(conversation_dir);
    let transcript = scan_transcript(&transcript_path(conversation_dir));
    let fallback_mtime = fallback_dir_mtime_rfc3339(conversation_dir);

    let summary_entry = if is_cli_flavor {
        None
    } else {
        summaries.get(&conversation_id)
    };

    let title = summary_entry
        .and_then(|entry| entry.title.clone())
        .filter(|title| !title.trim().is_empty())
        .or_else(|| transcript.first_request_title.clone());

    let cwd = summary_entry.and_then(|entry| entry.workspace.clone());

    let created_at = transcript.created_at.clone().or_else(|| fallback_mtime.clone());
    let updated_at = transcript
        .updated_at
        .clone()
        .or_else(|| transcript.created_at.clone())
        .or_else(|| fallback_mtime.clone());

    let parse_error = !transcript.has_events;

    SessionInfo {
        id: conversation_id,
        provider: ANTIGRAVITY_PROVIDER.to_string(),
        cwd,
        repo_root: None,
        repo_name: None,
        git_branch: None,
        summary: title,
        summary_count: None,
        created_at,
        updated_at,
        session_dir: conversation_dir.to_string_lossy().to_string(),
        parse_error,
        is_archived: false,
        notes: meta.notes,
        tags: meta.tags,
        has_plan: false,
        has_events: transcript.has_events,
    }
}

fn collect_conversation_dirs(brain_root: &Path) -> Vec<PathBuf> {
    let Ok(entries) = fs::read_dir(brain_root) else {
        return Vec::new();
    };

    let mut dirs = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        if transcript_path(&path).is_file() {
            dirs.push(path);
        }
    }
    dirs.sort();
    dirs
}

fn scan_flavor(
    antigravity_root: &Path,
    flavor: &str,
    connection: &Connection,
) -> Result<Vec<SessionInfo>, String> {
    let flavor_root = antigravity_root.join(flavor);
    let brain_root = flavor_root.join("brain");
    if !brain_root.exists() {
        return Ok(Vec::new());
    }

    let is_cli_flavor = flavor == "antigravity-cli";
    let summaries = if is_cli_flavor {
        HashMap::new()
    } else {
        parse_summaries_proto(&summaries_pb_path(&flavor_root))
    };

    let mut sessions = Vec::new();
    for conversation_dir in collect_conversation_dirs(&brain_root) {
        let mut session = parse_conversation(
            &conversation_dir,
            is_cli_flavor,
            &summaries,
            SessionMeta {
                notes: None,
                tags: Vec::new(),
            },
        );
        let meta = read_session_meta(connection, &session.id)?;
        session.notes = meta.notes;
        session.tags = meta.tags;
        sessions.push(session);
    }

    Ok(sessions)
}

pub(crate) fn build_antigravity_session_mtimes(sessions: &[SessionInfo]) -> HashMap<String, i64> {
    sessions
        .iter()
        .map(|session| {
            (
                session.session_dir.clone(),
                dir_mtime_secs(Path::new(&session.session_dir)),
            )
        })
        .collect()
}

pub(crate) fn scan_antigravity_sessions_internal(
    antigravity_root: &Path,
    _show_archived: bool,
    connection: &Connection,
) -> Result<Vec<SessionInfo>, String> {
    let mut sessions = Vec::new();
    for flavor in ANTIGRAVITY_FLAVORS {
        match scan_flavor(antigravity_root, flavor, connection) {
            Ok(flavor_sessions) => sessions.extend(flavor_sessions),
            Err(error) => {
                eprintln!("antigravity flavor '{flavor}' scan error (ignored): {error}");
            }
        }
    }
    Ok(sessions)
}

pub(crate) fn scan_antigravity_incremental_internal(
    antigravity_root: &Path,
    show_archived: bool,
    connection: &Connection,
    cache: &mut ProviderCache,
) -> Result<(), String> {
    // 每個 conversation 目錄可能含多個檔案（transcript/metadata），逐一比對 mtime 成本不低於重新掃描全部 flavor，
    // 因此以整體重掃 + 快取比對取代逐檔比對，維持與其他 provider 相同的介面。
    let fresh_sessions = scan_antigravity_sessions_internal(antigravity_root, show_archived, connection)?;
    let mut current_ids: HashSet<String> = HashSet::new();

    for mut session in fresh_sessions {
        let cache_key = session.session_dir.clone();
        let current_mtime = dir_mtime_secs(Path::new(&cache_key));
        let cached_mtime = cache.session_mtimes.get(&cache_key).copied().unwrap_or(-1);

        current_ids.insert(session.id.clone());

        if current_mtime != cached_mtime {
            let meta = read_session_meta(connection, &session.id)?;
            session.notes = meta.notes;
            session.tags = meta.tags;
            if let Some(pos) = cache.sessions.iter().position(|s| s.id == session.id) {
                cache.sessions[pos] = session;
            } else {
                cache.sessions.push(session);
            }
            cache.session_mtimes.insert(cache_key, current_mtime);
        }
    }

    cache.sessions.retain(|session| current_ids.contains(&session.id));
    let current_dirs: HashSet<String> = cache
        .sessions
        .iter()
        .map(|session| session.session_dir.clone())
        .collect();
    cache
        .session_mtimes
        .retain(|session_dir, _| current_dirs.contains(session_dir));

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_user_request_title_returns_trimmed_text() {
        let content = "<USER_REQUEST>\n你好，請幫我做事\n</USER_REQUEST>\n<ADDITIONAL_METADATA>ignore</ADDITIONAL_METADATA>";
        let title = extract_user_request_title(content);
        assert_eq!(title.as_deref(), Some("你好，請幫我做事"));
    }

    #[test]
    fn extract_user_request_title_returns_none_without_tags() {
        assert_eq!(extract_user_request_title("no tags here"), None);
    }

    #[test]
    fn url_decode_handles_percent_encoding() {
        let decoded = url_decode("file:///h:/Code/DIY/Sketchup%20extension/Style%20Engine");
        assert_eq!(decoded, "file:///h:/Code/DIY/Sketchup extension/Style Engine");
    }

    #[test]
    fn workspace_url_to_path_strips_file_scheme() {
        let path = workspace_url_to_path("file:///h:/Code/DIY/foo");
        assert_eq!(path, "h:/Code/DIY/foo");
    }

    #[test]
    fn parse_summaries_proto_returns_empty_map_for_missing_file() {
        let map = parse_summaries_proto(Path::new("/nonexistent/agyhub_summaries_proto.pb"));
        assert!(map.is_empty());
    }

    #[test]
    fn collect_conversation_dirs_returns_empty_for_missing_root() {
        let dirs = collect_conversation_dirs(Path::new("/nonexistent/brain"));
        assert!(dirs.is_empty());
    }

    #[test]
    #[ignore = "requires real ~/.gemini data on this machine; run manually with --ignored"]
    fn manual_smoke_scan_real_antigravity_root() {
        let user_profile = std::env::var("USERPROFILE").expect("USERPROFILE");
        let antigravity_root = Path::new(&user_profile).join(".gemini");
        let conn = rusqlite::Connection::open_in_memory().expect("open in-memory db");
        crate::db::init_db(&conn).expect("init db");
        let sessions =
            scan_antigravity_sessions_internal(&antigravity_root, false, &conn).expect("scan");
        println!("Found {} antigravity sessions", sessions.len());
        for s in sessions.iter().take(5) {
            println!(
                "id={} summary={:?} cwd={:?} updated_at={:?} parse_error={}",
                s.id, s.summary, s.cwd, s.updated_at, s.parse_error
            );
        }
        assert!(!sessions.is_empty(), "expected to find antigravity sessions");
    }
}
