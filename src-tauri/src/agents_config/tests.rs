
use super::*;
use std::collections::BTreeMap;
use std::env;
use std::sync::MutexGuard;
use std::thread;
use std::time::Duration;

fn lock_test() -> MutexGuard<'static, ()> {
    crate::shared_env_test_lock()
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
fn provider_aware_scan_unions_roots_and_keeps_provider_identity() {
    let _guard = lock_test();
    let saved_vars = ["COPILOT_SESSION_MANAGER_APPDATA_OVERRIDE", "APPDATA", "USERPROFILE"]
        .map(|key| (key, env::var_os(key)));
    let root = unique_test_dir("provider-aware-union");
    write_file(&root.join(".codex/skills/shared/SKILL.md"), "codex");
    write_file(&root.join(".agents/skills/shared/SKILL.md"), "agents");
    write_file(&root.join(".github/skills/copilot/SKILL.md"), "copilot");
    write_file(&root.join(".gemini/commands/review.toml"), "prompt = \"review\"");

    let appdata = unique_test_dir("provider-aware-appdata");
    fs::create_dir_all(&appdata).expect("appdata");
    unsafe {
        env::set_var("COPILOT_SESSION_MANAGER_APPDATA_OVERRIDE", &appdata);
        env::set_var("APPDATA", &appdata);
        env::set_var("USERPROFILE", &root);
    }
    let mut settings = AppSettings::default().expect("settings");
    settings.agents_source_root = root.join(".agents").to_string_lossy().to_string();
    settings.copilot_root = root.join(".copilot").to_string_lossy().to_string();
    settings.codex_root = root.join(".codex").to_string_lossy().to_string();
    settings.claude_root = root.join(".claude").to_string_lossy().to_string();
    settings.opencode_root = root.join(".opencode").to_string_lossy().to_string();
    settings.antigravity_root = root.join(".gemini").to_string_lossy().to_string();
    fs::create_dir_all(crate::settings::default_app_data_dir().expect("appdata path")).expect("settings parent");
    fs::write(
        crate::settings::settings_file_path().expect("settings path"),
        serde_json::to_vec_pretty(&settings).expect("settings JSON"),
    )
    .expect("write settings");

    let scope = AgentsScope::Project { project_cwd: normalize_display_path(&root) };
    let result = scan_agents_skills_with_providers(&scope, &[CODEX_PROVIDER.to_string(), COPILOT_PROVIDER.to_string()]).expect("skills");
    assert_eq!(result.enabled_providers, vec![CODEX_PROVIDER, COPILOT_PROVIDER]);
    assert!(result.skills.iter().any(|skill| skill.provider_id.as_deref() == Some(CODEX_PROVIDER) && skill.locations.len() == 2));
    assert!(result.skills.iter().any(|skill| skill.provider_id.as_deref() == Some(COPILOT_PROVIDER)));
    assert!(!result.skills.iter().any(|skill| skill.provider_id.as_deref() == Some(CLAUDE_PROVIDER)));

    let commands = scan_agents_commands_with_providers(&scope, &["antigravity".to_string()]).expect("commands");
    assert_eq!(commands.enabled_providers, vec!["antigravity"]);
    assert_eq!(commands.commands[0].name, "review");
    assert!(commands.commands[0].source_path.ends_with("review.toml"));

    let _ = fs::remove_dir_all(&root);
    let _ = fs::remove_dir_all(&appdata);
    for (key, value) in saved_vars {
        unsafe {
            match value {
                Some(value) => env::set_var(key, value),
                None => env::remove_var(key),
            }
        }
    }
}

