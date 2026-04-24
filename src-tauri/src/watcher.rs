use std::collections::{BTreeSet, HashMap};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::{fs, thread};

use notify::{recommended_watcher, RecursiveMode, Watcher};
use tauri::Emitter;

use crate::provider::{
    build_copilot_watch_snapshot, build_opencode_watch_snapshot, detect_copilot_integration_status,
    detect_opencode_integration_status, emit_provider_refresh, is_relevant_copilot_event,
    is_relevant_opencode_event, matched_bridge_providers, process_provider_bridge_event,
    should_emit_copilot_refresh, should_emit_opencode_refresh,
};
use crate::settings::{
    provider_bridge_dir, resolve_copilot_root, resolve_opencode_root, resolve_provider_bridge_path,
};
use crate::types::*;

use std::time::Instant;

pub(crate) fn create_sessions_watcher(
    app: &tauri::AppHandle,
    root: &Path,
    refresh_state: Arc<Mutex<HashMap<String, Instant>>>,
) -> Result<notify::RecommendedWatcher, String> {
    let app_handle = app.clone();
    let watch_root = root.to_path_buf();
    let session_roots = vec![
        watch_root.join("session-state"),
        watch_root.join("session-state-archive"),
    ];
    let snapshot_state = Arc::new(Mutex::new(build_copilot_watch_snapshot(&watch_root)));
    let last_event: Arc<Mutex<Instant>> = Arc::new(Mutex::new(Instant::now()));
    let debounce_running: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));

    let mut watcher = recommended_watcher(move |result: notify::Result<notify::Event>| {
        if let Ok(event) = result {
            if !is_relevant_copilot_event(&event, &session_roots) {
                return;
            }

            if let Ok(mut ts) = last_event.lock() {
                *ts = Instant::now();
            }

            if debounce_running
                .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
                .is_err()
            {
                return;
            }

            let le = Arc::clone(&last_event);
            let handle = app_handle.clone();
            let running = Arc::clone(&debounce_running);
            let refreshes = Arc::clone(&refresh_state);
            let watched_root = watch_root.clone();
            let tracked_snapshot = Arc::clone(&snapshot_state);
            thread::spawn(move || {
                loop {
                    thread::sleep(Duration::from_millis(COPILOT_DEBOUNCE_MS));
                    let elapsed = le.lock().map(|ts| ts.elapsed()).unwrap_or_default();
                    if elapsed >= Duration::from_millis(COPILOT_DEBOUNCE_MS) {
                        running.store(false, Ordering::SeqCst);
                        match should_emit_copilot_refresh(&watched_root, &tracked_snapshot) {
                            Ok(true) => {
                                let _ =
                                    emit_provider_refresh(&handle, &refreshes, COPILOT_PROVIDER);
                            }
                            Ok(false) => {}
                            Err(error) => {
                                eprintln!("failed to verify copilot watcher refresh: {error}");
                            }
                        }
                        break;
                    }
                }
            });
        }
    })
    .map_err(|error| format!("failed to create session watcher: {error}"))?;

    let session_state_dir = root.join("session-state");
    if session_state_dir.exists() {
        watcher
            .watch(&session_state_dir, RecursiveMode::Recursive)
            .map_err(|error| {
                format!("failed to watch {}: {error}", session_state_dir.display())
            })?;
    }

    let archive_dir = root.join("session-state-archive");
    if archive_dir.exists() {
        watcher
            .watch(&archive_dir, RecursiveMode::Recursive)
            .map_err(|error| format!("failed to watch {}: {error}", archive_dir.display()))?;
    }

    Ok(watcher)
}

