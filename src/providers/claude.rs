use crate::cli::ApplyMode;
use crate::fs_ops::{
    backup_and_remove_relative_paths, backup_relative, copy_render_dir,
    copy_render_file_with_extras, copy_render_relative_entries, copy_selected_scripts,
    create_backup_root, remove_if_exists, resolve_backup_root, restore_named_entry,
    restore_relative,
};
use crate::json_ops::{cleanup_claude_settings, read_json_or_empty, write_json_pretty};
use crate::layout::{
    CLAUDE_LEGACY_CLEANUP_PATHS, all_claude_harness_doc_paths, all_claude_skill_paths,
    claude_harness_doc_paths, claude_managed_paths_for, claude_skill_paths,
};
use crate::manifest::{BaselineMcp, BootstrapManifest};
use crate::repo_assets::stackpilot_dev_kit_claude_repo_root;
use crate::runtime::{command_exists, repo_root, run_command_in_home, timestamp_string};
use crate::state::{managed_mcp_names, read_installed_state, write_installed_state};
use anyhow::{Context, Result, bail};
use serde_json::{Value, json};
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

const CLAUDE_MANAGED_PATHS: &[&str] = &[
    "CLAUDE.md",
    "agents",
    "scripts",
    "RALPH_PLAN.md",
    "FOUNDER_LOOP.md",
    "OPERATING_REVIEW.md",
    "CONNECTORS.md",
    "AUTOMATIONS.md",
    "ENTRYPOINTS.md",
    "WORKFLOW.md",
    "SHIP_CHECKLIST.md",
    "OFFICE_HOURS.md",
    "INVESTIGATE.md",
    "AUTOPILOT.md",
    "TEAM.md",
    "REVIEW_AUTOMATION.md",
    "RETRO.md",
    "REVIEW.md",
    "QA.md",
    "SHIP.md",
    "settings.json",
    "RTK.md",
    "hooks/rtk-rewrite.sh",
    "stackpilot-state.json",
];

const CLAUDE_MANAGED_SKILL_PATHS: &[&str] = &[
    "skills/autopilot",
    "skills/deep-init",
    "skills/ralph-plan",
    "skills/founder-loop",
    "skills/investigate",
    "skills/operating-review",
    "skills/review",
    "skills/qa",
    "skills/ship",
    "skills/retro",
    "skills/office-hours",
    "skills/workflow-gate",
    "skills/team",
    "skills/ultrawork",
    "skills/review-automation",
];

const CLAUDE_BASELINE_PERMISSION_DENY: &[&str] = &[
    "Read(./.env)",
    "Read(./.env.*)",
    "Read(./secrets/**)",
    "Read(./**/secrets/**)",
    "Bash(rm -rf *)",
    "Bash(git reset --hard *)",
    "Bash(git checkout -- *)",
    "Bash(git clean -fdx *)",
    "Bash(git push --force*)",
];

const CLAUDE_BASELINE_PERMISSION_ASK: &[&str] = &[
    "Bash(git push *)",
    "Bash(curl *)",
    "Bash(wget *)",
    "Bash(ssh *)",
    "Bash(kubectl *)",
    "Bash(terraform apply*)",
];