#[test]
fn provider_aware_scan_supports_command_formats_and_disabled_providers() {
    let _guard = lock_test();
    let saved_vars = ["COPILOT_SESSION_MANAGER_APPDATA_OVERRIDE", "APPDATA", "USERPROFILE"]
        .map(|key| (key, env::var_os(key)));
    let root = unique_test_dir("provider-aware-formats");
    write_file(&root.join(".opencode/command/singular.md"), "singular");
    write_file(&root.join(".opencode/commands/plural.md"), "plural");
    write_file(&root.join(".github/prompts/copilot.prompt.md"), "copilot");
    write_file(&root.join(".gemini/commands/review.toml"), "prompt = \"review\"");
    write_file(&root.join(".claude/commands/disabled.md"), "disabled");

    let appdata = unique_test_dir("provider-aware-formats-appdata");
    fs::create_dir_all(&appdata).expect("appdata");
    unsafe {
        env::set_var("COPILOT_SESSION_MANAGER_APPDATA_OVERRIDE", &appdata);
        env::set_var("APPDATA", &appdata);
        env::set_var("USERPROFILE", &root);
    }
    let mut settings = AppSettings::default().expect("settings");
    settings.opencode_root = root.join(".opencode").to_string_lossy().to_string();
    settings.copilot_root = root.join(".copilot").to_string_lossy().to_string();
    settings.antigravity_root = root.join(".gemini").to_string_lossy().to_string();
    fs::create_dir_all(crate::settings::default_app_data_dir().expect("appdata path")).expect("settings parent");
    fs::write(
        crate::settings::settings_file_path().expect("settings path"),
        serde_json::to_vec_pretty(&settings).expect("settings JSON"),
    )
    .expect("write settings");

    let scope = AgentsScope::Project { project_cwd: normalize_display_path(&root) };
    let result = scan_agents_commands_with_providers(
        &scope,
        &[OPENCODE_PROVIDER.to_string(), COPILOT_PROVIDER.to_string(), "antigravity".to_string()],
    )
    .expect("commands");
    let names = result.commands.iter().map(|command| command.name.as_str()).collect::<Vec<_>>();
    assert!(names.contains(&"singular"));
    assert!(names.contains(&"plural"));
    assert!(names.contains(&"copilot"));
    assert!(names.contains(&"review"));
    assert!(!names.contains(&"disabled"));
    assert!(result.commands.iter().find(|command| command.name == "copilot").expect("copilot").source_path.ends_with("copilot.prompt.md"));
    assert!(result.commands.iter().find(|command| command.name == "review").expect("gemini").source_path.ends_with("review.toml"));

    let _ = fs::remove_dir_all(&root);
    let _ = fs::remove_dir_all(&appdata);
    for (key, value) in saved_vars {
        unsafe {
            match value {
                Some(value) => env::set_var(key, value),
                None => env::remove_var(key),
            }
        }
    }
}

#[test]
fn resource_scan_signature_changes_when_resource_file_changes() {
    let _guard = lock_test();
    let saved_vars = ["COPILOT_SESSION_MANAGER_APPDATA_OVERRIDE", "APPDATA", "USERPROFILE"]
        .map(|key| (key, env::var_os(key)));
    let root = unique_test_dir("resource-signature");
    let appdata = unique_test_dir("resource-signature-appdata");
    write_file(&root.join(".agents/skills/shared/SKILL.md"), "first");
    fs::create_dir_all(&appdata).expect("appdata");
    unsafe {
        env::set_var("COPILOT_SESSION_MANAGER_APPDATA_OVERRIDE", &appdata);
        env::set_var("APPDATA", &appdata);
        env::set_var("USERPROFILE", &root);
    }
    let scope = AgentsScope::Project { project_cwd: normalize_display_path(&root) };
    let before = resource_scan_signature(&scope, ResourceKind::Skill, &[CODEX_PROVIDER.to_string()]).expect("before signature");
    write_file(&root.join(".agents/skills/shared/SKILL.md"), "second content");
    let after = resource_scan_signature(&scope, ResourceKind::Skill, &[CODEX_PROVIDER.to_string()]).expect("after signature");
    assert_ne!(before, after);

    let _ = fs::remove_dir_all(&root);
    let _ = fs::remove_dir_all(&appdata);
    for (key, value) in saved_vars {
        unsafe {
            match value {
                Some(value) => env::set_var(key, value),
                None => env::remove_var(key),
            }
        }
    }
}

