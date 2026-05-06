use std::fs;
use std::path::{Path, PathBuf};

use crate::types::*;

fn scan_openspec_specs(specs_dir: &Path) -> Vec<OpenSpecSpec> {
    if !specs_dir.is_dir() {
        return Vec::new();
    }

    let mut result = Vec::new();
    if let Ok(entries) = fs::read_dir(specs_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let name = path
                    .file_name()
                    .and_then(|s| s.to_str())
                    .unwrap_or("unknown")
                    .to_string();
                let spec_md_path = path.join("spec.md");
                let spec_path = if spec_md_path.is_file() {
                    spec_md_path.to_string_lossy().to_string()
                } else {
                    path.to_string_lossy().to_string()
                };
                result.push(OpenSpecSpec {
                    name,
                    path: spec_path,
                });
            }
        }
    }

    result.sort_by(|a, b| a.name.cmp(&b.name));
    result
}

pub(crate) fn scan_openspec_change(change_dir: &Path) -> OpenSpecChange {
    let name = change_dir
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown")
        .to_string();

    let has_proposal = change_dir.join("proposal.md").is_file();
    let has_design = change_dir.join("design.md").is_file();
    let has_tasks = change_dir.join("tasks.md").is_file();

    let specs_dir = change_dir.join("specs");
    let specs = scan_openspec_specs(&specs_dir);
    let specs_count = specs.len();

    OpenSpecChange {
        name,
        has_proposal,
        has_design,
        has_tasks,
        specs_count,
        specs,
    }
}

pub(crate) fn scan_openspec_internal(project_dir: &Path) -> OpenSpecData {
    let openspec_dir = project_dir.join("openspec");

    if !openspec_dir.is_dir() {
        return OpenSpecData {
            schema: None,
            active_changes: Vec::new(),
            archived_changes: Vec::new(),
            specs: Vec::new(),
        };
    }

    let schema = {
        let config_path = openspec_dir.join("config.yaml");
        if config_path.is_file() {
            fs::read_to_string(&config_path)
                .ok()
                .and_then(|content| serde_yaml::from_str::<serde_yaml::Value>(&content).ok())
                .and_then(|value| {
                    value
                        .get("schema")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string())
                })
        } else {
            None
        }
    };

    let active_changes = {
        let changes_dir = openspec_dir.join("changes");
        if changes_dir.is_dir() {
            let mut result = Vec::new();
            if let Ok(entries) = fs::read_dir(&changes_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_dir() {
                        let dir_name = path.file_name().and_then(|s| s.to_str()).unwrap_or("");
                        if dir_name != "archive" {
                            result.push(scan_openspec_change(&path));
                        }
                    }
                }
            }
            result.sort_by(|a, b| a.name.cmp(&b.name));
            result
        } else {
            Vec::new()
        }
    };

    let archived_changes = {
        let archive_dir = openspec_dir.join("changes").join("archive");
        if archive_dir.is_dir() {
            let mut result = Vec::new();
            if let Ok(entries) = fs::read_dir(&archive_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_dir() {
                        result.push(scan_openspec_change(&path));
                    }
                }
            }
            result.sort_by(|a, b| a.name.cmp(&b.name));
            result
        } else {
            Vec::new()
        }
    };

    let specs = {
        let specs_dir = openspec_dir.join("specs");
        scan_openspec_specs(&specs_dir)
    };

    OpenSpecData {
        schema,
        active_changes,
        archived_changes,
        specs,
    }
}

pub(crate) fn read_openspec_file_internal(
    project_cwd: &str,
    relative_path: &str,
) -> Result<String, String> {
    let canonical_target = resolve_openspec_file_internal(project_cwd, relative_path)?;

    fs::read_to_string(&canonical_target).map_err(|e| format!("Failed to read file: {e}"))
}

pub(crate) fn write_openspec_file_internal(
    project_cwd: &str,
    relative_path: &str,
    content: &str,
) -> Result<(), String> {
    let canonical_target = resolve_openspec_file_internal(project_cwd, relative_path)?;

    fs::write(&canonical_target, content).map_err(|e| format!("Failed to write file: {e}"))
}

fn resolve_openspec_file_internal(project_cwd: &str, relative_path: &str) -> Result<PathBuf, String> {
    let base = PathBuf::from(project_cwd).join("openspec");
    let canonical_base = base
        .canonicalize()
        .map_err(|_| format!("openspec directory not found: {}", base.display()))?;

    let target = canonical_base.join(relative_path);
    let canonical_target = target
        .canonicalize()
        .map_err(|_| format!("File not found: {}", target.display()))?;

    // 路徑遍歷保護：確保目標在 openspec 目錄內
    if !canonical_target.starts_with(&canonical_base) {
        return Err("Access denied: path traversal detected".to_string());
    }

    if !canonical_target.is_file() {
        return Err(format!("Not a file: {}", canonical_target.display()));
    }

    Ok(canonical_target)
}