pub(crate) fn doctor_checks(
    home: &Path,
    enabled_mcp: &[BaselineMcp],
    rtk_enabled: bool,
    skills_enabled: bool,
    active_surfaces: &[String],
) -> Vec<PathBuf> {
    let root = home.join(".claude");
    let mut checks = vec![
        root.join("CLAUDE.md"),
        root.join("agents/planner.md"),
        root.join("agents/reviewer.md"),
        root.join("agents/executor.md"),
        root.join("agents/triage.md"),
        root.join("agents/verifier.md"),
    ];
    checks.extend(
        claude_harness_doc_paths(active_surfaces)
            .into_iter()
            .map(|relative| root.join(relative)),
    );
    if skills_enabled {
        checks.extend(
            claude_skill_paths(active_surfaces)
                .into_iter()
                .map(|relative| root.join(relative).join("SKILL.md")),
        );
    }

    if rtk_enabled {
        checks.push(root.join("settings.json"));
        checks.push(root.join("RTK.md"));
        checks.push(root.join("hooks/rtk-rewrite.sh"));
    }

    checks.extend(
        enabled_mcp
            .iter()
            .map(|mcp| root.join("scripts").join(mcp.script_name())),
    );
    checks.push(home.join(".claude.json"));
    checks
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn install(
    home: &Path,
    mode: ApplyMode,
    _manifest: &BootstrapManifest,
    enabled_mcp: &[BaselineMcp],
    rtk_enabled: bool,
    skills_enabled: bool,
    active_surfaces: &[String],
) -> Result<()> {
    ensure_claude_cli()?;

    let root = home.join(".claude");
    let template_root = repo_root().join("templates/claude");
    let addon_root = stackpilot_dev_kit_claude_repo_root();
    fs::create_dir_all(root.join("scripts"))?;
    fs::create_dir_all(root.join("hooks"))?;

    let backup_root = create_backup_root(&root, &timestamp_string()?)?;
    println!("[claude] backup {}", backup_root.display());

    for relative in CLAUDE_MANAGED_PATHS {
        backup_relative(&root, &backup_root, Path::new(relative))?;
    }
    for relative in CLAUDE_MANAGED_SKILL_PATHS {
        backup_relative(&root, &backup_root, Path::new(relative))?;
    }
    backup_home_file(home, &backup_root, ".claude.json", "claude.json")?;

    if mode == ApplyMode::Replace {
        for relative in CLAUDE_MANAGED_PATHS {
            remove_if_exists(&root.join(relative))?;
        }
        for relative in CLAUDE_MANAGED_SKILL_PATHS {
            remove_if_exists(&root.join(relative))?;
        }
        remove_legacy_paths(&root, &backup_root)?;
        remove_all_registered_mcp(home)?;
        fs::create_dir_all(root.join("scripts"))?;
        fs::create_dir_all(root.join("hooks"))?;
    }

    if rtk_enabled {
        run_rtk_init(home)?;
        write_rtk_rewrite_hook(&root)?;
    } else {
        remove_if_exists(&root.join("RTK.md"))?;
        remove_if_exists(&root.join("hooks/rtk-rewrite.sh"))?;
        cleanup_claude_settings(&root.join("settings.json"), true)?;
    }
    sync_claude_settings(&root, mode, rtk_enabled)?;

    copy_render_file_with_extras(
        &template_root.join("CLAUDE.md"),
        &root.join("CLAUDE.md"),
        false,
        home,
        &rtk_tokens(rtk_enabled),
    )?;
    copy_render_dir(&template_root.join("agents"), &root.join("agents"), home)?;
    copy_selected_scripts(
        &template_root.join("scripts"),
        &root.join("scripts"),
        home,
        enabled_mcp,
    )?;
    for relative in all_claude_harness_doc_paths() {
        remove_if_exists(&root.join(relative))?;
    }
    copy_render_relative_entries(
        &template_root,
        &root,
        &claude_harness_doc_paths(active_surfaces),
        home,
    )?;
    if skills_enabled {
        for relative in all_claude_skill_paths() {
            remove_if_exists(&root.join(relative))?;
        }
        copy_render_relative_entries(
            &addon_root,
            &root,
            &claude_skill_paths(active_surfaces),
            home,
        )?;
    } else {
        for relative in CLAUDE_MANAGED_SKILL_PATHS {
            remove_if_exists(&root.join(relative))?;
        }
    }

    sync_baseline_mcp(home, &root, enabled_mcp)?;
    sync_claude_settings(&root, mode, rtk_enabled)?;

    println!("[claude] installed {} ({})", root.display(), mode.name());
    Ok(())
}

pub(crate) fn uninstall(
    home: &Path,
    _enabled_mcp: &[BaselineMcp],
    rtk_enabled: bool,
) -> Result<()> {
    ensure_claude_cli()?;

    let root = home.join(".claude");
    if !root.exists() && !home.join(".claude.json").exists() {
        println!("[claude] skipped uninstall: no Claude home state found");
        return Ok(());
    }

    fs::create_dir_all(&root)?;
    let installed_state = read_installed_state(&root)?;
    let managed_paths = if installed_state.managed_paths.is_empty() {
        if installed_state.active_surfaces.is_empty() {
            CLAUDE_MANAGED_PATHS
                .iter()
                .map(|path| (*path).to_string())
                .chain(
                    CLAUDE_MANAGED_SKILL_PATHS
                        .iter()
                        .map(|path| (*path).to_string()),
                )
                .collect::<Vec<_>>()
        } else {
            claude_managed_paths_for(
                &installed_state.active_surfaces,
                crate::layout::provider_surface_enabled(&installed_state.active_surfaces),
                rtk_enabled,
            )
        }
    } else {
        installed_state.managed_paths
    };
    let backup_root = create_backup_root(&root, &timestamp_string()?)?;
    println!("[claude] backup {}", backup_root.display());

    for relative in &managed_paths {
        backup_relative(&root, &backup_root, Path::new(relative))?;
    }
    backup_home_file(home, &backup_root, ".claude.json", "claude.json")?;
    remove_legacy_paths(&root, &backup_root)?;

    remove_managed_mcp(home, &managed_mcp_names(&root)?)?;

    if rtk_enabled {
        run_rtk_uninstall(home)?;
    }

    for relative in &managed_paths {
        if relative == "settings.json" {
            continue;
        }
        remove_if_exists(&root.join(relative))?;
    }
    cleanup_claude_managed_settings(&root.join("settings.json"), rtk_enabled)?;

    println!("[claude] uninstalled {}", root.display());
    Ok(())
}

fn remove_legacy_paths(root: &Path, backup_root: &Path) -> Result<()> {
    let removed = backup_and_remove_relative_paths(root, backup_root, CLAUDE_LEGACY_CLEANUP_PATHS)?;
    if !removed.is_empty() {
        println!("[claude] removed legacy paths: {}", removed.join(","));
    }
    Ok(())
}

pub(crate) fn restore(home: &Path, backup_name: Option<&str>) -> Result<()> {
    ensure_claude_cli()?;

    let root = home.join(".claude");
    fs::create_dir_all(&root)?;
    let source_backup = resolve_backup_root(&root, backup_name)?;
    let installed_state = read_installed_state(&root)?;
    let managed_paths = if installed_state.managed_paths.is_empty() {
        CLAUDE_MANAGED_PATHS
            .iter()
            .map(|path| (*path).to_string())
            .chain(
                CLAUDE_MANAGED_SKILL_PATHS
                    .iter()
                    .map(|path| (*path).to_string()),
            )
            .collect::<Vec<_>>()
    } else {
        installed_state.managed_paths
    };
    let backup_root = create_backup_root(&root, &timestamp_string()?)?;
    println!("[claude] backup {}", backup_root.display());

    for relative in &managed_paths {
        backup_relative(&root, &backup_root, Path::new(relative))?;
    }
    backup_home_file(home, &backup_root, ".claude.json", "claude.json")?;

    for relative in &managed_paths {
        remove_if_exists(&root.join(relative))?;
    }
    remove_if_exists(&home.join(".claude.json"))?;

    for relative in &managed_paths {
        restore_relative(&root, &source_backup, Path::new(relative))?;
    }
    restore_named_entry(&source_backup, "claude.json", &home.join(".claude.json"))?;

    println!(
        "[claude] restored {} from {}",
        root.display(),
        source_backup.display()
    );
    Ok(())
}

fn ensure_claude_cli() -> Result<()> {
    if command_exists("claude") {
        Ok(())
    } else {
        bail!("Claude Code CLI is required for the claude provider")
    }
}

fn run_rtk_init(home: &Path) -> Result<()> {
    run_command_in_home(
        home,
        "rtk",
        ["init", "-g", "--auto-patch"],
        "initializing RTK for Claude Code",
    )
}

fn write_rtk_rewrite_hook(root: &Path) -> Result<()> {
    let hook_path = root.join("hooks/rtk-rewrite.sh");
    if let Some(parent) = hook_path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }

    fs::write(
        &hook_path,
        "#!/usr/bin/env bash\nset -euo pipefail\nexec rtk hook claude\n",
    )
    .with_context(|| format!("failed to write {}", hook_path.display()))?;
    fs::set_permissions(&hook_path, fs::Permissions::from_mode(0o755))
        .with_context(|| format!("failed to set executable bit on {}", hook_path.display()))?;
    Ok(())
}

