use std::fs;
use std::path::Path;

use crate::types::*;

pub(crate) fn extract_md_heading(content: &str) -> Option<String> {
    for line in content.lines() {
        let trimmed = line.trim();
        if let Some(heading) = trimmed.strip_prefix("# ") {
            let heading = heading.trim();
            if !heading.is_empty() {
                return Some(heading.to_string());
            }
        }
    }
    None
}

pub(crate) fn extract_md_tldr(content: &str) -> Option<String> {
    let mut in_tldr = false;
    let mut lines: Vec<&str> = Vec::new();

    for line in content.lines() {
        let trimmed = line.trim();
        if in_tldr {
            if trimmed.starts_with("## ") || trimmed.starts_with("# ") {
                break;
            }
            if !trimmed.is_empty() {
                lines.push(trimmed);
                if lines.len() >= 5 {
                    break;
                }
            }
        } else if trimmed.starts_with("## TL;DR") || trimmed.starts_with("## tl;dr") {
            in_tldr = true;
        }
    }

    if lines.is_empty() {
        None
    } else {
        Some(lines.join("\n"))
    }
}

pub(crate) fn scan_sisyphus_internal(project_dir: &Path) -> SisyphusData {
    let sisyphus_dir = project_dir.join(".sisyphus");

    if !sisyphus_dir.is_dir() {
        return SisyphusData {
            active_plan: None,
            plans: Vec::new(),
            notepads: Vec::new(),
            evidence_files: Vec::new(),
            draft_files: Vec::new(),
        };
    }

    let boulder = {
        let boulder_path = sisyphus_dir.join("boulder.json");
        if boulder_path.is_file() {
            fs::read_to_string(&boulder_path)
                .ok()
                .and_then(|content| serde_json::from_str::<SisyphusBoulder>(&content).ok())
        } else {
            None
        }
    };

    let active_plan_name = boulder
        .as_ref()
        .and_then(|b| b.active_plan.as_deref())
        .map(|p| {
            Path::new(p)
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or(p)
                .to_string()
        });

    let plans = {
        let plans_dir = sisyphus_dir.join("plans");
        if plans_dir.is_dir() {
            let mut result = Vec::new();
            if let Ok(entries) = fs::read_dir(&plans_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.extension().and_then(|e| e.to_str()) == Some("md") {
                        let name = path
                            .file_stem()
                            .and_then(|s| s.to_str())
                            .unwrap_or("unknown")
                            .to_string();

                        let content = fs::read_to_string(&path).unwrap_or_default();
                        let title = extract_md_heading(&content);
                        let tldr = extract_md_tldr(&content);
                        let is_active = active_plan_name
                            .as_deref()
                            .map(|ap| ap == name)
                            .unwrap_or(false);

                        result.push(SisyphusPlan {
                            name,
                            path: path.to_string_lossy().to_string(),
                            title,
                            tldr,
                            is_active,
                        });
                    }
                }
            }
            result.sort_by(|a, b| a.name.cmp(&b.name));
            result
        } else {
            Vec::new()
        }
    };

    let notepads = {
        let notepads_dir = sisyphus_dir.join("notepads");
        if notepads_dir.is_dir() {
            let mut result = Vec::new();
            if let Ok(entries) = fs::read_dir(&notepads_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_dir() {
                        let name = path
                            .file_name()
                            .and_then(|s| s.to_str())
                            .unwrap_or("unknown")
                            .to_string();
                        let has_issues = path.join("issues.md").is_file();
                        let has_learnings = path.join("learnings.md").is_file();
                        result.push(SisyphusNotepad {
                            name,
                            has_issues,
                            has_learnings,
                        });
                    }
                }
            }
            result.sort_by(|a, b| a.name.cmp(&b.name));
            result
        } else {
            Vec::new()
        }
    };

    let evidence_files = list_files_with_ext(&sisyphus_dir.join("evidence"), "txt");
    let draft_files = list_files_with_ext(&sisyphus_dir.join("drafts"), "md");

    SisyphusData {
        active_plan: boulder,
        plans,
        notepads,
        evidence_files,
        draft_files,
    }
}

pub(crate) fn list_files_with_ext(dir: &Path, ext: &str) -> Vec<String> {
    if !dir.is_dir() {
        return Vec::new();
    }
    let mut result = Vec::new();
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() && path.extension().and_then(|e| e.to_str()) == Some(ext) {
                if let Some(name) = path.file_name().and_then(|s| s.to_str()) {
                    result.push(name.to_string());
                }
            }
        }
    }
    result.sort();
    result
}
