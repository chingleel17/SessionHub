use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Instant;

use crate::db::{
    instant_from_unix_secs, load_scan_state_from_db, load_session_mtimes_from_db,
    load_sessions_cache_from_db, open_db_connection_and_init, persist_provider_cache,
};
use crate::settings::{resolve_copilot_root, resolve_opencode_root};
use crate::types::*;

pub mod copilot;
pub mod opencode;

pub(crate) use copilot::*;
pub(crate) use opencode::*;

pub(crate) fn get_sessions_internal(
    root_dir: Option<String>,
    opencode_root: Option<String>,
    show_archived: Option<bool>,
    enabled_providers: Option<Vec<String>>,
    force_full: Option<bool>,
    scan_cache: &ScanCache,
) -> Result<Vec<SessionInfo>, String> {
    let resolved_copilot = resolve_copilot_root(root_dir.as_deref())?;
    let resolved_opencode = resolve_opencode_root(opencode_root.as_deref()).ok();
    let providers = enabled_providers.unwrap_or_else(default_enabled_providers);
    let show_archived = show_archived.unwrap_or(false);
    let force = force_full.unwrap_or(false);

    let connection = open_db_connection_and_init()?;

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
                if let Err(error) =
                    persist_provider_cache(&connection, OPENCODE_PROVIDER, cache)
                {
                    eprintln!("failed to persist opencode cache: {error}");
                }
            }
        }
    }

    all_sessions.sort_by(|left, right| right.updated_at.cmp(&left.updated_at));

    Ok(all_sessions)
}