fn run_rtk_uninstall(home: &Path) -> Result<()> {
    if !command_exists("rtk") {
        println!("[warn] command rtk missing; skipping official Claude RTK uninstall");
        return Ok(());
    }

    run_command_in_home(
        home,
        "rtk",
        ["init", "-g", "--uninstall"],
        "uninstalling RTK for Claude Code",
    )
}

fn sync_baseline_mcp(home: &Path, root: &Path, enabled_mcp: &[BaselineMcp]) -> Result<()> {
    let previous = managed_mcp_names(root)?;
    for name in previous
        .iter()
        .filter(|name| !enabled_mcp.iter().any(|mcp| mcp.name() == name.as_str()))
    {
        remove_mcp(home, name)?;
    }

    for mcp in enabled_mcp {
        remove_mcp(home, mcp.name())?;
        let script = home
            .join(".claude/scripts")
            .join(mcp.script_name())
            .to_string_lossy()
            .to_string();
        run_command_in_home(
            home,
            "claude",
            ["mcp", "add", "--scope", "user", mcp.name(), "--", &script],
            &format!("adding Claude MCP {}", mcp.name()),
        )?;
    }
    write_installed_state(root, enabled_mcp, &crate::state::InstalledState::default())?;
    Ok(())
}

fn remove_managed_mcp(home: &Path, managed: &[String]) -> Result<()> {
    for name in managed {
        remove_mcp(home, name)?;
    }
    Ok(())
}