pub(crate) fn create_opencode_watcher(
    app: &tauri::AppHandle,
    opencode_root: &Path,
    refresh_state: Arc<Mutex<HashMap<String, Instant>>>,
) -> Result<notify::RecommendedWatcher, String> {
    let session_storage = opencode_root.join("storage").join("session");
    let message_storage = opencode_root.join("storage").join("message");

    if !session_storage.exists() {
        return Err(format!(
            "opencode session storage does not exist at {}",
            session_storage.display()
        ));
    }

    let app_handle = app.clone();
    let watch_root = opencode_root.to_path_buf();
    let snapshot_state = Arc::new(Mutex::new(build_opencode_watch_snapshot(&watch_root)));
    let last_event: Arc<Mutex<Instant>> = Arc::new(Mutex::new(Instant::now()));
    let debounce_running: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));

    let mut watcher = recommended_watcher(move |result: notify::Result<notify::Event>| {
        if let Ok(event) = result {
            if !is_relevant_opencode_event(&event, &watch_root) {
                return;
            }
            if let Ok(mut ts) = last_event.lock() {
                *ts = Instant::now();
            }
            if debounce_running
                .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
                .is_err()
            {
                return;
            }
            let le = Arc::clone(&last_event);
            let handle = app_handle.clone();
            let running = Arc::clone(&debounce_running);
            let refreshes = Arc::clone(&refresh_state);
            let watched_root = watch_root.clone();
            let tracked_snapshot = Arc::clone(&snapshot_state);
            thread::spawn(move || loop {
                thread::sleep(Duration::from_millis(OPENCODE_DEBOUNCE_MS));
                let elapsed = le.lock().map(|ts| ts.elapsed()).unwrap_or_default();
                if elapsed >= Duration::from_millis(OPENCODE_DEBOUNCE_MS) {
                    running.store(false, Ordering::SeqCst);
                    match should_emit_opencode_refresh(&watched_root, &tracked_snapshot) {
                        Ok(true) => {
                            let _ = emit_provider_refresh(&handle, &refreshes, OPENCODE_PROVIDER);
                        }
                        Ok(false) => {}
                        Err(error) => {
                            eprintln!("failed to verify opencode watcher refresh: {error}");
                        }
                    }
                    break;
                }
            });
        }
    })
    .map_err(|error| format!("failed to create opencode watcher: {error}"))?;

    watcher
        .watch(&session_storage, RecursiveMode::Recursive)
        .map_err(|error| format!("failed to watch {}: {error}", session_storage.display()))?;
    if message_storage.exists() {
        watcher
            .watch(&message_storage, RecursiveMode::NonRecursive)
            .map_err(|error| format!("failed to watch {}: {error}", message_storage.display()))?;
    }

    Ok(watcher)
}

pub(crate) fn create_provider_bridge_watcher(
    app: &tauri::AppHandle,
    providers: Vec<String>,
    refresh_state: Arc<Mutex<HashMap<String, Instant>>>,
    last_bridge_records: Arc<Mutex<HashMap<String, String>>>,
) -> Result<notify::RecommendedWatcher, String> {
    let bridge_dir = provider_bridge_dir()?;
    fs::create_dir_all(&bridge_dir).map_err(|error| {
        format!(
            "failed to create provider bridge directory {}: {error}",
            bridge_dir.display()
        )
    })?;

    let app_handle = app.clone();
    let provider_bridge_paths = providers.iter().try_fold(
        HashMap::new(),
        |mut acc: HashMap<String, PathBuf>, provider| -> Result<HashMap<String, PathBuf>, String> {
            acc.insert(provider.clone(), resolve_provider_bridge_path(provider)?);
            Ok(acc)
        },
    )?;
    let watched_bridge_paths = provider_bridge_paths.clone();
    let last_event: Arc<Mutex<Instant>> = Arc::new(Mutex::new(Instant::now()));
    let debounce_running: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));
    let pending_providers: Arc<Mutex<BTreeSet<String>>> = Arc::new(Mutex::new(BTreeSet::new()));

    let mut watcher = recommended_watcher(move |result: notify::Result<notify::Event>| {
        if let Ok(event) = result {
            let changed_providers = matched_bridge_providers(&event, &watched_bridge_paths);
            if changed_providers.is_empty() {
                return;
            }

            if let Ok(mut tracked_providers) = pending_providers.lock() {
                tracked_providers.extend(changed_providers);
            }

            if let Ok(mut ts) = last_event.lock() {
                *ts = Instant::now();
            }

            if debounce_running
                .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
                .is_err()
            {
                return;
            }

            let le = Arc::clone(&last_event);
            let handle = app_handle.clone();
            let running = Arc::clone(&debounce_running);
            let refreshes = Arc::clone(&refresh_state);
            let tracked_records = Arc::clone(&last_bridge_records);
            let watched_providers = providers.clone();
            let pending = Arc::clone(&pending_providers);
            thread::spawn(move || loop {
                thread::sleep(Duration::from_millis(PROVIDER_BRIDGE_DEBOUNCE_MS));
                let elapsed = le
                    .lock()
                    .map(|ts| {
                        let now = Instant::now();
                        now.saturating_duration_since(*ts)
                    })
                    .unwrap_or_else(|_| Duration::from_millis(PROVIDER_BRIDGE_DEBOUNCE_MS));
                if elapsed >= Duration::from_millis(PROVIDER_BRIDGE_DEBOUNCE_MS) {
                    running.store(false, Ordering::SeqCst);
                    let providers_to_process = pending
                        .lock()
                        .map(|mut tracked| {
                            let providers = tracked.iter().cloned().collect::<Vec<_>>();
                            tracked.clear();
                            providers
                        })
                        .unwrap_or_else(|_| watched_providers.clone());
                    for provider in &providers_to_process {
                        if let Err(error) = process_provider_bridge_event(
                            &handle,
                            &refreshes,
                            &tracked_records,
                            provider,
                        ) {
                            eprintln!("failed to process {provider} bridge event: {error}");
                            let _ = emit_provider_refresh(&handle, &refreshes, provider);
                        }
                    }
                    break;
                }
            });
        }
    })
    .map_err(|error| format!("failed to create provider bridge watcher: {error}"))?;

    watcher
        .watch(&bridge_dir, RecursiveMode::NonRecursive)
        .map_err(|error| format!("failed to watch {}: {error}", bridge_dir.display()))?;
    for bridge_path in provider_bridge_paths.values() {
        if bridge_path.exists() {
            watcher
                .watch(bridge_path, RecursiveMode::NonRecursive)
                .map_err(|error| format!("failed to watch {}: {error}", bridge_path.display()))?;
        }
    }

    Ok(watcher)
}

