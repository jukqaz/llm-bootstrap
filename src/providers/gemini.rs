use crate::cli::ApplyMode;
use crate::fs_ops::{
    backup_relative, copy_render_file_with_extras, copy_render_relative_entries,
    copy_selected_scripts, create_backup_root, remove_if_exists, resolve_backup_root,
    restore_relative,
};
use crate::json_ops::{
    cleanup_extension_enablement, cleanup_gemini_settings, merge_json,
    preserved_gemini_runtime_state, prune_rtk_gemini_hooks, read_json_or_empty, write_json_pretty,
};
use crate::layout::{
    all_gemini_bundle_doc_paths, all_gemini_extension_asset_paths, gemini_bundle_doc_paths,
    gemini_extension_asset_paths, gemini_extension_enablement_path, gemini_managed_paths,
    gemini_managed_paths_for,
};
use crate::manifest::{BaselineMcp, BootstrapManifest};
use crate::runtime::{command_exists, repo_root, run_command_in_home, timestamp_string};
use crate::state::read_installed_state;
use anyhow::Result;
use serde_json::{Map, Value, json};
use std::fs;
use std::path::{Path, PathBuf};

pub(crate) fn doctor_checks(
    home: &Path,
    enabled_mcp: &[BaselineMcp],
    rtk_enabled: bool,
    extension_enabled: bool,
    active_surfaces: &[String],
) -> Vec<PathBuf> {
    let root = home.join(".gemini");
    let mut checks = vec![root.join("GEMINI.md"), root.join("settings.json")];
    checks.extend(
        gemini_bundle_doc_paths(active_surfaces)
            .into_iter()
            .map(|relative| root.join(relative)),
    );
    if extension_enabled {
        checks.extend(
            gemini_extension_asset_paths(active_surfaces)
                .into_iter()
                .map(|relative| root.join(relative)),
        );
        checks.push(root.join(gemini_extension_enablement_path()));
    }
    if rtk_enabled {
        checks.push(root.join("hooks/rtk-hook-gemini.sh"));
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
    extension_enabled: bool,
    active_surfaces: &[String],
) -> Result<()> {
    let root = home.join(".gemini");
    let template_root = repo_root().join("templates/gemini");
    let bundle_root = repo_root().join("bundles/full/gemini");
    fs::create_dir_all(&root)?;
    fs::create_dir_all(root.join("hooks"))?;
    fs::create_dir_all(root.join("scripts"))?;
    fs::create_dir_all(root.join("extensions"))?;
    let backup_root = create_backup_root(&root, &timestamp_string()?)?;
    println!("[gemini] backup {}", backup_root.display());

    for relative in gemini_managed_paths() {
        backup_relative(&root, &backup_root, Path::new(relative))?;
    }

    let settings_path = root.join("settings.json");
    let existing_settings = read_json_or_empty(&settings_path)?;
    let enablement_path = root.join("extensions/extension-enablement.json");
    let existing_enablement = read_json_or_empty(&enablement_path)?;

    if mode == ApplyMode::Replace {
        for relative in gemini_managed_paths() {
            remove_if_exists(&root.join(relative))?;
        }
        fs::create_dir_all(root.join("hooks"))?;
        fs::create_dir_all(root.join("scripts"))?;
        fs::create_dir_all(root.join("extensions"))?;
    }

    if rtk_enabled {
        run_rtk_init(home)?;
    } else {
        remove_if_exists(&root.join("hooks/rtk-hook-gemini.sh"))?;
    }

    copy_render_file_with_extras(
        &template_root.join("GEMINI.md"),
        &root.join("GEMINI.md"),
        false,
        home,
        &rtk_tokens(rtk_enabled),
    )?;
    copy_selected_scripts(
        &template_root.join("scripts"),
        &root.join("scripts"),
        home,
        enabled_mcp,
    )?;
    for relative in all_gemini_bundle_doc_paths() {
        remove_if_exists(&root.join(relative))?;
    }
    copy_render_relative_entries(
        &bundle_root,
        &root,
        &gemini_bundle_doc_paths(active_surfaces),
        home,
    )?;
    for relative in all_gemini_extension_asset_paths() {
        remove_if_exists(&root.join(relative))?;
    }
    if extension_enabled {
        copy_render_relative_entries(
            &template_root,
            &root,
            &gemini_extension_asset_paths(active_surfaces),
            home,
        )?;
    } else {
        remove_if_exists(&root.join("extensions/llm-bootstrap-dev"))?;
    }

    let mut current_settings = match mode {
        ApplyMode::Merge => existing_settings.clone(),
        ApplyMode::Replace => preserved_gemini_runtime_state(&existing_settings),
    };
    merge_json(&mut current_settings, settings_patch(home, rtk_enabled));
    if !rtk_enabled {
        prune_rtk_gemini_hooks(&mut current_settings);
    }
    current_settings["mcpServers"] = mcp_servers(home, &existing_settings, enabled_mcp, mode);
    write_json_pretty(&settings_path, &current_settings)?;

    if extension_enabled {
        let mut enablement = match mode {
            ApplyMode::Merge => existing_enablement,
            ApplyMode::Replace => json!({}),
        };
        let override_path = format!("{}/{}", home.display(), "*");
        enablement["llm-bootstrap-dev"] = json!({
            "overrides": [override_path]
        });
        write_json_pretty(&enablement_path, &enablement)?;
    } else {
        remove_if_exists(&root.join("extensions/llm-bootstrap-dev"))?;
        cleanup_extension_enablement(&enablement_path)?;
    }

    let projects_registry_path = root.join("projects.json");
    if !projects_registry_path.exists() {
        write_json_pretty(&projects_registry_path, &json!({ "projects": {} }))?;
    }

    println!("[gemini] installed {} ({})", root.display(), mode.name());
    Ok(())
}

pub(crate) fn uninstall(
    home: &Path,
    manifest: &BootstrapManifest,
    rtk_enabled: bool,
) -> Result<()> {
    let root = home.join(".gemini");
    if !root.exists() {
        println!("[gemini] skipped uninstall: {} not found", root.display());
        return Ok(());
    }

    let installed_state = read_installed_state(&root)?;
    let uninstall_paths = if installed_state.managed_paths.is_empty() {
        if installed_state.active_surfaces.is_empty() {
            gemini_managed_paths()
                .into_iter()
                .map(ToOwned::to_owned)
                .collect::<Vec<_>>()
        } else {
            gemini_managed_paths_for(
                &installed_state.active_surfaces,
                crate::layout::provider_surface_enabled(&installed_state.active_surfaces),
                rtk_enabled,
            )
        }
    } else {
        installed_state.managed_paths
    };

    let backup_root = create_backup_root(&root, &timestamp_string()?)?;
    println!("[gemini] backup {}", backup_root.display());

    for relative in &uninstall_paths {
        backup_relative(&root, &backup_root, Path::new(relative))?;
    }

    if rtk_enabled {
        run_rtk_uninstall(home)?;
    }

    for relative in &uninstall_paths {
        if relative == "settings.json" || relative == "extensions/extension-enablement.json" {
            continue;
        }
        remove_if_exists(&root.join(relative))?;
    }

    cleanup_gemini_settings(&root.join("settings.json"), manifest, rtk_enabled)?;
    cleanup_extension_enablement(&root.join("extensions/extension-enablement.json"))?;

    println!("[gemini] uninstalled {}", root.display());
    Ok(())
}

pub(crate) fn restore(home: &Path, backup_name: Option<&str>) -> Result<()> {
    let root = home.join(".gemini");
    fs::create_dir_all(&root)?;
    let source_backup = resolve_backup_root(&root, backup_name)?;
    let installed_state = read_installed_state(&root)?;
    let managed_paths = if installed_state.managed_paths.is_empty() {
        gemini_managed_paths()
            .into_iter()
            .map(ToOwned::to_owned)
            .collect::<Vec<_>>()
    } else {
        installed_state.managed_paths
    };
    let backup_root = create_backup_root(&root, &timestamp_string()?)?;
    println!("[gemini] backup {}", backup_root.display());

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
        "[gemini] restored {} from {}",
        root.display(),
        source_backup.display()
    );
    Ok(())
}