fn remove_all_registered_mcp(home: &Path) -> Result<()> {
    let claude_json = home.join(".claude.json");
    if !claude_json.exists() {
        return Ok(());
    }

    let value = read_json_or_empty(&claude_json)?;
    let names = value
        .get("mcpServers")
        .and_then(Value::as_object)
        .map(|servers| servers.keys().cloned().collect::<Vec<_>>())
        .unwrap_or_default();

    for name in names {
        remove_mcp(home, &name)?;
    }
    Ok(())
}

fn remove_mcp(home: &Path, name: &str) -> Result<()> {
    let output = std::process::Command::new("claude")
        .env("HOME", home)
        .args(["mcp", "remove", "--scope", "user", name])
        .output()
        .with_context(|| format!("failed while removing Claude MCP {}", name))?;

    if output.status.success() {
        return Ok(());
    }

    let stderr = String::from_utf8_lossy(&output.stderr);
    if stderr.contains("No user-scoped MCP server found with name") {
        return Ok(());
    }

    bail!("failed to remove Claude MCP {}: {}", name, stderr.trim())
}

fn sync_claude_settings(root: &Path, mode: ApplyMode, rtk_enabled: bool) -> Result<()> {
    let path = root.join("settings.json");
    let existing = read_json_or_empty(&path)?;
    let mut settings = match mode {
        ApplyMode::Merge => existing,
        ApplyMode::Replace => json!({}),
    };

    if mode == ApplyMode::Replace && rtk_enabled {
        settings["hooks"] = rtk_claude_hooks(root);
    }

    set_json_pointer(
        &mut settings,
        "/$schema",
        json!("https://json.schemastore.org/claude-code-settings.json"),
    );
    set_json_pointer(&mut settings, "/autoUpdatesChannel", json!("stable"));
    set_json_pointer(&mut settings, "/autoMemoryEnabled", json!(true));
    set_json_pointer(&mut settings, "/awaySummaryEnabled", json!(true));
    set_json_pointer(&mut settings, "/cleanupPeriodDays", json!(365));
    remove_json_pointer(&mut settings, "/effortLevel");
    set_json_pointer(
        &mut settings,
        "/env/CLAUDE_CODE_EXPERIMENTAL_AGENT_TEAMS",
        json!("1"),
    );
    set_json_pointer_with_merge_default(
        &mut settings,
        mode,
        "/env/CLAUDE_CODE_EFFORT_LEVEL",
        json!("max"),
    );
    remove_json_pointer(&mut settings, "/env/CLAUDE_CODE_DISABLE_AUTO_MEMORY");
    set_json_pointer_with_merge_default(
        &mut settings,
        mode,
        "/fastModePerSessionOptIn",
        json!(true),
    );
    set_json_pointer(&mut settings, "/includeGitInstructions", json!(true));
    set_json_pointer_with_merge_default_or_known_old(
        &mut settings,
        mode,
        "/model",
        json!("opus[1m]"),
        &[json!("opus")],
    );
    set_json_pointer(&mut settings, "/permissions/defaultMode", json!("auto"));
    set_json_pointer(&mut settings, "/showThinkingSummaries", json!(true));
    set_json_pointer(&mut settings, "/useAutoModeDuringPlan", json!(false));
    append_json_string_array(
        &mut settings,
        "/permissions/deny",
        CLAUDE_BASELINE_PERMISSION_DENY,
    );
    append_json_string_array(
        &mut settings,
        "/permissions/ask",
        CLAUDE_BASELINE_PERMISSION_ASK,
    );

    write_json_pretty(&path, &settings)
}

