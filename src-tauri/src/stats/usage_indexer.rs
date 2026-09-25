use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

use rusqlite::Connection;

use crate::db::{
    replace_usage_sessions_batch, update_usage_ingestion_status, usage_ingestion_state,
};
use crate::types::{AnalyticsCoverageStatus, SessionInfo, UsageIngestionStateRecord};

use super::{
    build_claude_usage_events, build_opencode_usage_events, claude_usage_coverage,
    extract_codex_usage, extract_copilot_usage, refresh_missing_model_prices,
};

const PARSER_VERSION: i64 = 1;
const CLAUDE_PARSER_VERSION: i64 = 3;
const CODEX_PARSER_VERSION: i64 = 2;
const COPILOT_PARSER_VERSION: i64 = 2;
const OPENCODE_PARSER_VERSION: i64 = 3;
const MAX_USAGE_SESSIONS_PER_BATCH: usize = 50;

pub(crate) fn index_usage_batch_internal(
    connection: &Connection,
    sessions: &[SessionInfo],
) -> Result<usize, String> {
    let mut indexed = 0;
    let mut examined = 0;
    let mut replacements = Vec::new();
    for session in sessions {
        if examined >= MAX_USAGE_SESSIONS_PER_BATCH {
            break;
        }
        let source_path = PathBuf::from(&session.session_dir);
        let source_identity = source_path.to_string_lossy().to_string();
        let parser_version = parser_version_for_provider(&session.provider);
        let previous =
            usage_ingestion_state(connection, &session.provider, &session.id, &source_identity)?;
        let Some(fingerprint) = usage_source_fingerprint(&session.provider, &source_path) else {
            examined += 1;
            update_usage_ingestion_status(
                connection,
                &ingestion_state(
                    session,
                    source_identity,
                    None,
                    "error",
                    Some("Usage source is unavailable".to_string()),
                ),
            )?;
            continue;
        };
        if previous
            .as_ref()
            .is_some_and(|(previous_fingerprint, version, status)| {
                previous_fingerprint.as_deref() == Some(fingerprint.as_str())
                    && *version == parser_version
                    && status != "pending"
                    && status != "error"
            })
        {
            continue;
        }
        examined += 1;

        if let Some(jsonl_source) = usage_jsonl_source(&session.provider, &source_path) {
            match validate_jsonl_integrity(&jsonl_source) {
                Ok(()) => {}
                Err(JsonlIntegrityIssue::Pending) => {
                    update_usage_ingestion_status(
                        connection,
                        &ingestion_state(
                            session,
                            source_identity,
                            Some(fingerprint),
                            "pending",
                            Some("Usage source ends with an incomplete JSONL record".to_string()),
                        ),
                    )?;
                    continue;
                }
                Err(JsonlIntegrityIssue::Error(reason)) => {
                    update_usage_ingestion_status(
                        connection,
                        &ingestion_state(
                            session,
                            source_identity,
                            Some(fingerprint),
                            "error",
                            Some(reason),
                        ),
                    )?;
                    continue;
                }
            }
        }

        let extraction = (|| -> Result<_, String> {
            Ok(match session.provider.as_str() {
                "claude" => {
                    let events = build_claude_usage_events(&source_path, &session.id)?;
                    let (status, reason) = claude_usage_coverage(&source_path)?;
                    (events, Vec::new(), status, reason)
                }
                "opencode" => (
                    build_opencode_usage_events(&source_path, &session.id)?,
                    Vec::new(),
                    AnalyticsCoverageStatus::Complete,
                    None,
                ),
                "copilot" => {
                    let (events, summaries) = extract_copilot_usage(&source_path, &session.id)?;
                    (
                    events,
                    summaries,
                    AnalyticsCoverageStatus::Partial,
                    Some("Copilot assistant events expose output; shutdown usage is a session summary".to_string()),
                )
                }
                "codex" => {
                    let extraction = extract_codex_usage(&source_path, &session.id)?;
                    (
                        extraction.events,
                        extraction.session_summaries,
                        extraction.status,
                        extraction.reason,
                    )
                }
                "antigravity" => (
                    Vec::new(),
                    Vec::new(),
                    AnalyticsCoverageStatus::Unsupported,
                    Some("Token analytics is not supported for Antigravity".to_string()),
                ),
                _ => (
                    Vec::new(),
                    Vec::new(),
                    AnalyticsCoverageStatus::Unsupported,
                    Some("Provider usage source is not recognized".to_string()),
                ),
            })
        })();
        let (events, summaries, status, reason) = match extraction {
            Ok(result) => result,
            Err(error) => {
                update_usage_ingestion_status(
                    connection,
                    &ingestion_state(
                        session,
                        source_identity,
                        Some(fingerprint),
                        "error",
                        Some(error),
                    ),
                )?;
                continue;
            }
        };
        let status_name = coverage_status_name(&status);
        let summaries = summaries
            .into_iter()
            .map(|mut summary| {
                summary.cwd = session.cwd.clone();
                summary
            })
            .collect::<Vec<_>>();
        let mut events = events;
        for event in &mut events {
            event.cwd = session.cwd.clone();
        }
        let ingestion = ingestion_state(
            session,
            source_identity,
            Some(fingerprint),
            status_name,
            reason,
        );
        replacements.push((ingestion, events, summaries));
        indexed += 1;
    }
    replace_usage_sessions_batch(connection, &replacements)?;
    if let Err(error) = refresh_missing_model_prices(connection) {
        eprintln!("[usage-index] model pricing refresh failed: {error}");
    }
    Ok(indexed)
}

