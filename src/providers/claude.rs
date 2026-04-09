use crate::cli::ApplyMode;
use crate::fs_ops::{
    backup_relative, copy_render_dir, copy_render_file_with_extras, copy_render_relative_entries,
    copy_selected_scripts, create_backup_root, remove_if_exists, resolve_backup_root,
    restore_named_entry, restore_relative,
};
use crate::json_ops::{cleanup_claude_settings, read_json_or_empty};
use crate::layout::{
    all_claude_harness_doc_paths, all_claude_skill_paths, claude_harness_doc_paths,
    claude_managed_paths_for, claude_skill_paths,
};
use crate::manifest::{BaselineMcp, BootstrapManifest};
use crate::runtime::{command_exists, repo_root, run_command_in_home, timestamp_string};
use crate::state::{managed_mcp_names, read_installed_state, write_installed_state};
use anyhow::{Context, Result, bail};
use serde_json::Value;
use std::fs;
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
    "WORKFLOW.md",
    "SHIP_CHECKLIST.md",
    "OFFICE_HOURS.md",
    "INVESTIGATE.md",
    "AUTOPILOT.md",
    "RETRO.md",
    "REVIEW.md",
    "QA.md",
    "SHIP.md",
    "settings.json",
    "RTK.md",
    "hooks/rtk-rewrite.sh",
    "llm-bootstrap-state.json",
];

const CLAUDE_MANAGED_SKILL_PATHS: &[&str] = &[
    "skills/autopilot",
    "skills/ralph-plan",
    "skills/founder-loop",
    "skills/investigate",
    "skills/operating-review",
    "skills/review",
    "skills/qa",
    "skills/ship",
    "skills/retro",
    "skills/office-hours",
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
        remove_all_registered_mcp(home)?;
        fs::create_dir_all(root.join("scripts"))?;
        fs::create_dir_all(root.join("hooks"))?;
    }

    if rtk_enabled {
        run_rtk_init(home)?;
    } else {
        remove_if_exists(&root.join("RTK.md"))?;
        remove_if_exists(&root.join("hooks/rtk-rewrite.sh"))?;
        cleanup_claude_settings(&root.join("settings.json"), true)?;
    }

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
            &template_root,
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
    if rtk_enabled {
        cleanup_claude_settings(&root.join("settings.json"), true)?;
    }

    println!("[claude] uninstalled {}", root.display());
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