#[cfg(target_os = "windows")]
fn create_file_symlink(source_path: &Path, target_path: &Path) {
    std::os::windows::fs::symlink_file(source_path, target_path).expect("create file symlink");
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
        let broken_source = root.join(".agents").join("skills").join("broken-link-demo");
        write_file(&broken_source.join("SKILL.md"), "broken link demo");
        let linked_target = root.join(".claude").join("skills").join("broken-link-demo");
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
            .find(|skill| skill.name == "broken-link-demo")
            .expect("skill");
        let status = entry
            .targets
            .iter()
            .find(|target_status| target_status.target_id == CLAUDE_PROVIDER)
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

    let appdata_save =
        save_project_agents_prefs_internal(project.to_str().expect("project str"), &prefs, false)
            .expect("save appdata");
    assert!(!appdata_save.created_project_config_dir);
    assert!(!project_agents_prefs_path(project.to_str().expect("project str")).exists());
    assert!(
        appdata_project_agents_prefs_path(project.to_str().expect("project str"))
            .expect("fallback path")
            .exists()
    );

    let project_save =
        save_project_agents_prefs_internal(project.to_str().expect("project str"), &prefs, true)
            .expect("save project");
    assert!(project_save.created_project_config_dir);
    assert!(project_agents_prefs_path(project.to_str().expect("project str")).exists());

    let changed = ProjectAgentsPrefs {
        conflict_choice: Some("source-wins".to_string()),
        ignored_paths: Vec::new(),
        enabled_targets: default_enabled_targets(),
    };
    let existing_project_save =
        save_project_agents_prefs_internal(project.to_str().expect("project str"), &changed, false)
            .expect("save project existing");
    assert!(!existing_project_save.created_project_config_dir);
    let loaded =
        load_project_agents_prefs_internal(project.to_str().expect("project str")).expect("load");
    assert_eq!(loaded.conflict_choice.as_deref(), Some("source-wins"));

    fs::remove_dir_all(&project).expect("cleanup project");
    fs::remove_dir_all(&appdata).expect("cleanup appdata");
}

#[test]
fn skill_target_roots_project_scope_only_targets_claude() {
    let settings = AppSettings::default().expect("default settings");
    let targets = skill_target_roots(
        &AgentsScope::Project {
            project_cwd: "D:/demo".to_string(),
        },
        &settings,
    )
    .expect("targets");
    assert_eq!(targets.len(), 1);
    assert_eq!(targets[0].0, CLAUDE_PROVIDER);
}

#[test]
fn skill_target_roots_global_scope_default_source_only_targets_claude() {
    let mut settings = AppSettings::default().expect("default settings");
    settings.agents_source_root = String::new();
    let targets = skill_target_roots(&AgentsScope::Global, &settings).expect("targets");
    assert_eq!(targets.len(), 1);
    assert_eq!(targets[0].0, CLAUDE_PROVIDER);
}

#[test]
fn skill_target_roots_global_scope_custom_source_targets_agents_and_claude() {
    let mut settings = AppSettings::default().expect("default settings");
    settings.agents_source_root = "D:/custom/agents".to_string();
    let targets = skill_target_roots(&AgentsScope::Global, &settings).expect("targets");
    assert_eq!(targets.len(), 2);
    assert_eq!(targets[0].0, AGENTS_PROVIDER);
    assert_eq!(targets[1].0, CLAUDE_PROVIDER);
}

#[test]
fn agents_root_link_missing_creates_symlink() {
    let _guard = lock_test();
    let root = unique_test_dir("agents-root-link-missing");
    let agents_root = root.join(".agents");
    let source_root = root.join("custom-agents");
    fs::create_dir_all(&source_root).expect("create source");

    assert_eq!(
        check_agents_root_link_against(&agents_root, &source_root).expect("check"),
        AgentsRootLinkStatus::Missing
    );

    #[cfg(target_os = "windows")]
    {
        let status = link_agents_root_to(&agents_root, &source_root).expect("link");
        assert_eq!(status, AgentsRootLinkStatus::Linked);
        assert_eq!(
            check_agents_root_link_against(&agents_root, &source_root).expect("check"),
            AgentsRootLinkStatus::Linked
        );

        let result = link_agents_root_to(&agents_root, &source_root);
        assert!(result.is_err());
    }

    fs::remove_dir_all(&root).expect("cleanup root");
}