fn rtk_claude_hooks(root: &Path) -> Value {
    json!({
        "PreToolUse": [
            {
                "matcher": "Bash",
                "hooks": [
                    {
                        "type": "command",
                        "command": root.join("hooks/rtk-rewrite.sh").to_string_lossy()
                    }
                ]
            }
        ]
    })
}

fn set_json_pointer(settings: &mut Value, path: &str, value: Value) {
    let parts = path.trim_start_matches('/').split('/').collect::<Vec<_>>();
    let Some((last, parents)) = parts.split_last() else {
        return;
    };

    let mut cursor = settings;
    for part in parents {
        if !cursor.get(part).is_some_and(Value::is_object) {
            cursor[part] = json!({});
        }
        cursor = &mut cursor[part];
    }
    cursor[last] = value;
}

fn set_json_pointer_with_merge_default(
    settings: &mut Value,
    mode: ApplyMode,
    path: &str,
    value: Value,
) {
    if mode == ApplyMode::Merge && settings.pointer(path).is_some() {
        return;
    }
    set_json_pointer(settings, path, value);
}

fn set_json_pointer_with_merge_default_or_known_old(
    settings: &mut Value,
    mode: ApplyMode,
    path: &str,
    value: Value,
    known_old_values: &[Value],
) {
    if mode == ApplyMode::Merge
        && let Some(existing) = settings.pointer(path)
        && !known_old_values.iter().any(|old| old == existing)
    {
        return;
    }
    set_json_pointer(settings, path, value);
}

fn append_json_string_array(settings: &mut Value, path: &str, values: &[&str]) {
    let parts = path.trim_start_matches('/').split('/').collect::<Vec<_>>();
    let Some((last, parents)) = parts.split_last() else {
        return;
    };

    let mut cursor = settings;
    for part in parents {
        if !cursor.get(part).is_some_and(Value::is_object) {
            cursor[part] = json!({});
        }
        cursor = &mut cursor[part];
    }

    if !cursor.get(last).is_some_and(Value::is_array) {
        cursor[last] = json!([]);
    }
    let Some(array) = cursor[last].as_array_mut() else {
        return;
    };
    for value in values {
        if !array.iter().any(|entry| entry.as_str() == Some(value)) {
            array.push(json!(value));
        }
    }
}

fn cleanup_claude_managed_settings(path: &Path, rtk_enabled: bool) -> Result<()> {
    cleanup_claude_settings(path, rtk_enabled)?;
    if !path.exists() {
        return Ok(());
    }

    let mut settings = read_json_or_empty(path)?;
    remove_json_pointer_if_equal(
        &mut settings,
        "/$schema",
        &json!("https://json.schemastore.org/claude-code-settings.json"),
    );
    remove_json_pointer_if_equal(&mut settings, "/autoUpdatesChannel", &json!("stable"));
    remove_json_pointer_if_equal(&mut settings, "/autoMemoryEnabled", &json!(true));
    remove_json_pointer_if_equal(&mut settings, "/awaySummaryEnabled", &json!(true));
    remove_json_pointer_if_equal(&mut settings, "/cleanupPeriodDays", &json!(365));
    remove_json_pointer_if_equal(&mut settings, "/effortLevel", &json!("max"));
    remove_json_pointer_if_equal(&mut settings, "/effortLevel", &json!("high"));
    remove_json_pointer_if_equal(
        &mut settings,
        "/env/CLAUDE_CODE_EXPERIMENTAL_AGENT_TEAMS",
        &json!("1"),
    );
    remove_json_pointer_if_equal(
        &mut settings,
        "/env/CLAUDE_CODE_EFFORT_LEVEL",
        &json!("max"),
    );
    remove_empty_object_key(&mut settings, "env");
    remove_json_pointer_if_equal(&mut settings, "/fastModePerSessionOptIn", &json!(true));
    remove_json_pointer_if_equal(&mut settings, "/includeGitInstructions", &json!(true));
    remove_json_pointer_if_equal(&mut settings, "/model", &json!("opus[1m]"));
    remove_json_pointer_if_equal(&mut settings, "/model", &json!("opus"));
    remove_json_pointer_if_equal(&mut settings, "/model", &json!("sonnet[1m]"));
    remove_json_pointer_if_equal(&mut settings, "/model", &json!("sonnet"));
    remove_normalized_sonnet_1m_model(&mut settings);
    remove_json_pointer_if_equal(&mut settings, "/permissions/defaultMode", &json!("auto"));
    remove_json_pointer_if_equal(&mut settings, "/showThinkingSummaries", &json!(true));
    remove_json_pointer_if_equal(&mut settings, "/useAutoModeDuringPlan", &json!(false));
    remove_claude_permission_rules(&mut settings, "deny", CLAUDE_BASELINE_PERMISSION_DENY);
    remove_claude_permission_rules(&mut settings, "ask", CLAUDE_BASELINE_PERMISSION_ASK);

    if settings.as_object().is_some_and(|object| object.is_empty()) {
        remove_if_exists(path)?;
    } else {
        write_json_pretty(path, &settings)?;
    }
    Ok(())
}