pub(crate) fn restart_session_watcher_internal(
    app: &tauri::AppHandle,
    watcher_state: &WatcherState,
    copilot_root: Option<&str>,
    opencode_root: Option<&str>,
    enabled_providers: &[String],
) -> Result<(), String> {
    let copilot_bridge_active = enabled_providers.iter().any(|p| p == COPILOT_PROVIDER)
        && matches!(
            detect_copilot_integration_status(copilot_root).status,
            ProviderIntegrationState::Installed
        );
    let opencode_bridge_active = enabled_providers.iter().any(|p| p == OPENCODE_PROVIDER)
        && matches!(
            detect_opencode_integration_status().status,
            ProviderIntegrationState::Installed
        );

    let mut bridge_providers = Vec::new();
    if copilot_bridge_active {
        bridge_providers.push(COPILOT_PROVIDER.to_string());
    }
    if opencode_bridge_active {
        bridge_providers.push(OPENCODE_PROVIDER.to_string());
    }

    if bridge_providers.is_empty() {
        let mut provider_bridge = watcher_state
            .provider_bridge
            .lock()
            .map_err(|_| "failed to lock provider bridge watcher state".to_string())?;
        *provider_bridge = None;
    } else {
        let watcher = create_provider_bridge_watcher(
            app,
            bridge_providers,
            Arc::clone(&watcher_state.last_provider_refresh),
            Arc::clone(&watcher_state.last_bridge_records),
        )?;
        let mut provider_bridge = watcher_state
            .provider_bridge
            .lock()
            .map_err(|_| "failed to lock provider bridge watcher state".to_string())?;
        *provider_bridge = Some(watcher);
    }

    // Copilot watcher
    if enabled_providers.iter().any(|p| p == COPILOT_PROVIDER) && !copilot_bridge_active {
        let root = resolve_copilot_root(copilot_root)?;
        match create_sessions_watcher(app, &root, Arc::clone(&watcher_state.last_provider_refresh))
        {
            Ok(watcher) => {
                let mut session_watcher = watcher_state
                    .sessions
                    .lock()
                    .map_err(|_| "failed to lock session watcher state".to_string())?;
                *session_watcher = Some(watcher);
            }
            Err(error) => {
                eprintln!("failed to start copilot session watcher: {error}");
            }
        }
    } else {
        let mut session_watcher = watcher_state
            .sessions
            .lock()
            .map_err(|_| "failed to lock session watcher state".to_string())?;
        *session_watcher = None;
    }

    // OpenCode watcher
    if enabled_providers.iter().any(|p| p == OPENCODE_PROVIDER) && !opencode_bridge_active {
        if let Ok(oc_root) = resolve_opencode_root(opencode_root) {
            match create_opencode_watcher(
                app,
                &oc_root,
                Arc::clone(&watcher_state.last_provider_refresh),
            ) {
                Ok(watcher) => {
                    let mut oc_watcher = watcher_state
                        .opencode
                        .lock()
                        .map_err(|_| "failed to lock opencode watcher state".to_string())?;
                    *oc_watcher = Some(watcher);
                }
                Err(error) => {
                    eprintln!("failed to start opencode watcher: {error}");
                }
            }
        }
    } else {
        let mut oc_watcher = watcher_state
            .opencode
            .lock()
            .map_err(|_| "failed to lock opencode watcher state".to_string())?;
        *oc_watcher = None;
    }

    Ok(())
}

