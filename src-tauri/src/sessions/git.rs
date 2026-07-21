use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

use crate::types::{SessionInfo, CREATE_NO_WINDOW};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct GitMetadata {
    pub(crate) repo_root: String,
    pub(crate) repo_name: String,
    pub(crate) git_branch: Option<String>,
}

fn build_git_command(cwd: &Path) -> Command {
    let mut command = Command::new("git");
    command.arg("-C").arg(cwd);
    #[cfg(target_os = "windows")]
    {
        command.creation_flags(CREATE_NO_WINDOW);
    }
    command
}

fn run_git_command(cwd: &Path, args: &[&str]) -> Option<String> {
    let output = build_git_command(cwd).args(args).output().ok()?;
    if !output.status.success() {
        return None;
    }

    let value = String::from_utf8(output.stdout).ok()?;
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return None;
    }

    Some(trimmed.to_string())
}

fn normalize_display_path(path: &Path) -> String {
    let resolved = fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
    resolved
        .to_string_lossy()
        .trim_start_matches(r"\\?\")
        .to_string()
}

fn parse_repo_name_from_remote(remote: &str) -> Option<String> {
    let trimmed = remote.trim().trim_end_matches('/');
    let tail = trimmed.rsplit(['/', '\\', ':']).next()?.trim();
    let name = tail.strip_suffix(".git").unwrap_or(tail).trim();
    if name.is_empty() {
        return None;
    }
    Some(name.to_string())
}

fn resolve_git_branch(cwd: &Path) -> Option<String> {
    let branch = run_git_command(cwd, &["rev-parse", "--abbrev-ref", "HEAD"])?;
    if branch != "HEAD" {
        return Some(branch);
    }

    run_git_command(cwd, &["rev-parse", "--short", "HEAD"])
}

pub(crate) fn resolve_git_metadata(cwd: &str) -> Option<GitMetadata> {
    let cwd_path = Path::new(cwd);
    if !cwd_path.exists() {
        return None;
    }

    let repo_root_raw = run_git_command(cwd_path, &["rev-parse", "--show-toplevel"])?;
    let repo_root_path = PathBuf::from(&repo_root_raw);
    let repo_root = normalize_display_path(&repo_root_path);
    let repo_name = run_git_command(cwd_path, &["config", "--get", "remote.origin.url"])
        .and_then(|remote| parse_repo_name_from_remote(&remote))
        .or_else(|| {
            repo_root_path
                .file_name()
                .map(|value| value.to_string_lossy().to_string())
        })?;

    Some(GitMetadata {
        repo_root,
        repo_name,
        git_branch: resolve_git_branch(cwd_path),
    })
}

pub(crate) fn enrich_sessions_with_git_metadata(sessions: &mut [SessionInfo]) {
    let mut cache: HashMap<String, Option<GitMetadata>> = HashMap::new();

    for session in sessions {
        let Some(cwd) = session
            .cwd
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
        else {
            continue;
        };

        let metadata = cache
            .entry(cwd.to_string())
            .or_insert_with(|| resolve_git_metadata(cwd));

        if let Some(metadata) = metadata.as_ref() {
            session.repo_root = Some(metadata.repo_root.clone());
            session.repo_name = Some(metadata.repo_name.clone());
            session.git_branch = metadata.git_branch.clone();
        }
    }
}
