use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::io::Read;
use std::path::{Component, Path, PathBuf};
use std::time::UNIX_EPOCH;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use walkdir::{DirEntry, WalkDir};

use crate::db::ensure_parent_dir;
use crate::settings::{
    default_app_data_dir, default_opencode_config_root, load_settings_internal,
    resolve_claude_root, resolve_codex_root, resolve_copilot_root,
};
use crate::types::{
    AppSettings, CLAUDE_PROVIDER, CODEX_PROVIDER, COPILOT_PROVIDER, OPENCODE_PROVIDER,
};

const AGENTS_FILE_NAME: &str = "AGENTS.md";
const CLAUDE_FILE_NAME: &str = "CLAUDE.md";
const SKILL_FILE_NAME: &str = "SKILL.md";
const AGENTS_PREFS_FILE_NAME: &str = "agents.json";
const MAX_SCAN_DEPTH: usize = 8;
const MAX_SCANNED_DIRS: u64 = 20_000;
const FIXED_IGNORED_DIRS: &[&str] = &[
    "node_modules",
    ".git",
    "dist",
    "build",
    "vendor",
    ".next",
    ".nuxt",
    "target",
    ".sessionhub",
];

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum SyncStatus {
    InSync,
    TargetMissing,
    Differs,
    SourceMissing,
    Linked,
    LinkBroken,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct FileFingerprint {
    pub(crate) path: String,
    pub(crate) exists: bool,
    pub(crate) hash: Option<String>,
    pub(crate) mtime_ms: Option<u64>,
    pub(crate) size: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AgentsMdEntry {
    pub(crate) dir: String,
    pub(crate) rel_dir: String,
    pub(crate) source: FileFingerprint,
    pub(crate) target: FileFingerprint,
    pub(crate) status: SyncStatus,
    pub(crate) target_newer: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AgentsMdScanResult {
    pub(crate) root: String,
    pub(crate) entries: Vec<AgentsMdEntry>,
    pub(crate) truncated: bool,
    pub(crate) scanned_dirs: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct TargetInfo {
    pub(crate) target_id: String,
    pub(crate) root: String,
    pub(crate) root_exists: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct TargetStatus {
    pub(crate) target_id: String,
    pub(crate) target_root: String,
    pub(crate) status: SyncStatus,
    pub(crate) target_newer: bool,
    pub(crate) reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SkillEntry {
    pub(crate) name: String,
    pub(crate) source_dir: String,
    pub(crate) skill_md_path: String,
    pub(crate) file_count: u64,
    pub(crate) targets: Vec<TargetStatus>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SkillsScanResult {
    pub(crate) source_root: String,
    pub(crate) skills: Vec<SkillEntry>,
    pub(crate) targets: Vec<TargetInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CommandEntry {
    pub(crate) name: String,
    pub(crate) source_path: String,
    pub(crate) sync_source_path: String,
    pub(crate) targets: Vec<TargetStatus>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CommandsScanResult {
    pub(crate) source_root: String,
    pub(crate) commands: Vec<CommandEntry>,
    pub(crate) targets: Vec<TargetInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub(crate) enum AgentsScope {
    // enum 層級的 rename_all 只影響 variant 名稱，欄位需另行標註才會轉為 camelCase。
    #[serde(rename_all = "camelCase")]
    Project {
        project_cwd: String,
    },
    Global,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum SyncMode {
    Copy,
    Link,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum SyncDirection {
    SourceToTarget,
    TargetToSource,
    Skip,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum SyncItemKind {
    File,
    Directory,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum SyncAction {
    Create,
    Overwrite,
    SkipInSync,
    Conflict,
    Error,
    LinkFallbackCopy,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SyncItem {
    pub(crate) source: String,
    pub(crate) target: String,
    #[serde(default = "default_sync_item_kind")]
    pub(crate) item_kind: SyncItemKind,
    #[serde(default)]
    pub(crate) direction: Option<SyncDirection>,
    #[serde(default)]
    pub(crate) target_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SyncRequest {
    pub(crate) items: Vec<SyncItem>,
    #[serde(default)]
    pub(crate) dry_run: bool,
    #[serde(default)]
    pub(crate) force: bool,
    #[serde(default = "default_sync_mode")]
    pub(crate) mode: SyncMode,
    #[serde(default)]
    pub(crate) project_cwd: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SyncActionResult {
    pub(crate) source: String,
    pub(crate) target: String,
    pub(crate) action: SyncAction,
    pub(crate) reason: Option<String>,
    pub(crate) bytes: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SyncReport {
    pub(crate) dry_run: bool,
    pub(crate) actions: Vec<SyncActionResult>,
    pub(crate) conflicts: u32,
    pub(crate) errors: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ProjectAgentsPrefs {
    #[serde(default)]
    pub(crate) conflict_choice: Option<String>,
    #[serde(default)]
    pub(crate) ignored_paths: Vec<String>,
    #[serde(default = "default_enabled_targets")]
    pub(crate) enabled_targets: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SaveProjectAgentsPrefsResult {
    pub(crate) stored_path: String,
    pub(crate) created_project_config_dir: bool,
}

pub(crate) fn default_sync_mode() -> SyncMode {
    SyncMode::Copy
}

pub(crate) fn default_sync_item_kind() -> SyncItemKind {
    SyncItemKind::File
}

pub(crate) fn default_enabled_targets() -> Vec<String> {
    vec![
        CLAUDE_PROVIDER.to_string(),
        CODEX_PROVIDER.to_string(),
        OPENCODE_PROVIDER.to_string(),
        COPILOT_PROVIDER.to_string(),
    ]
}

impl Default for ProjectAgentsPrefs {
    fn default() -> Self {
        Self {
            conflict_choice: None,
            ignored_paths: Vec::new(),
            enabled_targets: default_enabled_targets(),
        }
    }
}

#[derive(Debug, Clone)]
struct AggregateFingerprint {
    fingerprint: FileFingerprint,
    file_count: u64,
}

trait CommandAdapter {
    fn adapt(&self, _target_id: Option<&str>, content: Vec<u8>) -> Result<Vec<u8>, String>;
}

struct PassthroughAdapter;

impl CommandAdapter for PassthroughAdapter {
    fn adapt(&self, _target_id: Option<&str>, content: Vec<u8>) -> Result<Vec<u8>, String> {
        Ok(content)
    }
}

pub(crate) fn scan_agents_md_internal(project_cwd: &str) -> Result<AgentsMdScanResult, String> {
    let prefs = load_project_agents_prefs_internal(project_cwd)?;
    let root = PathBuf::from(project_cwd);
    scan_agents_md_root(&root, &prefs)
}

pub(crate) fn scan_global_agents_md_internal() -> Result<AgentsMdScanResult, String> {
    let settings = load_agents_settings()?;
    let mut entries = Vec::new();
    let mut roots = global_instruction_roots(&settings)?;
    roots.sort_by(|left, right| left.0.cmp(&right.0));

    for (_, root) in roots {
        let source = fingerprint_file(&root.join(AGENTS_FILE_NAME));
        let target = fingerprint_file(&root.join(CLAUDE_FILE_NAME));
        if !source.exists && !target.exists {
            continue;
        }
        let (status, target_newer) = classify_file_status(&source, &target);
        entries.push(AgentsMdEntry {
            dir: normalize_display_path(&root),
            rel_dir: root
                .file_name()
                .and_then(|value| value.to_str())
                .unwrap_or_default()
                .to_string(),
            source,
            target,
            status,
            target_newer,
        });
    }

    Ok(AgentsMdScanResult {
        root: "global".to_string(),
        entries,
        truncated: false,
        scanned_dirs: 0,
    })
}

pub(crate) fn scan_agents_skills_internal(scope: &AgentsScope) -> Result<SkillsScanResult, String> {
    let settings = load_agents_settings()?;
    let prefs = match scope {
        AgentsScope::Project { project_cwd } => load_project_agents_prefs_internal(project_cwd)?,
        AgentsScope::Global => ProjectAgentsPrefs::default(),
    };
    let source_root = skills_source_root(scope)?;
    let targets = skill_target_roots(scope, &settings)?;
    let target_infos = targets
        .iter()
        .map(|(target_id, root)| TargetInfo {
            target_id: target_id.clone(),
            root: normalize_display_path(root),
            root_exists: root.exists(),
        })
        .collect::<Vec<_>>();

    // 先從來源目錄探索，再從各目標目錄補上僅存在於目標端的 skill。
    let mut discovered = BTreeMap::<String, PathBuf>::new();
    collect_skill_dirs(&source_root, &mut discovered);
    for (_, target_root) in &targets {
        collect_skill_dirs(target_root, &mut discovered);
    }

    let mut skills = Vec::new();
    for (name, discovered_dir) in discovered {
        let skill_dir = source_root.join(&name);
        let preview_dir = if skill_dir.join(SKILL_FILE_NAME).is_file() {
            skill_dir.clone()
        } else {
            discovered_dir
        };
        let source_fp =
            fingerprint_directory(&skill_dir, Some(&source_root), &prefs.ignored_paths)?;
        let mut statuses = Vec::new();
        for (target_id, target_root) in &targets {
            let target_dir = target_root.join(&name);
            statuses.push(compare_directory_target(
                target_id,
                target_root,
                &skill_dir,
                &target_dir,
                &source_fp,
                &prefs.ignored_paths,
            )?);
        }

        skills.push(SkillEntry {
            name,
            source_dir: normalize_display_path(&skill_dir),
            skill_md_path: normalize_display_path(&preview_dir.join(SKILL_FILE_NAME)),
            file_count: source_fp.file_count,
            targets: statuses,
        });
    }

    Ok(SkillsScanResult {
        source_root: normalize_display_path(&source_root),
        skills,
        targets: target_infos,
    })
}

/// Copilot 的 `.github/prompts/` 慣例要求副檔名為 `.prompt.md`（VS Code Copilot prompt files
/// 規則），其餘 target（claude/codex/opencode）與來源端一律使用 `.md`。剝離／組裝檔名時皆須
/// 依 target 分流，否則 copilot 端的 `xxx.prompt.md` 會被誤判為獨立於 `xxx` 的 command。
fn command_file_suffix(target_id: &str) -> &'static str {
    if target_id == COPILOT_PROVIDER {
        ".prompt.md"
    } else {
        ".md"
    }
}

fn strip_command_file_suffix(relative: &Path, target_id: &str) -> Option<String> {
    let raw = relative.to_string_lossy().replace('\\', "/");
    let suffix = command_file_suffix(target_id);
    raw.strip_suffix(suffix).map(|stem| stem.to_string())
}

fn command_relative_path(name: &str, target_id: &str) -> PathBuf {
    let file_name = format!("{name}{}", command_file_suffix(target_id));
    PathBuf::from(file_name.replace('/', "\\"))
}

pub(crate) fn scan_agents_commands_internal(
    scope: &AgentsScope,
) -> Result<CommandsScanResult, String> {
    let settings = load_agents_settings()?;
    let source_root = commands_source_root(scope)?;
    let targets = command_target_roots(scope, &settings)?;
    let target_infos = targets
        .iter()
        .map(|(target_id, root)| TargetInfo {
            target_id: target_id.clone(),
            root: normalize_display_path(root),
            root_exists: root.exists(),
        })
        .collect::<Vec<_>>();

    let mut discovered = BTreeMap::<String, PathBuf>::new();

    if source_root.is_dir() {
        let walker = WalkDir::new(&source_root)
            .follow_links(false)
            .sort_by_file_name()
            .into_iter();
        for entry in walker.filter_map(Result::ok) {
            let path = entry.path();
            if !entry.file_type().is_file()
                || path.extension().and_then(|value| value.to_str()) != Some("md")
            {
                continue;
            }
            let relative = match path.strip_prefix(&source_root) {
                Ok(value) => value,
                Err(_) => continue,
            };
            // 來源端固定使用 `.md`（非 copilot 慣例）。
            let Some(name) = strip_command_file_suffix(relative, "") else {
                continue;
            };
            discovered.entry(name).or_insert_with(|| path.to_path_buf());
        }
    }

    for (target_id, target_root) in &targets {
        if !target_root.is_dir() {
            continue;
        }
        let walker = WalkDir::new(target_root)
            .follow_links(false)
            .sort_by_file_name()
            .into_iter();
        for entry in walker.filter_map(Result::ok) {
            let path = entry.path();
            if !entry.file_type().is_file()
                || path.extension().and_then(|value| value.to_str()) != Some("md")
            {
                continue;
            }
            let relative = match path.strip_prefix(target_root) {
                Ok(value) => value,
                Err(_) => continue,
            };
            let Some(name) = strip_command_file_suffix(relative, target_id) else {
                continue;
            };
            discovered.entry(name).or_insert_with(|| path.to_path_buf());
        }
    }

    let mut commands = Vec::new();
    for (name, preview_path) in discovered {
        let source_relative = command_relative_path(&name, "");
        let sync_source_path = source_root.join(&source_relative);
        let source_fp = fingerprint_file(&sync_source_path);
        let preview_source_path = if source_fp.exists {
            sync_source_path.clone()
        } else {
            preview_path
        };

        let mut statuses = Vec::new();
        for (target_id, target_root) in &targets {
            let target_relative = command_relative_path(&name, target_id);
            let target_path = target_root.join(&target_relative);
            let target_fp = fingerprint_file(&target_path);
            let (status, target_newer) = classify_file_status(&source_fp, &target_fp);
            statuses.push(TargetStatus {
                target_id: target_id.clone(),
                target_root: normalize_display_path(target_root),
                status,
                target_newer,
                reason: None,
            });
        }
        commands.push(CommandEntry {
            name,
            source_path: normalize_display_path(&preview_source_path),
            sync_source_path: normalize_display_path(&sync_source_path),
            targets: statuses,
        });
    }

    commands.sort_by(|left, right| left.name.cmp(&right.name));
    Ok(CommandsScanResult {
        source_root: normalize_display_path(&source_root),
        commands,
        targets: target_infos,
    })
}

pub(crate) fn sync_agents_items_internal(request: &SyncRequest) -> Result<SyncReport, String> {
    let prefs = match request.project_cwd.as_deref() {
        Some(project_cwd) => load_project_agents_prefs_internal(project_cwd)?,
        None => ProjectAgentsPrefs::default(),
    };
    let adapter = PassthroughAdapter;
    let mut actions = Vec::new();
    let mut conflicts = 0u32;
    let mut errors = 0u32;

    for item in &request.items {
        let result = sync_one_item(item, request, &prefs, &adapter)?;
        if result.action == SyncAction::Conflict {
            conflicts += 1;
        }
        if result.action == SyncAction::Error {
            errors += 1;
        }
        actions.push(result);
    }

    Ok(SyncReport {
        dry_run: request.dry_run,
        actions,
        conflicts,
        errors,
    })
}

pub(crate) fn read_agents_file_internal(file_path: &str) -> Result<String, String> {
    fs::read_to_string(file_path).map_err(|error| format!("failed to read agents file: {error}"))
}

pub(crate) fn write_agents_file_internal(
    scope_root: &str,
    file_path: &str,
    content: &str,
) -> Result<(), String> {
    let resolved = resolve_scoped_write_path(Path::new(scope_root), Path::new(file_path))?;
    atomic_write_file(&resolved, content.as_bytes())
}

pub(crate) fn load_project_agents_prefs_internal(
    project_cwd: &str,
) -> Result<ProjectAgentsPrefs, String> {
    let project_path = project_agents_prefs_path(project_cwd);
    if project_path.is_file() {
        return read_project_agents_prefs(&project_path);
    }

    let fallback_path = appdata_project_agents_prefs_path(project_cwd)?;
    if fallback_path.is_file() {
        return read_project_agents_prefs(&fallback_path);
    }

    Ok(ProjectAgentsPrefs::default())
}

pub(crate) fn save_project_agents_prefs_internal(
    project_cwd: &str,
    prefs: &ProjectAgentsPrefs,
    allow_create_project_config_dir: bool,
) -> Result<SaveProjectAgentsPrefsResult, String> {
    let project_path = project_agents_prefs_path(project_cwd);
    let project_dir = project_path
        .parent()
        .ok_or_else(|| format!("failed to resolve project prefs parent for {}", project_path.display()))?;
    let should_write_project = project_path.is_file() || allow_create_project_config_dir;
    let created_project_config_dir = should_write_project && !project_dir.exists();
    let target_path = if should_write_project {
        project_path
    } else {
        appdata_project_agents_prefs_path(project_cwd)?
    };
    write_project_agents_prefs(&target_path, prefs)?;
    Ok(SaveProjectAgentsPrefsResult {
        stored_path: normalize_display_path(&target_path),
        created_project_config_dir,
    })
}

fn scan_agents_md_root(
    root: &Path,
    prefs: &ProjectAgentsPrefs,
) -> Result<AgentsMdScanResult, String> {
    let root = root
        .canonicalize()
        .map_err(|error| format!("failed to resolve project root {}: {error}", root.display()))?;
    let mut entries = Vec::new();
    let mut scanned_dirs = 0u64;
    let mut truncated = false;

    let mut walker = WalkDir::new(&root)
        .max_depth(MAX_SCAN_DEPTH)
        .follow_links(false)
        .sort_by_file_name()
        .into_iter();

    loop {
        let next = walker.next();
        let entry = match next {
            Some(Ok(entry)) => entry,
            Some(Err(_)) => continue,
            None => break,
        };

        if should_skip_walk_entry(&entry, &root, &prefs.ignored_paths) {
            if entry.file_type().is_dir() {
                walker.skip_current_dir();
            }
            continue;
        }

        if !entry.file_type().is_dir() {
            continue;
        }
        scanned_dirs += 1;
        if scanned_dirs > MAX_SCANNED_DIRS {
            truncated = true;
            break;
        }

        let dir_path = entry.path();
        let source = fingerprint_file(&dir_path.join(AGENTS_FILE_NAME));
        let target = fingerprint_file(&dir_path.join(CLAUDE_FILE_NAME));
        if !source.exists && !target.exists {
            continue;
        }

        let rel_dir = dir_path
            .strip_prefix(&root)
            .unwrap_or(dir_path)
            .to_string_lossy()
            .replace('\\', "/");
        let (status, target_newer) = classify_file_status(&source, &target);
        entries.push(AgentsMdEntry {
            dir: normalize_display_path(dir_path),
            rel_dir,
            source,
            target,
            status,
            target_newer,
        });
    }

    entries.sort_by(|left, right| left.dir.cmp(&right.dir));
    Ok(AgentsMdScanResult {
        root: normalize_display_path(&root),
        entries,
        truncated,
        scanned_dirs,
    })
}

fn collect_skill_dirs(root: &Path, discovered: &mut BTreeMap<String, PathBuf>) {
    if !root.is_dir() {
        return;
    }

    // skill 目錄可能是符號連結（例如集中存放後 link 到 .agents/skills），
    // 因此需要 follow_links 才能探索連結後的內容；迴圈連結會由 walkdir 回報錯誤而被濾除。
    let walker = WalkDir::new(root)
        .follow_links(true)
        .sort_by_file_name()
        .into_iter();
    for entry in walker.filter_map(Result::ok) {
        if !entry.file_type().is_dir() {
            continue;
        }

        let dir = entry.path();
        if !dir.join(SKILL_FILE_NAME).is_file() {
            continue;
        }

        let Ok(relative) = dir.strip_prefix(root) else {
            continue;
        };
        let name = relative.to_string_lossy().replace('\\', "/");
        if name.is_empty() {
            continue;
        }
        discovered.entry(name).or_insert_with(|| dir.to_path_buf());
    }
}

fn classify_file_status(source: &FileFingerprint, target: &FileFingerprint) -> (SyncStatus, bool) {
    if source.exists && target.exists {
        if source.hash == target.hash {
            return (SyncStatus::InSync, false);
        }
        return (SyncStatus::Differs, is_target_newer(source, target));
    }
    if source.exists {
        return (SyncStatus::TargetMissing, false);
    }
    if target.exists {
        return (SyncStatus::SourceMissing, false);
    }
    // 兩者皆不存在：commands 矩陣的列可能是從「其他 target」反向探索出的名稱，
    // 對這個 target 而言單純是「尚未同步」而非例外錯誤，統一視為 target-missing。
    (SyncStatus::TargetMissing, false)
}

fn compare_directory_target(
    target_id: &str,
    target_root: &Path,
    source_dir: &Path,
    target_dir: &Path,
    source_fp: &AggregateFingerprint,
    ignored_paths: &[String],
) -> Result<TargetStatus, String> {
    let target_root_display = normalize_display_path(target_root);
    let symlink_meta = fs::symlink_metadata(target_dir);
    if let Ok(metadata) = symlink_meta {
        if metadata.file_type().is_symlink() {
            return Ok(inspect_directory_symlink(
                target_id,
                &target_root_display,
                source_dir,
                target_dir,
            ));
        }
    }

    if !target_dir.exists() {
        return Ok(TargetStatus {
            target_id: target_id.to_string(),
            target_root: target_root_display,
            status: SyncStatus::TargetMissing,
            target_newer: false,
            reason: None,
        });
    }

    if !source_fp.fingerprint.exists {
        return Ok(TargetStatus {
            target_id: target_id.to_string(),
            target_root: target_root_display,
            status: SyncStatus::SourceMissing,
            target_newer: false,
            reason: None,
        });
    }

    let target_fp = fingerprint_directory(target_dir, Some(target_root), ignored_paths)?;
    let status = if source_fp.fingerprint.hash == target_fp.fingerprint.hash {
        SyncStatus::InSync
    } else {
        SyncStatus::Differs
    };
    let target_newer = status == SyncStatus::Differs
        && is_target_newer(&source_fp.fingerprint, &target_fp.fingerprint);
    Ok(TargetStatus {
        target_id: target_id.to_string(),
        target_root: target_root_display,
        status,
        target_newer,
        reason: None,
    })
}

fn inspect_directory_symlink(
    target_id: &str,
    target_root: &str,
    source_dir: &Path,
    target_dir: &Path,
) -> TargetStatus {
    let link_target = match fs::read_link(target_dir) {
        Ok(path) => path,
        Err(error) => {
            return TargetStatus {
                target_id: target_id.to_string(),
                target_root: target_root.to_string(),
                status: SyncStatus::Error,
                target_newer: false,
                reason: Some(format!("failed to inspect symlink: {error}")),
            }
        }
    };

    let canonical_source = source_dir.canonicalize();
    let canonical_target = canonicalize_link_target(target_dir, &link_target);
    match (canonical_source, canonical_target) {
        (Ok(source), Ok(target)) if source == target => TargetStatus {
            target_id: target_id.to_string(),
            target_root: target_root.to_string(),
            status: SyncStatus::Linked,
            target_newer: false,
            reason: None,
        },
        (Ok(_), Ok(_)) => TargetStatus {
            target_id: target_id.to_string(),
            target_root: target_root.to_string(),
            status: SyncStatus::Error,
            target_newer: false,
            reason: Some("symlink points to a different source".to_string()),
        },
        _ => TargetStatus {
            target_id: target_id.to_string(),
            target_root: target_root.to_string(),
            status: SyncStatus::LinkBroken,
            target_newer: false,
            reason: Some("symlink target is missing".to_string()),
        },
    }
}

fn sync_one_item(
    item: &SyncItem,
    request: &SyncRequest,
    prefs: &ProjectAgentsPrefs,
    adapter: &dyn CommandAdapter,
) -> Result<SyncActionResult, String> {
    validate_sync_path(Path::new(&item.source))?;
    validate_sync_path(Path::new(&item.target))?;

    let source_path = PathBuf::from(&item.source);
    let target_path = PathBuf::from(&item.target);
    let source_exists = source_path.exists();
    let target_exists = target_path.exists() || fs::symlink_metadata(&target_path).is_ok();
    let source_fp = fingerprint_path_for_sync(&source_path, &item.item_kind)?;
    let target_fp = fingerprint_path_for_sync(&target_path, &item.item_kind)?;

    if matches!(item.direction, Some(SyncDirection::Skip)) {
        return Ok(SyncActionResult {
            source: item.source.clone(),
            target: item.target.clone(),
            action: SyncAction::SkipInSync,
            reason: Some("skipped by user choice".to_string()),
            bytes: None,
        });
    }

    if source_exists && target_exists && source_fp.hash == target_fp.hash {
        return Ok(SyncActionResult {
            source: item.source.clone(),
            target: item.target.clone(),
            action: SyncAction::SkipInSync,
            reason: None,
            bytes: None,
        });
    }

    if !source_exists && !target_exists {
        return Ok(SyncActionResult {
            source: item.source.clone(),
            target: item.target.clone(),
            action: SyncAction::Error,
            reason: Some("both source and target are missing".to_string()),
            bytes: None,
        });
    }

    let remembered_direction = match prefs.conflict_choice.as_deref() {
        Some("source-wins") => Some(SyncDirection::SourceToTarget),
        Some("target-wins") => Some(SyncDirection::TargetToSource),
        _ => None,
    };
    let explicit_direction = item.direction.clone().or(remembered_direction);

    if !source_exists && target_exists {
        if matches!(explicit_direction, Some(SyncDirection::TargetToSource)) {
            return execute_sync(
                &target_path,
                &source_path,
                &item.item_kind,
                SyncAction::Create,
                request,
                item.target_id.as_deref(),
                adapter,
            );
        }
        return Ok(SyncActionResult {
            source: item.source.clone(),
            target: item.target.clone(),
            action: SyncAction::Conflict,
            reason: Some("source file is missing".to_string()),
            bytes: None,
        });
    }

    if source_exists && !target_exists {
        if matches!(explicit_direction, Some(SyncDirection::TargetToSource)) {
            return Ok(SyncActionResult {
                source: item.source.clone(),
                target: item.target.clone(),
                action: SyncAction::Conflict,
                reason: Some("target is missing for target-to-source sync".to_string()),
                bytes: None,
            });
        }
        return execute_sync(
            &source_path,
            &target_path,
            &item.item_kind,
            SyncAction::Create,
            request,
            item.target_id.as_deref(),
            adapter,
        );
    }

    let target_newer = target_fp.mtime_ms.unwrap_or(0) > source_fp.mtime_ms.unwrap_or(0);
    let direction = if let Some(direction) = explicit_direction {
        direction
    } else if request.force {
        SyncDirection::SourceToTarget
    } else if target_newer {
        return Ok(SyncActionResult {
            source: item.source.clone(),
            target: item.target.clone(),
            action: SyncAction::Conflict,
            reason: Some("target is newer than source".to_string()),
            bytes: None,
        });
    } else {
        SyncDirection::SourceToTarget
    };

    match direction {
        SyncDirection::SourceToTarget => execute_sync(
            &source_path,
            &target_path,
            &item.item_kind,
            SyncAction::Overwrite,
            request,
            item.target_id.as_deref(),
            adapter,
        ),
        SyncDirection::TargetToSource => execute_sync(
            &target_path,
            &source_path,
            &item.item_kind,
            SyncAction::Overwrite,
            request,
            item.target_id.as_deref(),
            adapter,
        ),
        SyncDirection::Skip => Ok(SyncActionResult {
            source: item.source.clone(),
            target: item.target.clone(),
            action: SyncAction::SkipInSync,
            reason: Some("skipped by user choice".to_string()),
            bytes: None,
        }),
    }
}

fn execute_sync(
    source_path: &Path,
    target_path: &Path,
    item_kind: &SyncItemKind,
    default_action: SyncAction,
    request: &SyncRequest,
    target_id: Option<&str>,
    adapter: &dyn CommandAdapter,
) -> Result<SyncActionResult, String> {
    if request.dry_run {
        let bytes = if source_path.is_file() {
            Some(
                fs::metadata(source_path)
                    .map(|value| value.len())
                    .unwrap_or(0),
            )
        } else {
            None
        };
        return Ok(SyncActionResult {
            source: normalize_display_path(source_path),
            target: normalize_display_path(target_path),
            action: default_action,
            reason: None,
            bytes,
        });
    }

    match item_kind {
        SyncItemKind::File => {
            let bytes = sync_file(source_path, target_path, target_id, adapter)?;
            Ok(SyncActionResult {
                source: normalize_display_path(source_path),
                target: normalize_display_path(target_path),
                action: default_action,
                reason: None,
                bytes: Some(bytes),
            })
        }
        SyncItemKind::Directory => {
            if request.mode == SyncMode::Link {
                match sync_directory_link(source_path, target_path) {
                    Ok(action) => {
                        return Ok(SyncActionResult {
                            source: normalize_display_path(source_path),
                            target: normalize_display_path(target_path),
                            action,
                            reason: None,
                            bytes: None,
                        });
                    }
                    Err(error) if is_symlink_privilege_error(&error) => {
                        sync_directory_copy(source_path, target_path, adapter)?;
                        return Ok(SyncActionResult {
                            source: normalize_display_path(source_path),
                            target: normalize_display_path(target_path),
                            action: SyncAction::LinkFallbackCopy,
                            reason: Some(error),
                            bytes: None,
                        });
                    }
                    Err(error) => {
                        return Ok(SyncActionResult {
                            source: normalize_display_path(source_path),
                            target: normalize_display_path(target_path),
                            action: SyncAction::Error,
                            reason: Some(error),
                            bytes: None,
                        });
                    }
                }
            }

            sync_directory_copy(source_path, target_path, adapter)?;
            Ok(SyncActionResult {
                source: normalize_display_path(source_path),
                target: normalize_display_path(target_path),
                action: default_action,
                reason: None,
                bytes: None,
            })
        }
    }
}

fn sync_file(
    source_path: &Path,
    target_path: &Path,
    target_id: Option<&str>,
    adapter: &dyn CommandAdapter,
) -> Result<u64, String> {
    let content = fs::read(source_path).map_err(|error| {
        format!(
            "failed to read source file {}: {error}",
            source_path.display()
        )
    })?;
    let transformed = adapter.adapt(target_id, content)?;
    let bytes = transformed.len() as u64;
    atomic_write_file(target_path, &transformed)?;
    Ok(bytes)
}

fn sync_directory_copy(
    source_path: &Path,
    target_path: &Path,
    adapter: &dyn CommandAdapter,
) -> Result<(), String> {
    let source_root = source_path.canonicalize().map_err(|error| {
        format!(
            "failed to resolve source directory {}: {error}",
            source_path.display()
        )
    })?;
    let walker = WalkDir::new(&source_root)
        .follow_links(false)
        .sort_by_file_name()
        .into_iter();
    for entry in walker.filter_map(Result::ok) {
        let path = entry.path();
        if entry.file_type().is_dir() {
            continue;
        }
        let relative = path
            .strip_prefix(&source_root)
            .map_err(|error| format!("failed to compute relative path: {error}"))?;
        let destination = target_path.join(relative);
        sync_file(path, &destination, None, adapter)?;
    }
    Ok(())
}

fn sync_directory_link(source_path: &Path, target_path: &Path) -> Result<SyncAction, String> {
    if let Ok(metadata) = fs::symlink_metadata(target_path) {
        if metadata.file_type().is_symlink() {
            let link_target = fs::read_link(target_path).map_err(|error| {
                format!("failed to read symlink {}: {error}", target_path.display())
            })?;
            let resolved_link =
                canonicalize_link_target(target_path, &link_target).map_err(|error| {
                    format!(
                        "failed to resolve symlink {}: {error}",
                        target_path.display()
                    )
                })?;
            let resolved_source = source_path.canonicalize().map_err(|error| {
                format!(
                    "failed to resolve source directory {}: {error}",
                    source_path.display()
                )
            })?;
            if resolved_link == resolved_source {
                return Ok(SyncAction::SkipInSync);
            }
            return Err("target symlink points to a different source".to_string());
        }
        return Err("target already contains physical files".to_string());
    }

    if let Some(parent) = target_path.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("failed to create directory {}: {error}", parent.display()))?;
    }
    create_directory_symlink(source_path, target_path)?;
    Ok(SyncAction::Create)
}

fn fingerprint_path_for_sync(path: &Path, kind: &SyncItemKind) -> Result<FileFingerprint, String> {
    match kind {
        SyncItemKind::File => Ok(fingerprint_file(path)),
        SyncItemKind::Directory => Ok(fingerprint_directory(path, Some(path), &[])?.fingerprint),
    }
}

fn fingerprint_file(path: &Path) -> FileFingerprint {
    let display_path = normalize_display_path(path);
    let metadata = match fs::metadata(path) {
        Ok(metadata) => metadata,
        Err(_) => {
            return FileFingerprint {
                path: display_path,
                exists: false,
                hash: None,
                mtime_ms: None,
                size: None,
            }
        }
    };
    let mut file = match fs::File::open(path) {
        Ok(file) => file,
        Err(_) => {
            return FileFingerprint {
                path: display_path,
                exists: false,
                hash: None,
                mtime_ms: None,
                size: None,
            }
        }
    };
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 8192];
    loop {
        match file.read(&mut buffer) {
            Ok(0) => break,
            Ok(bytes) => hasher.update(&buffer[..bytes]),
            Err(_) => {
                return FileFingerprint {
                    path: display_path,
                    exists: false,
                    hash: None,
                    mtime_ms: None,
                    size: None,
                }
            }
        }
    }

    FileFingerprint {
        path: display_path,
        exists: true,
        hash: Some(format!("{:x}", hasher.finalize())),
        mtime_ms: metadata.modified().ok().and_then(system_time_to_millis),
        size: Some(metadata.len()),
    }
}

fn fingerprint_directory(
    path: &Path,
    relative_root: Option<&Path>,
    ignored_paths: &[String],
) -> Result<AggregateFingerprint, String> {
    if !path.exists() {
        return Ok(AggregateFingerprint {
            fingerprint: FileFingerprint {
                path: normalize_display_path(path),
                exists: false,
                hash: None,
                mtime_ms: None,
                size: None,
            },
            file_count: 0,
        });
    }

    let root = path
        .canonicalize()
        .map_err(|error| format!("failed to resolve directory {}: {error}", path.display()))?;
    // 符號連結的 skill 目錄 canonicalize 後會脫離 relative_root，
    // 因此改用「原始路徑」推導名稱前綴，再與目錄內相對路徑組合，確保雜湊格式一致。
    let prefix = relative_root
        .and_then(|value| path.strip_prefix(value).ok())
        .map(Path::to_path_buf)
        .unwrap_or_default();
    let mut outer_hasher = Sha256::new();
    let mut file_count = 0u64;
    let mut total_size = 0u64;
    let mut latest_mtime: Option<u64> = None;
    let walker = WalkDir::new(&root)
        .follow_links(true)
        .sort_by_file_name()
        .into_iter();
    for entry in walker.filter_map(Result::ok) {
        if should_skip_walk_entry(&entry, &root, &[]) {
            continue;
        }
        if !entry.file_type().is_file() {
            continue;
        }
        let file_fp = fingerprint_file(entry.path());
        let relative = prefix
            .join(entry.path().strip_prefix(&root).unwrap_or(entry.path()))
            .to_string_lossy()
            .replace('\\', "/");
        if is_ignored_relative(&relative, ignored_paths) {
            continue;
        }
        outer_hasher.update(relative.as_bytes());
        outer_hasher.update([0]);
        if let Some(hash) = &file_fp.hash {
            outer_hasher.update(hash.as_bytes());
        }
        total_size += file_fp.size.unwrap_or(0);
        latest_mtime = match (latest_mtime, file_fp.mtime_ms) {
            (Some(left), Some(right)) => Some(left.max(right)),
            (None, Some(value)) => Some(value),
            (value, None) => value,
        };
        file_count += 1;
    }

    Ok(AggregateFingerprint {
        fingerprint: FileFingerprint {
            path: normalize_display_path(&root),
            exists: true,
            hash: Some(format!("{:x}", outer_hasher.finalize())),
            mtime_ms: latest_mtime,
            size: Some(total_size),
        },
        file_count,
    })
}

fn should_skip_walk_entry(entry: &DirEntry, root: &Path, ignored_paths: &[String]) -> bool {
    if entry.depth() == 0 {
        return false;
    }
    if entry
        .file_name()
        .to_str()
        .map(|value| FIXED_IGNORED_DIRS.contains(&value))
        .unwrap_or(false)
    {
        return true;
    }
    let relative = entry
        .path()
        .strip_prefix(root)
        .unwrap_or(entry.path())
        .to_string_lossy()
        .replace('\\', "/")
        .to_lowercase();
    is_ignored_relative(&relative, ignored_paths)
}

fn is_ignored_relative(relative: &str, ignored_paths: &[String]) -> bool {
    let relative = relative.to_lowercase();
    ignored_paths.iter().any(|ignored| {
        let ignored = ignored.replace('\\', "/").trim_matches('/').to_lowercase();
        !ignored.is_empty()
            && (relative == ignored || relative.starts_with(format!("{ignored}/").as_str()))
    })
}

fn load_agents_settings() -> Result<AppSettings, String> {
    load_settings_internal().or_else(|_| AppSettings::default())
}

fn global_instruction_roots(settings: &AppSettings) -> Result<Vec<(String, PathBuf)>, String> {
    Ok(vec![
        ("agents".to_string(), default_agents_root()?),
        (
            "agents-instructions".to_string(),
            default_agents_root()?.join("instructions"),
        ),
        (
            CLAUDE_PROVIDER.to_string(),
            resolve_claude_root(Some(settings.claude_root.as_str()))?,
        ),
        (
            CODEX_PROVIDER.to_string(),
            resolve_codex_root(Some(settings.codex_root.as_str()))?,
        ),
        (
            OPENCODE_PROVIDER.to_string(),
            default_opencode_config_root()?,
        ),
        (
            COPILOT_PROVIDER.to_string(),
            resolve_copilot_root(Some(settings.copilot_root.as_str()))?,
        ),
    ])
}

fn skills_source_root(scope: &AgentsScope) -> Result<PathBuf, String> {
    match scope {
        AgentsScope::Project { project_cwd } => {
            Ok(PathBuf::from(project_cwd).join(".agents").join("skills"))
        }
        AgentsScope::Global => Ok(default_agents_root()?.join("skills")),
    }
}

fn commands_source_root(scope: &AgentsScope) -> Result<PathBuf, String> {
    Ok(skills_source_root(scope)?.join("command"))
}

fn skill_target_roots(
    scope: &AgentsScope,
    settings: &AppSettings,
) -> Result<Vec<(String, PathBuf)>, String> {
    match scope {
        AgentsScope::Project { project_cwd } => {
            let project_root = PathBuf::from(project_cwd);
            Ok(vec![
                (
                    CLAUDE_PROVIDER.to_string(),
                    project_root.join(".claude").join("skills"),
                ),
                (
                    CODEX_PROVIDER.to_string(),
                    project_root.join(".codex").join("skills"),
                ),
                (
                    OPENCODE_PROVIDER.to_string(),
                    project_root.join(".opencode").join("skills"),
                ),
                (
                    COPILOT_PROVIDER.to_string(),
                    resolve_project_target_root(
                        &project_root.join(".github").join("skills"),
                        &project_root.join(".copilot").join("skills"),
                    ),
                ),
            ])
        }
        AgentsScope::Global => Ok(vec![
            (
                CLAUDE_PROVIDER.to_string(),
                resolve_claude_root(Some(settings.claude_root.as_str()))?.join("skills"),
            ),
            (
                CODEX_PROVIDER.to_string(),
                resolve_codex_root(Some(settings.codex_root.as_str()))?.join("skills"),
            ),
            (
                OPENCODE_PROVIDER.to_string(),
                default_opencode_config_root()?.join("skills"),
            ),
            (
                COPILOT_PROVIDER.to_string(),
                resolve_copilot_root(Some(settings.copilot_root.as_str()))?.join("skills"),
            ),
        ]),
    }
}

fn command_target_roots(
    scope: &AgentsScope,
    settings: &AppSettings,
) -> Result<Vec<(String, PathBuf)>, String> {
    match scope {
        AgentsScope::Project { project_cwd } => {
            let project_root = PathBuf::from(project_cwd);
            Ok(vec![
                (
                    CLAUDE_PROVIDER.to_string(),
                    project_root.join(".claude").join("commands"),
                ),
                (
                    CODEX_PROVIDER.to_string(),
                    project_root.join(".codex").join("prompts"),
                ),
                (
                    OPENCODE_PROVIDER.to_string(),
                    project_root.join(".opencode").join("command"),
                ),
                (
                    COPILOT_PROVIDER.to_string(),
                    resolve_project_target_root(
                        &project_root.join(".github").join("prompts"),
                        &project_root.join(".copilot").join("prompts"),
                    ),
                ),
            ])
        }
        AgentsScope::Global => Ok(vec![
            (
                CLAUDE_PROVIDER.to_string(),
                resolve_claude_root(Some(settings.claude_root.as_str()))?.join("commands"),
            ),
            (
                CODEX_PROVIDER.to_string(),
                resolve_codex_root(Some(settings.codex_root.as_str()))?.join("prompts"),
            ),
            (
                OPENCODE_PROVIDER.to_string(),
                default_opencode_config_root()?.join("command"),
            ),
            (
                COPILOT_PROVIDER.to_string(),
                resolve_copilot_root(Some(settings.copilot_root.as_str()))?.join("prompts"),
            ),
        ]),
    }
}

fn default_agents_root() -> Result<PathBuf, String> {
    let user_profile = env::var("USERPROFILE")
        .map_err(|_| "USERPROFILE environment variable is not set".to_string())?;
    Ok(PathBuf::from(user_profile).join(".agents"))
}

fn resolve_project_target_root(primary: &Path, fallback: &Path) -> PathBuf {
    if primary.exists() || !fallback.exists() {
        primary.to_path_buf()
    } else {
        fallback.to_path_buf()
    }
}

fn project_agents_prefs_path(project_cwd: &str) -> PathBuf {
    PathBuf::from(project_cwd)
        .join(".sessionhub")
        .join(AGENTS_PREFS_FILE_NAME)
}

fn appdata_project_agents_prefs_path(project_cwd: &str) -> Result<PathBuf, String> {
    let normalized = project_cwd.to_lowercase();
    let digest = Sha256::digest(normalized.as_bytes());
    let file_name = format!("{:x}.json", digest);
    Ok(default_app_data_dir()?
        .join("project-agents")
        .join(file_name))
}

fn read_project_agents_prefs(path: &Path) -> Result<ProjectAgentsPrefs, String> {
    let content = fs::read_to_string(path)
        .map_err(|error| format!("failed to read agents prefs {}: {error}", path.display()))?;
    serde_json::from_str::<ProjectAgentsPrefs>(&content)
        .map_err(|error| format!("failed to parse agents prefs {}: {error}", path.display()))
}

fn write_project_agents_prefs(path: &Path, prefs: &ProjectAgentsPrefs) -> Result<(), String> {
    let content = serde_json::to_vec_pretty(prefs)
        .map_err(|error| format!("failed to serialize agents prefs: {error}"))?;
    atomic_write_file(path, &content)
}

fn resolve_scoped_write_path(scope_root: &Path, file_path: &Path) -> Result<PathBuf, String> {
    let canonical_root = scope_root.canonicalize().map_err(|error| {
        format!(
            "failed to resolve scope root {}: {error}",
            scope_root.display()
        )
    })?;
    let target = if file_path.is_absolute() {
        file_path.to_path_buf()
    } else {
        canonical_root.join(file_path)
    };
    let resolved = resolve_existing_ancestor_path(&target)?;
    if !resolved.starts_with(&canonical_root) {
        return Err("Access denied: path traversal detected".to_string());
    }
    Ok(resolved)
}

fn resolve_existing_ancestor_path(path: &Path) -> Result<PathBuf, String> {
    let mut pending = Vec::new();
    let mut current = path;
    loop {
        if has_parent_dir_component(current) {
            return Err("Access denied: path traversal detected".to_string());
        }
        if current.exists() {
            let mut resolved = current.canonicalize().map_err(|error| {
                format!("failed to resolve path {}: {error}", current.display())
            })?;
            for component in pending.iter().rev() {
                resolved.push(component);
            }
            return Ok(resolved);
        }

        let file_name = current
            .file_name()
            .ok_or_else(|| format!("failed to resolve path {}", path.display()))?
            .to_os_string();
        pending.push(file_name);
        current = current
            .parent()
            .ok_or_else(|| format!("failed to resolve path {}", path.display()))?;
    }
}

fn validate_sync_path(path: &Path) -> Result<(), String> {
    if has_parent_dir_component(path) {
        return Err(format!(
            "invalid path {}: parent traversal is not allowed",
            path.display()
        ));
    }
    Ok(())
}

fn has_parent_dir_component(path: &Path) -> bool {
    path.components()
        .any(|component| matches!(component, Component::ParentDir))
}

fn atomic_write_file(path: &Path, content: &[u8]) -> Result<(), String> {
    ensure_parent_dir(path)?;
    let file_name = path
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or_else(|| format!("invalid target path {}", path.display()))?;
    let temp_path = path.with_file_name(format!("{file_name}.tmp-sessionhub"));
    fs::write(&temp_path, content)
        .map_err(|error| format!("failed to write temp file {}: {error}", temp_path.display()))?;

    if path.exists() {
        #[cfg(target_os = "windows")]
        {
            use windows_sys::Win32::Storage::FileSystem::{
                MoveFileExW, MOVEFILE_REPLACE_EXISTING, MOVEFILE_WRITE_THROUGH,
            };

            let source = widestring(&temp_path);
            let target = widestring(path);
            // 使用系統層級 replace，避免 Windows 上 rename 無法覆蓋既有檔案。
            let ok = unsafe {
                MoveFileExW(
                    source.as_ptr(),
                    target.as_ptr(),
                    MOVEFILE_REPLACE_EXISTING | MOVEFILE_WRITE_THROUGH,
                )
            };
            if ok == 0 {
                let _ = fs::remove_file(&temp_path);
                return Err(format!("failed to replace file {}", path.display()));
            }
            return Ok(());
        }
        #[cfg(not(target_os = "windows"))]
        {
            fs::rename(&temp_path, path)
                .map_err(|error| format!("failed to replace file {}: {error}", path.display()))?;
            return Ok(());
        }
    }

    fs::rename(&temp_path, path).map_err(|error| {
        format!(
            "failed to move temp file into place {}: {error}",
            path.display()
        )
    })
}

#[cfg(target_os = "windows")]
fn widestring(path: &Path) -> Vec<u16> {
    use std::os::windows::ffi::OsStrExt;

    path.as_os_str()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect()
}

fn canonicalize_link_target(base: &Path, link_target: &Path) -> Result<PathBuf, String> {
    let joined = if link_target.is_absolute() {
        link_target.to_path_buf()
    } else {
        base.parent().unwrap_or(base).join(link_target)
    };
    joined.canonicalize().map_err(|error| {
        format!(
            "failed to resolve link target {}: {error}",
            joined.display()
        )
    })
}

fn system_time_to_millis(value: std::time::SystemTime) -> Option<u64> {
    value
        .duration_since(UNIX_EPOCH)
        .ok()
        .map(|duration| duration.as_millis() as u64)
}

fn is_target_newer(source: &FileFingerprint, target: &FileFingerprint) -> bool {
    target.mtime_ms.unwrap_or(0) > source.mtime_ms.unwrap_or(0)
}

fn is_symlink_privilege_error(error: &str) -> bool {
    error.contains("1314") || error.contains("privilege")
}

#[cfg(target_os = "windows")]
fn create_directory_symlink(source_path: &Path, target_path: &Path) -> Result<(), String> {
    std::os::windows::fs::symlink_dir(source_path, target_path).map_err(|error| {
        format!(
            "failed to create directory symlink {} -> {}: {error}",
            target_path.display(),
            source_path.display()
        )
    })
}

#[cfg(not(target_os = "windows"))]
fn create_directory_symlink(source_path: &Path, target_path: &Path) -> Result<(), String> {
    std::os::unix::fs::symlink(source_path, target_path).map_err(|error| {
        format!(
            "failed to create directory symlink {} -> {}: {error}",
            target_path.display(),
            source_path.display()
        )
    })
}

fn normalize_display_path(path: &Path) -> String {
    path.display().to_string().replace("\\\\?\\", "")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;
    use std::sync::{Mutex, MutexGuard, OnceLock};
    use std::thread;
    use std::time::Duration;

    fn test_lock() -> &'static Mutex<()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
    }

    fn lock_test() -> MutexGuard<'static, ()> {
        test_lock()
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    fn unique_test_dir(name: &str) -> PathBuf {
        let suffix = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        std::env::temp_dir().join(format!("session-hub-agents-{name}-{suffix}"))
    }

    fn write_file(path: &Path, content: &str) {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("create parent");
        }
        fs::write(path, content).expect("write file");
    }

    #[test]
    fn scan_agents_md_respects_ignore_and_depth_limits() {
        let _guard = lock_test();
        let root = unique_test_dir("scan-ignore");
        write_file(&root.join("AGENTS.md"), "root");
        write_file(&root.join("feature").join("AGENTS.md"), "feature");
        write_file(
            &root.join("node_modules").join("ignored").join("AGENTS.md"),
            "ignored",
        );
        write_file(
            &root
                .join("a")
                .join("b")
                .join("c")
                .join("d")
                .join("e")
                .join("f")
                .join("g")
                .join("h")
                .join("i")
                .join("AGENTS.md"),
            "deep",
        );

        let result = scan_agents_md_root(&root, &ProjectAgentsPrefs::default()).expect("scan");
        assert_eq!(result.entries.len(), 2);
        assert!(result
            .entries
            .iter()
            .all(|entry| !entry.dir.contains("node_modules")));
        assert!(result
            .entries
            .iter()
            .all(|entry| !entry.dir.contains("\\i") && !entry.dir.ends_with("/i")));

        fs::remove_dir_all(&root).expect("cleanup");
    }

    #[test]
    fn scan_agents_md_marks_all_statuses() {
        let _guard = lock_test();
        let root = unique_test_dir("scan-statuses");
        write_file(&root.join("sync").join("AGENTS.md"), "same");
        write_file(&root.join("sync").join("CLAUDE.md"), "same");
        write_file(&root.join("missing-target").join("AGENTS.md"), "source");
        write_file(&root.join("source-missing").join("CLAUDE.md"), "target");
        write_file(&root.join("differs").join("AGENTS.md"), "old");
        thread::sleep(Duration::from_millis(5));
        write_file(&root.join("differs").join("CLAUDE.md"), "new");

        let result = scan_agents_md_root(&root, &ProjectAgentsPrefs::default()).expect("scan");
        let map = result
            .entries
            .iter()
            .map(|entry| (entry.rel_dir.clone(), entry.clone()))
            .collect::<BTreeMap<_, _>>();
        assert_eq!(map["sync"].status, SyncStatus::InSync);
        assert_eq!(map["missing-target"].status, SyncStatus::TargetMissing);
        assert_eq!(map["source-missing"].status, SyncStatus::SourceMissing);
        assert_eq!(map["differs"].status, SyncStatus::Differs);
        assert!(map["differs"].target_newer);

        fs::remove_dir_all(&root).expect("cleanup");
    }

    #[test]
    fn scan_agents_md_sets_truncated_when_limit_exceeded() {
        let _guard = lock_test();
        let root = unique_test_dir("scan-truncated");
        fs::create_dir_all(&root).expect("root");
        for index in 0..(MAX_SCANNED_DIRS + 5) {
            fs::create_dir_all(root.join(format!("dir-{index}"))).expect("create dir");
        }

        let result = scan_agents_md_root(&root, &ProjectAgentsPrefs::default()).expect("scan");
        assert!(result.truncated);

        fs::remove_dir_all(&root).expect("cleanup");
    }

    #[test]
    fn write_agents_file_blocks_path_traversal() {
        let _guard = lock_test();
        let root = unique_test_dir("write-guard");
        fs::create_dir_all(&root).expect("root");
        let result = write_agents_file_internal(
            root.to_str().expect("root str"),
            root.join("..")
                .join("outside.md")
                .to_str()
                .expect("target str"),
            "nope",
        );
        assert!(result.is_err());

        fs::remove_dir_all(&root).expect("cleanup");
    }

    #[test]
    fn sync_agents_items_dry_run_does_not_write_file() {
        let _guard = lock_test();
        let root = unique_test_dir("sync-dry-run");
        let source = root.join("AGENTS.md");
        let target = root.join("CLAUDE.md");
        write_file(&source, "content");

        let report = sync_agents_items_internal(&SyncRequest {
            items: vec![SyncItem {
                source: normalize_display_path(&source),
                target: normalize_display_path(&target),
                item_kind: SyncItemKind::File,
                direction: None,
                target_id: None,
            }],
            dry_run: true,
            force: false,
            mode: SyncMode::Copy,
            project_cwd: None,
        })
        .expect("sync");
        assert_eq!(report.actions[0].action, SyncAction::Create);
        assert!(!target.exists());

        fs::remove_dir_all(&root).expect("cleanup");
    }

    #[test]
    fn sync_agents_items_conflict_and_force_behaviour() {
        let _guard = lock_test();
        let root = unique_test_dir("sync-conflict");
        let source = root.join("AGENTS.md");
        let target = root.join("CLAUDE.md");
        write_file(&source, "old");
        thread::sleep(Duration::from_millis(5));
        write_file(&target, "new");

        let conflict = sync_agents_items_internal(&SyncRequest {
            items: vec![SyncItem {
                source: normalize_display_path(&source),
                target: normalize_display_path(&target),
                item_kind: SyncItemKind::File,
                direction: None,
                target_id: None,
            }],
            dry_run: false,
            force: false,
            mode: SyncMode::Copy,
            project_cwd: None,
        })
        .expect("sync");
        assert_eq!(conflict.actions[0].action, SyncAction::Conflict);

        let forced = sync_agents_items_internal(&SyncRequest {
            items: vec![SyncItem {
                source: normalize_display_path(&source),
                target: normalize_display_path(&target),
                item_kind: SyncItemKind::File,
                direction: None,
                target_id: None,
            }],
            dry_run: false,
            force: true,
            mode: SyncMode::Copy,
            project_cwd: None,
        })
        .expect("sync");
        assert_eq!(forced.actions[0].action, SyncAction::Overwrite);
        assert_eq!(fs::read_to_string(&target).expect("read target"), "old");

        fs::remove_dir_all(&root).expect("cleanup");
    }

    #[test]
    fn sync_agents_items_source_missing_is_conflict_even_with_force() {
        let _guard = lock_test();
        let root = unique_test_dir("sync-source-missing");
        let source = root.join("AGENTS.md");
        let target = root.join("CLAUDE.md");
        write_file(&target, "target");

        let report = sync_agents_items_internal(&SyncRequest {
            items: vec![SyncItem {
                source: normalize_display_path(&source),
                target: normalize_display_path(&target),
                item_kind: SyncItemKind::File,
                direction: None,
                target_id: None,
            }],
            dry_run: false,
            force: true,
            mode: SyncMode::Copy,
            project_cwd: None,
        })
        .expect("sync");
        assert_eq!(report.actions[0].action, SyncAction::Conflict);

        fs::remove_dir_all(&root).expect("cleanup");
    }

    #[test]
    fn sync_agents_items_can_copy_directory_and_detect_broken_link() {
        let _guard = lock_test();
        let root = unique_test_dir("sync-dir");
        let source = root.join(".agents").join("skills").join("demo");
        let target = root.join(".claude").join("skills").join("demo");
        write_file(&source.join("SKILL.md"), "skill");
        write_file(&source.join("docs").join("usage.md"), "usage");

        let report = sync_agents_items_internal(&SyncRequest {
            items: vec![SyncItem {
                source: normalize_display_path(&source),
                target: normalize_display_path(&target),
                item_kind: SyncItemKind::Directory,
                direction: None,
                target_id: Some(CLAUDE_PROVIDER.to_string()),
            }],
            dry_run: false,
            force: false,
            mode: SyncMode::Copy,
            project_cwd: None,
        })
        .expect("sync");
        assert_eq!(report.actions[0].action, SyncAction::Create);
        assert!(target.join("SKILL.md").is_file());
        assert!(target.join("docs").join("usage.md").is_file());

        #[cfg(target_os = "windows")]
        {
            let linked_target = root.join(".codex").join("skills").join("demo");
            fs::create_dir_all(linked_target.parent().expect("parent")).expect("parent");
            let missing_source = root.join("missing-skill-source");
            std::os::windows::fs::symlink_dir(&missing_source, &linked_target).expect("symlink");
            let result = scan_agents_skills_internal(&AgentsScope::Project {
                project_cwd: normalize_display_path(&root),
            })
            .expect("scan");
            let entry = result
                .skills
                .iter()
                .find(|skill| skill.name == "demo")
                .expect("skill");
            let status = entry
                .targets
                .iter()
                .find(|target_status| target_status.target_id == CODEX_PROVIDER)
                .expect("target");
            assert!(matches!(
                status.status,
                SyncStatus::LinkBroken | SyncStatus::Error
            ));
        }

        if root.exists() {
            fs::remove_dir_all(&root).expect("cleanup");
        }
    }

    #[test]
    fn project_agents_prefs_respect_project_and_appdata_targets() {
        let _guard = lock_test();
        let project = unique_test_dir("prefs-project");
        let appdata = unique_test_dir("prefs-appdata");
        fs::create_dir_all(&project).expect("project");
        fs::create_dir_all(&appdata).expect("appdata");

        unsafe {
            env::set_var("COPILOT_SESSION_MANAGER_APPDATA_OVERRIDE", &appdata);
        }

        let prefs = ProjectAgentsPrefs {
            conflict_choice: Some("target-wins".to_string()),
            ignored_paths: vec!["tmp".to_string()],
            enabled_targets: vec![CLAUDE_PROVIDER.to_string()],
        };

        let appdata_save = save_project_agents_prefs_internal(
            project.to_str().expect("project str"),
            &prefs,
            false,
        )
        .expect("save appdata");
        assert!(!appdata_save.created_project_config_dir);
        assert!(!project_agents_prefs_path(project.to_str().expect("project str")).exists());
        assert!(
            appdata_project_agents_prefs_path(project.to_str().expect("project str"))
                .expect("fallback path")
                .exists()
        );

        let project_save = save_project_agents_prefs_internal(
            project.to_str().expect("project str"),
            &prefs,
            true,
        )
        .expect("save project");
        assert!(project_save.created_project_config_dir);
        assert!(project_agents_prefs_path(project.to_str().expect("project str")).exists());

        let changed = ProjectAgentsPrefs {
            conflict_choice: Some("source-wins".to_string()),
            ignored_paths: Vec::new(),
            enabled_targets: default_enabled_targets(),
        };
        let existing_project_save = save_project_agents_prefs_internal(
            project.to_str().expect("project str"),
            &changed,
            false,
        )
        .expect("save project existing");
        assert!(!existing_project_save.created_project_config_dir);
        let loaded = load_project_agents_prefs_internal(project.to_str().expect("project str"))
            .expect("load");
        assert_eq!(loaded.conflict_choice.as_deref(), Some("source-wins"));

        fs::remove_dir_all(&project).expect("cleanup project");
        fs::remove_dir_all(&appdata).expect("cleanup appdata");
    }

    #[test]
    fn project_target_root_prefers_github_when_present() {
        let _guard = lock_test();
        let root = unique_test_dir("project-target-root");
        let github_root = root.join(".github").join("skills");
        let legacy_root = root.join(".copilot").join("skills");
        fs::create_dir_all(&github_root).expect("create github root");
        fs::create_dir_all(&legacy_root).expect("create legacy root");

        let resolved = resolve_project_target_root(&github_root, &legacy_root);
        assert_eq!(resolved, github_root);

        fs::remove_dir_all(&root).expect("cleanup root");
    }

    #[test]
    fn project_skills_scan_reads_agents_source_with_github_target() {
        let _guard = lock_test();
        let root = unique_test_dir("project-skills-github");
        let source_skill = root.join(".agents").join("skills").join("demo-skill");
        let github_skill = root.join(".github").join("skills").join("demo-skill");

        write_file(&source_skill.join("SKILL.md"), "demo skill");
        write_file(&github_skill.join("SKILL.md"), "demo skill");

        let result = scan_agents_skills_internal(&AgentsScope::Project {
            project_cwd: normalize_display_path(&root),
        })
        .expect("scan skills");

        assert_eq!(result.skills.len(), 1);
        assert_eq!(result.skills[0].name, "demo-skill");
        let copilot = result.skills[0]
            .targets
            .iter()
            .find(|target| target.target_id == COPILOT_PROVIDER)
            .expect("copilot target");
        assert_eq!(copilot.status, SyncStatus::InSync);

        fs::remove_dir_all(&root).expect("cleanup root");
    }

    #[test]
    fn agents_scope_deserializes_camel_case_payload() {
        let scope: AgentsScope =
            serde_json::from_str(r#"{"kind":"project","projectCwd":"D:/demo"}"#)
                .expect("deserialize project scope");
        assert_eq!(
            scope,
            AgentsScope::Project {
                project_cwd: "D:/demo".to_string()
            }
        );
        let global: AgentsScope =
            serde_json::from_str(r#"{"kind":"global"}"#).expect("deserialize global scope");
        assert_eq!(global, AgentsScope::Global);
    }

    #[test]
    fn project_skills_scan_includes_target_only_entries() {
        let _guard = lock_test();
        let root = unique_test_dir("project-skills-union");
        let claude_skill = root.join(".claude").join("skills").join("target-only");
        write_file(&claude_skill.join("SKILL.md"), "target only skill");

        let result = scan_agents_skills_internal(&AgentsScope::Project {
            project_cwd: normalize_display_path(&root),
        })
        .expect("scan skills");

        let entry = result
            .skills
            .iter()
            .find(|skill| skill.name == "target-only")
            .expect("target-only skill");
        assert!(entry.skill_md_path.ends_with("SKILL.md"));
        assert!(Path::new(&entry.skill_md_path).is_file());
        let claude = entry
            .targets
            .iter()
            .find(|target| target.target_id == CLAUDE_PROVIDER)
            .expect("claude target");
        assert_eq!(claude.status, SyncStatus::SourceMissing);

        fs::remove_dir_all(&root).expect("cleanup root");
    }

    #[test]
    fn project_skills_scan_includes_nested_skill_directories() {
        let _guard = lock_test();
        let root = unique_test_dir("project-skills-nested");
        let nested_skill = root
            .join(".agents")
            .join("skills")
            .join("team")
            .join("demo-skill");

        write_file(&nested_skill.join("SKILL.md"), "nested skill");

        let result = scan_agents_skills_internal(&AgentsScope::Project {
            project_cwd: normalize_display_path(&root),
        })
        .expect("scan skills");

        let entry = result
            .skills
            .iter()
            .find(|skill| skill.name == "team/demo-skill")
            .expect("nested skill entry");
        assert!(entry.source_dir.ends_with("team\\demo-skill") || entry.source_dir.ends_with("team/demo-skill"));

        fs::remove_dir_all(&root).expect("cleanup root");
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn project_skills_scan_discovers_symlinked_skill_directories() {
        let _guard = lock_test();
        let root = unique_test_dir("project-skills-symlink");
        let real_skill = root.join("real-skills").join("linked-skill");
        write_file(&real_skill.join("SKILL.md"), "linked skill");

        let skills_root = root.join(".agents").join("skills");
        fs::create_dir_all(&skills_root).expect("skills root");
        std::os::windows::fs::symlink_dir(&real_skill, skills_root.join("linked-skill"))
            .expect("symlink skill");

        let claude_skill = root.join(".claude").join("skills").join("linked-skill");
        write_file(&claude_skill.join("SKILL.md"), "linked skill");

        let result = scan_agents_skills_internal(&AgentsScope::Project {
            project_cwd: normalize_display_path(&root),
        })
        .expect("scan skills");

        let entry = result
            .skills
            .iter()
            .find(|skill| skill.name == "linked-skill")
            .expect("symlinked skill entry");
        assert_eq!(entry.file_count, 1);
        let claude = entry
            .targets
            .iter()
            .find(|target| target.target_id == CLAUDE_PROVIDER)
            .expect("claude target");
        assert_eq!(claude.status, SyncStatus::InSync);

        fs::remove_dir_all(&root).expect("cleanup root");
    }

    #[test]
    fn project_commands_scan_includes_target_only_entries() {
        let _guard = lock_test();
        let root = unique_test_dir("project-commands-union");
        let claude_command = root
            .join(".claude")
            .join("commands")
            .join("opsx")
            .join("apply.md");
        let opencode_command = root.join(".opencode").join("command").join("build.md");

        write_file(&claude_command, "# apply");
        write_file(&opencode_command, "# build");

        let result = scan_agents_commands_internal(&AgentsScope::Project {
            project_cwd: normalize_display_path(&root),
        })
        .expect("scan commands");

        assert!(result
            .commands
            .iter()
            .any(|entry| entry.name == "opsx/apply"));
        assert!(result.commands.iter().any(|entry| entry.name == "build"));

        fs::remove_dir_all(&root).expect("cleanup root");
    }

    #[test]
    fn project_commands_scan_matches_copilot_prompt_suffix_to_shared_name() {
        let _guard = lock_test();
        let root = unique_test_dir("project-commands-copilot-prompt-suffix");
        let source_command = root
            .join(".agents")
            .join("skills")
            .join("command")
            .join("opsx-apply.md");
        let claude_command = root.join(".claude").join("commands").join("opsx-apply.md");
        // GitHub Copilot prompt files 慣例：副檔名固定為 .prompt.md。
        let copilot_command = root
            .join(".github")
            .join("prompts")
            .join("opsx-apply.prompt.md");

        write_file(&source_command, "# apply");
        write_file(&claude_command, "# apply");
        write_file(&copilot_command, "# apply");

        let result = scan_agents_commands_internal(&AgentsScope::Project {
            project_cwd: normalize_display_path(&root),
        })
        .expect("scan commands");

        // 不應出現被誤剝離出的 "opsx-apply.prompt" 這個獨立項目。
        assert!(!result
            .commands
            .iter()
            .any(|entry| entry.name == "opsx-apply.prompt"));

        let entry = result
            .commands
            .iter()
            .find(|entry| entry.name == "opsx-apply")
            .expect("opsx-apply entry");

        let claude_status = entry
            .targets
            .iter()
            .find(|target| target.target_id == CLAUDE_PROVIDER)
            .expect("claude target");
        assert_eq!(claude_status.status, SyncStatus::InSync);

        let copilot_status = entry
            .targets
            .iter()
            .find(|target| target.target_id == COPILOT_PROVIDER)
            .expect("copilot target");
        assert_eq!(copilot_status.status, SyncStatus::InSync);

        fs::remove_dir_all(&root).expect("cleanup root");
    }

    #[test]
    fn project_commands_scan_marks_missing_other_targets_as_target_missing() {
        let _guard = lock_test();
        let root = unique_test_dir("project-commands-missing-target-status");
        let codex_command = root.join(".codex").join("prompts").join("opsx-apply.md");

        write_file(&codex_command, "# apply");

        let result = scan_agents_commands_internal(&AgentsScope::Project {
            project_cwd: normalize_display_path(&root),
        })
        .expect("scan commands");

        let entry = result
            .commands
            .iter()
            .find(|command| command.name == "opsx-apply")
            .expect("opsx-apply entry");

        let codex_status = entry
            .targets
            .iter()
            .find(|target| target.target_id == CODEX_PROVIDER)
            .expect("codex target");
        assert_eq!(codex_status.status, SyncStatus::SourceMissing);

        let claude_status = entry
            .targets
            .iter()
            .find(|target| target.target_id == CLAUDE_PROVIDER)
            .expect("claude target");
        assert_eq!(claude_status.status, SyncStatus::TargetMissing);

        fs::remove_dir_all(&root).expect("cleanup root");
    }
}