#[test]
fn agents_root_link_physical_directory_statuses_and_create_protection() {
    let _guard = lock_test();
    let root = unique_test_dir("agents-root-link-physical");
    let agents_root = root.join(".agents");
    let source_root = root.join("custom-agents");
    fs::create_dir_all(source_root.join("instructions")).expect("create source instructions");
    fs::create_dir_all(source_root.join("skills")).expect("create source skills");
    write_file(&source_root.join("AGENTS.md"), "source instructions");
    fs::create_dir_all(&agents_root).expect("create physical agents root");
    write_file(&agents_root.join("marker.txt"), "existing content");

    assert_eq!(
        check_agents_root_link_against(&agents_root, &source_root).expect("check"),
        AgentsRootLinkStatus::UnlinkedPhysical
    );

    let result = link_agents_root_to(&agents_root, &source_root);
    assert!(result.is_err());
    assert!(agents_root.join("marker.txt").is_file());

    #[cfg(target_os = "windows")]
    {
        create_directory_symlink(
            &source_root.join("instructions"),
            &agents_root.join("instructions"),
        )
        .expect("link instructions");
        assert_eq!(
            check_agents_root_link_against(&agents_root, &source_root).expect("check partial"),
            AgentsRootLinkStatus::Partial {
                unmatched_items: vec!["AGENTS.md".to_string(), "skills".to_string()],
            }
        );
        assert!(link_agents_root_to(&agents_root, &source_root).is_err());

        create_directory_symlink(&source_root.join("skills"), &agents_root.join("skills"))
            .expect("link skills");
        create_file_symlink(
            &source_root.join("AGENTS.md"),
            &agents_root.join("AGENTS.md"),
        );
        assert_eq!(
            check_agents_root_link_against(&agents_root, &source_root).expect("check linked"),
            AgentsRootLinkStatus::Linked
        );

        fs::remove_dir(&agents_root.join("skills")).expect("remove skills link");
        fs::create_dir_all(agents_root.join("skills")).expect("create physical skills copy");
        assert_eq!(
            check_agents_root_link_against(&agents_root, &source_root).expect("check copied item"),
            AgentsRootLinkStatus::Partial {
                unmatched_items: vec!["skills".to_string()],
            }
        );
    }

    fs::remove_dir_all(&root).expect("cleanup root");
}