fn remove_empty_object_key(settings: &mut Value, key: &str) {
    let Some(root) = settings.as_object_mut() else {
        return;
    };
    if root
        .get(key)
        .and_then(Value::as_object)
        .is_some_and(|object| object.is_empty())
    {
        root.remove(key);
    }
}

fn remove_json_pointer(settings: &mut Value, path: &str) {
    let parts = path.trim_start_matches('/').split('/').collect::<Vec<_>>();
    let Some((last, parents)) = parts.split_last() else {
        return;
    };

    let mut cursor = settings;
    for part in parents {
        let Some(next) = cursor.get_mut(part) else {
            return;
        };
        cursor = next;
    }
    if let Some(object) = cursor.as_object_mut() {
        object.remove(*last);
    }
}

fn remove_json_pointer_if_equal(settings: &mut Value, path: &str, expected: &Value) {
    if settings.pointer(path) != Some(expected) {
        return;
    }
    let parts = path.trim_start_matches('/').split('/').collect::<Vec<_>>();
    let Some((last, parents)) = parts.split_last() else {
        return;
    };

    let mut cursor = settings;
    for part in parents {
        let Some(next) = cursor.get_mut(part) else {
            return;
        };
        cursor = next;
    }
    if let Some(object) = cursor.as_object_mut() {
        object.remove(*last);
    }
}

fn remove_normalized_sonnet_1m_model(settings: &mut Value) {
    let is_normalized_sonnet_1m = settings
        .get("model")
        .and_then(Value::as_str)
        .is_some_and(|model| model.starts_with("sonnet-") && model.ends_with("[1m]"));
    if is_normalized_sonnet_1m {
        remove_json_pointer(settings, "/model");
    }
}

fn remove_claude_permission_rules(settings: &mut Value, key: &str, rules: &[&str]) {
    let Some(root) = settings.as_object_mut() else {
        return;
    };
    let Some(permissions) = root.get_mut("permissions").and_then(Value::as_object_mut) else {
        return;
    };
    let Some(entries) = permissions.get_mut(key).and_then(Value::as_array_mut) else {
        return;
    };

    entries.retain(|entry| !entry.as_str().is_some_and(|value| rules.contains(&value)));
    if entries.is_empty() {
        permissions.remove(key);
    }
    if permissions.is_empty() {
        root.remove("permissions");
    }
}

fn backup_home_file(
    home: &Path,
    backup_root: &Path,
    source_name: &str,
    backup_name: &str,
) -> Result<()> {
    let source = home.join(source_name);
    if !source.exists() {
        return Ok(());
    }

    let destination = backup_root.join(backup_name);
    if let Some(parent) = destination.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }
    fs::copy(&source, &destination).with_context(|| {
        format!(
            "failed to copy {} -> {}",
            source.display(),
            destination.display()
        )
    })?;
    Ok(())
}

fn rtk_tokens(rtk_enabled: bool) -> Vec<(&'static str, &'static str)> {
    if rtk_enabled {
        vec![("__RTK_CLAUDE_IMPORT__", "@RTK.md")]
    } else {
        vec![("__RTK_CLAUDE_IMPORT__", "")]
    }
}

#[cfg(test)]
pub(crate) fn claude_user_mcp(home: &Path) -> Result<Value> {
    read_json_or_empty(&home.join(".claude.json"))
}
