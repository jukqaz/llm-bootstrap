use crate::cli::ApplyMode;
use crate::fs_ops::{
    backup_and_remove_relative_paths, backup_relative, copy_render_dir, copy_render_file,
    copy_render_file_with_extras, copy_render_relative_entries, copy_selected_scripts,
    create_backup_root, remove_if_exists, resolve_backup_root, restore_relative, toml_table_key,
};
use crate::layout::{
    CODEX_LEGACY_CLEANUP_PATHS, all_codex_bundle_doc_paths, all_codex_bundle_plugin_asset_paths,
    all_codex_plugin_asset_paths, codex_bundle_doc_paths, codex_bundle_plugin_asset_paths,
    codex_managed_paths, codex_managed_paths_for, codex_plugin_asset_paths,
};
use crate::manifest::{BaselineMcp, BootstrapManifest};
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
                        root.join("plugins/llm-dev-kit").join(relative),
                        root.join("plugins/cache/llm-bootstrap/llm-dev-kit/local")
                            .join(relative),
                    ]
                }),
        );
        checks.extend(
            codex_bundle_plugin_asset_paths(active_surfaces)
                .into_iter()
                .flat_map(|relative| {
                    [
                        root.join("plugins/llm-dev-kit").join(relative),
                        root.join("plugins/cache/llm-bootstrap/llm-dev-kit/local")
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
    let plugin_root = repo_root().join("plugins/llm-dev-kit");
    let marketplace_path = repo_root().join(".agents/plugins/marketplace.json");
    let installed_plugin_root = root.join("plugins/cache/llm-bootstrap/llm-dev-kit/local");
    let bundle_root = repo_root().join("bundles/full/codex");
    let bundle_plugin_root = repo_root().join("bundles/full/plugins/llm-dev-kit");
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
    copy_render_file_with_extras(
        &template_root.join("config.toml"),
        &root.join("config.toml"),
        false,
        home,
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
        remove_if_exists(&root.join("plugins/llm-dev-kit"))?;
        remove_if_exists(&root.join("plugins/cache/llm-bootstrap/llm-dev-kit"))?;
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
            remove_if_exists(&root.join("plugins/llm-dev-kit").join(relative))?;
            remove_if_exists(&installed_plugin_root.join(relative))?;
        }
        for relative in all_codex_bundle_plugin_asset_paths() {
            remove_if_exists(&root.join("plugins/llm-dev-kit").join(relative))?;
            remove_if_exists(&installed_plugin_root.join(relative))?;
        }
        copy_render_relative_entries(
            &plugin_root,
            &root.join("plugins/llm-dev-kit"),
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
            &root.join("plugins/llm-dev-kit"),
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
    let removed = backup_and_remove_relative_paths(root, backup_root, CODEX_LEGACY_CLEANUP_PATHS)?;
    if !removed.is_empty() {
        println!("[codex] removed legacy paths: {}", removed.join(","));
    }
    Ok(())
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
            format!(
                "[mcp_servers.{name}]\ncommand = \"{command}\"\nenabled = true",
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
        "[plugins.\"llm-dev-kit@llm-bootstrap\"]\nenabled = true".to_string()
    } else {
        String::new()
    }
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