fn run_rtk_init(home: &Path) -> Result<()> {
    run_command_in_home(
        home,
        "rtk",
        ["init", "-g", "--gemini", "--auto-patch"],
        "initializing RTK for Gemini",
    )
}

fn run_rtk_uninstall(home: &Path) -> Result<()> {
    if !command_exists("rtk") {
        println!("[warn] command rtk missing; skipping official Gemini RTK uninstall");
        return Ok(());
    }

    run_command_in_home(
        home,
        "rtk",
        ["init", "-g", "--gemini", "--uninstall", "--auto-patch"],
        "uninstalling RTK for Gemini",
    )
}

fn settings_patch(home: &Path, rtk_enabled: bool) -> Value {
    let hook_path = home.join(".gemini/hooks/rtk-hook-gemini.sh");
    let hooks = if rtk_enabled {
        json!({
            "BeforeTool": [
                {
                    "hooks": [
                        {
                            "command": hook_path.to_string_lossy(),
                            "type": "command"
                        }
                    ],
                    "matcher": "run_shell_command"
                }
            ]
        })
    } else {
        json!({})
    };

    json!({
        "general": {
            "defaultApprovalMode": "plan",
            "enableAutoUpdate": false,
            "enableNotifications": true,
            "plan": {
                "directory": "",
                "modelRouting": true
            },
            "vimMode": false
        },
        "hooks": hooks,
        "ideMode": true,
        "output": {
            "format": "text"
        },
        "showLineNumbers": false,
        "showMemoryUsage": true,
        "ui": {
            "autoThemeSwitching": true,
            "errorVerbosity": "full",
            "hideTips": true,
            "hideWindowTitle": false,
            "inlineThinkingMode": "full",
            "loadingPhrases": "all",
            "showMemoryUsage": true,
            "showStatusInTitle": true
        }
    })
}

fn mcp_servers(
    home: &Path,
    existing_settings: &Value,
    enabled_mcp: &[BaselineMcp],
    mode: ApplyMode,
) -> Value {
    let gemini_home = home.join(".gemini");
    let mut servers = match mode {
        ApplyMode::Merge => existing_settings
            .get("mcpServers")
            .and_then(Value::as_object)
            .map(|existing| {
                existing
                    .iter()
                    .filter(|(name, _)| {
                        !BaselineMcp::all()
                            .iter()
                            .any(|mcp| mcp.name() == name.as_str())
                    })
                    .map(|(name, value)| (name.clone(), value.clone()))
                    .collect::<Map<String, Value>>()
            })
            .unwrap_or_default(),
        ApplyMode::Replace => Map::new(),
    };

    for mcp in enabled_mcp {
        servers.insert(
            mcp.name().to_string(),
            json!({
                "command": gemini_home.join("scripts").join(mcp.script_name()).to_string_lossy().to_string()
            }),
        );
    }

    Value::Object(servers)
}

fn rtk_tokens(rtk_enabled: bool) -> Vec<(&'static str, &'static str)> {
    if rtk_enabled {
        vec![(
            "__RTK_GEMINI_RULE__",
            "- Prefer `rtk <command>` for large or noisy shell commands.",
        )]
    } else {
        vec![("__RTK_GEMINI_RULE__", "")]
    }
}
