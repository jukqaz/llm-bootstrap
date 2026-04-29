use crate::cli::ApplyMode;
use crate::fs_ops::{
    backup_and_remove_relative_paths, backup_relative, copy_render_dir, copy_render_file,
    copy_render_file_with_extras, copy_render_relative_entries, copy_selected_scripts,
    create_backup_root, remove_if_exists, render_template_file_with_extras, resolve_backup_root,
    restore_relative, toml_table_key,
};
use crate::layout::{
    CODEX_LEGACY_CLEANUP_PATHS, all_codex_bundle_doc_paths, all_codex_bundle_plugin_asset_paths,
    all_codex_plugin_asset_paths, codex_bundle_doc_paths, codex_bundle_plugin_asset_paths,
    codex_managed_paths, codex_managed_paths_for, codex_plugin_asset_paths,
};
use crate::manifest::{BaselineMcp, BootstrapManifest};
use crate::repo_assets::{
    stackpilot_dev_kit_codex_bundle_root, stackpilot_dev_kit_codex_repo_root,
};
use crate::runtime::{command_exists, repo_root, run_command_in_home, timestamp_string};
use crate::state::read_installed_state;
use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};
use toml::{Value, map::Map as TomlMap};

pub(crate) fn doctor_checks(
    home: &Path,
    _manifest: &BootstrapManifest,
    enabled_mcp: &[BaselineMcp],
    rtk_enabled: bool,
    plugin_enabled: bool,
    active_surfaces: &[String],
) -> Vec<PathBuf> {
    let root = home.join(".codex");
    let mut checks = vec![
        root.join("config.toml"),
        root.join("AGENTS.md"),
        root.join("agents/planner.toml"),
    ];
    checks.extend(
        codex_bundle_doc_paths(active_surfaces)
            .into_iter()
            .map(|relative| root.join(relative)),
    );
    if plugin_enabled {
        checks.push(root.join(".agents/plugins/marketplace.json"));
        checks.extend(
            codex_plugin_asset_paths(active_surfaces)
                .into_iter()
                .flat_map(|relative| {
                    [
                        root.join("plugins/stackpilot-dev-kit").join(relative),
                        root.join("plugins/cache/stackpilot/stackpilot-dev-kit/local")
                            .join(relative),
                    ]
                }),
        );
        checks.extend(
            codex_bundle_plugin_asset_paths(active_surfaces)
                .into_iter()
                .flat_map(|relative| {
                    [
                        root.join("plugins/stackpilot-dev-kit").join(relative),
                        root.join("plugins/cache/stackpilot/stackpilot-dev-kit/local")
                            .join(relative),
                    ]
                }),
        );
    }
    if rtk_enabled {
        checks.push(root.join("RTK.md"));
    }

    checks.extend(
        enabled_mcp
            .iter()
            .map(|mcp| root.join("scripts").join(mcp.script_name())),
    );
    checks
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn install(
    home: &Path,
    mode: ApplyMode,
    _manifest: &BootstrapManifest,
    enabled_mcp: &[BaselineMcp],
    rtk_enabled: bool,
    plugin_enabled: bool,
    active_surfaces: &[String],
) -> Result<()> {
    let root = home.join(".codex");
    let template_root = repo_root().join("templates/codex");
    let plugin_root = stackpilot_dev_kit_codex_repo_root();
    let marketplace_path = repo_root().join(".agents/plugins/marketplace.json");
    let installed_plugin_root = root.join("plugins/cache/stackpilot/stackpilot-dev-kit/local");
    let bundle_root = repo_root().join("bundles/full/codex");
    let bundle_plugin_root = stackpilot_dev_kit_codex_bundle_root();
    fs::create_dir_all(&root)?;
    let backup_root = create_backup_root(&root, &timestamp_string()?)?;
    println!("[codex] backup {}", backup_root.display());

    for relative in codex_managed_paths() {
        backup_relative(&root, &backup_root, Path::new(relative))?;
    }

    if mode == ApplyMode::Replace {
        for relative in codex_managed_paths() {
            remove_if_exists(&root.join(relative))?;
        }
        remove_legacy_paths(&root, &backup_root)?;
    }

    if rtk_enabled {
        run_rtk_init(home)?;
    } else {
        remove_if_exists(&root.join("RTK.md"))?;
    }

    let codex_mcp_blocks = mcp_blocks(home, &root, enabled_mcp, mode)?;
    let codex_plugin_blocks = plugin_blocks(plugin_enabled);
    write_codex_config(
        &template_root.join("config.toml"),
        &root.join("config.toml"),
        home,
        mode,
        &[
            ("__CODEX_MCP_BLOCKS__", codex_mcp_blocks.as_str()),
            ("__CODEX_PLUGIN_BLOCKS__", codex_plugin_blocks.as_str()),
        ],
    )?;
    copy_render_file_with_extras(
        &template_root.join("AGENTS.md"),
        &root.join("AGENTS.md"),
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
    if plugin_enabled {
        copy_render_file(
            &marketplace_path,
            &root.join(".agents/plugins/marketplace.json"),
            false,
            home,
        )?;
    } else {
        remove_if_exists(&root.join(".agents/plugins/marketplace.json"))?;
        remove_if_exists(&root.join("plugins/stackpilot-dev-kit"))?;
        remove_if_exists(&root.join("plugins/cache/stackpilot/stackpilot-dev-kit"))?;
    }
    for relative in all_codex_bundle_doc_paths() {
        remove_if_exists(&root.join(relative))?;
    }
    copy_render_relative_entries(
        &bundle_root,
        &root,
        &codex_bundle_doc_paths(active_surfaces),
        home,
    )?;
    if plugin_enabled {
        for relative in all_codex_plugin_asset_paths() {
            remove_if_exists(&root.join("plugins/stackpilot-dev-kit").join(relative))?;
            remove_if_exists(&installed_plugin_root.join(relative))?;
        }
        for relative in all_codex_bundle_plugin_asset_paths() {
            remove_if_exists(&root.join("plugins/stackpilot-dev-kit").join(relative))?;
            remove_if_exists(&installed_plugin_root.join(relative))?;
        }
        copy_render_relative_entries(
            &plugin_root,
            &root.join("plugins/stackpilot-dev-kit"),
            &codex_plugin_asset_paths(active_surfaces),
            home,
        )?;
        copy_render_relative_entries(
            &plugin_root,
            &installed_plugin_root,
            &codex_plugin_asset_paths(active_surfaces),
            home,
        )?;
        copy_render_relative_entries(
            &bundle_plugin_root,
            &root.join("plugins/stackpilot-dev-kit"),
            &codex_bundle_plugin_asset_paths(active_surfaces),
            home,
        )?;
        copy_render_relative_entries(
            &bundle_plugin_root,
            &installed_plugin_root,
            &codex_bundle_plugin_asset_paths(active_surfaces),
            home,
        )?;
    }
    println!("[codex] installed {} ({})", root.display(), mode.name());
    Ok(())
}

pub(crate) fn uninstall(home: &Path, rtk_enabled: bool) -> Result<()> {
    let root = home.join(".codex");
    if !root.exists() {
        println!("[codex] skipped uninstall: {} not found", root.display());
        return Ok(());
    }

    let installed_state = read_installed_state(&root)?;
    let uninstall_paths = if installed_state.managed_paths.is_empty() {
        if installed_state.active_surfaces.is_empty() {
            codex_managed_paths()
                .into_iter()
                .map(ToOwned::to_owned)
                .collect::<Vec<_>>()
        } else {
            codex_managed_paths_for(
                &installed_state.active_surfaces,
                crate::layout::provider_surface_enabled(&installed_state.active_surfaces),
                rtk_enabled,
            )
        }
    } else {
        installed_state.managed_paths
    };

    let backup_root = create_backup_root(&root, &timestamp_string()?)?;
    println!("[codex] backup {}", backup_root.display());

    for relative in &uninstall_paths {
        backup_relative(&root, &backup_root, Path::new(relative))?;
    }
    remove_legacy_paths(&root, &backup_root)?;

    if rtk_enabled {
        run_rtk_uninstall(home)?;
    }

    for relative in &uninstall_paths {
        remove_if_exists(&root.join(relative))?;
    }

    println!("[codex] uninstalled {}", root.display());
    Ok(())
}

fn remove_legacy_paths(root: &Path, backup_root: &Path) -> Result<()> {
    let mut removed =
        backup_and_remove_relative_paths(root, backup_root, CODEX_LEGACY_CLEANUP_PATHS)?;
    removed.extend(remove_tmp_plugin_noise(root, backup_root)?);
    if !removed.is_empty() {
        println!("[codex] removed legacy paths: {}", removed.join(","));
    }
    Ok(())
}

fn remove_tmp_plugin_noise(root: &Path, backup_root: &Path) -> Result<Vec<String>> {
    let tmp_root = root.join(".tmp");
    if !tmp_root.exists() {
        return Ok(Vec::new());
    }

    let mut removed = Vec::new();
    for entry in
        fs::read_dir(&tmp_root).with_context(|| format!("failed to read {}", tmp_root.display()))?
    {
        let entry = entry?;
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if name == "plugins" || name == "plugins.sha" || name.starts_with("plugins-backup-") {
            let relative = PathBuf::from(".tmp").join(name.as_ref());
            backup_relative(root, backup_root, &relative)?;
            remove_if_exists(&root.join(&relative))?;
            removed.push(relative.display().to_string());
        }
    }

    Ok(removed)
}

pub(crate) fn restore(home: &Path, backup_name: Option<&str>) -> Result<()> {
    let root = home.join(".codex");
    fs::create_dir_all(&root)?;
    let source_backup = resolve_backup_root(&root, backup_name)?;
    let installed_state = read_installed_state(&root)?;
    let managed_paths = if installed_state.managed_paths.is_empty() {
        codex_managed_paths()
            .into_iter()
            .map(ToOwned::to_owned)
            .collect::<Vec<_>>()
    } else {
        installed_state.managed_paths
    };
    let backup_root = create_backup_root(&root, &timestamp_string()?)?;
    println!("[codex] backup {}", backup_root.display());

    for relative in &managed_paths {
        backup_relative(&root, &backup_root, Path::new(relative))?;
    }

    for relative in &managed_paths {
        remove_if_exists(&root.join(relative))?;
    }
    for relative in &managed_paths {
        restore_relative(&root, &source_backup, Path::new(relative))?;
    }

    println!(
        "[codex] restored {} from {}",
        root.display(),
        source_backup.display()
    );
    Ok(())
}

pub(crate) fn mcp_blocks(
    home: &Path,
    root: &Path,
    enabled_mcp: &[BaselineMcp],
    mode: ApplyMode,
) -> Result<String> {
    let codex_home = home.join(".codex");

    let unmanaged = match mode {
        ApplyMode::Merge => unmanaged_mcp_blocks(root)?,
        ApplyMode::Replace => Vec::new(),
    };

    let managed = enabled_mcp
        .iter()
        .map(|mcp| {
            let env_vars = mcp
                .env_var()
                .map(|env_var| format!("\nenv_vars = [\"{env_var}\"]"))
                .unwrap_or_default();
            format!(
                "[mcp_servers.{name}]\ncommand = \"{command}\"\nenabled = true\nstartup_timeout_sec = 20\ntool_timeout_sec = 120{env_vars}",
                name = toml_table_key(mcp.name()),
                command = codex_home.join("scripts").join(mcp.script_name()).display()
            )
        })
        .collect::<Vec<_>>();

    Ok(unmanaged
        .into_iter()
        .chain(managed)
        .collect::<Vec<_>>()
        .join("\n\n"))
}

pub(crate) fn plugin_blocks(plugin_enabled: bool) -> String {
    if plugin_enabled {
        "[plugins.\"stackpilot-dev-kit@stackpilot\"]\nenabled = true".to_string()
    } else {
        String::new()
    }
}

fn write_codex_config(
    source: &Path,
    destination: &Path,
    home: &Path,
    mode: ApplyMode,
    extra_tokens: &[(&str, &str)],
) -> Result<()> {
    let rendered = render_template_file_with_extras(source, home, extra_tokens)?;
    if mode == ApplyMode::Replace || !destination.exists() {
        fs::write(destination, rendered)
            .with_context(|| format!("failed to write {}", destination.display()))?;
        cleanup_codex_config_compatibility(destination)?;
        return Ok(());
    }

    let existing_raw = fs::read_to_string(destination)
        .with_context(|| format!("failed to read {}", destination.display()))?;
    let existing = existing_raw
        .parse::<Value>()
        .with_context(|| format!("failed to parse {}", destination.display()))?;
    let managed = rendered
        .parse::<Value>()
        .with_context(|| format!("failed to parse rendered {}", source.display()))?;
    let merged = merge_codex_config_value(managed, existing);
    fs::write(destination, toml::to_string(&merged)?)
        .with_context(|| format!("failed to write {}", destination.display()))?;
    cleanup_codex_config_compatibility(destination)?;
    Ok(())
}

fn cleanup_codex_config_compatibility(path: &Path) -> Result<()> {
    let raw =
        fs::read_to_string(path).with_context(|| format!("failed to read {}", path.display()))?;
    let mut parsed = raw
        .parse::<Value>()
        .with_context(|| format!("failed to parse {}", path.display()))?;

    let mut changed = false;

    let Some(root) = parsed.as_table_mut() else {
        return Ok(());
    };
    let multi_agent_v2_enabled = root
        .get("features")
        .and_then(Value::as_table)
        .and_then(|features| features.get("multi_agent_v2"))
        .and_then(Value::as_bool)
        .unwrap_or(false);
    if multi_agent_v2_enabled
        && let Some(agents) = root.get_mut("agents").and_then(Value::as_table_mut)
    {
        changed |= agents.remove("max_threads").is_some();
    }

    if let Some(features) = root.get_mut("features").and_then(Value::as_table_mut) {
        for key in [
            "apply_patch_freeform",
            "artifact",
            "child_agents_md",
            "code_mode",
            "codex_git_commit",
            "enable_fanout",
            "image_detail_original",
            "image_generation",
            "js_repl",
            "multi_agent_v2",
            "runtime_metrics",
            "shell_zsh_fork",
            "tool_search",
        ] {
            changed |= features.remove(key).is_some();
        }
    }

    if let Some(memories) = root.get_mut("memories").and_then(Value::as_table_mut) {
        changed |= memories
            .remove("no_memories_if_mcp_or_web_search")
            .is_some();
    }

    if changed {
        fs::write(path, toml::to_string(&parsed)?)
            .with_context(|| format!("failed to write {}", path.display()))?;
    }
    Ok(())
}

fn merge_codex_config_value(managed: Value, existing: Value) -> Value {
    merge_codex_config_value_at(managed, existing, &[])
}

fn merge_codex_config_value_at(managed: Value, existing: Value, path: &[String]) -> Value {
    if existing_codex_preference_wins(path) {
        return existing;
    }

    match (managed, existing) {
        (Value::Table(managed_table), Value::Table(existing_table)) => {
            let mut merged = existing_table;
            for (key, managed_value) in managed_table {
                let mut next_path = path.to_vec();
                next_path.push(key.clone());
                let next_value = match merged.remove(&key) {
                    Some(existing_value) => {
                        merge_codex_config_value_at(managed_value, existing_value, &next_path)
                    }
                    None => managed_value,
                };
                merged.insert(key, next_value);
            }
            Value::Table(merged)
        }
        (managed_value, _) => managed_value,
    }
}

fn existing_codex_preference_wins(path: &[String]) -> bool {
    if path.len() == 1 {
        return matches!(
            path[0].as_str(),
            "model_reasoning_summary"
                | "model_verbosity"
                | "personality"
                | "service_tier"
                | "web_search"
                | "zsh_path"
                | "notify"
        );
    }

    if path.first().map(String::as_str) == Some("features") {
        return path.get(1).is_some_and(|key| key != "memories");
    }

    matches!(
        path.first().map(String::as_str),
        Some("tools" | "history" | "marketplaces" | "projects")
    )
}

fn run_rtk_init(home: &Path) -> Result<()> {
    run_command_in_home(
        home,
        "rtk",
        ["init", "-g", "--codex"],
        "initializing RTK for Codex",
    )
}

fn run_rtk_uninstall(home: &Path) -> Result<()> {
    if !command_exists("rtk") {
        println!("[warn] command rtk missing; skipping official Codex RTK uninstall");
        return Ok(());
    }

    run_command_in_home(
        home,
        "rtk",
        ["init", "-g", "--codex", "--uninstall"],
        "uninstalling RTK for Codex",
    )
}

fn rtk_tokens(rtk_enabled: bool) -> Vec<(&'static str, &'static str)> {
    if rtk_enabled {
        vec![
            (
                "__RTK_CODEX_RULE__",
                "- Prefer `rtk <command>` for shell commands that can generate large or noisy output, especially `git`, `curl`, test, build, and diff workflows.",
            ),
            (
                "__RTK_CODEX_HELPER__",
                "- `RTK.md` describes the preferred `rtk` shell wrapper workflow.",
            ),
            ("__RTK_CODEX_IMPORT__", "@RTK.md"),
        ]
    } else {
        vec![
            ("__RTK_CODEX_RULE__", ""),
            ("__RTK_CODEX_HELPER__", ""),
            ("__RTK_CODEX_IMPORT__", ""),
        ]
    }
}

fn unmanaged_mcp_blocks(root: &Path) -> Result<Vec<String>> {
    let config_path = root.join("config.toml");
    if !config_path.exists() {
        return Ok(Vec::new());
    }

    let raw = fs::read_to_string(&config_path)
        .with_context(|| format!("failed to read {}", config_path.display()))?;
    let value: Value = raw
        .parse::<Value>()
        .with_context(|| format!("failed to parse {}", config_path.display()))?;
    let Some(mcp_servers) = value.get("mcp_servers").and_then(Value::as_table) else {
        return Ok(Vec::new());
    };

    let mut blocks = Vec::new();
    for (name, table) in mcp_servers {
        if BaselineMcp::all().iter().any(|mcp| mcp.name() == name) {
            continue;
        }

        let mut mcp_servers_table = TomlMap::new();
        mcp_servers_table.insert(name.clone(), table.clone());

        let mut root_table = TomlMap::new();
        root_table.insert("mcp_servers".to_string(), Value::Table(mcp_servers_table));
        blocks.push(
            toml::to_string(&Value::Table(root_table))?
                .trim()
                .to_string(),
        );
    }
    Ok(blocks)
}