fn ingestion_state(
    session: &SessionInfo,
    source_identity: String,
    source_fingerprint: Option<String>,
    status: &str,
    error_message: Option<String>,
) -> UsageIngestionStateRecord {
    UsageIngestionStateRecord {
        provider: session.provider.clone(),
        session_id: session.id.clone(),
        source_identity,
        source_fingerprint,
        parser_version: parser_version_for_provider(&session.provider),
        integrity_status: status.to_string(),
        ingestion_status: status.to_string(),
        error_message,
    }
}

fn parser_version_for_provider(provider: &str) -> i64 {
    match provider {
        "claude" => CLAUDE_PARSER_VERSION,
        "codex" => CODEX_PARSER_VERSION,
        "copilot" => COPILOT_PARSER_VERSION,
        "opencode" => OPENCODE_PARSER_VERSION,
        _ => PARSER_VERSION,
    }
}

enum JsonlIntegrityIssue {
    Pending,
    Error(String),
}

fn usage_jsonl_source(provider: &str, session_path: &Path) -> Option<PathBuf> {
    match provider {
        "copilot" => Some(session_path.join("events.jsonl")),
        "claude" => Some(session_path.to_path_buf()),
        _ => None,
    }
}

fn validate_jsonl_integrity(path: &Path) -> Result<(), JsonlIntegrityIssue> {
    let content = std::fs::read(path).map_err(|error| {
        JsonlIntegrityIssue::Error(format!("failed to read usage source: {error}"))
    })?;
    let text = std::str::from_utf8(&content).map_err(|error| {
        JsonlIntegrityIssue::Error(format!("usage source is not UTF-8: {error}"))
    })?;
    let lines = text.lines().collect::<Vec<_>>();
    for (index, line) in lines.iter().enumerate() {
        if line.trim().is_empty() || serde_json::from_str::<serde_json::Value>(line).is_ok() {
            continue;
        }
        if index + 1 == lines.len() && !content.ends_with(b"\n") {
            return Err(JsonlIntegrityIssue::Pending);
        }
        return Err(JsonlIntegrityIssue::Error(format!(
            "invalid JSONL record at line {}",
            index + 1
        )));
    }
    Ok(())
}

fn coverage_status_name(status: &AnalyticsCoverageStatus) -> &'static str {
    match status {
        AnalyticsCoverageStatus::Complete => "complete",
        AnalyticsCoverageStatus::Partial => "partial",
        AnalyticsCoverageStatus::Pending => "pending",
        AnalyticsCoverageStatus::Unsupported => "unsupported",
        AnalyticsCoverageStatus::Error => "error",
        AnalyticsCoverageStatus::SummaryOnly => "summary_only",
    }
}

