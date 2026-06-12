use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Instant;

use rusqlite::Connection;

use crate::db::{
    instant_from_unix_secs, load_scan_state_from_db, load_session_mtimes_from_db,
    load_sessions_cache_from_db, persist_provider_cache,
};
use crate::settings::{
    resolve_claude_root, resolve_codex_root, resolve_copilot_root, resolve_opencode_root,
};
use crate::types::*;

pub mod claude;
pub mod codex;
pub mod copilot;
pub mod git;
pub mod opencode;

pub(crate) use claude::*;
pub(crate) use codex::*;
pub(crate) use copilot::*;
pub(crate) use git::*;
pub(crate) use opencode::*;

pub(crate) fn get_sessions_internal(
    root_dir: Option<String>,
    opencode_root: Option<String>,
    codex_root: Option<String>,
    claude_root: Option<String>,
    show_archived: Option<bool>,
    enabled_providers: Option<Vec<String>>,
    force_full: Option<bool>,
    scan_cache: &ScanCache,
    connection: &Connection,
) -> Result<Vec<SessionInfo>, String> {
    let resolved_copilot = resolve_copilot_root(root_dir.as_deref())?;
    let resolved_opencode = resolve_opencode_root(opencode_root.as_deref()).ok();
    let resolved_codex = resolve_codex_root(codex_root.as_deref()).ok();
    let resolved_claude = resolve_claude_root(claude_root.as_deref()).ok();
    let providers = enabled_providers.unwrap_or_else(default_enabled_providers);
    let show_archived = show_archived.unwrap_or(false);
    let force = force_full.unwrap_or(false);

    let mut all_sessions: Vec<SessionInfo> = Vec::new();

    // ── Copilot provider ──────────────────────────────────────────────────────
    if providers.iter().any(|p| p == "copilot") {
        let mut copilot_guard = scan_cache
            .copilot
            .lock()
            .map_err(|_| "failed to lock copilot scan cache".to_string())?;

        if copilot_guard.is_none() {
            let sessions = load_sessions_cache_from_db(&connection, Some(COPILOT_PROVIDER))
                .unwrap_or_default();
            let mtimes =
                load_session_mtimes_from_db(&connection, COPILOT_PROVIDER).unwrap_or_default();
            let (last_full_scan_at, last_cursor) =
                load_scan_state_from_db(&connection, COPILOT_PROVIDER).unwrap_or((0, 0));
            *copilot_guard = Some(ProviderCache {
                sessions,
                session_mtimes: mtimes,
                last_full_scan_at: instant_from_unix_secs(last_full_scan_at),
                last_cursor,
            });
        }

        let copilot_force_full = force
            || copilot_guard
                .as_ref()
                .map(|cache| cache.sessions.is_empty())
                .unwrap_or(true);

        if should_full_scan(&copilot_guard, copilot_force_full) {
            let mut sessions =
                scan_session_dir(&resolved_copilot.join("session-state"), false, &connection)?;
            if show_archived {
                sessions.extend(scan_session_dir(
                    &resolved_copilot.join("session-state-archive"),
                    true,
                    &connection,
                )?);
            }

            let mut mtimes = HashMap::new();
            for session in &sessions {
                let dir = PathBuf::from(&session.session_dir);
                mtimes.insert(session.id.clone(), dir_mtime_secs(&dir));
            }

            *copilot_guard = Some(ProviderCache {
                sessions: sessions.clone(),
                session_mtimes: mtimes,
                last_full_scan_at: Instant::now(),
                last_cursor: 0,
            });

            all_sessions.extend(sessions);
        } else {
            let cache = copilot_guard
                .as_mut()
                .expect("cache is Some after should_full_scan check");
            scan_copilot_incremental_internal(
                &resolved_copilot.join("session-state"),
                false,
                &connection,
                cache,
            )?;
            if show_archived {
                scan_copilot_incremental_internal(
                    &resolved_copilot.join("session-state-archive"),
                    true,
                    &connection,
                    cache,
                )?;
            }
            all_sessions.extend(cache.sessions.iter().cloned());
        }

        if let Some(cache) = copilot_guard.as_ref() {
            if let Err(error) = persist_provider_cache(&connection, COPILOT_PROVIDER, cache) {
                eprintln!("failed to persist copilot cache: {error}");
            }
        }
    }

    // ── OpenCode provider ─────────────────────────────────────────────────────
    if providers.iter().any(|p| p == "opencode") {
        if let Some(oc_root) = &resolved_opencode {
            let mut oc_guard = scan_cache
                .opencode
                .lock()
                .map_err(|_| "failed to lock opencode scan cache".to_string())?;

            if oc_guard.is_none() {
                let sessions = load_sessions_cache_from_db(&connection, Some(OPENCODE_PROVIDER))
                    .unwrap_or_default();
                let (last_full_scan_at, last_cursor) =
                    load_scan_state_from_db(&connection, OPENCODE_PROVIDER).unwrap_or((0, 0));
                *oc_guard = Some(ProviderCache {
                    sessions,
                    session_mtimes: HashMap::new(),
                    last_full_scan_at: instant_from_unix_secs(last_full_scan_at),
                    last_cursor,
                });
            }

            let opencode_force_full = force
                || oc_guard
                    .as_ref()
                    .map(|cache| cache.sessions.is_empty())
                    .unwrap_or(true);

            if should_full_scan(&oc_guard, opencode_force_full) {
                match scan_opencode_sessions_internal(oc_root, show_archived, &connection) {
                    Ok(sessions) => {
                        let max_cursor = get_opencode_max_cursor(oc_root).unwrap_or(0);
                        *oc_guard = Some(ProviderCache {
                            sessions: sessions.clone(),
                            session_mtimes: HashMap::new(),
                            last_full_scan_at: Instant::now(),
                            last_cursor: max_cursor,
                        });
                        all_sessions.extend(sessions);
                    }
                    Err(error) => {
                        eprintln!("opencode provider error (ignored): {error}");
                    }
                }
            } else {
                let cache = oc_guard
                    .as_mut()
                    .expect("cache is Some after should_full_scan check");
                if let Err(e) =
                    scan_opencode_incremental_internal(oc_root, show_archived, &connection, cache)
                {
                    eprintln!("opencode incremental scan error (ignored): {e}");
                }
                all_sessions.extend(cache.sessions.iter().cloned());
            }

            if let Some(cache) = oc_guard.as_ref() {
                if let Err(error) = persist_provider_cache(&connection, OPENCODE_PROVIDER, cache) {
                    eprintln!("failed to persist opencode cache: {error}");
                }
            }
        }
    }

    // ── Codex provider ────────────────────────────────────────────────────────
    if providers.iter().any(|p| p == CODEX_PROVIDER) {
        if let Some(codex_root) = &resolved_codex {
            let mut codex_guard = scan_cache
                .codex
                .lock()
                .map_err(|_| "failed to lock codex scan cache".to_string())?;

            if codex_guard.is_none() {
                let sessions = load_sessions_cache_from_db(&connection, Some(CODEX_PROVIDER))
                    .unwrap_or_default();
                let mtimes =
                    load_session_mtimes_from_db(&connection, CODEX_PROVIDER).unwrap_or_default();
                let (last_full_scan_at, last_cursor) =
                    load_scan_state_from_db(&connection, CODEX_PROVIDER).unwrap_or((0, 0));
                *codex_guard = Some(ProviderCache {
                    sessions,
                    session_mtimes: mtimes,
                    last_full_scan_at: instant_from_unix_secs(last_full_scan_at),
                    last_cursor,
                });
            }

            let codex_force_full = force
                || codex_guard
                    .as_ref()
                    .map(|cache| cache.sessions.is_empty())
                    .unwrap_or(true);

            if should_full_scan(&codex_guard, codex_force_full) {
                match scan_codex_sessions_internal(codex_root, show_archived, &connection) {
                    Ok(sessions) => {
                        let mtimes = build_codex_session_mtimes(&sessions);
                        *codex_guard = Some(ProviderCache {
                            sessions: sessions.clone(),
                            session_mtimes: mtimes,
                            last_full_scan_at: Instant::now(),
                            last_cursor: 0,
                        });
                        all_sessions.extend(sessions);
                    }
                    Err(error) => {
                        eprintln!("codex provider error (ignored): {error}");
                    }
                }
            } else {
                let cache = codex_guard
                    .as_mut()
                    .expect("cache is Some after should_full_scan check");
                if let Err(error) =
                    scan_codex_incremental_internal(codex_root, show_archived, &connection, cache)
                {
                    eprintln!("codex incremental scan error (ignored): {error}");
                }
                all_sessions.extend(cache.sessions.iter().cloned());
            }

            if let Some(cache) = codex_guard.as_ref() {
                if let Err(error) = persist_provider_cache(&connection, CODEX_PROVIDER, cache) {
                    eprintln!("failed to persist codex cache: {error}");
                }
            }
        }
    }

    // ── Claude provider ───────────────────────────────────────────────────────
    if providers.iter().any(|p| p == CLAUDE_PROVIDER) {
        if let Some(claude_root) = &resolved_claude {
            let mut claude_guard = scan_cache
                .claude
                .lock()
                .map_err(|_| "failed to lock claude scan cache".to_string())?;

            if claude_guard.is_none() {
                let sessions = load_sessions_cache_from_db(&connection, Some(CLAUDE_PROVIDER))
                    .unwrap_or_default();
                let mtimes =
                    load_session_mtimes_from_db(&connection, CLAUDE_PROVIDER).unwrap_or_default();
                let (last_full_scan_at, last_cursor) =
                    load_scan_state_from_db(&connection, CLAUDE_PROVIDER).unwrap_or((0, 0));
                *claude_guard = Some(ProviderCache {
                    sessions,
                    session_mtimes: mtimes,
                    last_full_scan_at: instant_from_unix_secs(last_full_scan_at),
                    last_cursor,
                });
            }

            let claude_force_full = force
                || claude_guard
                    .as_ref()
                    .map(|cache| cache.sessions.is_empty())
                    .unwrap_or(true);

            if should_full_scan(&claude_guard, claude_force_full) {
                match scan_claude_sessions_internal(claude_root, show_archived, &connection) {
                    Ok(sessions) => {
                        let mtimes = build_claude_session_mtimes(&sessions);
                        *claude_guard = Some(ProviderCache {
                            sessions: sessions.clone(),
                            session_mtimes: mtimes,
                            last_full_scan_at: Instant::now(),
                            last_cursor: 0,
                        });
                        all_sessions.extend(sessions);
                    }
                    Err(error) => {
                        eprintln!("claude provider error (ignored): {error}");
                    }
                }
            } else {
                let cache = claude_guard
                    .as_mut()
                    .expect("cache is Some after should_full_scan check");
                if let Err(error) =
                    scan_claude_incremental_internal(claude_root, show_archived, &connection, cache)
                {
                    eprintln!("claude incremental scan error (ignored): {error}");
                }
                all_sessions.extend(cache.sessions.iter().cloned());
            }

            if let Some(cache) = claude_guard.as_ref() {
                if let Err(error) = persist_provider_cache(&connection, CLAUDE_PROVIDER, cache) {
                    eprintln!("failed to persist claude cache: {error}");
                }
            }
        }
    }

    enrich_sessions_with_git_metadata(&mut all_sessions);
    all_sessions.sort_by(|left, right| right.updated_at.cmp(&left.updated_at));

    Ok(all_sessions)
}