#[test]
fn agents_root_link_rejects_symlink_to_different_source() {
    let _guard = lock_test();
    let root = unique_test_dir("agents-root-link-not-linked");
    let agents_root = root.join(".agents");
    let source_root = root.join("custom-agents");
    let other_source = root.join("other-agents");
    fs::create_dir_all(&source_root).expect("create source");
    fs::create_dir_all(&other_source).expect("create other source");

    #[cfg(target_os = "windows")]
    {
        create_directory_symlink(&other_source, &agents_root).expect("create other source link");
        assert_eq!(
            check_agents_root_link_against(&agents_root, &source_root).expect("check"),
            AgentsRootLinkStatus::NotLinked
        );
        assert!(link_agents_root_to(&agents_root, &source_root).is_err());
    }

    fs::remove_dir_all(&root).expect("cleanup root");
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
fn project_skills_scan_only_targets_claude() {
    let _guard = lock_test();
    let root = unique_test_dir("project-skills-claude-only");
    let source_skill = root.join(".agents").join("skills").join("demo-skill");
    let claude_skill = root.join(".claude").join("skills").join("demo-skill");

    write_file(&source_skill.join("SKILL.md"), "demo skill");
    write_file(&claude_skill.join("SKILL.md"), "demo skill");

    let result = scan_agents_skills_internal(&AgentsScope::Project {
        project_cwd: normalize_display_path(&root),
    })
    .expect("scan skills");

    assert_eq!(result.targets.len(), 1);
    assert_eq!(result.targets[0].target_id, CLAUDE_PROVIDER);
    assert_eq!(result.skills.len(), 1);
    assert_eq!(result.skills[0].name, "demo-skill");
    let claude = result.skills[0]
        .targets
        .iter()
        .find(|target| target.target_id == CLAUDE_PROVIDER)
        .expect("claude target");
    assert_eq!(claude.status, SyncStatus::InSync);

    fs::remove_dir_all(&root).expect("cleanup root");
}

#[test]
fn project_skills_scan_extracts_description_from_frontmatter() {
    let _guard = lock_test();
    let root = unique_test_dir("project-skills-description");
    let source_skill = root.join(".agents").join("skills").join("demo-skill");
    write_file(
        &source_skill.join(SKILL_FILE_NAME),
        "---\nname: demo-skill\ndescription: Does the demo thing.\n---\n\nbody",
    );

    let result = scan_agents_skills_internal(&AgentsScope::Project {
        project_cwd: normalize_display_path(&root),
    })
    .expect("scan skills");

    let entry = result
        .skills
        .iter()
        .find(|skill| skill.name == "demo-skill")
        .expect("demo-skill");
    assert_eq!(entry.description.as_deref(), Some("Does the demo thing."));

    fs::remove_dir_all(&root).expect("cleanup root");
}

#[test]
fn project_skills_scan_description_absent_without_frontmatter() {
    let _guard = lock_test();
    let root = unique_test_dir("project-skills-no-description");
    let source_skill = root.join(".agents").join("skills").join("plain-skill");
    write_file(&source_skill.join(SKILL_FILE_NAME), "no frontmatter here");

    let result = scan_agents_skills_internal(&AgentsScope::Project {
        project_cwd: normalize_display_path(&root),
    })
    .expect("scan skills");

    let entry = result
        .skills
        .iter()
        .find(|skill| skill.name == "plain-skill")
        .expect("plain-skill");
    assert_eq!(entry.description, None);

    fs::remove_dir_all(&root).expect("cleanup root");
}

#[test]
fn project_commands_scan_extracts_description_from_frontmatter() {
    let _guard = lock_test();
    let root = unique_test_dir("project-commands-description");
    let source_command = root
        .join(".agents")
        .join("skills")
        .join("command")
        .join("apply.md");
    write_file(
        &source_command,
        "---\ndescription: Apply the change.\n---\n\n# apply",
    );

    let result = scan_agents_commands_internal(&AgentsScope::Project {
        project_cwd: normalize_display_path(&root),
    })
    .expect("scan commands");

    let entry = result
        .commands
        .iter()
        .find(|command| command.name == "apply")
        .expect("apply command");
    assert_eq!(entry.description.as_deref(), Some("Apply the change."));

    fs::remove_dir_all(&root).expect("cleanup root");
}

#[test]
fn project_commands_scan_description_absent_without_frontmatter() {
    let _guard = lock_test();
    let root = unique_test_dir("project-commands-no-description");
    let source_command = root
        .join(".agents")
        .join("skills")
        .join("command")
        .join("build.md");
    write_file(&source_command, "# build\nno frontmatter");

    let result = scan_agents_commands_internal(&AgentsScope::Project {
        project_cwd: normalize_display_path(&root),
    })
    .expect("scan commands");

    let entry = result
        .commands
        .iter()
        .find(|command| command.name == "build")
        .expect("build command");
    assert_eq!(entry.description, None);

    fs::remove_dir_all(&root).expect("cleanup root");
}

#[test]
fn agents_scope_deserializes_camel_case_payload() {
    let scope: AgentsScope = serde_json::from_str(r#"{"kind":"project","projectCwd":"D:/demo"}"#)
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
    assert!(
        entry.source_dir.ends_with("team\\demo-skill")
            || entry.source_dir.ends_with("team/demo-skill")
    );

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