fn usage_source_fingerprint(provider: &str, session_path: &Path) -> Option<String> {
    let source = match provider {
        "copilot" => session_path.join("events.jsonl"),
        "claude" => session_path.to_path_buf(),
        "opencode" => {
            let storage_root = session_path.parent()?.parent()?;
            let database = storage_root.parent()?.join("opencode.db");
            if database.exists() {
                database
            } else {
                session_path.to_path_buf()
            }
        }
        "codex" | "antigravity" => session_path.to_path_buf(),
        _ => return None,
    };
    let metadata = std::fs::metadata(&source).ok()?;
    let modified_nanos = metadata
        .modified()
        .ok()?
        .duration_since(UNIX_EPOCH)
        .ok()?
        .as_nanos();
    Some(format!("{}:{modified_nanos}", metadata.len()))
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    use rusqlite::Connection;

    use crate::db::init_db;
    use crate::types::SessionInfo;

    use super::{index_usage_batch_internal, MAX_USAGE_SESSIONS_PER_BATCH};

    fn temp_dir() -> PathBuf {
        let suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock should be valid")
            .as_nanos();
        let path = std::env::temp_dir().join(format!("session-hub-usage-index-{suffix}"));
        fs::create_dir_all(&path).expect("create temp dir");
        path
    }

    fn copilot_session(path: PathBuf, id: String) -> SessionInfo {
        SessionInfo {
            id,
            provider: "copilot".to_string(),
            cwd: Some("D:\\fixture".to_string()),
            repo_root: None,
            repo_name: None,
            git_branch: None,
            summary: None,
            summary_count: None,
            created_at: None,
            updated_at: None,
            session_dir: path.to_string_lossy().to_string(),
            parse_error: false,
            is_archived: false,
            notes: None,
            tags: Vec::new(),
            has_plan: false,
            has_events: true,
        }
    }

    #[test]
    fn indexes_at_most_fifty_sessions_and_skips_unchanged_sources() {
        let root = temp_dir();
        let connection = Connection::open_in_memory().expect("open test db");
        init_db(&connection).expect("initialize test db");
        let fixture = include_str!("../../tests/fixtures/usage/copilot.jsonl");
        let sessions = (0..=MAX_USAGE_SESSIONS_PER_BATCH)
            .map(|index| {
                let id = format!("session-{index}");
                let path = root.join(&id);
                fs::create_dir_all(&path).expect("create session directory");
                fs::write(path.join("events.jsonl"), fixture).expect("write events fixture");
                copilot_session(path, id)
            })
            .collect::<Vec<_>>();

        assert_eq!(
            index_usage_batch_internal(&connection, &sessions).expect("first batch"),
            50
        );
        assert_eq!(
            index_usage_batch_internal(&connection, &sessions).expect("second batch"),
            1
        );
        connection
            .execute(
                "UPDATE usage_ingestion_state SET parser_version = 0 WHERE session_id = 'session-0'",
                [],
            )
            .expect("simulate an older parser version");
        assert_eq!(
            index_usage_batch_internal(&connection, &sessions).expect("rebuild old parser output"),
            1
        );
        let rebuilt_event_count: i64 = connection
            .query_row(
                "SELECT COUNT(*) FROM usage_events WHERE session_id = 'session-0'",
                [],
                |row| row.get(0),
            )
            .expect("count rebuilt events");
        assert_eq!(rebuilt_event_count, 3);
        let revision_after_index = connection
            .query_row(
                "SELECT revision FROM analytics_revision WHERE singleton = 1",
                [],
                |row| row.get::<_, i64>(0),
            )
            .expect("read revision");
        assert_eq!(
            index_usage_batch_internal(&connection, &sessions).expect("unchanged scan"),
            0
        );
        let revision_after_repeat = connection
            .query_row(
                "SELECT revision FROM analytics_revision WHERE singleton = 1",
                [],
                |row| row.get::<_, i64>(0),
            )
            .expect("read revision after unchanged scan");
        assert_eq!(revision_after_repeat, revision_after_index);
        let event_count: i64 = connection
            .query_row("SELECT COUNT(*) FROM usage_events", [], |row| row.get(0))
            .expect("count indexed usage events");
        assert_eq!(event_count, 3 * 51);
        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn keeps_last_complete_events_for_partial_source_and_continues_after_error() {
        let root = temp_dir();
        let database_path = root.join("metadata.db");
        let connection = Connection::open(&database_path).expect("open test db");
        init_db(&connection).expect("initialize test db");
        let fixture = include_str!("../../tests/fixtures/usage/copilot.jsonl");
        let complete_dir = root.join("complete-session");
        fs::create_dir_all(&complete_dir).expect("create complete session directory");
        let events_path = complete_dir.join("events.jsonl");
        fs::write(&events_path, fixture).expect("write complete source");
        let complete = copilot_session(complete_dir.clone(), "complete-session".to_string());

        assert_eq!(
            index_usage_batch_internal(&connection, &[complete.clone()]).expect("initial ingest"),
            1
        );
        assert_eq!(
            fs::read_to_string(&events_path).expect("read original source after indexing"),
            fixture
        );
        let initial_event_count: i64 = connection
            .query_row(
                "SELECT COUNT(*) FROM usage_events WHERE session_id = 'complete-session'",
                [],
                |row| row.get(0),
            )
            .expect("count initial events");
        assert_eq!(initial_event_count, 3);

        fs::write(
            &events_path,
            format!(
                "{fixture}{{\"type\":\"assistant.message\",\"timestamp\":\"2026-04-12T10:00:00Z\""
            ),
        )
        .expect("write partial final record");
        assert_eq!(
            index_usage_batch_internal(&connection, &[complete.clone()]).expect("pending ingest"),
            0
        );
        let retained_event_count: i64 = connection
            .query_row(
                "SELECT COUNT(*) FROM usage_events WHERE session_id = 'complete-session'",
                [],
                |row| row.get(0),
            )
            .expect("count retained events");
        assert_eq!(retained_event_count, initial_event_count);
        let pending_status: String = connection
            .query_row(
                "SELECT ingestion_status FROM usage_ingestion_state WHERE session_id = 'complete-session'",
                [],
                |row| row.get(0),
            )
            .expect("read pending state");
        assert_eq!(pending_status, "pending");

        drop(connection);
        let connection = Connection::open(&database_path).expect("reopen db after interruption");
        init_db(&connection).expect("resume db after interruption");

        let repaired_content = format!(
            "{fixture}{{\"type\":\"assistant.message\",\"timestamp\":\"2026-04-12T10:00:00Z\",\"data\":{{\"outputTokens\":70}}}}\n"
        );
        fs::write(&events_path, repaired_content).expect("finish the interrupted JSONL record");
        assert_eq!(
            index_usage_batch_internal(&connection, &[complete.clone()]).expect("resume ingest"),
            1
        );
        let resumed_event_count: i64 = connection
            .query_row(
                "SELECT COUNT(*) FROM usage_events WHERE session_id = 'complete-session'",
                [],
                |row| row.get(0),
            )
            .expect("count resumed events");
        assert_eq!(resumed_event_count, 4);

        let invalid_dir = root.join("invalid-session");
        fs::create_dir_all(&invalid_dir).expect("create invalid session directory");
        fs::write(invalid_dir.join("events.jsonl"), "{invalid json}\n")
            .expect("write invalid source");
        let invalid = copilot_session(invalid_dir, "invalid-session".to_string());
        let healthy_dir = root.join("healthy-session");
        fs::create_dir_all(&healthy_dir).expect("create healthy session directory");
        fs::write(healthy_dir.join("events.jsonl"), fixture).expect("write healthy source");
        let healthy = copilot_session(healthy_dir, "healthy-session".to_string());
        assert_eq!(
            index_usage_batch_internal(&connection, &[invalid, healthy]).expect("continue batch"),
            1
        );
        let error_status: String = connection
            .query_row(
                "SELECT ingestion_status FROM usage_ingestion_state WHERE session_id = 'invalid-session'",
                [],
                |row| row.get(0),
            )
            .expect("read error state");
        assert_eq!(error_status, "error");
        let retained_after_error: i64 = connection
            .query_row(
                "SELECT COUNT(*) FROM usage_events WHERE session_id = 'complete-session'",
                [],
                |row| row.get(0),
            )
            .expect("count events after other source error");
        assert_eq!(retained_after_error, resumed_event_count);
        let healthy_event_count: i64 = connection
            .query_row(
                "SELECT COUNT(*) FROM usage_events WHERE session_id = 'healthy-session'",
                [],
                |row| row.get(0),
            )
            .expect("count healthy session events");
        assert_eq!(healthy_event_count, 3);
        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn rewrites_and_truncations_replace_the_prior_session_event_set() {
        let root = temp_dir();
        let connection = Connection::open_in_memory().expect("open test db");
        init_db(&connection).expect("initialize test db");
        let fixture = include_str!("../../tests/fixtures/usage/copilot.jsonl");
        let session_dir = root.join("session");
        fs::create_dir_all(&session_dir).expect("create session directory");
        let source = session_dir.join("events.jsonl");
        fs::write(&source, fixture).expect("write full event source");
        let session = copilot_session(session_dir, "rewrite-session".to_string());

        assert_eq!(
            index_usage_batch_internal(&connection, &[session.clone()])
                .expect("index full session"),
            1
        );
        let first_count: i64 = connection
            .query_row(
                "SELECT COUNT(*) FROM usage_events WHERE session_id = 'rewrite-session'",
                [],
                |row| row.get(0),
            )
            .expect("count first event set");
        assert_eq!(first_count, 3);

        let shortened = fixture.lines().take(2).collect::<Vec<_>>().join("\n") + "\n";
        fs::write(&source, shortened).expect("rewrite source with shorter content");
        assert_eq!(
            index_usage_batch_internal(&connection, &[session]).expect("reindex rewritten session"),
            1
        );
        let second_count: i64 = connection
            .query_row(
                "SELECT COUNT(*) FROM usage_events WHERE session_id = 'rewrite-session'",
                [],
                |row| row.get(0),
            )
            .expect("count replacement event set");
        assert_eq!(second_count, 1);
        fs::remove_dir_all(root).ok();
    }
}