pub(crate) fn watch_plan_file_internal(
    app: &tauri::AppHandle,
    watcher_state: &WatcherState,
    session_dir: &str,
) -> Result<(), String> {
    let plan_path = PathBuf::from(session_dir).join("plan.md");

    if !plan_path.exists() {
        return Err("plan.md does not exist".to_string());
    }

    let app_handle = app.clone();
    let watched_session_dir = session_dir.to_string();
    let mut watcher = recommended_watcher(move |result: notify::Result<notify::Event>| {
        if result.is_ok() {
            let _ = app_handle.emit("plan-file-changed", watched_session_dir.clone());
        }
    })
    .map_err(|error| format!("failed to create plan watcher: {error}"))?;

    watcher
        .watch(&plan_path, RecursiveMode::NonRecursive)
        .map_err(|error| format!("failed to watch {}: {error}", plan_path.display()))?;

    let mut plan_watcher = watcher_state
        .plan
        .lock()
        .map_err(|_| "failed to lock plan watcher state".to_string())?;
    *plan_watcher = Some(watcher);
    Ok(())
}

pub(crate) fn is_relevant_project_event(event: &notify::Event, project_dir: &Path) -> bool {
    let sisyphus_dir = project_dir.join(".sisyphus");
    let openspec_dir = project_dir.join("openspec");

    event.paths.iter().any(|path| {
        path == &sisyphus_dir
            || path.starts_with(&sisyphus_dir)
            || path == &openspec_dir
            || path.starts_with(&openspec_dir)
    })
}

pub(crate) fn watch_project_files_internal(
    app: &tauri::AppHandle,
    watcher_state: &WatcherState,
    project_dir: &str,
) -> Result<(), String> {
    let project_path = PathBuf::from(project_dir);
    if !project_path.is_dir() {
        return Err(format!("project directory not found: {}", project_path.display()));
    }

    let app_handle = app.clone();
    let watched_project_dir = project_dir.to_string();
    let relevant_root = project_path.clone();
    let last_event: Arc<Mutex<Instant>> = Arc::new(Mutex::new(Instant::now()));
    let debounce_running: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));

    let mut watcher = recommended_watcher(move |result: notify::Result<notify::Event>| {
        if let Ok(event) = result {
            if !is_relevant_project_event(&event, &relevant_root) {
                return;
            }

            if let Ok(mut ts) = last_event.lock() {
                *ts = Instant::now();
            }

            if debounce_running
                .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
                .is_err()
            {
                return;
            }

            let le = Arc::clone(&last_event);
            let handle = app_handle.clone();
            let running = Arc::clone(&debounce_running);
            let payload = watched_project_dir.clone();
            thread::spawn(move || loop {
                thread::sleep(Duration::from_millis(PROJECT_FILES_DEBOUNCE_MS));
                let elapsed = le.lock().map(|ts| ts.elapsed()).unwrap_or_default();
                if elapsed >= Duration::from_millis(PROJECT_FILES_DEBOUNCE_MS) {
                    running.store(false, Ordering::SeqCst);
                    let _ = handle.emit("project-files-changed", payload.clone());
                    break;
                }
            });
        }
    })
    .map_err(|error| format!("failed to create project watcher: {error}"))?;

    watcher
        .watch(&project_path, RecursiveMode::Recursive)
        .map_err(|error| format!("failed to watch {}: {error}", project_path.display()))?;

    let mut project_watcher = watcher_state
        .project
        .lock()
        .map_err(|_| "failed to lock project watcher state".to_string())?;
    *project_watcher = Some(watcher);
    Ok(())
}
