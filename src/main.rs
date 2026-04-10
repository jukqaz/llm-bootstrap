mod cli;
mod fs_ops;
mod json_ops;
mod layout;
mod manifest;
mod providers;
mod runtime;
mod state;

use anyhow::{Context, Result, bail};
use clap::Parser;
use cli::{
    BackupsArgs, Cli, Command, DoctorArgs, InstallArgs, PackArgs, Provider, ProviderArgs,
    RecordArgs, RecordSurface, RestoreArgs, UninstallArgs, WizardArgs,
};
use dialoguer::{Confirm, MultiSelect, Password, Select, theme::ColorfulTheme};
use fs_ops::{backup_relative, list_backup_entries, remove_if_exists};
use indexmap::IndexSet;
use manifest::{BaselineMcp, BootstrapManifest, DistributionTarget};
use providers::{claude, codex, gemini};
use runtime::{command_exists, ensure_runtime_dependencies, home_dir, repo_root, timestamp_string};
use serde::Serialize;
use state::{RequestedState, read_installed_state, write_installed_state};
use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::hash::Hash;
use std::path::{Path, PathBuf};
use std::process::Command as ProcessCommand;

use crate::layout::{
    HOME_LEGACY_CLEANUP_PATHS, LEGACY_ENV_KEYS, claude_managed_paths_for, codex_managed_paths_for,
    gemini_managed_paths_for,
};

const ZSHRC_MARKER_START: &str = "# >>> llm-bootstrap env >>>";
const ZSHRC_MARKER_END: &str = "# <<< llm-bootstrap env <<<";
const ZSHRC_ENV_RELATIVE_PATH: &str = ".zshrc.d/llm-bootstrap-env.zsh";

fn main() -> Result<()> {
    let cli = Cli::parse();
    let manifest = load_manifest()?;

    match cli.command.unwrap_or_else(default_command) {
        Command::Install(args) => install(args, &manifest),
        Command::Restore(args) => restore(args, &manifest),
        Command::Backups(args) => backups(args, &manifest),
        Command::Uninstall(args) => uninstall(args, &manifest),
        Command::Doctor(args) => doctor(args, &manifest),
        Command::Record(args) => record(args, &manifest),
        Command::Wizard(args) => wizard(args, &manifest),
    }
}

fn default_command() -> Command {
    Command::Wizard(WizardArgs::default())
}

fn load_manifest() -> Result<BootstrapManifest> {
    let path = repo_root().join("bootstrap.toml");
    let raw =
        fs::read_to_string(&path).with_context(|| format!("failed to read {}", path.display()))?;
    let manifest: BootstrapManifest =
        toml::from_str(&raw).with_context(|| format!("failed to parse {}", path.display()))?;
    validate_manifest(&manifest)?;
    Ok(manifest)
}

fn install(args: InstallArgs, manifest: &BootstrapManifest) -> Result<()> {
    let providers = selected_providers(&args.provider_args, manifest);
    let mode = args.mode.unwrap_or(manifest.bootstrap.default_mode);
    let rtk_enabled = is_rtk_enabled(args.without_rtk, manifest);
    let resolved = resolve_plan(manifest, &args.pack_args)?;
    let home = home_dir()?;
    if args.dry_run {
        print_install_plan(&home, &providers, manifest, mode, rtk_enabled, &resolved);
        return Ok(());
    }
    ensure_runtime_dependencies(rtk_enabled)?;
    install_with(&home, &providers, manifest, mode, rtk_enabled, &resolved)
}

fn uninstall(args: UninstallArgs, manifest: &BootstrapManifest) -> Result<()> {
    let home = home_dir()?;
    let providers = selected_providers(&args.provider_args, manifest);
    let rtk_enabled = is_rtk_enabled(args.without_rtk, manifest);
    let enabled_mcp = all_manifest_mcp(manifest);

    if args.dry_run {
        print_uninstall_plan(&home, &providers, rtk_enabled, &enabled_mcp);
        return Ok(());
    }

    cleanup_home_legacy_artifacts(&home)?;
    cleanup_legacy_env_vars(&home)?;

    for provider in &providers {
        match *provider {
            Provider::Codex => codex::uninstall(&home, rtk_enabled)?,
            Provider::Gemini => gemini::uninstall(&home, manifest, rtk_enabled)?,
            Provider::Claude => claude::uninstall(&home, &enabled_mcp, rtk_enabled)?,
        }
    }

    println!(
        "uninstalled providers: {} (rtk: {})",
        provider_names(&providers),
        if rtk_enabled { "enabled" } else { "disabled" }
    );
    Ok(())
}

fn restore(args: RestoreArgs, manifest: &BootstrapManifest) -> Result<()> {
    let home = home_dir()?;
    let providers = selected_providers(&args.provider_args, manifest);
    if args.list {
        return list_backups_for(&home, &providers, args.json);
    }
    if args.dry_run {
        return print_restore_plan(&home, &providers, args.backup.as_deref(), args.json);
    }

    for provider in &providers {
        match *provider {
            Provider::Codex => codex::restore(&home, args.backup.as_deref())?,
            Provider::Gemini => gemini::restore(&home, args.backup.as_deref())?,
            Provider::Claude => claude::restore(&home, args.backup.as_deref())?,
        }
    }

    println!(
        "restored providers: {} (backup: {})",
        provider_names(&providers),
        args.backup.as_deref().unwrap_or("latest"),
    );
    Ok(())
}

fn backups(args: BackupsArgs, manifest: &BootstrapManifest) -> Result<()> {
    let home = home_dir()?;
    let providers = selected_providers(&args.provider_args, manifest);
    list_backups_for(&home, &providers, args.json)
}

fn doctor(args: DoctorArgs, manifest: &BootstrapManifest) -> Result<()> {
    let home = home_dir()?;
    let providers = selected_providers(&args.provider_args, manifest);
    let resolved = resolve_plan(manifest, &args.pack_args)?;
    doctor_with(
        &home,
        &providers,
        manifest,
        is_rtk_enabled(args.without_rtk, manifest),
        args.json,
        &resolved,
    )
}

fn record(args: RecordArgs, manifest: &BootstrapManifest) -> Result<()> {
    record_with(&args, manifest)
}

fn doctor_with(
    home: &std::path::Path,
    providers: &[Provider],
    manifest: &BootstrapManifest,
    rtk_enabled: bool,
    json: bool,
    resolved: &ResolvedPlan,
) -> Result<()> {
    let mut failures = Vec::new();
    let mut warnings = Vec::new();
    let mut command_checks = Vec::new();
    let mut env_checks = Vec::new();
    let mut provider_reports = Vec::new();
    let catalog_report = doctor_catalog_report(
        manifest,
        &resolved.selection,
        &resolved.requested_mcp,
        &resolved.enabled_mcp,
        &resolved.distribution_state,
    );
    let codex_plugin_enabled = resolved
        .distribution_state
        .enabled(DistributionTarget::CodexPlugin);
    let gemini_extension_enabled = resolved
        .distribution_state
        .enabled(DistributionTarget::GeminiExtension);
    let claude_skills_enabled = resolved
        .distribution_state
        .enabled(DistributionTarget::ClaudeSkills);

    if !json {
        println!("[doctor] commands");
    }
    let mut commands = vec!["node", "npx"];
    if rtk_enabled {
        commands.insert(0, "rtk");
    }
    if providers.contains(&Provider::Claude) {
        commands.insert(0, "claude");
    }
    for command in commands {
        if command_exists(command) {
            if !json {
                println!("[ok] command {}", command);
            }
            command_checks.push(DoctorCheck {
                target: command.to_string(),
                status: "ok".to_string(),
                detail: None,
            });
        } else {
            if !json {
                println!("[missing] command {}", command);
            }
            failures.push(PathBuf::from(command));
            command_checks.push(DoctorCheck {
                target: command.to_string(),
                status: "missing".to_string(),
                detail: None,
            });
        }
    }

    if !json {
        println!("[doctor] api");
    }
    for gated in &resolved.env_gates {
        if !resolved.requested_mcp.contains(&gated.name) {
            continue;
        }
        if gated.enabled {
            if !json {
                println!("[ok] env {} enables {}", gated.env, gated.name.name());
            }
            env_checks.push(DoctorEnvCheck {
                name: gated.name.name().to_string(),
                env: gated.env.clone(),
                status: "ok".to_string(),
                detail: None,
            });
        } else {
            let detail = env_warning(&gated.env).to_string();
            if !json {
                println!(
                    "[warn] mcp {} disabled: env {} not set; {}",
                    gated.name.name(),
                    gated.env,
                    detail
                );
            }
            warnings.push(format!("{} disabled", gated.name.name()));
            env_checks.push(DoctorEnvCheck {
                name: gated.name.name().to_string(),
                env: gated.env.clone(),
                status: "warn".to_string(),
                detail: Some(detail),
            });
        }
    }

    for provider in providers {
        let installed_state = read_installed_state(&provider_root(home, *provider))?;
        let requested_surfaces = match provider {
            Provider::Codex => &resolved.surfaces.codex,
            Provider::Gemini => &resolved.surfaces.gemini,
            Provider::Claude => &resolved.surfaces.claude,
        };
        let requested_managed_paths = provider_managed_paths(
            *provider,
            rtk_enabled,
            &resolved.distribution_state,
            &resolved.surfaces,
        );
        let requested_state = RequestedState {
            active_preset: resolved.selection.preset.as_deref(),
            active_packs: &resolved.selection.packs,
            active_harnesses: &resolved.selection.harnesses,
            active_connectors: &catalog_report.active_connectors,
            active_automations: &catalog_report.active_automations,
            active_record_templates: &catalog_report.active_record_templates,
            active_surfaces: requested_surfaces,
            managed_paths: &requested_managed_paths,
        };
        let state_mismatch = installed_state.mismatch(&requested_state);
        if !json {
            println!("[doctor] provider {}", provider.name());
            if state_mismatch {
                println!(
                    "[warn] installed state differs: preset={}, packs={}",
                    installed_state
                        .active_preset
                        .as_deref()
                        .unwrap_or("unknown"),
                    installed_state.active_packs.join(",")
                );
            }
        }
        let checks = match provider {
            Provider::Codex => codex::doctor_checks(
                home,
                manifest,
                &resolved.enabled_mcp,
                rtk_enabled,
                codex_plugin_enabled,
                &resolved.surfaces.codex,
            ),
            Provider::Gemini => gemini::doctor_checks(
                home,
                &resolved.enabled_mcp,
                rtk_enabled,
                gemini_extension_enabled,
                &resolved.surfaces.gemini,
            ),
            Provider::Claude => claude::doctor_checks(
                home,
                &resolved.enabled_mcp,
                rtk_enabled,
                claude_skills_enabled,
                &resolved.surfaces.claude,
            ),
        };
        let mut provider_checks = Vec::new();

        for path in checks {
            if path.exists() {
                if !json {
                    println!("[ok] {}", path.display());
                }
                provider_checks.push(DoctorCheck {
                    target: path.display().to_string(),
                    status: "ok".to_string(),
                    detail: None,
                });
            } else {
                if !json {
                    println!("[missing] {}", path.display());
                }
                failures.push(path.clone());
                provider_checks.push(DoctorCheck {
                    target: path.display().to_string(),
                    status: "missing".to_string(),
                    detail: None,
                });
            }
        }
        provider_reports.push(DoctorProviderReport {
            provider: provider.name().to_string(),
            installed_preset: installed_state.active_preset,
            installed_packs: installed_state.active_packs,
            installed_harnesses: installed_state.active_harnesses,
            installed_connectors: installed_state.active_connectors,
            installed_automations: installed_state.active_automations,
            installed_record_templates: installed_state.active_record_templates,
            installed_surfaces: installed_state.active_surfaces,
            installed_managed_paths: installed_state.managed_paths,
            requested_managed_paths,
            state_mismatch,
            checks: provider_checks,
        });
    }

    if !json {
        println!("[doctor] catalog");
        println!("[ok] default preset: {}", catalog_report.default_preset);
        println!(
            "[ok] active packs: {}",
            catalog_report.active_packs.join(",")
        );
        println!(
            "[ok] active preset: {}",
            catalog_report.active_preset.as_deref().unwrap_or("custom")
        );
        println!(
            "[ok] active harnesses: {}",
            catalog_report.active_harnesses.join(",")
        );
        println!(
            "[ok] requested mcp: {}",
            catalog_report.requested_mcp_servers.join(",")
        );
        println!(
            "[ok] active mcp: {}",
            catalog_report.active_mcp_servers.join(",")
        );
        println!(
            "[ok] active connectors: {}",
            catalog_report.active_connectors.join(",")
        );
        println!(
            "[ok] active automations: {}",
            catalog_report.active_automations.join(",")
        );
        println!(
            "[ok] active record templates: {}",
            catalog_report.active_record_templates.join(",")
        );
        println!(
            "[ok] requested distribution targets: {}",
            catalog_report.requested_distribution_targets.join(",")
        );
        println!(
            "[ok] active distribution targets: {}",
            catalog_report.active_distribution_targets.join(",")
        );
        for harness in &catalog_report.harnesses {
            println!(
                "[ok] harness {} (category: {}, default: {})",
                harness.name,
                harness.category,
                if harness.default_enabled { "yes" } else { "no" }
            );
        }
        for pack in &catalog_report.packs {
            println!(
                "[ok] pack {} (scope: {}, lane: {}, harnesses: {}, connector apps: {}, mcp: {}, selected: {}, targets: {})",
                pack.name,
                pack.scope,
                pack.lane,
                pack.harnesses.join(","),
                pack.connector_apps.join(","),
                pack.mcp_servers.join(","),
                if pack.selected { "yes" } else { "no" },
                pack.resolved_distribution_targets.join(",")
            );
        }
        for connector in &catalog_report.connectors {
            println!(
                "[ok] connector {} (category: {}, access: {}, approval: {}, active: {})",
                connector.name,
                connector.category,
                connector.access,
                connector.approval,
                if connector.active { "yes" } else { "no" }
            );
        }
        for automation in &catalog_report.automations {
            println!(
                "[ok] automation {} (cadence: {}, artifact: {}, active: {})",
                automation.name,
                automation.cadence,
                automation.artifact,
                if automation.active { "yes" } else { "no" }
            );
        }
        println!(
            "[ok] record readiness: system={}, templates={}, missing_handoffs={}",
            catalog_report.record_readiness.record_system,
            catalog_report.record_readiness.active_templates.join(","),
            catalog_report.record_readiness.missing_handoffs.join(",")
        );
    }

    let report = DoctorReport {
        ok: failures.is_empty(),
        warning_count: warnings.len(),
        command_checks,
        env_checks,
        providers: provider_reports,
        catalog: catalog_report,
    };

    if json {
        println!("{}", serde_json::to_string_pretty(&report)?);
        if failures.is_empty() {
            return Ok(());
        }
        std::process::exit(1);
    }

    if failures.is_empty() {
        if warnings.is_empty() {
            println!("[doctor] complete: no blocking issues");
        } else {
            println!("[doctor] complete: {} warning(s)", warnings.len());
        }
        Ok(())
    } else {
        bail!("doctor found missing commands or files")
    }
}

fn wizard(_args: WizardArgs, manifest: &BootstrapManifest) -> Result<()> {
    let defaults = selected_providers(&ProviderArgs { providers: None }, manifest);
    let default_mode = manifest.bootstrap.default_mode;
    let default_rtk = manifest.external.rtk.enabled;
    let default_pack_selection = selected_pack_names(
        &PackArgs {
            preset: None,
            packs: None,
        },
        manifest,
    )?;
    let theme = ColorfulTheme::default();
    let provider_items = [Provider::Codex, Provider::Gemini, Provider::Claude];
    let provider_labels = provider_items
        .iter()
        .map(|provider| provider.name())
        .collect::<Vec<_>>();
    let provider_defaults = provider_items
        .iter()
        .map(|provider| defaults.contains(provider))
        .collect::<Vec<_>>();

    println!("llm-bootstrap wizard");
    println!(
        "defaults: providers={}, mode={}, rtk={}, preset={}",
        provider_names(&defaults),
        default_mode.name(),
        if default_rtk { "enabled" } else { "disabled" },
        default_pack_selection.preset.as_deref().unwrap_or("custom")
    );

    let preset_items = manifest
        .presets
        .iter()
        .map(|preset| format!("{}: {}", preset.name, preset.description))
        .collect::<Vec<_>>();
    let default_preset_index = default_pack_selection
        .preset
        .as_ref()
        .and_then(|name| {
            manifest
                .presets
                .iter()
                .position(|preset| &preset.name == name)
        })
        .unwrap_or(0);

    let selected_indices = MultiSelect::with_theme(&theme)
        .with_prompt("providers")
        .items(&provider_labels)
        .defaults(&provider_defaults)
        .interact()?;
    let providers = if selected_indices.is_empty() {
        defaults
    } else {
        selected_indices
            .into_iter()
            .map(|index| provider_items[index])
            .collect::<Vec<_>>()
    };

    let pack_selection = if preset_items.is_empty() {
        default_pack_selection
    } else {
        let preset_index = Select::with_theme(&theme)
            .with_prompt("set menu")
            .items(&preset_items)
            .default(default_preset_index)
            .interact()?;
        selected_pack_names(
            &PackArgs {
                preset: Some(manifest.presets[preset_index].name.clone()),
                packs: None,
            },
            manifest,
        )?
    };
    let active_record_templates = selected_record_template_names(manifest, &pack_selection.packs);
    let record_surface_items = [
        "local docs only",
        "GitHub issues + repo docs",
        "local docs + GitHub issue",
    ];
    let record_surface = if active_record_templates.is_empty() {
        RecordSurface::LocalDocs
    } else {
        match Select::with_theme(&theme)
            .with_prompt("record surface")
            .items(record_surface_items)
            .default(0)
            .interact()?
        {
            0 => RecordSurface::LocalDocs,
            1 => RecordSurface::GithubIssue,
            2 => RecordSurface::Both,
            _ => unreachable!(),
        }
    };

    let mode = match Select::with_theme(&theme)
        .with_prompt("mode")
        .items(["merge", "replace"])
        .default(if default_mode == cli::ApplyMode::Merge {
            0
        } else {
            1
        })
        .interact()?
    {
        0 => cli::ApplyMode::Merge,
        1 => cli::ApplyMode::Replace,
        _ => unreachable!(),
    };
    let rtk_enabled = Confirm::with_theme(&theme)
        .with_prompt("enable RTK?")
        .default(default_rtk)
        .interact()?;

    let exa_key = prompt_secret_with_dialoguer(
        &theme,
        "EXA_API_KEY (leave blank to keep current or disabled)",
    )?;
    let context7_key = prompt_secret_with_dialoguer(
        &theme,
        "CONTEXT7_API_KEY (leave blank to keep current or disabled)",
    )?;
    let persistence_items = [
        "GUI apps (launchctl setenv)",
        "CLI shells (~/.zshrc + ~/.zshrc.d/llm-bootstrap-env.zsh)",
    ];
    let persistence_defaults = [true, true];
    let persistence = MultiSelect::with_theme(&theme)
        .with_prompt("persist entered keys for")
        .items(persistence_items)
        .defaults(&persistence_defaults)
        .interact()?;
    let persist_gui = persistence.contains(&0);
    let persist_cli = persistence.contains(&1);
    let apply_now = Confirm::with_theme(&theme)
        .with_prompt("run install now?")
        .default(true)
        .interact()?;

    let env_overrides = wizard_env_overrides(&exa_key, &context7_key);
    let env_gates = resolved_env_gates(manifest, |name| {
        env_overrides
            .get(name)
            .copied()
            .unwrap_or_else(|| env_is_set(name))
    });
    let resolved = build_resolved_plan(manifest, pack_selection.clone(), env_gates);

    let keys_to_persist = [
        ("EXA_API_KEY", exa_key.as_deref()),
        ("CONTEXT7_API_KEY", context7_key.as_deref()),
    ];
    persist_env_keys(&keys_to_persist, persist_gui, persist_cli)?;

    println!(
        "wizard summary: providers={}, mode={}, rtk={}, preset={}, packs={}, record_surface={}, record_templates={}, exa={}, context7={}, gui_persist={}, cli_persist={}",
        provider_names(&providers),
        mode.name(),
        if rtk_enabled { "enabled" } else { "disabled" },
        resolved.selection.preset.as_deref().unwrap_or("custom"),
        resolved.selection.packs.join(","),
        record_surface.name(),
        selected_record_template_names(manifest, &resolved.selection.packs).join(","),
        if resolved.enabled_mcp.contains(&BaselineMcp::Exa) {
            "enabled"
        } else {
            "disabled"
        },
        if resolved.enabled_mcp.contains(&BaselineMcp::Context7) {
            "enabled"
        } else {
            "disabled"
        },
        if persist_gui { "yes" } else { "no" },
        if persist_cli { "yes" } else { "no" },
    );

    if apply_now {
        ensure_runtime_dependencies(rtk_enabled)?;
        let home = home_dir()?;
        install_with(&home, &providers, manifest, mode, rtk_enabled, &resolved)?;
        doctor_with(&home, &providers, manifest, rtk_enabled, false, &resolved)?;
    }

    Ok(())
}

fn install_with(
    home: &std::path::Path,
    providers: &[Provider],
    manifest: &BootstrapManifest,
    mode: cli::ApplyMode,
    rtk_enabled: bool,
    resolved: &ResolvedPlan,
) -> Result<()> {
    let codex_plugin_enabled = resolved
        .distribution_state
        .enabled(DistributionTarget::CodexPlugin);
    let gemini_extension_enabled = resolved
        .distribution_state
        .enabled(DistributionTarget::GeminiExtension);
    let claude_skills_enabled = resolved
        .distribution_state
        .enabled(DistributionTarget::ClaudeSkills);
    let active_connectors = selected_connector_names(manifest, &resolved.selection.packs);
    let active_automations = selected_automation_names(manifest, &resolved.selection.packs);
    let active_record_templates =
        selected_record_template_names(manifest, &resolved.selection.packs);
    if mode == cli::ApplyMode::Replace {
        cleanup_home_legacy_artifacts(home)?;
        cleanup_legacy_env_vars(home)?;
    }
    for provider in providers {
        match *provider {
            Provider::Codex => codex::install(
                home,
                mode,
                manifest,
                &resolved.enabled_mcp,
                rtk_enabled,
                codex_plugin_enabled,
                &resolved.surfaces.codex,
            )?,
            Provider::Gemini => gemini::install(
                home,
                mode,
                manifest,
                &resolved.enabled_mcp,
                rtk_enabled,
                gemini_extension_enabled,
                &resolved.surfaces.gemini,
            )?,
            Provider::Claude => claude::install(
                home,
                mode,
                manifest,
                &resolved.enabled_mcp,
                rtk_enabled,
                claude_skills_enabled,
                &resolved.surfaces.claude,
            )?,
        }
        write_installed_state(
            &provider_root(home, *provider),
            &resolved.enabled_mcp,
            &state::InstalledState {
                active_preset: resolved.selection.preset.clone(),
                active_packs: resolved.selection.packs.clone(),
                active_harnesses: resolved.selection.harnesses.clone(),
                active_connectors: active_connectors.clone(),
                active_automations: active_automations.clone(),
                active_record_templates: active_record_templates.clone(),
                active_surfaces: provider_surfaces(*provider, &resolved.surfaces).to_vec(),
                managed_paths: provider_managed_paths(
                    *provider,
                    rtk_enabled,
                    &resolved.distribution_state,
                    &resolved.surfaces,
                ),
            },
        )?;
    }

    println!(
        "installed providers: {} (mode: {}, rtk: {}, preset: {}, packs: {}, requested_targets: {}, targets: {})",
        provider_names(providers),
        mode.name(),
        if rtk_enabled { "enabled" } else { "disabled" },
        resolved.selection.preset.as_deref().unwrap_or("custom"),
        resolved.selection.packs.join(","),
        resolved
            .distribution_state
            .requested
            .iter()
            .map(|target| target.name())
            .collect::<Vec<_>>()
            .join(","),
        resolved
            .distribution_state
            .effective
            .iter()
            .map(|target| target.name())
            .collect::<Vec<_>>()
            .join(","),
    );
    Ok(())
}

fn cleanup_home_legacy_artifacts(home: &std::path::Path) -> Result<()> {
    let existing = HOME_LEGACY_CLEANUP_PATHS
        .iter()
        .copied()
        .filter(|relative| home.join(relative).exists())
        .collect::<Vec<_>>();
    if existing.is_empty() {
        return Ok(());
    }

    let backups_dir = home.join(".llm-bootstrap-legacy-backups");
    let timestamp = timestamp_string()?;
    let mut backup_root = backups_dir.join(format!("legacy-cleanup-{timestamp}"));
    let mut suffix = 1usize;
    while backup_root.exists() {
        backup_root = backups_dir.join(format!("legacy-cleanup-{timestamp}-{suffix}"));
        suffix += 1;
    }
    fs::create_dir_all(&backup_root)
        .with_context(|| format!("failed to create {}", backup_root.display()))?;

    for relative in &existing {
        backup_relative(home, &backup_root, Path::new(relative))?;
    }
    for relative in &existing {
        remove_if_exists(&home.join(relative))?;
    }

    println!(
        "[legacy] removed home paths: {} (backup: {})",
        existing.join(","),
        backup_root.display()
    );
    Ok(())
}

fn cleanup_legacy_env_vars(home: &std::path::Path) -> Result<()> {
    let launchctl_removed = unset_launchctl_legacy_env_vars();
    let cli_removed = remove_legacy_managed_cli_env(&home.join(ZSHRC_ENV_RELATIVE_PATH))?;

    if !launchctl_removed.is_empty() {
        println!(
            "[legacy] removed launchctl env: {}",
            launchctl_removed.join(",")
        );
    }
    if !cli_removed.is_empty() {
        println!("[legacy] removed CLI env: {}", cli_removed.join(","));
    }
    Ok(())
}

fn unset_launchctl_legacy_env_vars() -> Vec<String> {
    LEGACY_ENV_KEYS
        .iter()
        .filter_map(|name| {
            launchctl_env_value(name)?;
            let status = ProcessCommand::new("launchctl")
                .args(["unsetenv", name])
                .status()
                .ok()?;
            status.success().then(|| (*name).to_string())
        })
        .collect()
}

fn remove_legacy_managed_cli_env(path: &Path) -> Result<Vec<String>> {
    let mut entries = read_managed_env_entries(path)?;
    let removed = entries
        .keys()
        .filter(|name| is_legacy_env_key(name))
        .cloned()
        .collect::<Vec<_>>();
    if removed.is_empty() {
        return Ok(Vec::new());
    }

    for name in &removed {
        entries.remove(name);
    }
    write_managed_env_entries(path, &entries)?;
    Ok(removed)
}

fn is_legacy_env_key(name: &str) -> bool {
    let upper = name.to_ascii_uppercase();
    upper == "OMX"
        || upper == "OMG"
        || upper == "OMC"
        || upper.starts_with("OMX_")
        || upper.starts_with("OMG_")
        || upper.starts_with("OMC_")
        || upper.starts_with("OH_MY_")
}

fn print_install_plan(
    home: &Path,
    providers: &[Provider],
    _manifest: &BootstrapManifest,
    mode: cli::ApplyMode,
    rtk_enabled: bool,
    resolved: &ResolvedPlan,
) {
    println!("[dry-run] install");
    println!("providers: {}", provider_names(providers));
    println!("mode: {}", mode.name());
    println!(
        "preset: {}",
        resolved.selection.preset.as_deref().unwrap_or("custom")
    );
    println!("packs: {}", resolved.selection.packs.join(","));
    println!("harnesses: {}", resolved.selection.harnesses.join(","));
    println!(
        "requested_distribution_targets: {}",
        resolved
            .distribution_state
            .requested
            .iter()
            .map(|target| target.name())
            .collect::<Vec<_>>()
            .join(",")
    );
    println!(
        "distribution_targets: {}",
        resolved
            .distribution_state
            .effective
            .iter()
            .map(|target| target.name())
            .collect::<Vec<_>>()
            .join(",")
    );
    println!("rtk: {}", if rtk_enabled { "enabled" } else { "disabled" });
    println!(
        "requested_mcp: {}",
        resolved
            .requested_mcp
            .iter()
            .map(|mcp| mcp.name())
            .collect::<Vec<_>>()
            .join(",")
    );
    println!(
        "baseline_mcp: {}",
        resolved
            .enabled_mcp
            .iter()
            .map(|mcp| mcp.name())
            .collect::<Vec<_>>()
            .join(",")
    );
    for provider in providers {
        println!(
            "- {} root: {}",
            provider.name(),
            provider_root(home, *provider).display()
        );
    }
}

fn print_uninstall_plan(
    home: &Path,
    providers: &[Provider],
    rtk_enabled: bool,
    enabled_mcp: &[BaselineMcp],
) {
    println!("[dry-run] uninstall");
    println!("providers: {}", provider_names(providers));
    println!("rtk: {}", if rtk_enabled { "enabled" } else { "disabled" });
    for provider in providers {
        println!(
            "- {} root: {}",
            provider.name(),
            provider_root(home, *provider).display()
        );
    }
    if providers.contains(&Provider::Claude) {
        println!(
            "claude managed_mcp removal target: {}",
            enabled_mcp
                .iter()
                .map(|mcp| mcp.name())
                .collect::<Vec<_>>()
                .join(",")
        );
    }
}

fn print_restore_plan(
    home: &Path,
    providers: &[Provider],
    backup_name: Option<&str>,
    json: bool,
) -> Result<()> {
    let mut plans = Vec::new();
    for provider in providers {
        let root = provider_root(home, *provider);
        let backup = fs_ops::resolve_backup_root(&root, backup_name)?;
        plans.push(RestorePlan {
            provider: provider.name().to_string(),
            root: root.display().to_string(),
            backup: backup.display().to_string(),
        });
    }

    if json {
        println!("{}", serde_json::to_string_pretty(&plans)?);
        return Ok(());
    }

    println!("[dry-run] restore");
    for plan in plans {
        println!(
            "- {} root={} backup={}",
            plan.provider, plan.root, plan.backup
        );
    }
    Ok(())
}

fn list_backups_for(home: &Path, providers: &[Provider], json: bool) -> Result<()> {
    let mut reports = Vec::new();
    for provider in providers {
        let root = provider_root(home, *provider);
        let backups = list_backup_entries(&root)?
            .into_iter()
            .map(|entry| BackupSummary {
                name: entry.name,
                path: entry.path.display().to_string(),
            })
            .collect::<Vec<_>>();
        reports.push(ProviderBackups {
            provider: provider.name().to_string(),
            root: root.display().to_string(),
            backups,
        });
    }

    if json {
        println!("{}", serde_json::to_string_pretty(&reports)?);
        return Ok(());
    }

    println!("[backups]");
    for report in reports {
        println!("provider: {}", report.provider);
        println!("root: {}", report.root);
        if report.backups.is_empty() {
            println!("backups: none");
            continue;
        }
        for backup in report.backups {
            println!("- {} ({})", backup.name, backup.path);
        }
    }
    Ok(())
}

#[derive(Serialize)]
struct DoctorReport {
    ok: bool,
    warning_count: usize,
    command_checks: Vec<DoctorCheck>,
    env_checks: Vec<DoctorEnvCheck>,
    providers: Vec<DoctorProviderReport>,
    catalog: DoctorCatalogReport,
}

#[derive(Serialize)]
struct DoctorProviderReport {
    provider: String,
    installed_preset: Option<String>,
    installed_packs: Vec<String>,
    installed_harnesses: Vec<String>,
    installed_connectors: Vec<String>,
    installed_automations: Vec<String>,
    installed_record_templates: Vec<String>,
    installed_surfaces: Vec<String>,
    installed_managed_paths: Vec<String>,
    requested_managed_paths: Vec<String>,
    state_mismatch: bool,
    checks: Vec<DoctorCheck>,
}

#[derive(Serialize)]
struct DoctorCheck {
    target: String,
    status: String,
    detail: Option<String>,
}

#[derive(Serialize)]
struct DoctorEnvCheck {
    name: String,
    env: String,
    status: String,
    detail: Option<String>,
}

#[derive(Serialize)]
struct DoctorCatalogReport {
    default_preset: String,
    active_preset: Option<String>,
    active_packs: Vec<String>,
    active_harnesses: Vec<String>,
    requested_mcp_servers: Vec<String>,
    active_mcp_servers: Vec<String>,
    active_connectors: Vec<String>,
    active_automations: Vec<String>,
    active_record_templates: Vec<String>,
    requested_distribution_targets: Vec<String>,
    active_distribution_targets: Vec<String>,
    harnesses: Vec<DoctorHarnessReport>,
    packs: Vec<DoctorPackReport>,
    presets: Vec<DoctorPresetReport>,
    connectors: Vec<DoctorConnectorReport>,
    automations: Vec<DoctorAutomationReport>,
    record_templates: Vec<DoctorRecordTemplateReport>,
    runtime_handoff: DoctorRuntimeHandoffReport,
    record_readiness: DoctorRecordReadinessReport,
}

#[derive(Serialize)]
struct DoctorRuntimeHandoffReport {
    active_app_connector_count: usize,
    pending_app_verification_count: usize,
    active_automation_count: usize,
    pending_scheduler_registration_count: usize,
    connector_queue: Vec<String>,
    automation_queue: Vec<String>,
    next_steps: Vec<String>,
}

#[derive(Serialize)]
struct DoctorPresetReport {
    name: String,
    packs: Vec<String>,
    selected: bool,
    description: String,
}

#[derive(Serialize)]
struct DoctorConnectorReport {
    name: String,
    category: String,
    tool_source: String,
    access: String,
    approval: String,
    automation_allowed: bool,
    active: bool,
    health: String,
    auth_state: String,
    runtime_owner: String,
    verification_mode: String,
    connection_status: String,
    next_step: Option<String>,
    detail: Option<String>,
    description: String,
}

#[derive(Serialize)]
struct DoctorAutomationReport {
    name: String,
    cadence: String,
    packs: Vec<String>,
    connectors: Vec<String>,
    artifact: String,
    active: bool,
    status: String,
    scheduler_owner: String,
    registration_status: String,
    next_step: Option<String>,
    detail: Option<String>,
    description: String,
}

#[derive(Serialize)]
struct DoctorRecordTemplateReport {
    name: String,
    record_type: String,
    stage: String,
    packs: Vec<String>,
    surfaces: Vec<String>,
    active: bool,
    runtime_owner: String,
    description: String,
}

#[derive(Serialize)]
struct DoctorRecordReadinessReport {
    enabled: bool,
    record_system: String,
    runtime_owner: String,
    active_templates: Vec<String>,
    missing_handoffs: Vec<String>,
    next_action: String,
}

struct ResolvedDistributionState {
    requested: Vec<DistributionTarget>,
    effective: Vec<DistributionTarget>,
}

#[derive(Clone)]
struct ActiveSelection {
    preset: Option<String>,
    packs: Vec<String>,
    harnesses: Vec<String>,
}

struct ProviderSurfaces {
    codex: Vec<String>,
    gemini: Vec<String>,
    claude: Vec<String>,
}

struct ResolvedPlan {
    selection: ActiveSelection,
    env_gates: Vec<ResolvedEnvGate>,
    requested_mcp: Vec<BaselineMcp>,
    enabled_mcp: Vec<BaselineMcp>,
    distribution_state: ResolvedDistributionState,
    surfaces: ProviderSurfaces,
}

impl ResolvedDistributionState {
    fn resolve(manifest: &BootstrapManifest, active_packs: &[String]) -> Self {
        Self {
            requested: selected_distribution_targets(manifest, active_packs),
            effective: effective_distribution_targets(manifest, active_packs),
        }
    }

    fn enabled(&self, target: DistributionTarget) -> bool {
        has_distribution_target(&self.effective, target)
    }
}

#[derive(Serialize)]
struct DoctorHarnessReport {
    name: String,
    category: String,
    default_enabled: bool,
    description: String,
}

#[derive(Serialize)]
struct DoctorPackReport {
    name: String,
    scope: String,
    lane: String,
    harnesses: Vec<String>,
    connector_apps: Vec<String>,
    mcp_servers: Vec<String>,
    connectors: Vec<String>,
    codex_surfaces: Vec<String>,
    gemini_surfaces: Vec<String>,
    claude_surfaces: Vec<String>,
    resolved_distribution_targets: Vec<String>,
    selected: bool,
    description: String,
}

#[derive(Serialize)]
struct ProviderBackups {
    provider: String,
    root: String,
    backups: Vec<BackupSummary>,
}

#[derive(Serialize)]
struct BackupSummary {
    name: String,
    path: String,
}

#[derive(Serialize)]
struct RestorePlan {
    provider: String,
    root: String,
    backup: String,
}

fn env_warning(name: &str) -> &'static str {
    match name {
        "EXA_API_KEY" => "Exa stays disabled until EXA_API_KEY is exported",
        "CONTEXT7_API_KEY" => "Context7 stays disabled until CONTEXT7_API_KEY is exported",
        _ => "recommended runtime env is missing",
    }
}

fn doctor_catalog_report(
    manifest: &BootstrapManifest,
    selection: &ActiveSelection,
    requested_mcp: &[BaselineMcp],
    active_mcp: &[BaselineMcp],
    distribution_state: &ResolvedDistributionState,
) -> DoctorCatalogReport {
    let active_connectors = selected_connector_names(manifest, &selection.packs);
    let active_automations = selected_automation_names(manifest, &selection.packs);
    let active_record_templates = selected_record_template_names(manifest, &selection.packs);

    DoctorCatalogReport {
        default_preset: manifest.bootstrap.default_preset.clone(),
        active_preset: selection.preset.clone(),
        active_packs: selection.packs.clone(),
        active_harnesses: selection.harnesses.clone(),
        requested_mcp_servers: requested_mcp
            .iter()
            .map(|mcp| mcp.name().to_string())
            .collect(),
        active_mcp_servers: active_mcp
            .iter()
            .map(|mcp| mcp.name().to_string())
            .collect(),
        active_connectors: active_connectors.clone(),
        active_automations: active_automations.clone(),
        active_record_templates: active_record_templates.clone(),
        requested_distribution_targets: distribution_state
            .requested
            .iter()
            .map(|target| target.name().to_string())
            .collect(),
        active_distribution_targets: distribution_state
            .effective
            .iter()
            .map(|target| target.name().to_string())
            .collect(),
        harnesses: manifest
            .harnesses
            .iter()
            .map(|harness| DoctorHarnessReport {
                name: harness.name.clone(),
                category: harness.category.name().to_string(),
                default_enabled: harness.default_enabled,
                description: harness.description.clone(),
            })
            .collect(),
        packs: manifest
            .packs
            .iter()
            .map(|pack| DoctorPackReport {
                name: pack.name.clone(),
                scope: pack.scope.name().to_string(),
                lane: pack.lane.name().to_string(),
                harnesses: pack.harnesses.clone(),
                connector_apps: pack_app_names(manifest, pack),
                mcp_servers: pack
                    .mcp_servers
                    .iter()
                    .map(|mcp| mcp.name().to_string())
                    .collect(),
                connectors: pack.connectors.clone(),
                codex_surfaces: pack.codex_surfaces.clone(),
                gemini_surfaces: pack.gemini_surfaces.clone(),
                claude_surfaces: pack.claude_surfaces.clone(),
                resolved_distribution_targets: pack_distribution_targets(pack)
                    .into_iter()
                    .map(|target| target.name().to_string())
                    .collect(),
                selected: selection.packs.contains(&pack.name),
                description: pack.description.clone(),
            })
            .collect(),
        presets: manifest
            .presets
            .iter()
            .map(|preset| DoctorPresetReport {
                name: preset.name.clone(),
                packs: preset.packs.clone(),
                selected: selection.preset.as_deref() == Some(preset.name.as_str()),
                description: preset.description.clone(),
            })
            .collect(),
        connectors: manifest
            .connectors
            .iter()
            .map(|connector| DoctorConnectorReport {
                name: connector.name.clone(),
                category: connector.category.name().to_string(),
                tool_source: connector.tool_source.name().to_string(),
                access: connector.access.name().to_string(),
                approval: connector.approval.name().to_string(),
                automation_allowed: connector.automation_allowed,
                active: active_connectors.contains(&connector.name),
                health: connector_health(connector, &active_connectors).to_string(),
                auth_state: connector_auth_state(connector, &active_connectors).to_string(),
                runtime_owner: connector_runtime_owner(connector).to_string(),
                verification_mode: connector_verification_mode(connector).to_string(),
                connection_status: connector_connection_status(connector, &active_connectors)
                    .to_string(),
                next_step: connector_next_step(connector, &active_connectors),
                detail: connector_detail(connector, &active_connectors),
                description: connector.description.clone(),
            })
            .collect(),
        automations: manifest
            .automations
            .iter()
            .map(|automation| DoctorAutomationReport {
                name: automation.name.clone(),
                cadence: automation.cadence.name().to_string(),
                packs: automation.packs.clone(),
                connectors: automation.connectors.clone(),
                artifact: automation.artifact.clone(),
                active: active_automations.contains(&automation.name),
                status: automation_status(automation, &active_automations).to_string(),
                scheduler_owner: automation_scheduler_owner().to_string(),
                registration_status: automation_registration_status(
                    automation,
                    &active_automations,
                )
                .to_string(),
                next_step: automation_next_step(automation, &active_automations),
                detail: automation_detail(automation, &active_automations),
                description: automation.description.clone(),
            })
            .collect(),
        record_templates: manifest
            .record_templates
            .iter()
            .map(|record| DoctorRecordTemplateReport {
                name: record.name.clone(),
                record_type: record.record_type.clone(),
                stage: record.stage.clone(),
                packs: record.packs.clone(),
                surfaces: record.surfaces.clone(),
                active: active_record_templates.contains(&record.name),
                runtime_owner: record_runtime_owner(record, &active_record_templates).to_string(),
                description: record.description.clone(),
            })
            .collect(),
        runtime_handoff: doctor_runtime_handoff_report(
            manifest,
            &active_connectors,
            &active_automations,
        ),
        record_readiness: doctor_record_readiness_report(
            &active_connectors,
            &active_record_templates,
        ),
    }
}

fn doctor_record_readiness_report(
    active_connectors: &[String],
    active_record_templates: &[String],
) -> DoctorRecordReadinessReport {
    let missing_handoffs = active_connectors
        .iter()
        .filter(|connector| connector.as_str() != "github")
        .cloned()
        .collect::<Vec<_>>();

    let next_action = if active_record_templates.is_empty() {
        "no record templates are active for the selected packs".to_string()
    } else if missing_handoffs.is_empty() {
        "write operating records to local docs or GitHub issue and repo-doc links".to_string()
    } else {
        "write operating records to local docs or GitHub links, then verify external source-of-truth handoffs"
            .to_string()
    };

    DoctorRecordReadinessReport {
        enabled: !active_record_templates.is_empty(),
        record_system: "local-docs+github-issues".to_string(),
        runtime_owner: "bootstrap-contracts+external-tools".to_string(),
        active_templates: active_record_templates.to_vec(),
        missing_handoffs,
        next_action,
    }
}

fn record_runtime_owner(
    record: &manifest::RecordTemplateDefinition,
    active_record_templates: &[String],
) -> &'static str {
    if !active_record_templates.contains(&record.name) {
        return "not-requested";
    }

    if record
        .surfaces
        .iter()
        .any(|surface| surface == "github-issues")
    {
        "external-tools"
    } else {
        "bootstrap-contract"
    }
}

fn doctor_runtime_handoff_report(
    manifest: &BootstrapManifest,
    active_connectors: &[String],
    active_automations: &[String],
) -> DoctorRuntimeHandoffReport {
    let connector_queue = manifest
        .connectors
        .iter()
        .filter(|connector| active_connectors.contains(&connector.name))
        .filter(|connector| {
            matches!(
                connector.tool_source,
                manifest::ConnectorToolSource::App | manifest::ConnectorToolSource::Native
            )
        })
        .map(|connector| connector.name.clone())
        .collect::<Vec<_>>();

    let automation_queue = manifest
        .automations
        .iter()
        .filter(|automation| active_automations.contains(&automation.name))
        .map(|automation| automation.name.clone())
        .collect::<Vec<_>>();

    let mut next_steps = Vec::new();
    if !connector_queue.is_empty() {
        next_steps.push(
            "open each provider runtime and verify active app connectors with one real read action"
                .to_string(),
        );
    }
    if !automation_queue.is_empty() {
        next_steps.push(
            "register active automation contracts in the target runtime scheduler before expecting recurring runs"
                .to_string(),
        );
    }
    if next_steps.is_empty() {
        next_steps.push("no runtime handoff work is pending for the active preset".to_string());
    }

    DoctorRuntimeHandoffReport {
        active_app_connector_count: connector_queue.len(),
        pending_app_verification_count: connector_queue.len(),
        active_automation_count: automation_queue.len(),
        pending_scheduler_registration_count: automation_queue.len(),
        connector_queue,
        automation_queue,
        next_steps,
    }
}

fn connector_health(
    connector: &manifest::ConnectorDefinition,
    active_connectors: &[String],
) -> &'static str {
    if !active_connectors.contains(&connector.name) {
        return "inactive";
    }

    match connector.tool_source {
        manifest::ConnectorToolSource::App => "runtime-managed",
        manifest::ConnectorToolSource::Mcp => "managed",
        manifest::ConnectorToolSource::Native => "ready",
    }
}

fn connector_auth_state(
    connector: &manifest::ConnectorDefinition,
    active_connectors: &[String],
) -> &'static str {
    if !active_connectors.contains(&connector.name) {
        return "not-needed";
    }

    match connector.tool_source {
        manifest::ConnectorToolSource::App => "external-runtime",
        manifest::ConnectorToolSource::Mcp => "bootstrap-managed",
        manifest::ConnectorToolSource::Native => "native-runtime",
    }
}

fn connector_runtime_owner(connector: &manifest::ConnectorDefinition) -> &'static str {
    match connector.tool_source {
        manifest::ConnectorToolSource::App => "provider-runtime",
        manifest::ConnectorToolSource::Mcp => "bootstrap",
        manifest::ConnectorToolSource::Native => "provider-native",
    }
}

fn connector_verification_mode(connector: &manifest::ConnectorDefinition) -> &'static str {
    match connector.tool_source {
        manifest::ConnectorToolSource::App => "manual-runtime-check",
        manifest::ConnectorToolSource::Mcp => "bootstrap-check",
        manifest::ConnectorToolSource::Native => "native-check",
    }
}

fn connector_connection_status(
    connector: &manifest::ConnectorDefinition,
    active_connectors: &[String],
) -> &'static str {
    if !active_connectors.contains(&connector.name) {
        return "not-requested";
    }

    match connector.tool_source {
        manifest::ConnectorToolSource::App => "not-verified",
        manifest::ConnectorToolSource::Mcp => "managed",
        manifest::ConnectorToolSource::Native => "ready",
    }
}

fn connector_next_step(
    connector: &manifest::ConnectorDefinition,
    active_connectors: &[String],
) -> Option<String> {
    if !active_connectors.contains(&connector.name) {
        return None;
    }

    match connector.tool_source {
        manifest::ConnectorToolSource::App => Some(format!(
            "verify {} inside the provider runtime and confirm the account session is connected",
            connector.name
        )),
        manifest::ConnectorToolSource::Mcp => Some(format!(
            "run doctor or the target MCP client and confirm {} is callable",
            connector.name
        )),
        manifest::ConnectorToolSource::Native => None,
    }
}

fn connector_detail(
    connector: &manifest::ConnectorDefinition,
    active_connectors: &[String],
) -> Option<String> {
    if !active_connectors.contains(&connector.name) {
        return None;
    }

    match connector.tool_source {
        manifest::ConnectorToolSource::App => Some(
            "app connector auth is owned by the provider runtime and not verified by bootstrap"
                .to_string(),
        ),
        manifest::ConnectorToolSource::Mcp => Some(
            "connector is expected to be available through bootstrap-managed MCP wiring"
                .to_string(),
        ),
        manifest::ConnectorToolSource::Native => None,
    }
}

fn automation_status(
    automation: &manifest::AutomationDefinition,
    active_automations: &[String],
) -> &'static str {
    if active_automations.contains(&automation.name) {
        "rendered"
    } else {
        "inactive"
    }
}

fn automation_scheduler_owner() -> &'static str {
    "runtime-managed"
}

fn automation_registration_status(
    automation: &manifest::AutomationDefinition,
    active_automations: &[String],
) -> &'static str {
    if !active_automations.contains(&automation.name) {
        return "not-requested";
    }

    "not-registered"
}

fn automation_next_step(
    automation: &manifest::AutomationDefinition,
    active_automations: &[String],
) -> Option<String> {
    if !active_automations.contains(&automation.name) {
        return None;
    }

    Some(format!(
        "register {} in the target runtime scheduler if you want recurring execution",
        automation.name
    ))
}

fn automation_detail(
    automation: &manifest::AutomationDefinition,
    active_automations: &[String],
) -> Option<String> {
    if !active_automations.contains(&automation.name) {
        return None;
    }

    Some(
        "automation contract is rendered into the installed runtime state; recurring scheduler registration stays runtime-managed"
            .to_string(),
    )
}

fn record_with(args: &RecordArgs, manifest: &BootstrapManifest) -> Result<()> {
    let record_template = manifest
        .record_templates
        .iter()
        .find(|record| record.record_type == args.record_type.record_type());
    let body = render_record_body(args, record_template)?;
    let local_path = if args.surface.includes_local_docs() {
        let output_dir = resolve_record_output_dir(&args.output_dir)?;
        let path = output_dir.join(record_file_name(args)?);
        if args.dry_run {
            println!("[dry-run] local record: {}", path.display());
        } else {
            fs::create_dir_all(&output_dir)
                .with_context(|| format!("failed to create {}", output_dir.display()))?;
            fs::write(&path, &body)
                .with_context(|| format!("failed to write {}", path.display()))?;
            println!("record written: {}", path.display());
        }
        Some(path)
    } else {
        None
    };

    if args.surface.includes_github_issue() {
        if args.dry_run {
            println!(
                "[dry-run] github issue: title={} repo={}",
                args.title,
                args.github_repo.as_deref().unwrap_or("current")
            );
        } else {
            create_github_issue(args, &body, local_path.as_deref())?;
        }
    }

    if args.dry_run {
        println!("{body}");
    }

    Ok(())
}

fn render_record_body(
    args: &RecordArgs,
    template: Option<&manifest::RecordTemplateDefinition>,
) -> Result<String> {
    let id = format!("rec_{}", timestamp_string()?);
    let updated_at = timestamp_string()?;
    let stage = template
        .map(|record| record.stage.as_str())
        .unwrap_or("manual");
    let template_name = template
        .map(|record| record.name.as_str())
        .unwrap_or("manual-record");
    let description = template
        .map(|record| record.description.as_str())
        .unwrap_or("Manual operating record.");
    let owner = args.owner.as_deref().unwrap_or("");
    let next_action = args.next_action.as_deref().unwrap_or("");

    Ok(format!(
        "# {title}\n\n```yaml\nid: \"{id}\"\ntype: \"{record_type}\"\ntemplate: \"{template_name}\"\nstage: \"{stage}\"\ntitle: \"{title}\"\nstatus: \"{status}\"\nsource: \"llm-bootstrap record\"\nowner: \"{owner}\"\nupdated_at: \"{updated_at}\"\nnext_action: \"{next_action}\"\nlinked_tools:\n  github: \"\"\n  linear: \"\"\n  figma: \"\"\n  docs: \"\"\n  calendar: \"\"\n  crm: \"\"\n  helpdesk: \"\"\n  analytics: \"\"\ncontext:\n  summary: \"\"\n  assumptions: []\ndecision:\n  chosen: \"\"\n  alternatives: []\n  rationale: \"\"\nevidence:\n  links: []\n  notes: []\napprovals:\n  required: false\n  reason: \"\"\n  approver: \"\"\nhandoff:\n  runtime_owner: \"\"\n  external_object_id: \"\"\n  next_step: \"\"\n```\n\n## Description\n\n{description}\n\n## Notes\n\n- Keep this record compact.\n- Link to external source-of-truth systems instead of copying their data.\n- Require approval before customer sends, legal/finance decisions, or external writes.\n",
        title = yaml_string(&args.title),
        id = id,
        record_type = args.record_type.record_type(),
        template_name = yaml_string(template_name),
        stage = yaml_string(stage),
        status = yaml_string(&args.status),
        owner = yaml_string(owner),
        updated_at = updated_at,
        next_action = yaml_string(next_action),
        description = description,
    ))
}

fn resolve_record_output_dir(path: &Path) -> Result<PathBuf> {
    if path.is_absolute() {
        Ok(path.to_path_buf())
    } else {
        Ok(env::current_dir()
            .context("failed to read current directory")?
            .join(path))
    }
}

fn record_file_name(args: &RecordArgs) -> Result<String> {
    let timestamp = timestamp_string()?;
    Ok(format!(
        "{}-{}-{}.md",
        timestamp,
        args.record_type.name(),
        slugify(&args.title)
    ))
}

fn create_github_issue(args: &RecordArgs, body: &str, local_path: Option<&Path>) -> Result<()> {
    if !command_exists("gh") {
        bail!("gh is required for --surface github-issue or both");
    }

    let body_path = match local_path {
        Some(path) => path.to_path_buf(),
        None => {
            let path = env::temp_dir().join(format!(
                "llm-bootstrap-record-{}-{}.md",
                args.record_type.name(),
                timestamp_string()?
            ));
            fs::write(&path, body)
                .with_context(|| format!("failed to write {}", path.display()))?;
            path
        }
    };

    let mut command = ProcessCommand::new("gh");
    command.args(["issue", "create", "--title", &args.title, "--body-file"]);
    command.arg(&body_path);
    if let Some(repo) = &args.github_repo {
        command.args(["--repo", repo]);
    }
    let output = command
        .output()
        .with_context(|| "failed while creating GitHub issue with gh")?;
    if !output.status.success() {
        bail!(
            "gh issue create failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }

    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if stdout.is_empty() {
        println!("github issue created");
    } else {
        println!("github issue created: {stdout}");
    }
    Ok(())
}

fn yaml_string(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

fn slugify(value: &str) -> String {
    let mut slug = String::new();
    let mut previous_dash = false;
    for character in value.chars().flat_map(char::to_lowercase) {
        if character.is_ascii_alphanumeric() {
            slug.push(character);
            previous_dash = false;
        } else if !previous_dash {
            slug.push('-');
            previous_dash = true;
        }
    }
    let slug = slug.trim_matches('-');
    if slug.is_empty() {
        "record".to_string()
    } else {
        slug.to_string()
    }
}

fn provider_root(home: &Path, provider: Provider) -> PathBuf {
    match provider {
        Provider::Codex => home.join(".codex"),
        Provider::Gemini => home.join(".gemini"),
        Provider::Claude => home.join(".claude"),
    }
}

fn provider_managed_paths(
    provider: Provider,
    rtk_enabled: bool,
    distribution_state: &ResolvedDistributionState,
    surfaces: &ProviderSurfaces,
) -> Vec<String> {
    match provider {
        Provider::Codex => codex_managed_paths_for(
            &surfaces.codex,
            distribution_state.enabled(DistributionTarget::CodexPlugin),
            rtk_enabled,
        ),
        Provider::Gemini => gemini_managed_paths_for(
            &surfaces.gemini,
            distribution_state.enabled(DistributionTarget::GeminiExtension),
            rtk_enabled,
        ),
        Provider::Claude => claude_managed_paths_for(
            &surfaces.claude,
            distribution_state.enabled(DistributionTarget::ClaudeSkills),
            rtk_enabled,
        ),
    }
}

fn provider_surfaces(provider: Provider, surfaces: &ProviderSurfaces) -> &[String] {
    match provider {
        Provider::Codex => &surfaces.codex,
        Provider::Gemini => &surfaces.gemini,
        Provider::Claude => &surfaces.claude,
    }
}

fn env_is_set(name: &str) -> bool {
    env_is_set_with(
        name,
        process_env_value,
        launchctl_env_value,
        managed_cli_env_value,
    )
}

fn env_is_set_with<F, G, H>(
    name: &str,
    process_lookup: F,
    launchctl_lookup: G,
    managed_lookup: H,
) -> bool
where
    F: Fn(&str) -> Option<String>,
    G: Fn(&str) -> Option<String>,
    H: Fn(&str) -> Option<String>,
{
    process_lookup(name)
        .or_else(|| launchctl_lookup(name))
        .or_else(|| managed_lookup(name))
        .map(|value| !value.trim().is_empty())
        .unwrap_or(false)
}

fn process_env_value(name: &str) -> Option<String> {
    env::var(name).ok()
}

fn launchctl_env_value(name: &str) -> Option<String> {
    let output = ProcessCommand::new("launchctl")
        .args(["getenv", name])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }

    let value = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if value.is_empty() { None } else { Some(value) }
}

fn managed_cli_env_value(name: &str) -> Option<String> {
    let path = managed_zsh_env_path().ok()?;
    let raw = fs::read_to_string(path).ok()?;
    parse_managed_env_content(&raw, name)
}

fn selected_providers(args: &ProviderArgs, manifest: &BootstrapManifest) -> Vec<Provider> {
    args.providers
        .clone()
        .unwrap_or_else(|| manifest.bootstrap.providers.clone())
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ResolvedPackSelection {
    preset: Option<String>,
    packs: Vec<String>,
}

fn selected_pack_names(
    args: &PackArgs,
    manifest: &BootstrapManifest,
) -> Result<ResolvedPackSelection> {
    let (requested_preset, requested) = if let Some(packs) = &args.packs {
        (None, packs.clone())
    } else if let Some(preset) = &args.preset {
        let preset_definition = manifest
            .presets
            .iter()
            .find(|candidate| candidate.name == *preset)
            .with_context(|| format!("unknown preset: {}", preset))?;
        (
            Some(preset_definition.name.clone()),
            preset_definition.packs.clone(),
        )
    } else {
        let preset_definition = manifest
            .presets
            .iter()
            .find(|candidate| candidate.name == manifest.bootstrap.default_preset)
            .with_context(|| {
                format!(
                    "default preset {} is not declared",
                    manifest.bootstrap.default_preset
                )
            })?;
        (
            Some(preset_definition.name.clone()),
            preset_definition.packs.clone(),
        )
    };
    let mut selected = IndexSet::new();

    for pack in requested {
        if !manifest
            .packs
            .iter()
            .any(|candidate| candidate.name == pack)
        {
            bail!("unknown pack: {}", pack);
        }
        selected.insert(pack);
    }

    Ok(ResolvedPackSelection {
        preset: requested_preset,
        packs: selected.into_iter().collect(),
    })
}

fn selected_harness_names(manifest: &BootstrapManifest, active_packs: &[String]) -> Vec<String> {
    selected_pack_items(manifest, active_packs, |pack| pack.harnesses.as_slice())
}

fn selected_connector_names(manifest: &BootstrapManifest, active_packs: &[String]) -> Vec<String> {
    selected_pack_items(manifest, active_packs, |pack| pack.connectors.as_slice())
}

fn selected_automation_names(manifest: &BootstrapManifest, active_packs: &[String]) -> Vec<String> {
    manifest
        .automations
        .iter()
        .filter(|automation| {
            automation
                .packs
                .iter()
                .all(|pack| active_packs.contains(pack))
        })
        .map(|automation| automation.name.clone())
        .collect()
}

fn selected_record_template_names(
    manifest: &BootstrapManifest,
    active_packs: &[String],
) -> Vec<String> {
    manifest
        .record_templates
        .iter()
        .filter(|record| record.packs.iter().any(|pack| active_packs.contains(pack)))
        .map(|record| record.name.clone())
        .collect()
}

fn pack_app_names(manifest: &BootstrapManifest, pack: &manifest::PackDefinition) -> Vec<String> {
    pack.connectors
        .iter()
        .filter(|name| {
            manifest.connectors.iter().any(|connector| {
                connector.name == **name
                    && connector.tool_source == manifest::ConnectorToolSource::App
            })
        })
        .cloned()
        .collect()
}

fn pack_distribution_targets(pack: &manifest::PackDefinition) -> Vec<DistributionTarget> {
    let mut targets = IndexSet::new();

    if !pack.codex_surfaces.is_empty() {
        targets.insert(DistributionTarget::CodexPlugin);
    }
    if !pack.gemini_surfaces.is_empty() {
        targets.insert(DistributionTarget::GeminiExtension);
    }
    if !pack.claude_surfaces.is_empty() {
        targets.insert(DistributionTarget::ClaudeSkills);
    }

    targets.into_iter().collect()
}

fn selected_distribution_targets(
    manifest: &BootstrapManifest,
    active_packs: &[String],
) -> Vec<DistributionTarget> {
    let codex_surfaces = selected_codex_surfaces(manifest, active_packs);
    let gemini_surfaces = selected_gemini_surfaces(manifest, active_packs);
    let claude_surfaces = selected_claude_surfaces(manifest, active_packs);
    let mut targets = Vec::new();

    if !codex_surfaces.is_empty() {
        targets.push(DistributionTarget::CodexPlugin);
    }
    if !gemini_surfaces.is_empty() {
        targets.push(DistributionTarget::GeminiExtension);
    }
    if !claude_surfaces.is_empty() {
        targets.push(DistributionTarget::ClaudeSkills);
    }

    targets
}

fn effective_distribution_targets(
    manifest: &BootstrapManifest,
    active_packs: &[String],
) -> Vec<DistributionTarget> {
    selected_distribution_targets(manifest, active_packs)
        .into_iter()
        .filter(|target| distribution_target_enabled(manifest, active_packs, *target))
        .collect()
}

fn has_distribution_target(targets: &[DistributionTarget], expected: DistributionTarget) -> bool {
    targets.contains(&expected)
}

fn distribution_target_enabled(
    manifest: &BootstrapManifest,
    active_packs: &[String],
    target: DistributionTarget,
) -> bool {
    match target {
        DistributionTarget::CodexPlugin => {
            layout::provider_surface_enabled(&selected_codex_surfaces(manifest, active_packs))
        }
        DistributionTarget::GeminiExtension => {
            layout::provider_surface_enabled(&selected_gemini_surfaces(manifest, active_packs))
        }
        DistributionTarget::ClaudeSkills => {
            layout::provider_surface_enabled(&selected_claude_surfaces(manifest, active_packs))
        }
    }
}

fn validate_manifest(manifest: &BootstrapManifest) -> Result<()> {
    let mut errors = Vec::new();
    let mut harness_names = IndexSet::new();
    let mut pack_names = IndexSet::new();
    let mut preset_names = IndexSet::new();
    let mut connector_names = IndexSet::new();
    let mut record_template_names = IndexSet::new();

    for harness in &manifest.harnesses {
        if !harness_names.insert(harness.name.clone()) {
            errors.push(format!("duplicate harness: {}", harness.name));
        }
    }

    for pack in &manifest.packs {
        if !pack_names.insert(pack.name.clone()) {
            errors.push(format!("duplicate pack: {}", pack.name));
        }
        if pack.harnesses.is_empty() {
            errors.push(format!("pack {} has no harnesses", pack.name));
        }
        if pack.connectors.is_empty() {
            errors.push(format!("pack {} has no connectors", pack.name));
        }
        if pack.codex_surfaces.is_empty()
            && pack.gemini_surfaces.is_empty()
            && pack.claude_surfaces.is_empty()
        {
            errors.push(format!("pack {} has no provider surfaces", pack.name));
        }
        for harness in &pack.harnesses {
            if !harness_names.contains(harness) {
                errors.push(format!(
                    "pack {} references unknown harness {}",
                    pack.name, harness
                ));
            }
        }
    }

    if !manifest
        .presets
        .iter()
        .any(|preset| preset.name == manifest.bootstrap.default_preset)
    {
        errors.push(format!(
            "default preset {} is not declared",
            manifest.bootstrap.default_preset
        ));
    }

    for preset in &manifest.presets {
        if !preset_names.insert(preset.name.clone()) {
            errors.push(format!("duplicate preset: {}", preset.name));
        }
        if preset.packs.is_empty() {
            errors.push(format!("preset {} has no packs", preset.name));
        }
        for pack in &preset.packs {
            if !pack_names.contains(pack) {
                errors.push(format!(
                    "preset {} references unknown pack {}",
                    preset.name, pack
                ));
            }
        }
    }

    for connector in &manifest.connectors {
        if !connector_names.insert(connector.name.clone()) {
            errors.push(format!("duplicate connector: {}", connector.name));
        }
    }

    for pack in &manifest.packs {
        for connector in &pack.connectors {
            if !connector_names.contains(connector) {
                errors.push(format!(
                    "pack {} references unknown connector {}",
                    pack.name, connector
                ));
            }
        }
        for mcp in &pack.mcp_servers {
            let declared = manifest.mcp.always_on.contains(mcp)
                || manifest
                    .mcp
                    .env_gated
                    .iter()
                    .any(|candidate| candidate.name == *mcp);
            if !declared {
                errors.push(format!(
                    "pack {} references undeclared mcp {}",
                    pack.name,
                    mcp.name()
                ));
            }
        }
    }

    for automation in &manifest.automations {
        if automation.packs.is_empty() {
            errors.push(format!("automation {} has no packs", automation.name));
        }
        if automation.connectors.is_empty() {
            errors.push(format!("automation {} has no connectors", automation.name));
        }
        for pack in &automation.packs {
            if !pack_names.contains(pack) {
                errors.push(format!(
                    "automation {} references unknown pack {}",
                    automation.name, pack
                ));
            }
        }
        for connector in &automation.connectors {
            if !connector_names.contains(connector) {
                errors.push(format!(
                    "automation {} references unknown connector {}",
                    automation.name, connector
                ));
            }
        }
    }

    for record in &manifest.record_templates {
        if !record_template_names.insert(record.name.clone()) {
            errors.push(format!("duplicate record template: {}", record.name));
        }
        if record.packs.is_empty() {
            errors.push(format!("record template {} has no packs", record.name));
        }
        if record.surfaces.is_empty() {
            errors.push(format!("record template {} has no surfaces", record.name));
        }
        for pack in &record.packs {
            if !pack_names.contains(pack) {
                errors.push(format!(
                    "record template {} references unknown pack {}",
                    record.name, pack
                ));
            }
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        bail!("invalid bootstrap manifest:\n- {}", errors.join("\n- "))
    }
}

fn is_rtk_enabled(without_rtk: bool, manifest: &BootstrapManifest) -> bool {
    manifest.external.rtk.enabled && !without_rtk
}

#[derive(Clone)]
struct ResolvedEnvGate {
    name: BaselineMcp,
    env: String,
    enabled: bool,
}

fn resolved_env_gates<F>(manifest: &BootstrapManifest, is_enabled: F) -> Vec<ResolvedEnvGate>
where
    F: Fn(&str) -> bool,
{
    manifest
        .mcp
        .env_gated
        .iter()
        .map(|gated| ResolvedEnvGate {
            name: gated.name,
            env: gated.env.clone(),
            enabled: is_enabled(&gated.env),
        })
        .collect()
}

fn enabled_mcp_from_gates(
    manifest: &BootstrapManifest,
    env_gates: &[ResolvedEnvGate],
    requested: &[BaselineMcp],
) -> Vec<BaselineMcp> {
    let requested_set = if requested.is_empty() {
        all_manifest_mcp(manifest)
    } else {
        requested.to_vec()
    };
    let mut enabled = manifest
        .mcp
        .always_on
        .iter()
        .copied()
        .filter(|mcp| requested_set.contains(mcp))
        .collect::<Vec<_>>();
    enabled.extend(
        env_gates
            .iter()
            .filter(|gated| requested_set.contains(&gated.name) && gated.enabled)
            .map(|gated| gated.name),
    );
    enabled
}

fn selected_pack_mcp_names(
    manifest: &BootstrapManifest,
    active_packs: &[String],
) -> Vec<BaselineMcp> {
    selected_pack_items(manifest, active_packs, |pack| pack.mcp_servers.as_slice())
}

fn selected_codex_surfaces(manifest: &BootstrapManifest, active_packs: &[String]) -> Vec<String> {
    selected_pack_items(manifest, active_packs, |pack| {
        pack.codex_surfaces.as_slice()
    })
}

fn selected_gemini_surfaces(manifest: &BootstrapManifest, active_packs: &[String]) -> Vec<String> {
    selected_pack_items(manifest, active_packs, |pack| {
        pack.gemini_surfaces.as_slice()
    })
}

fn selected_claude_surfaces(manifest: &BootstrapManifest, active_packs: &[String]) -> Vec<String> {
    selected_pack_items(manifest, active_packs, |pack| {
        pack.claude_surfaces.as_slice()
    })
}

fn selected_pack_items<T, F>(
    manifest: &BootstrapManifest,
    active_packs: &[String],
    items: F,
) -> Vec<T>
where
    T: Clone + Eq + Hash,
    F: Fn(&manifest::PackDefinition) -> &[T],
{
    let mut selected = IndexSet::new();

    for pack in &manifest.packs {
        if !active_packs.contains(&pack.name) {
            continue;
        }
        for item in items(pack) {
            selected.insert(item.clone());
        }
    }

    selected.into_iter().collect()
}

fn all_manifest_mcp(manifest: &BootstrapManifest) -> Vec<BaselineMcp> {
    let mut selected = IndexSet::new();
    for mcp in &manifest.mcp.always_on {
        selected.insert(*mcp);
    }
    for gated in &manifest.mcp.env_gated {
        selected.insert(gated.name);
    }
    selected.into_iter().collect()
}

fn prompt_secret_with_dialoguer(theme: &ColorfulTheme, label: &str) -> Result<Option<String>> {
    let value = Password::with_theme(theme)
        .with_prompt(label)
        .allow_empty_password(true)
        .interact()?;
    if value.trim().is_empty() {
        Ok(None)
    } else {
        Ok(Some(value))
    }
}

#[cfg(test)]
fn parse_provider_list(value: &str) -> Result<Vec<Provider>> {
    let mut providers = Vec::new();
    for item in value
        .split(',')
        .map(|item| item.trim())
        .filter(|item| !item.is_empty())
    {
        let provider = match item {
            "codex" => Provider::Codex,
            "gemini" => Provider::Gemini,
            "claude" => Provider::Claude,
            other => anyhow::bail!("unsupported provider: {}", other),
        };
        if !providers.contains(&provider) {
            providers.push(provider);
        }
    }

    if providers.is_empty() {
        anyhow::bail!("at least one provider is required");
    }

    Ok(providers)
}

fn persist_env_keys(
    keys: &[(&str, Option<&str>)],
    persist_gui: bool,
    persist_cli: bool,
) -> Result<()> {
    let persisted = keys
        .iter()
        .filter_map(|(name, value)| value.map(|value| (*name, value)))
        .collect::<Vec<_>>();

    if persist_gui {
        for (name, value) in &persisted {
            persist_launchctl_env_key(name, value)?;
        }
    }

    if persist_cli && !persisted.is_empty() {
        persist_cli_env_keys(&persisted)?;
    }

    Ok(())
}

fn persist_launchctl_env_key(name: &str, value: &str) -> Result<()> {
    if !value.trim().is_empty() {
        let status = ProcessCommand::new("launchctl")
            .args(["setenv", name, value])
            .status()?;
        if !status.success() {
            anyhow::bail!("launchctl setenv failed for {}", name);
        }
    }
    Ok(())
}

fn persist_cli_env_keys(keys: &[(&str, &str)]) -> Result<()> {
    let zshrc_env_path = managed_zsh_env_path()?;
    let zshrc_dir = zshrc_env_path
        .parent()
        .context("managed zsh env file must have parent directory")?;
    fs::create_dir_all(zshrc_dir)
        .with_context(|| format!("failed to create {}", zshrc_dir.display()))?;

    let mut existing = read_managed_env_entries(&zshrc_env_path)?;
    for (name, value) in keys {
        if !value.trim().is_empty() {
            existing.insert((*name).to_string(), (*value).to_string());
        }
    }
    write_managed_env_entries(&zshrc_env_path, &existing)?;
    ensure_zshrc_sources_managed_env()?;
    Ok(())
}

fn managed_zsh_env_path() -> Result<PathBuf> {
    Ok(home_dir()?.join(ZSHRC_ENV_RELATIVE_PATH))
}

fn zshrc_path() -> Result<PathBuf> {
    Ok(home_dir()?.join(".zshrc"))
}

fn read_managed_env_entries(path: &Path) -> Result<BTreeMap<String, String>> {
    if !path.exists() {
        return Ok(BTreeMap::new());
    }
    let raw =
        fs::read_to_string(path).with_context(|| format!("failed to read {}", path.display()))?;
    let mut entries = BTreeMap::new();
    for line in raw.lines() {
        if let Some((name, value)) = parse_managed_env_line(line) {
            entries.insert(name, value);
        }
    }
    Ok(entries)
}

fn write_managed_env_entries(path: &Path, entries: &BTreeMap<String, String>) -> Result<()> {
    let mut body = String::from("# managed by llm-bootstrap\n");
    for (name, value) in entries {
        body.push_str("export ");
        body.push_str(name);
        body.push('=');
        body.push_str(&shell_single_quote(value));
        body.push('\n');
    }
    fs::write(path, body).with_context(|| format!("failed to write {}", path.display()))
}

fn ensure_zshrc_sources_managed_env() -> Result<()> {
    let zshrc = zshrc_path()?;
    let existing = if zshrc.exists() {
        fs::read_to_string(&zshrc).with_context(|| format!("failed to read {}", zshrc.display()))?
    } else {
        String::new()
    };

    if existing.contains("llm-bootstrap-env.zsh") || zshrc_has_zshrc_d_loader(&existing) {
        return Ok(());
    }

    let block = format!(
        "{start}\nif [[ -r \"$HOME/{path}\" ]]; then\n  source \"$HOME/{path}\"\nfi\n{end}\n",
        start = ZSHRC_MARKER_START,
        path = ZSHRC_ENV_RELATIVE_PATH,
        end = ZSHRC_MARKER_END,
    );
    let updated = upsert_managed_block(&existing, &block);
    fs::write(&zshrc, updated).with_context(|| format!("failed to write {}", zshrc.display()))
}

fn zshrc_has_zshrc_d_loader(raw: &str) -> bool {
    raw.contains(".zshrc.d") && raw.contains("*.zsh")
}

fn upsert_managed_block(existing: &str, block: &str) -> String {
    if let Some(start) = existing.find(ZSHRC_MARKER_START)
        && let Some(end_rel) = existing[start..].find(ZSHRC_MARKER_END)
    {
        let end = start + end_rel + ZSHRC_MARKER_END.len();
        let mut updated = String::new();
        updated.push_str(existing[..start].trim_end());
        if !updated.is_empty() {
            updated.push_str("\n\n");
        }
        updated.push_str(block.trim_end());
        updated.push('\n');
        let tail = existing[end..].trim_start_matches('\n');
        if !tail.is_empty() {
            updated.push('\n');
            updated.push_str(tail);
        }
        return updated;
    }

    let mut updated = existing.trim_end().to_string();
    if !updated.is_empty() {
        updated.push_str("\n\n");
    }
    updated.push_str(block.trim_end());
    updated.push('\n');
    updated
}

fn parse_managed_env_content(raw: &str, target: &str) -> Option<String> {
    raw.lines().find_map(|line| {
        let (name, value) = parse_managed_env_line(line)?;
        (name == target).then_some(value)
    })
}

fn parse_managed_env_line(line: &str) -> Option<(String, String)> {
    let line = line.trim();
    let remainder = line.strip_prefix("export ")?;
    let (name, raw_value) = remainder.split_once('=')?;
    let value = parse_shell_single_quoted_value(raw_value.trim())?;
    Some((name.trim().to_string(), value))
}

fn parse_shell_single_quoted_value(raw: &str) -> Option<String> {
    let inner = raw.strip_prefix('\'')?.strip_suffix('\'')?;
    Some(inner.replace("'\\''", "'"))
}

fn shell_single_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}

fn wizard_env_overrides(
    exa_key: &Option<String>,
    context7_key: &Option<String>,
) -> BTreeMap<&'static str, bool> {
    let mut overrides = BTreeMap::new();
    if let Some(value) = exa_key {
        overrides.insert("EXA_API_KEY", !value.trim().is_empty());
    }
    if let Some(value) = context7_key {
        overrides.insert("CONTEXT7_API_KEY", !value.trim().is_empty());
    }
    overrides
}

fn provider_names(providers: &[Provider]) -> String {
    providers
        .iter()
        .map(|provider| provider.name())
        .collect::<Vec<_>>()
        .join(",")
}

fn resolve_plan(manifest: &BootstrapManifest, pack_args: &PackArgs) -> Result<ResolvedPlan> {
    let pack_selection = selected_pack_names(pack_args, manifest)?;
    let env_gates = resolved_env_gates(manifest, env_is_set);
    Ok(build_resolved_plan(manifest, pack_selection, env_gates))
}

fn build_resolved_plan(
    manifest: &BootstrapManifest,
    pack_selection: ResolvedPackSelection,
    env_gates: Vec<ResolvedEnvGate>,
) -> ResolvedPlan {
    let selection = ActiveSelection {
        preset: pack_selection.preset,
        harnesses: selected_harness_names(manifest, &pack_selection.packs),
        packs: pack_selection.packs,
    };
    let requested_mcp = selected_pack_mcp_names(manifest, &selection.packs);
    let enabled_mcp = enabled_mcp_from_gates(manifest, &env_gates, &requested_mcp);
    let distribution_state = ResolvedDistributionState::resolve(manifest, &selection.packs);
    let surfaces = ProviderSurfaces {
        codex: selected_codex_surfaces(manifest, &selection.packs),
        gemini: selected_gemini_surfaces(manifest, &selection.packs),
        claude: selected_claude_surfaces(manifest, &selection.packs),
    };

    ResolvedPlan {
        selection,
        env_gates,
        requested_mcp,
        enabled_mcp,
        distribution_state,
        surfaces,
    }
}

#[cfg(test)]
mod tests {
    use crate::cli::ApplyMode;
    use crate::fs_ops::render_tokens;
    use crate::json_ops::{
        cleanup_extension_enablement, merge_json, preserved_gemini_runtime_state,
        prune_rtk_claude_hooks, prune_rtk_gemini_hooks, remove_baseline_mcp_servers,
    };
    use crate::manifest::{
        AutomationCadence, AutomationDefinition, BaselineMcp, BootstrapManifest, BootstrapSection,
        ConnectorAccess, ConnectorApproval, ConnectorCategory, ConnectorDefinition,
        ConnectorToolSource, DistributionTarget, EnvGatedMcp, ExternalSection, HarnessCategory,
        HarnessDefinition, McpSection, PackDefinition, PackLane, PackScope, PresetDefinition,
        RecordTemplateDefinition, RtkSection,
    };
    use crate::providers::{claude, codex, gemini};
    use serde_json::json;
    use std::{
        fs,
        path::{Path, PathBuf},
        sync::atomic::{AtomicU64, Ordering},
    };

    static TEMP_HOME_COUNTER: AtomicU64 = AtomicU64::new(0);

    fn test_manifest() -> BootstrapManifest {
        BootstrapManifest {
            bootstrap: BootstrapSection {
                providers: vec![super::Provider::Codex, super::Provider::Gemini],
                default_mode: ApplyMode::Merge,
                default_preset: "normal".to_string(),
            },
            external: ExternalSection {
                rtk: RtkSection { enabled: true },
            },
            mcp: McpSection {
                always_on: vec![BaselineMcp::ChromeDevtools],
                env_gated: vec![
                    EnvGatedMcp {
                        name: BaselineMcp::Context7,
                        env: "CONTEXT7_API_KEY".to_string(),
                    },
                    EnvGatedMcp {
                        name: BaselineMcp::Exa,
                        env: "EXA_API_KEY".to_string(),
                    },
                ],
            },
            harnesses: vec![
                HarnessDefinition {
                    name: "ralph-loop".to_string(),
                    category: HarnessCategory::Core,
                    default_enabled: true,
                    description: "Shared control loop for plan, execute, review, and verify."
                        .to_string(),
                },
                HarnessDefinition {
                    name: "delivery".to_string(),
                    category: HarnessCategory::Development,
                    default_enabled: true,
                    description: "Software delivery harness from planning to ship.".to_string(),
                },
                HarnessDefinition {
                    name: "ralph-plan".to_string(),
                    category: HarnessCategory::Core,
                    default_enabled: true,
                    description: "Plan contract with owners, evidence, and next slice.".to_string(),
                },
                HarnessDefinition {
                    name: "review-gate".to_string(),
                    category: HarnessCategory::Quality,
                    default_enabled: true,
                    description: "Review, QA, and verification gate.".to_string(),
                },
                HarnessDefinition {
                    name: "incident".to_string(),
                    category: HarnessCategory::Quality,
                    default_enabled: true,
                    description: "Incident and regression response harness.".to_string(),
                },
                HarnessDefinition {
                    name: "founder-loop".to_string(),
                    category: HarnessCategory::Company,
                    default_enabled: false,
                    description: "Founder and product direction review loop.".to_string(),
                },
                HarnessDefinition {
                    name: "operating-review".to_string(),
                    category: HarnessCategory::Company,
                    default_enabled: false,
                    description: "Company operating review loop.".to_string(),
                },
            ],
            packs: vec![
                PackDefinition {
                    name: "delivery-pack".to_string(),
                    scope: PackScope::Development,
                    lane: PackLane::Core,
                    harnesses: vec![
                        "ralph-loop".to_string(),
                        "ralph-plan".to_string(),
                        "delivery".to_string(),
                        "review-gate".to_string(),
                    ],
                    mcp_servers: vec![BaselineMcp::ChromeDevtools, BaselineMcp::Context7],
                    connectors: vec!["github".to_string(), "linear".to_string()],
                    codex_surfaces: vec!["llm-dev-kit".to_string(), "delivery-skills".to_string()],
                    gemini_surfaces: vec![
                        "llm-bootstrap-dev".to_string(),
                        "delivery-commands".to_string(),
                    ],
                    claude_surfaces: vec![
                        "claude-skills".to_string(),
                        "delivery-skills".to_string(),
                    ],
                    description: "Core software delivery pack.".to_string(),
                },
                PackDefinition {
                    name: "incident-pack".to_string(),
                    scope: PackScope::Development,
                    lane: PackLane::Core,
                    harnesses: vec!["incident".to_string(), "review-gate".to_string()],
                    mcp_servers: vec![BaselineMcp::ChromeDevtools, BaselineMcp::Context7],
                    connectors: vec!["github".to_string(), "linear".to_string()],
                    codex_surfaces: vec!["llm-dev-kit".to_string(), "incident-skills".to_string()],
                    gemini_surfaces: vec![
                        "llm-bootstrap-dev".to_string(),
                        "incident-commands".to_string(),
                    ],
                    claude_surfaces: vec![
                        "claude-skills".to_string(),
                        "incident-skills".to_string(),
                    ],
                    description: "Incident response pack.".to_string(),
                },
                PackDefinition {
                    name: "founder-pack".to_string(),
                    scope: PackScope::Company,
                    lane: PackLane::Optional,
                    harnesses: vec![
                        "ralph-plan".to_string(),
                        "founder-loop".to_string(),
                        "operating-review".to_string(),
                    ],
                    mcp_servers: vec![BaselineMcp::Exa],
                    connectors: vec![
                        "linear".to_string(),
                        "gmail".to_string(),
                        "calendar".to_string(),
                        "drive".to_string(),
                        "figma".to_string(),
                        "stitch".to_string(),
                    ],
                    codex_surfaces: vec!["llm-dev-kit".to_string(), "company-skills".to_string()],
                    gemini_surfaces: vec![
                        "llm-bootstrap-dev".to_string(),
                        "company-commands".to_string(),
                    ],
                    claude_surfaces: vec![
                        "claude-skills".to_string(),
                        "company-skills".to_string(),
                    ],
                    description: "Founder and company operating pack.".to_string(),
                },
                PackDefinition {
                    name: "ops-pack".to_string(),
                    scope: PackScope::Company,
                    lane: PackLane::Optional,
                    harnesses: vec!["ralph-plan".to_string(), "operating-review".to_string()],
                    mcp_servers: vec![BaselineMcp::Exa],
                    connectors: vec![
                        "linear".to_string(),
                        "gmail".to_string(),
                        "calendar".to_string(),
                        "drive".to_string(),
                    ],
                    codex_surfaces: vec!["llm-dev-kit".to_string(), "company-skills".to_string()],
                    gemini_surfaces: vec![
                        "llm-bootstrap-dev".to_string(),
                        "company-commands".to_string(),
                    ],
                    claude_surfaces: vec![
                        "claude-skills".to_string(),
                        "company-skills".to_string(),
                    ],
                    description: "Operating review pack.".to_string(),
                },
            ],
            presets: vec![
                PresetDefinition {
                    name: "light".to_string(),
                    packs: vec!["delivery-pack".to_string()],
                    description: "Lean development baseline.".to_string(),
                },
                PresetDefinition {
                    name: "normal".to_string(),
                    packs: vec!["delivery-pack".to_string(), "incident-pack".to_string()],
                    description: "Default development baseline.".to_string(),
                },
                PresetDefinition {
                    name: "full".to_string(),
                    packs: vec![
                        "delivery-pack".to_string(),
                        "incident-pack".to_string(),
                        "founder-pack".to_string(),
                        "ops-pack".to_string(),
                    ],
                    description: "Development and company packs.".to_string(),
                },
                PresetDefinition {
                    name: "company".to_string(),
                    packs: vec!["founder-pack".to_string(), "ops-pack".to_string()],
                    description: "Company operating packs.".to_string(),
                },
            ],
            connectors: vec![
                ConnectorDefinition {
                    name: "github".to_string(),
                    category: ConnectorCategory::Delivery,
                    tool_source: ConnectorToolSource::App,
                    access: ConnectorAccess::ReadWrite,
                    approval: ConnectorApproval::OnWrite,
                    automation_allowed: true,
                    description: "Delivery context.".to_string(),
                },
                ConnectorDefinition {
                    name: "linear".to_string(),
                    category: ConnectorCategory::Delivery,
                    tool_source: ConnectorToolSource::App,
                    access: ConnectorAccess::ReadWrite,
                    approval: ConnectorApproval::OnWrite,
                    automation_allowed: true,
                    description: "Project and roadmap context.".to_string(),
                },
                ConnectorDefinition {
                    name: "gmail".to_string(),
                    category: ConnectorCategory::Communication,
                    tool_source: ConnectorToolSource::App,
                    access: ConnectorAccess::ReadOnly,
                    approval: ConnectorApproval::OnWrite,
                    automation_allowed: true,
                    description: "Inbox context.".to_string(),
                },
                ConnectorDefinition {
                    name: "calendar".to_string(),
                    category: ConnectorCategory::Communication,
                    tool_source: ConnectorToolSource::App,
                    access: ConnectorAccess::ReadOnly,
                    approval: ConnectorApproval::OnWrite,
                    automation_allowed: true,
                    description: "Calendar context.".to_string(),
                },
                ConnectorDefinition {
                    name: "drive".to_string(),
                    category: ConnectorCategory::Knowledge,
                    tool_source: ConnectorToolSource::App,
                    access: ConnectorAccess::ReadOnly,
                    approval: ConnectorApproval::OnWrite,
                    automation_allowed: true,
                    description: "Document context.".to_string(),
                },
                ConnectorDefinition {
                    name: "figma".to_string(),
                    category: ConnectorCategory::Design,
                    tool_source: ConnectorToolSource::App,
                    access: ConnectorAccess::ReadOnly,
                    approval: ConnectorApproval::OnWrite,
                    automation_allowed: false,
                    description: "Design file context.".to_string(),
                },
                ConnectorDefinition {
                    name: "stitch".to_string(),
                    category: ConnectorCategory::Design,
                    tool_source: ConnectorToolSource::App,
                    access: ConnectorAccess::ReadOnly,
                    approval: ConnectorApproval::OnWrite,
                    automation_allowed: false,
                    description: "Design exploration context.".to_string(),
                },
            ],
            automations: vec![
                AutomationDefinition {
                    name: "daily-founder-brief".to_string(),
                    cadence: AutomationCadence::Daily,
                    packs: vec!["founder-pack".to_string()],
                    connectors: vec![
                        "linear".to_string(),
                        "gmail".to_string(),
                        "calendar".to_string(),
                        "drive".to_string(),
                    ],
                    artifact: "Founder Brief".to_string(),
                    description: "Daily founder brief.".to_string(),
                },
                AutomationDefinition {
                    name: "weekly-operating-review".to_string(),
                    cadence: AutomationCadence::Weekly,
                    packs: vec!["founder-pack".to_string(), "ops-pack".to_string()],
                    connectors: vec![
                        "linear".to_string(),
                        "gmail".to_string(),
                        "calendar".to_string(),
                        "drive".to_string(),
                    ],
                    artifact: "Operating Review".to_string(),
                    description: "Weekly operating review.".to_string(),
                },
            ],
            record_templates: vec![
                RecordTemplateDefinition {
                    name: "project-record".to_string(),
                    record_type: "ProjectRecord".to_string(),
                    stage: "build".to_string(),
                    packs: vec!["delivery-pack".to_string(), "founder-pack".to_string()],
                    surfaces: vec!["local-docs".to_string(), "github-issues".to_string()],
                    description: "Project brief and execution record.".to_string(),
                },
                RecordTemplateDefinition {
                    name: "ops-record".to_string(),
                    record_type: "OpsRecord".to_string(),
                    stage: "learn".to_string(),
                    packs: vec!["ops-pack".to_string()],
                    surfaces: vec!["local-docs".to_string(), "github-issues".to_string()],
                    description: "Weekly operating record.".to_string(),
                },
            ],
        }
    }

    fn test_active_harnesses() -> Vec<String> {
        super::selected_harness_names(&test_manifest(), &["delivery-pack".to_string()])
    }

    fn test_active_surfaces() -> Vec<String> {
        vec![
            "delivery-skills".to_string(),
            "delivery-commands".to_string(),
        ]
    }

    #[test]
    fn doctor_catalog_report_exposes_harnesses_and_packs() {
        let manifest = test_manifest();
        let active_packs = vec!["delivery-pack".to_string()];
        let active_harnesses = test_active_harnesses();
        let selection = super::ActiveSelection {
            preset: Some("normal".to_string()),
            packs: active_packs.clone(),
            harnesses: active_harnesses.clone(),
        };
        let requested_mcp = super::selected_pack_mcp_names(&manifest, &active_packs);
        let distribution_state =
            super::ResolvedDistributionState::resolve(&manifest, &active_packs);
        let report = super::doctor_catalog_report(
            &manifest,
            &selection,
            &requested_mcp,
            &requested_mcp,
            &distribution_state,
        );

        assert_eq!(report.harnesses.len(), 7);
        assert_eq!(report.packs.len(), 4);
        assert_eq!(report.default_preset, "normal".to_string());
        assert_eq!(report.active_preset, Some("normal".to_string()));
        assert_eq!(report.active_packs, vec!["delivery-pack".to_string()]);
        assert_eq!(
            report.requested_mcp_servers,
            vec!["chrome-devtools".to_string(), "context7".to_string()]
        );
        assert_eq!(
            report.active_harnesses,
            vec![
                "ralph-loop".to_string(),
                "ralph-plan".to_string(),
                "delivery".to_string(),
                "review-gate".to_string()
            ]
        );
        assert_eq!(
            report.active_connectors,
            vec!["github".to_string(), "linear".to_string()]
        );
        assert!(report.active_automations.is_empty());
        assert_eq!(
            report.requested_distribution_targets,
            vec![
                "codex-plugin".to_string(),
                "gemini-extension".to_string(),
                "claude-skills".to_string()
            ]
        );
        assert_eq!(
            report.active_distribution_targets,
            vec![
                "codex-plugin".to_string(),
                "gemini-extension".to_string(),
                "claude-skills".to_string()
            ]
        );
        assert_eq!(report.harnesses[0].name, "ralph-loop");
        assert_eq!(report.harnesses[0].category, "core");
        assert!(report.harnesses[0].default_enabled);
        assert_eq!(report.packs[0].name, "delivery-pack");
        assert_eq!(
            report.packs[0].connector_apps,
            vec!["github".to_string(), "linear".to_string()]
        );
        assert_eq!(
            report.packs[0].mcp_servers,
            vec!["chrome-devtools".to_string(), "context7".to_string()]
        );
        assert_eq!(report.packs[0].scope, "development");
        assert_eq!(report.packs[0].lane, "core");
        assert!(report.packs[0].selected);
        assert_eq!(report.connectors.len(), 7);
        assert_eq!(report.automations.len(), 2);
        assert_eq!(report.runtime_handoff.active_app_connector_count, 2);
        assert_eq!(report.runtime_handoff.pending_app_verification_count, 2);
        assert_eq!(report.runtime_handoff.active_automation_count, 0);
        assert_eq!(
            report.runtime_handoff.pending_scheduler_registration_count,
            0
        );
        assert_eq!(
            report.runtime_handoff.connector_queue,
            vec!["github".to_string(), "linear".to_string()]
        );
        assert!(report.runtime_handoff.automation_queue.is_empty());
        assert_eq!(
            report.active_record_templates,
            vec!["project-record".to_string()]
        );
        assert_eq!(report.record_templates.len(), 2);
        assert!(report.record_templates[0].active);
        assert_eq!(report.record_templates[0].runtime_owner, "external-tools");
        assert_eq!(
            report.record_readiness.record_system,
            "local-docs+github-issues"
        );
        assert_eq!(
            report.record_readiness.active_templates,
            vec!["project-record".to_string()]
        );
        assert_eq!(
            report.record_readiness.missing_handoffs,
            vec!["linear".to_string()]
        );
        assert_eq!(
            report.runtime_handoff.next_steps,
            vec![
                "open each provider runtime and verify active app connectors with one real read action"
                    .to_string()
            ]
        );
        let github = report
            .connectors
            .iter()
            .find(|connector| connector.name == "github")
            .unwrap();
        assert_eq!(github.health, "runtime-managed");
        assert_eq!(github.auth_state, "external-runtime");
        assert_eq!(github.runtime_owner, "provider-runtime");
        assert_eq!(github.verification_mode, "manual-runtime-check");
        assert_eq!(github.connection_status, "not-verified");
        assert_eq!(
            github.next_step.as_deref(),
            Some(
                "verify github inside the provider runtime and confirm the account session is connected"
            )
        );
        let gmail = report
            .connectors
            .iter()
            .find(|connector| connector.name == "gmail")
            .unwrap();
        assert_eq!(gmail.connection_status, "not-requested");
        assert!(gmail.next_step.is_none());
        let founder_brief = report
            .automations
            .iter()
            .find(|automation| automation.name == "daily-founder-brief")
            .unwrap();
        assert_eq!(founder_brief.status, "inactive");
        assert_eq!(founder_brief.scheduler_owner, "runtime-managed");
        assert_eq!(founder_brief.registration_status, "not-requested");
        assert!(founder_brief.next_step.is_none());
    }

    #[test]
    fn installed_state_mismatch_includes_managed_paths() {
        let requested = crate::state::InstalledState {
            active_preset: Some("normal".to_string()),
            active_packs: vec!["delivery-pack".to_string(), "incident-pack".to_string()],
            active_harnesses: vec![
                "ralph-loop".to_string(),
                "ralph-plan".to_string(),
                "delivery".to_string(),
                "review-gate".to_string(),
            ],
            active_connectors: vec!["github".to_string(), "linear".to_string()],
            active_automations: Vec::new(),
            active_record_templates: vec!["project-record".to_string()],
            active_surfaces: vec!["llm-dev-kit".to_string(), "delivery-skills".to_string()],
            managed_paths: vec!["config.toml".to_string(), "AGENTS.md".to_string()],
        };

        let requested_state = crate::state::RequestedState {
            active_preset: Some("normal"),
            active_packs: &requested.active_packs,
            active_harnesses: &requested.active_harnesses,
            active_connectors: &requested.active_connectors,
            active_automations: &requested.active_automations,
            active_record_templates: &requested.active_record_templates,
            active_surfaces: &requested.active_surfaces,
            managed_paths: &["config.toml".to_string()],
        };

        assert!(requested.mismatch(&requested_state));
    }

    #[test]
    fn doctor_catalog_report_separates_requested_and_effective_targets() {
        let manifest = test_manifest();
        let active_packs = vec!["founder-pack".to_string(), "ops-pack".to_string()];
        let active_harnesses = super::selected_harness_names(&manifest, &active_packs);
        let selection = super::ActiveSelection {
            preset: None,
            packs: active_packs.clone(),
            harnesses: active_harnesses.clone(),
        };
        let requested_mcp = super::selected_pack_mcp_names(&manifest, &active_packs);
        let active_mcp = super::enabled_mcp_from_gates(
            &manifest,
            &[super::ResolvedEnvGate {
                name: BaselineMcp::Exa,
                env: "EXA_API_KEY".to_string(),
                enabled: true,
            }],
            &requested_mcp,
        );
        let distribution_state =
            super::ResolvedDistributionState::resolve(&manifest, &active_packs);
        let report = super::doctor_catalog_report(
            &manifest,
            &selection,
            &requested_mcp,
            &active_mcp,
            &distribution_state,
        );

        assert_eq!(
            report.requested_distribution_targets,
            vec![
                "codex-plugin".to_string(),
                "gemini-extension".to_string(),
                "claude-skills".to_string()
            ]
        );
        assert_eq!(report.requested_mcp_servers, vec!["exa".to_string()]);
        assert_eq!(
            report.active_distribution_targets,
            vec![
                "codex-plugin".to_string(),
                "gemini-extension".to_string(),
                "claude-skills".to_string()
            ]
        );
        assert_eq!(
            report.active_connectors,
            vec![
                "linear".to_string(),
                "gmail".to_string(),
                "calendar".to_string(),
                "drive".to_string(),
                "figma".to_string(),
                "stitch".to_string()
            ]
        );
        assert_eq!(
            report.active_automations,
            vec![
                "daily-founder-brief".to_string(),
                "weekly-operating-review".to_string()
            ]
        );
        assert_eq!(
            report.active_record_templates,
            vec!["project-record".to_string(), "ops-record".to_string()]
        );
        assert_eq!(
            report.record_readiness.missing_handoffs,
            vec![
                "linear".to_string(),
                "gmail".to_string(),
                "calendar".to_string(),
                "drive".to_string(),
                "figma".to_string(),
                "stitch".to_string()
            ]
        );
        assert_eq!(report.runtime_handoff.active_app_connector_count, 6);
        assert_eq!(report.runtime_handoff.pending_app_verification_count, 6);
        assert_eq!(report.runtime_handoff.active_automation_count, 2);
        assert_eq!(
            report.runtime_handoff.pending_scheduler_registration_count,
            2
        );
        assert_eq!(
            report.runtime_handoff.connector_queue,
            vec![
                "linear".to_string(),
                "gmail".to_string(),
                "calendar".to_string(),
                "drive".to_string(),
                "figma".to_string(),
                "stitch".to_string()
            ]
        );
        assert_eq!(
            report.runtime_handoff.automation_queue,
            vec![
                "daily-founder-brief".to_string(),
                "weekly-operating-review".to_string()
            ]
        );
        assert_eq!(
            report.runtime_handoff.next_steps,
            vec![
                "open each provider runtime and verify active app connectors with one real read action"
                    .to_string(),
                "register active automation contracts in the target runtime scheduler before expecting recurring runs"
                    .to_string()
            ]
        );
        let drive = report
            .connectors
            .iter()
            .find(|connector| connector.name == "drive")
            .unwrap();
        assert_eq!(drive.health, "runtime-managed");
        assert_eq!(drive.runtime_owner, "provider-runtime");
        assert_eq!(drive.verification_mode, "manual-runtime-check");
        assert_eq!(drive.connection_status, "not-verified");
        assert_eq!(
            drive.next_step.as_deref(),
            Some(
                "verify drive inside the provider runtime and confirm the account session is connected"
            )
        );
        let operating_review = report
            .automations
            .iter()
            .find(|automation| automation.name == "weekly-operating-review")
            .unwrap();
        assert_eq!(operating_review.status, "rendered");
        assert_eq!(operating_review.scheduler_owner, "runtime-managed");
        assert_eq!(operating_review.registration_status, "not-registered");
        assert_eq!(
            operating_review.next_step.as_deref(),
            Some(
                "register weekly-operating-review in the target runtime scheduler if you want recurring execution"
            )
        );
    }

    #[test]
    fn bootstrap_toml_parses_harnesses_and_packs() {
        let path = crate::runtime::repo_root().join("bootstrap.toml");
        let raw = fs::read_to_string(path).unwrap();
        let manifest: BootstrapManifest = toml::from_str(&raw).unwrap();

        assert!(manifest.bootstrap.default_preset == "normal");
        assert!(
            manifest
                .harnesses
                .iter()
                .any(|harness| harness.name == "ralph-plan" && harness.default_enabled)
        );
        assert!(manifest.packs.iter().any(|pack| {
            pack.name == "delivery-pack"
                && pack.scope == PackScope::Development
                && pack.connectors.contains(&"linear".to_string())
                && pack.mcp_servers.contains(&BaselineMcp::Context7)
                && pack.codex_surfaces.contains(&"delivery-skills".to_string())
        }));
        assert!(
            manifest
                .packs
                .iter()
                .any(|pack| pack.name == "founder-pack" && pack.scope == PackScope::Company)
        );
        assert!(
            manifest
                .presets
                .iter()
                .any(|preset| preset.name == "full"
                    && preset.packs.contains(&"ops-pack".to_string()))
        );
        assert!(
            manifest
                .connectors
                .iter()
                .any(|connector| connector.name == "gmail")
        );
        assert!(
            manifest
                .connectors
                .iter()
                .any(|connector| connector.name == "linear")
        );
        assert!(
            manifest
                .connectors
                .iter()
                .any(|connector| connector.name == "figma")
        );
        assert!(
            manifest
                .automations
                .iter()
                .any(|automation| automation.name == "weekly-operating-review")
        );
        assert!(manifest.record_templates.iter().any(|record| {
            record.name == "ops-record"
                && record.record_type == "OpsRecord"
                && record.surfaces.contains(&"github-issues".to_string())
        }));
    }

    #[test]
    fn codex_bundle_qa_browser_skill_has_frontmatter() {
        let path = crate::runtime::repo_root()
            .join("bundles/full/plugins/llm-dev-kit/skills/qa-browser/SKILL.md");
        let raw = fs::read_to_string(path).unwrap();
        assert!(raw.starts_with("---\n"));
    }

    fn temp_home() -> PathBuf {
        let counter = TEMP_HOME_COUNTER.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir()
            .join(format!(
                "llm-bootstrap-test-{}-{}-{}",
                std::process::id(),
                std::thread::current().name().unwrap_or("anon"),
                counter
            ))
            .join(format!(
                "{}",
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_nanos()
            ));
        fs::create_dir_all(&path).unwrap();
        path
    }

    #[test]
    fn home_legacy_cleanup_removes_old_omx_omc_omg_without_touching_history() {
        let home = temp_home();
        fs::create_dir_all(home.join(".omx/cache")).unwrap();
        fs::write(home.join(".omx/cache/state.json"), "{}").unwrap();
        fs::create_dir_all(home.join(".omg")).unwrap();
        fs::write(home.join(".omg/config.json"), "{}").unwrap();
        fs::create_dir_all(home.join(".config/omc")).unwrap();
        fs::write(home.join(".config/omc/state.json"), "{}").unwrap();
        fs::create_dir_all(home.join(".local/bin")).unwrap();
        fs::write(home.join(".local/bin/oh-my-opencode"), "#!/bin/sh\n").unwrap();
        fs::create_dir_all(home.join(".codex/sessions")).unwrap();
        fs::write(home.join(".codex/sessions/session.jsonl"), "keep").unwrap();

        super::cleanup_home_legacy_artifacts(&home).unwrap();

        assert!(!home.join(".omx").exists());
        assert!(!home.join(".omg").exists());
        assert!(!home.join(".config/omc").exists());
        assert!(!home.join(".local/bin/oh-my-opencode").exists());
        assert_eq!(
            fs::read_to_string(home.join(".codex/sessions/session.jsonl")).unwrap(),
            "keep"
        );
        let backup_root = fs::read_dir(home.join(".llm-bootstrap-legacy-backups"))
            .unwrap()
            .next()
            .unwrap()
            .unwrap()
            .path();
        assert!(backup_root.join(".omx/cache/state.json").exists());
        assert!(backup_root.join(".omg/config.json").exists());
        assert!(backup_root.join(".config/omc/state.json").exists());
        assert!(backup_root.join(".local/bin/oh-my-opencode").exists());

        fs::remove_dir_all(home).unwrap();
    }

    #[test]
    fn legacy_env_cleanup_removes_oh_my_keys_from_managed_cli_env_only() {
        let home = temp_home();
        let env_path = home.join(".zshrc.d/llm-bootstrap-env.zsh");
        fs::create_dir_all(env_path.parent().unwrap()).unwrap();
        fs::write(
            &env_path,
            "# managed by llm-bootstrap\nexport EXA_API_KEY='exa-key'\nexport OMX_API_KEY='old-omx'\nexport OH_MY_OPENCODE_API_KEY='old-oh-my'\n",
        )
        .unwrap();

        let removed = super::remove_legacy_managed_cli_env(&env_path).unwrap();

        assert_eq!(
            removed,
            vec![
                "OH_MY_OPENCODE_API_KEY".to_string(),
                "OMX_API_KEY".to_string()
            ]
        );
        let after = fs::read_to_string(&env_path).unwrap();
        assert!(after.contains("EXA_API_KEY"));
        assert!(!after.contains("OMX_API_KEY"));
        assert!(!after.contains("OH_MY_OPENCODE_API_KEY"));

        fs::remove_dir_all(home).unwrap();
    }

    #[test]
    fn merge_json_overwrites_scalars_and_keeps_unknown_keys() {
        let mut target = json!({
            "general": {
                "existing": true,
                "nested": {
                    "keep": "yes"
                }
            },
            "selectedAuthType": "oauth-personal"
        });

        merge_json(
            &mut target,
            json!({
                "general": {
                    "defaultApprovalMode": "plan",
                    "nested": {
                        "replace": "value"
                    }
                }
            }),
        );

        assert_eq!(target["general"]["existing"], json!(true));
        assert_eq!(target["general"]["defaultApprovalMode"], json!("plan"));
        assert_eq!(target["general"]["nested"]["keep"], json!("yes"));
        assert_eq!(target["general"]["nested"]["replace"], json!("value"));
        assert_eq!(target["selectedAuthType"], json!("oauth-personal"));
    }

    #[test]
    fn render_tokens_replaces_provider_paths() {
        let rendered = render_tokens(
            "__HOME__ __CODEX_HOME__ __GEMINI_HOME__",
            Path::new("/tmp/home"),
        );
        assert_eq!(rendered, "/tmp/home /tmp/home/.codex /tmp/home/.gemini");
    }

    #[test]
    fn codex_mcp_blocks_include_unified_baseline() {
        let enabled = vec![BaselineMcp::ChromeDevtools];
        let root = temp_home().join(".codex");
        fs::create_dir_all(&root).unwrap();
        let blocks =
            codex::mcp_blocks(Path::new("/tmp/home"), &root, &enabled, ApplyMode::Merge).unwrap();
        assert!(blocks.contains("chrome-devtools-mcp.sh"));
        assert!(!blocks.contains("context7-mcp.sh"));
        assert!(!blocks.contains("exa-mcp.sh"));
        assert!(!blocks.contains("playwright-mcp.sh"));
        assert!(!blocks.contains("github-mcp.sh"));
        fs::remove_dir_all(root.parent().unwrap()).unwrap();
    }

    #[test]
    fn codex_plugin_blocks_follow_distribution_target_selection() {
        assert!(codex::plugin_blocks(true).contains("llm-dev-kit@llm-bootstrap"));
        assert!(codex::plugin_blocks(false).is_empty());
    }

    #[test]
    fn apply_mode_names_match_cli_values() {
        assert_eq!(ApplyMode::Merge.name(), "merge");
        assert_eq!(ApplyMode::Replace.name(), "replace");
    }

    #[test]
    fn default_command_is_wizard() {
        match super::default_command() {
            super::Command::Wizard(_) => {}
            _ => panic!("default command must be wizard"),
        }
    }

    #[test]
    fn parse_provider_list_accepts_unique_values() {
        let providers = super::parse_provider_list("codex, gemini, codex").unwrap();
        assert_eq!(
            providers,
            vec![super::Provider::Codex, super::Provider::Gemini]
        );
    }

    #[test]
    fn selected_pack_names_use_manifest_defaults() {
        let selection = super::selected_pack_names(
            &super::PackArgs {
                preset: None,
                packs: None,
            },
            &test_manifest(),
        )
        .unwrap();
        assert_eq!(selection.preset, Some("normal".to_string()));
        assert_eq!(
            selection.packs,
            vec!["delivery-pack".to_string(), "incident-pack".to_string()]
        );
    }

    #[test]
    fn selected_pack_names_reject_unknown_values() {
        let err = super::selected_pack_names(
            &super::PackArgs {
                preset: None,
                packs: Some(vec!["missing-pack".to_string()]),
            },
            &test_manifest(),
        )
        .unwrap_err();
        assert!(err.to_string().contains("unknown pack"));
    }

    #[test]
    fn selected_pack_names_resolve_named_preset() {
        let selection = super::selected_pack_names(
            &super::PackArgs {
                preset: Some("full".to_string()),
                packs: None,
            },
            &test_manifest(),
        )
        .unwrap();
        assert_eq!(selection.preset, Some("full".to_string()));
        assert_eq!(
            selection.packs,
            vec![
                "delivery-pack".to_string(),
                "incident-pack".to_string(),
                "founder-pack".to_string(),
                "ops-pack".to_string()
            ]
        );
    }

    #[test]
    fn selected_automation_names_require_all_declared_packs() {
        let founder_only =
            super::selected_automation_names(&test_manifest(), &["founder-pack".to_string()]);
        let founder_and_ops = super::selected_automation_names(
            &test_manifest(),
            &["founder-pack".to_string(), "ops-pack".to_string()],
        );

        assert_eq!(founder_only, vec!["daily-founder-brief".to_string()]);
        assert_eq!(
            founder_and_ops,
            vec![
                "daily-founder-brief".to_string(),
                "weekly-operating-review".to_string()
            ]
        );
    }

    #[test]
    fn selected_distribution_targets_are_deduplicated_from_active_packs() {
        let targets = super::selected_distribution_targets(
            &test_manifest(),
            &["delivery-pack".to_string(), "founder-pack".to_string()],
        );
        assert_eq!(
            targets,
            vec![
                DistributionTarget::CodexPlugin,
                DistributionTarget::GeminiExtension,
                DistributionTarget::ClaudeSkills
            ]
        );
    }

    #[test]
    fn selected_pack_mcp_names_follow_active_packs() {
        let delivery =
            super::selected_pack_mcp_names(&test_manifest(), &["delivery-pack".to_string()]);
        let founder =
            super::selected_pack_mcp_names(&test_manifest(), &["founder-pack".to_string()]);

        assert_eq!(
            delivery,
            vec![BaselineMcp::ChromeDevtools, BaselineMcp::Context7]
        );
        assert_eq!(founder, vec![BaselineMcp::Exa]);
    }

    #[test]
    fn enabled_mcp_from_gates_respects_requested_pack_mcp() {
        let env_gates = vec![
            super::ResolvedEnvGate {
                name: BaselineMcp::Context7,
                env: "CONTEXT7_API_KEY".to_string(),
                enabled: true,
            },
            super::ResolvedEnvGate {
                name: BaselineMcp::Exa,
                env: "EXA_API_KEY".to_string(),
                enabled: true,
            },
        ];

        let delivery = super::enabled_mcp_from_gates(
            &test_manifest(),
            &env_gates,
            &[BaselineMcp::ChromeDevtools, BaselineMcp::Context7],
        );
        let company =
            super::enabled_mcp_from_gates(&test_manifest(), &env_gates, &[BaselineMcp::Exa]);

        assert_eq!(
            delivery,
            vec![BaselineMcp::ChromeDevtools, BaselineMcp::Context7]
        );
        assert_eq!(company, vec![BaselineMcp::Exa]);
    }

    #[test]
    fn selected_record_template_names_follow_active_packs() {
        let delivery =
            super::selected_record_template_names(&test_manifest(), &["delivery-pack".to_string()]);
        let ops =
            super::selected_record_template_names(&test_manifest(), &["ops-pack".to_string()]);

        assert_eq!(delivery, vec!["project-record".to_string()]);
        assert_eq!(ops, vec!["ops-record".to_string()]);
    }

    #[test]
    fn record_with_writes_local_record() {
        let home = temp_home();
        let output_dir = home.join("records");
        let args = super::RecordArgs {
            record_type: crate::cli::RecordKind::Project,
            title: "Build operating records".to_string(),
            status: "active".to_string(),
            owner: Some("owner".to_string()),
            next_action: Some("verify record flow".to_string()),
            surface: crate::cli::RecordSurface::LocalDocs,
            output_dir: output_dir.clone(),
            github_repo: None,
            dry_run: false,
        };

        super::record_with(&args, &test_manifest()).unwrap();

        let files = fs::read_dir(&output_dir)
            .unwrap()
            .map(|entry| entry.unwrap().path())
            .collect::<Vec<_>>();
        assert_eq!(files.len(), 1);
        let raw = fs::read_to_string(&files[0]).unwrap();
        assert!(raw.contains("type: \"ProjectRecord\""));
        assert!(raw.contains("template: \"project-record\""));
        assert!(raw.contains("title: \"Build operating records\""));
        assert!(raw.contains("next_action: \"verify record flow\""));

        fs::remove_dir_all(home).unwrap();
    }

    #[test]
    fn slugify_produces_stable_ascii_slug() {
        assert_eq!(
            super::slugify("Build Operating Records!"),
            "build-operating-records"
        );
        assert_eq!(super::slugify("기록"), "record");
    }

    #[test]
    fn effective_distribution_targets_drop_dev_surfaces_for_company_only_harnesses() {
        let active_packs = vec!["founder-pack".to_string()];
        let targets = super::effective_distribution_targets(&test_manifest(), &active_packs);
        assert_eq!(
            targets,
            vec![
                DistributionTarget::CodexPlugin,
                DistributionTarget::GeminiExtension,
                DistributionTarget::ClaudeSkills
            ]
        );
    }

    #[test]
    fn effective_distribution_targets_keep_gemini_extension_for_incident_only_pack() {
        let active_packs = vec!["incident-pack".to_string()];
        let targets = super::effective_distribution_targets(&test_manifest(), &active_packs);
        assert!(targets.contains(&DistributionTarget::GeminiExtension));
    }

    #[test]
    fn validate_manifest_rejects_unknown_pack_harness() {
        let mut manifest = test_manifest();
        manifest.packs[0].harnesses.push("not-real".to_string());

        let err = super::validate_manifest(&manifest).unwrap_err();
        assert!(
            err.to_string()
                .contains("references unknown harness not-real")
        );
    }

    #[test]
    fn validate_manifest_rejects_missing_default_preset() {
        let mut manifest = test_manifest();
        manifest.bootstrap.default_preset = "missing".to_string();

        let err = super::validate_manifest(&manifest).unwrap_err();
        assert!(
            err.to_string()
                .contains("default preset missing is not declared")
        );
    }

    #[test]
    fn validate_manifest_rejects_unknown_pack_connector() {
        let mut manifest = test_manifest();
        manifest.packs[0]
            .connectors
            .push("missing-connector".to_string());

        let err = super::validate_manifest(&manifest).unwrap_err();
        assert!(
            err.to_string()
                .contains("pack delivery-pack references unknown connector missing-connector")
        );
    }

    #[test]
    fn validate_manifest_rejects_unknown_record_template_pack() {
        let mut manifest = test_manifest();
        manifest.record_templates[0]
            .packs
            .push("missing-pack".to_string());

        let err = super::validate_manifest(&manifest).unwrap_err();
        assert!(
            err.to_string()
                .contains("record template project-record references unknown pack missing-pack")
        );
    }

    #[test]
    fn wizard_env_overrides_marks_only_non_empty_keys() {
        let overrides =
            super::wizard_env_overrides(&Some("exa-key".to_string()), &Some("".to_string()));
        assert_eq!(overrides.get("EXA_API_KEY"), Some(&true));
        assert_eq!(overrides.get("CONTEXT7_API_KEY"), Some(&false));
    }

    #[test]
    fn codex_agent_templates_parse_and_only_long_context_roles_pin_windows() {
        let agents_dir = crate::runtime::repo_root().join("templates/codex/agents");
        let mut pinned = Vec::new();

        for entry in fs::read_dir(&agents_dir).unwrap() {
            let path = entry.unwrap().path();
            if path.extension().and_then(|ext| ext.to_str()) != Some("toml") {
                continue;
            }

            let raw = fs::read_to_string(&path).unwrap();
            let parsed: toml::Value = raw.parse().unwrap();
            let name = parsed["name"].as_str().unwrap().to_string();
            assert!(parsed.get("model").is_some(), "missing model in {}", name);
            assert!(
                parsed.get("model_reasoning_effort").is_some(),
                "missing effort in {}",
                name
            );
            if parsed.get("model_context_window").is_some() {
                assert_eq!(
                    parsed["model_context_window"].as_integer(),
                    Some(1_000_000),
                    "unexpected context window in {}",
                    name
                );
                assert_eq!(
                    parsed["model_auto_compact_token_limit"].as_integer(),
                    Some(900_000),
                    "unexpected auto compact limit in {}",
                    name
                );
                pinned.push(name);
            }
        }

        pinned.sort();
        assert_eq!(pinned, vec!["architect-1m", "planner-1m", "reviewer-1m"]);
    }

    #[test]
    fn claude_agent_templates_use_official_frontmatter_model_fields() {
        let agents_dir = crate::runtime::repo_root().join("templates/claude/agents");
        let expected = [
            ("executor.md", "model: inherit"),
            ("planner.md", "model: inherit"),
            ("reviewer.md", "model: sonnet"),
            ("triage.md", "model: haiku"),
            ("verifier.md", "model: sonnet"),
        ];

        for (file, needle) in expected {
            let raw = fs::read_to_string(agents_dir.join(file)).unwrap();
            assert!(
                raw.starts_with("---\n"),
                "{file} should start with frontmatter"
            );
            assert!(raw.contains("name:"), "{file} missing name frontmatter");
            assert!(
                raw.contains("description:"),
                "{file} missing description frontmatter"
            );
            assert!(raw.contains(needle), "{file} missing expected model");
        }
    }

    #[test]
    fn gemini_agent_templates_include_required_frontmatter() {
        let agents_dir = crate::runtime::repo_root()
            .join("templates/gemini/extensions/llm-bootstrap-dev/agents");
        let expected = [
            "docs-researcher.md",
            "executor.md",
            "planner.md",
            "qa.md",
            "reviewer.md",
            "triage.md",
            "verifier.md",
        ];

        for file in expected {
            let raw = fs::read_to_string(agents_dir.join(file)).unwrap();
            assert!(
                raw.starts_with("---\n"),
                "{file} should start with frontmatter"
            );
            assert!(raw.contains("name:"), "{file} missing name frontmatter");
            assert!(
                raw.contains("description:"),
                "{file} missing description frontmatter"
            );
        }

        let bundled_qa = crate::runtime::repo_root()
            .join("bundles/full/gemini/extensions/llm-bootstrap-dev/agents/qa.md");
        let bundled_raw = fs::read_to_string(bundled_qa).unwrap();
        assert!(
            bundled_raw.starts_with("---\n"),
            "bundled Gemini QA agent should start with frontmatter"
        );
        assert!(
            bundled_raw.contains("name:"),
            "bundled Gemini QA agent missing name frontmatter"
        );
        assert!(
            bundled_raw.contains("description:"),
            "bundled Gemini QA agent missing description frontmatter"
        );
    }

    #[test]
    fn preserved_gemini_runtime_state_keeps_auth_shape_only() {
        let existing = json!({
            "selectedAuthType": "oauth-personal",
            "security": {
                "auth": {
                    "selectedType": "oauth-personal"
                },
                "unmanagedPolicy": true
            },
            "accounts": [{"email": "dev@example.com"}],
            "general": {"defaultApprovalMode": "plan"},
            "mcpServers": {"legacy": {"command": "noop"}}
        });

        let preserved = preserved_gemini_runtime_state(&existing);

        assert_eq!(preserved["selectedAuthType"], json!("oauth-personal"));
        assert_eq!(
            preserved["security"]["auth"]["selectedType"],
            json!("oauth-personal")
        );
        assert_eq!(preserved["accounts"][0]["email"], json!("dev@example.com"));
        assert!(preserved["security"].get("unmanagedPolicy").is_none());
        assert!(preserved.get("general").is_none());
        assert!(preserved.get("mcpServers").is_none());
    }

    #[test]
    fn enabled_mcp_turns_on_env_gated_entries_only_when_keys_exist() {
        let manifest = test_manifest();
        let requested = vec![
            BaselineMcp::ChromeDevtools,
            BaselineMcp::Context7,
            BaselineMcp::Exa,
        ];
        let gates = super::resolved_env_gates(&manifest, |name| name == "EXA_API_KEY");
        let enabled = super::enabled_mcp_from_gates(&manifest, &gates, &requested);

        assert!(enabled.contains(&BaselineMcp::ChromeDevtools));
        assert!(enabled.contains(&BaselineMcp::Exa));
        assert!(!enabled.contains(&BaselineMcp::Context7));
    }

    #[test]
    fn env_is_set_with_reuses_launchctl_value_when_process_env_is_missing() {
        assert!(super::env_is_set_with(
            "EXA_API_KEY",
            |_| None,
            |name| (name == "EXA_API_KEY").then(|| "saved-key".to_string()),
            |_| None,
        ));
    }

    #[test]
    fn env_is_set_with_prefers_process_env_over_launchctl() {
        assert!(super::env_is_set_with(
            "EXA_API_KEY",
            |name| (name == "EXA_API_KEY").then(|| "process-key".to_string()),
            |_| Some("saved-key".to_string()),
            |_| Some("managed-key".to_string()),
        ));
    }

    #[test]
    fn env_is_set_with_treats_blank_values_as_disabled() {
        assert!(!super::env_is_set_with(
            "EXA_API_KEY",
            |_| Some("   ".to_string()),
            |_| Some("".to_string()),
            |_| Some("".to_string()),
        ));
    }

    #[test]
    fn env_is_set_with_reuses_managed_cli_value_when_process_and_launchctl_are_missing() {
        assert!(super::env_is_set_with(
            "EXA_API_KEY",
            |_| None,
            |_| None,
            |name| (name == "EXA_API_KEY").then(|| "managed-key".to_string()),
        ));
    }

    #[test]
    fn parse_managed_env_content_reads_exported_key() {
        let raw = "# managed by llm-bootstrap\nexport EXA_API_KEY='exa-key'\nexport CONTEXT7_API_KEY='ctx-key'\n";
        assert_eq!(
            super::parse_managed_env_content(raw, "CONTEXT7_API_KEY"),
            Some("ctx-key".to_string())
        );
    }

    #[test]
    fn upsert_managed_block_appends_when_missing() {
        let existing = "export PATH=\"$HOME/.local/bin:$PATH\"\n";
        let block = "# >>> llm-bootstrap env >>>\nsource test\n# <<< llm-bootstrap env <<<\n";
        let updated = super::upsert_managed_block(existing, block);
        assert!(updated.contains("llm-bootstrap env"));
        assert!(updated.contains("source test"));
    }

    #[test]
    fn prune_rtk_hook_removes_run_shell_command_entry_only() {
        let mut settings = json!({
            "hooks": {
                "BeforeTool": [
                    {
                        "matcher": "run_shell_command",
                        "hooks": [
                            {
                                "type": "command",
                                "command": "/tmp/.gemini/hooks/rtk-hook-gemini.sh"
                            }
                        ]
                    },
                    {
                        "matcher": "edit_file",
                        "hooks": [
                            {
                                "type": "command",
                                "command": "/tmp/other-hook.sh"
                            }
                        ]
                    },
                    {
                        "matcher": "run_shell_command",
                        "hooks": [
                            {
                                "type": "command",
                                "command": "/tmp/custom-run-shell.sh"
                            }
                        ]
                    }
                ]
            }
        });

        prune_rtk_gemini_hooks(&mut settings);

        assert_eq!(settings["hooks"]["BeforeTool"].as_array().unwrap().len(), 2);
        assert_eq!(
            settings["hooks"]["BeforeTool"][0]["matcher"],
            json!("edit_file")
        );
        assert_eq!(
            settings["hooks"]["BeforeTool"][1]["hooks"][0]["command"],
            json!("/tmp/custom-run-shell.sh")
        );
    }

    #[test]
    fn prune_rtk_claude_hook_removes_bash_entry_only() {
        let mut settings = json!({
            "hooks": {
                "PreToolUse": [
                    {
                        "matcher": "Bash",
                        "hooks": [
                            {
                                "type": "command",
                                "command": "/tmp/.claude/hooks/rtk-rewrite.sh"
                            }
                        ]
                    },
                    {
                        "matcher": "Edit",
                        "hooks": [
                            {
                                "type": "command",
                                "command": "/tmp/other-hook.sh"
                            }
                        ]
                    },
                    {
                        "matcher": "Bash",
                        "hooks": [
                            {
                                "type": "command",
                                "command": "/tmp/custom-bash-hook.sh"
                            }
                        ]
                    }
                ]
            }
        });

        prune_rtk_claude_hooks(&mut settings);

        assert_eq!(settings["hooks"]["PreToolUse"].as_array().unwrap().len(), 2);
        assert_eq!(settings["hooks"]["PreToolUse"][0]["matcher"], json!("Edit"));
        assert_eq!(
            settings["hooks"]["PreToolUse"][1]["hooks"][0]["command"],
            json!("/tmp/custom-bash-hook.sh")
        );
    }

    #[test]
    fn remove_baseline_mcp_servers_keeps_unmanaged_entries() {
        let manifest = test_manifest();
        let mut settings = json!({
            "mcpServers": {
                "chrome-devtools": {"command": "a"},
                "context7": {"command": "b"},
                "exa": {"command": "c"},
                "manual-tool": {"command": "keep"}
            }
        });

        remove_baseline_mcp_servers(&mut settings, &manifest);

        assert!(settings["mcpServers"].get("chrome-devtools").is_none());
        assert!(settings["mcpServers"].get("context7").is_none());
        assert!(settings["mcpServers"].get("exa").is_none());
        assert_eq!(
            settings["mcpServers"]["manual-tool"]["command"],
            json!("keep")
        );
    }

    #[test]
    fn cleanup_extension_enablement_removes_only_llm_bootstrap_entry() {
        let temp = temp_home();
        fs::create_dir_all(&temp).unwrap();
        let path = temp.join("extension-enablement.json");
        fs::write(
            &path,
            "{\n  \"llm-bootstrap-dev\": {\"overrides\": [\"/tmp/*\"]},\n  \"other\": {\"overrides\": [\"/keep/*\"]}\n}\n",
        )
        .unwrap();

        cleanup_extension_enablement(&path).unwrap();

        let after = fs::read_to_string(&path).unwrap();
        assert!(!after.contains("llm-bootstrap-dev"));
        assert!(after.contains("\"other\""));

        fs::remove_dir_all(temp).unwrap();
    }

    #[test]
    fn codex_install_uninstall_round_trip_without_rtk() {
        let home = temp_home();
        let manifest = test_manifest();

        let enabled = vec![BaselineMcp::ChromeDevtools];
        codex::install(
            &home,
            ApplyMode::Merge,
            &manifest,
            &enabled,
            false,
            true,
            &test_active_surfaces(),
        )
        .unwrap();
        let codex_home = home.join(".codex");
        assert!(codex_home.join("config.toml").exists());
        assert!(codex_home.join("AGENTS.md").exists());
        assert!(
            codex_home
                .join("plugins/llm-dev-kit/.codex-plugin/plugin.json")
                .exists()
        );
        assert!(!codex_home.join("RTK.md").exists());
        fs::create_dir_all(codex_home.join("vendor_imports/skills/old")).unwrap();
        fs::write(codex_home.join("vendor_imports/skills/old/SKILL.md"), "old").unwrap();

        codex::uninstall(&home, false).unwrap();
        assert!(!codex_home.join("config.toml").exists());
        assert!(!codex_home.join("AGENTS.md").exists());
        assert!(!codex_home.join("vendor_imports/skills").exists());
        assert!(codex_home.join("backups").exists());

        fs::remove_dir_all(home).unwrap();
    }

    #[test]
    fn codex_replace_removes_legacy_plugin_cache() {
        let home = temp_home();
        let manifest = test_manifest();
        let codex_home = home.join(".codex");
        fs::create_dir_all(codex_home.join(".tmp/plugins/omx")).unwrap();
        fs::write(codex_home.join(".tmp/plugins/omx/plugin.json"), "{}").unwrap();

        codex::install(
            &home,
            ApplyMode::Replace,
            &manifest,
            &[BaselineMcp::ChromeDevtools],
            false,
            true,
            &test_active_surfaces(),
        )
        .unwrap();

        assert!(!codex_home.join(".tmp/plugins").exists());
        assert!(codex_home.join("backups").exists());

        fs::remove_dir_all(home).unwrap();
    }

    #[test]
    fn codex_merge_preserves_unmanaged_mcp_blocks() {
        let home = temp_home();
        let codex_home = home.join(".codex");
        fs::create_dir_all(&codex_home).unwrap();
        fs::write(
            codex_home.join("config.toml"),
            "[mcp_servers.manual-tool]\ncommand = \"manual-tool\"\nenabled = true\n",
        )
        .unwrap();

        let blocks = codex::mcp_blocks(
            &home,
            &codex_home,
            &[BaselineMcp::ChromeDevtools],
            ApplyMode::Merge,
        )
        .unwrap();

        assert!(blocks.contains("[mcp_servers.manual-tool]"));
        assert!(blocks.contains("[mcp_servers.\"chrome-devtools\"]"));

        fs::remove_dir_all(home).unwrap();
    }

    #[test]
    fn codex_restore_recovers_latest_backup() {
        let home = temp_home();
        let codex_home = home.join(".codex");
        fs::create_dir_all(&codex_home).unwrap();
        fs::write(codex_home.join("AGENTS.md"), "old codex agents").unwrap();

        codex::install(
            &home,
            ApplyMode::Replace,
            &test_manifest(),
            &[BaselineMcp::ChromeDevtools],
            false,
            true,
            &test_active_surfaces(),
        )
        .unwrap();

        assert_ne!(
            fs::read_to_string(codex_home.join("AGENTS.md")).unwrap(),
            "old codex agents"
        );

        codex::restore(&home, None).unwrap();

        assert_eq!(
            fs::read_to_string(codex_home.join("AGENTS.md")).unwrap(),
            "old codex agents"
        );

        fs::remove_dir_all(home).unwrap();
    }

    #[test]
    fn codex_install_skips_plugin_assets_when_distribution_target_is_disabled() {
        let home = temp_home();
        let manifest = test_manifest();
        let enabled = vec![BaselineMcp::ChromeDevtools];

        codex::install(
            &home,
            ApplyMode::Merge,
            &manifest,
            &enabled,
            false,
            false,
            &test_active_surfaces(),
        )
        .unwrap();

        let codex_home = home.join(".codex");
        let config = fs::read_to_string(codex_home.join("config.toml")).unwrap();
        assert!(!config.contains("llm-dev-kit@llm-bootstrap"));
        assert!(!codex_home.join(".agents/plugins/marketplace.json").exists());
        assert!(!codex_home.join("plugins/llm-dev-kit").exists());

        fs::remove_dir_all(home).unwrap();
    }

    #[test]
    fn codex_install_slices_plugin_skills_by_active_surfaces() {
        let home = temp_home();
        let manifest = test_manifest();
        let enabled = vec![BaselineMcp::ChromeDevtools];
        let incident_only = vec!["incident-skills".to_string()];

        codex::install(
            &home,
            ApplyMode::Merge,
            &manifest,
            &enabled,
            false,
            true,
            &incident_only,
        )
        .unwrap();

        let codex_home = home.join(".codex");
        assert!(
            codex_home
                .join("plugins/llm-dev-kit/.codex-plugin/plugin.json")
                .exists()
        );
        assert!(
            codex_home
                .join("plugins/llm-dev-kit/skills/investigate/SKILL.md")
                .exists()
        );
        assert!(
            codex_home
                .join("plugins/llm-dev-kit/skills/repo-radar/SKILL.md")
                .exists()
        );
        assert!(
            !codex_home
                .join("plugins/llm-dev-kit/skills/autopilot/SKILL.md")
                .exists()
        );
        assert!(
            !codex_home
                .join("plugins/llm-dev-kit/skills/delivery-loop/SKILL.md")
                .exists()
        );
        assert!(
            !codex_home
                .join("plugins/llm-dev-kit/skills/qa-browser/SKILL.md")
                .exists()
        );

        fs::remove_dir_all(home).unwrap();
    }

    #[test]
    fn gemini_install_uninstall_round_trip_without_rtk_preserves_custom_hooks() {
        let home = temp_home();
        let manifest = test_manifest();
        let gemini_home = home.join(".gemini");
        fs::create_dir_all(gemini_home.join("hooks")).unwrap();
        fs::write(
            gemini_home.join("settings.json"),
            "{\n  \"selectedAuthType\": \"oauth-personal\",\n  \"hooks\": {\n    \"BeforeTool\": [\n      {\n        \"matcher\": \"run_shell_command\",\n        \"hooks\": [\n          {\n            \"type\": \"command\",\n            \"command\": \"/tmp/custom-run-shell.sh\"\n          }\n        ]\n      }\n    ]\n  }\n}\n",
        )
        .unwrap();

        let enabled = vec![BaselineMcp::ChromeDevtools];
        gemini::install(
            &home,
            ApplyMode::Merge,
            &manifest,
            &enabled,
            false,
            true,
            &test_active_surfaces(),
        )
        .unwrap();
        let installed = fs::read_to_string(gemini_home.join("settings.json")).unwrap();
        assert!(installed.contains("/tmp/custom-run-shell.sh"));
        assert!(gemini_home.join("GEMINI.md").exists());
        assert!(
            gemini_home
                .join("extensions/llm-bootstrap-dev/commands/doctor.toml")
                .exists()
        );
        assert!(
            gemini_home
                .join("extensions/llm-bootstrap-dev/commands/autopilot.toml")
                .exists()
        );
        assert!(!gemini_home.join("hooks/rtk-hook-gemini.sh").exists());
        fs::create_dir_all(gemini_home.join("extensions/oh-my-gemini")).unwrap();
        fs::write(
            gemini_home.join("extensions/oh-my-gemini/gemini-extension.json"),
            "{}",
        )
        .unwrap();

        gemini::uninstall(&home, &manifest, false).unwrap();
        let uninstalled = fs::read_to_string(gemini_home.join("settings.json")).unwrap();
        assert!(uninstalled.contains("/tmp/custom-run-shell.sh"));
        assert!(!gemini_home.join("GEMINI.md").exists());
        assert!(!gemini_home.join("extensions/llm-bootstrap-dev").exists());
        assert!(!gemini_home.join("extensions/oh-my-gemini").exists());
        assert!(gemini_home.join("backups").exists());

        fs::remove_dir_all(home).unwrap();
    }

    #[test]
    fn gemini_uninstall_preserves_other_extension_enablement_and_removes_bundle_docs() {
        let home = temp_home();
        let manifest = test_manifest();
        let gemini_home = home.join(".gemini");

        gemini::install(
            &home,
            ApplyMode::Merge,
            &manifest,
            &[BaselineMcp::ChromeDevtools],
            false,
            true,
            &test_active_surfaces(),
        )
        .unwrap();
        fs::write(
            gemini_home.join("llm-bootstrap-state.json"),
            "{\n  \"managed_mcp\": [\"chrome-devtools\"]\n}\n",
        )
        .unwrap();
        fs::write(
            gemini_home.join("extensions/extension-enablement.json"),
            "{\n  \"llm-bootstrap-dev\": {\"overrides\": [\"/tmp/*\"]},\n  \"other-extension\": {\"overrides\": [\"/opt/*\"]}\n}\n",
        )
        .unwrap();

        gemini::uninstall(&home, &manifest, false).unwrap();

        let enablement: serde_json::Value = serde_json::from_str(
            &fs::read_to_string(gemini_home.join("extensions/extension-enablement.json")).unwrap(),
        )
        .unwrap();
        assert!(enablement.get("llm-bootstrap-dev").is_none());
        assert!(enablement.get("other-extension").is_some());
        assert!(!gemini_home.join("WORKFLOW.md").exists());
        assert!(!gemini_home.join("SHIP_CHECKLIST.md").exists());
        assert!(!gemini_home.join("llm-bootstrap-state.json").exists());

        fs::remove_dir_all(home).unwrap();
    }

    #[test]
    fn gemini_merge_preserves_unmanaged_mcp_servers() {
        let home = temp_home();
        let manifest = test_manifest();
        let gemini_home = home.join(".gemini");
        fs::create_dir_all(&gemini_home).unwrap();
        fs::create_dir_all(gemini_home.join("extensions/oh-my-gemini-cli")).unwrap();
        fs::write(
            gemini_home.join("extensions/oh-my-gemini-cli/gemini-extension.json"),
            "{}",
        )
        .unwrap();
        fs::write(
            gemini_home.join("settings.json"),
            "{\n  \"mcpServers\": {\n    \"legacy-memory\": {\"command\": \"legacy-memory\"},\n    \"manual-tool\": {\"command\": \"manual-tool\"}\n  },\n  \"selectedAuthType\": \"oauth-personal\",\n  \"security\": {\n    \"auth\": {\n      \"selectedType\": \"oauth-personal\"\n    }\n  }\n}\n",
        )
        .unwrap();

        gemini::install(
            &home,
            ApplyMode::Merge,
            &manifest,
            &[BaselineMcp::ChromeDevtools],
            false,
            true,
            &test_active_surfaces(),
        )
        .unwrap();

        let after: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(gemini_home.join("settings.json")).unwrap())
                .unwrap();
        assert!(after["mcpServers"].get("legacy-memory").is_some());
        assert!(after["mcpServers"].get("manual-tool").is_some());
        assert!(after["mcpServers"].get("chrome-devtools").is_some());

        fs::remove_dir_all(home).unwrap();
    }

    #[test]
    fn gemini_replace_keeps_only_baseline_mcp_servers() {
        let home = temp_home();
        let manifest = test_manifest();
        let gemini_home = home.join(".gemini");
        fs::create_dir_all(&gemini_home).unwrap();
        fs::write(
            gemini_home.join("settings.json"),
            "{\n  \"mcpServers\": {\n    \"legacy-memory\": {\"command\": \"legacy-memory\"},\n    \"manual-tool\": {\"command\": \"manual-tool\"}\n  },\n  \"selectedAuthType\": \"oauth-personal\",\n  \"security\": {\n    \"auth\": {\n      \"selectedType\": \"oauth-personal\"\n    }\n  }\n}\n",
        )
        .unwrap();

        gemini::install(
            &home,
            ApplyMode::Replace,
            &manifest,
            &[BaselineMcp::ChromeDevtools],
            false,
            true,
            &test_active_surfaces(),
        )
        .unwrap();

        let after: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(gemini_home.join("settings.json")).unwrap())
                .unwrap();
        assert!(after["mcpServers"].get("legacy-memory").is_none());
        assert!(after["mcpServers"].get("manual-tool").is_none());
        assert!(after["mcpServers"].get("chrome-devtools").is_some());
        assert_eq!(after["selectedAuthType"], json!("oauth-personal"));
        assert_eq!(
            after["security"]["auth"]["selectedType"],
            json!("oauth-personal")
        );
        assert!(!gemini_home.join("extensions/oh-my-gemini-cli").exists());

        fs::remove_dir_all(home).unwrap();
    }

    #[test]
    fn gemini_install_bootstraps_projects_registry() {
        let home = temp_home();
        let manifest = test_manifest();

        gemini::install(
            &home,
            ApplyMode::Merge,
            &manifest,
            &[BaselineMcp::ChromeDevtools],
            false,
            true,
            &test_active_surfaces(),
        )
        .unwrap();

        let registry: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(home.join(".gemini/projects.json")).unwrap())
                .unwrap();
        assert_eq!(registry, json!({ "projects": {} }));

        fs::remove_dir_all(home).unwrap();
    }

    #[test]
    fn gemini_restore_recovers_latest_backup() {
        let home = temp_home();
        let manifest = test_manifest();
        let gemini_home = home.join(".gemini");
        fs::create_dir_all(&gemini_home).unwrap();
        fs::write(
            gemini_home.join("settings.json"),
            "{\n  \"mcpServers\": {\"manual-tool\": {\"command\": \"manual-tool\"}},\n  \"selectedAuthType\": \"oauth-personal\"\n}\n",
        )
        .unwrap();

        gemini::install(
            &home,
            ApplyMode::Replace,
            &manifest,
            &[BaselineMcp::ChromeDevtools],
            false,
            true,
            &test_active_surfaces(),
        )
        .unwrap();

        let after_replace: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(gemini_home.join("settings.json")).unwrap())
                .unwrap();
        assert!(after_replace["mcpServers"].get("manual-tool").is_none());

        gemini::restore(&home, None).unwrap();

        let restored: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(gemini_home.join("settings.json")).unwrap())
                .unwrap();
        assert!(restored["mcpServers"].get("manual-tool").is_some());

        fs::remove_dir_all(home).unwrap();
    }

    #[test]
    fn gemini_install_skips_extension_assets_when_distribution_target_is_disabled() {
        let home = temp_home();
        let manifest = test_manifest();

        gemini::install(
            &home,
            ApplyMode::Merge,
            &manifest,
            &[BaselineMcp::ChromeDevtools],
            false,
            false,
            &test_active_surfaces(),
        )
        .unwrap();

        let gemini_home = home.join(".gemini");
        assert!(gemini_home.join("GEMINI.md").exists());
        assert!(
            !gemini_home
                .join("extensions/llm-bootstrap-dev/gemini-extension.json")
                .exists()
        );
        assert!(
            !gemini_home
                .join("extensions/llm-bootstrap-dev/commands/autopilot.toml")
                .exists()
        );
        let enablement = gemini_home.join("extensions/extension-enablement.json");
        if enablement.exists() {
            let raw = fs::read_to_string(enablement).unwrap();
            assert!(!raw.contains("llm-bootstrap-dev"));
        }

        fs::remove_dir_all(home).unwrap();
    }

    #[test]
    fn claude_install_uninstall_round_trip_without_rtk() {
        if !crate::runtime::command_exists("claude") {
            return;
        }

        let home = temp_home();
        let manifest = test_manifest();
        let enabled = vec![BaselineMcp::ChromeDevtools];

        claude::install(
            &home,
            ApplyMode::Merge,
            &manifest,
            &enabled,
            false,
            true,
            &test_active_surfaces(),
        )
        .unwrap();
        let claude_home = home.join(".claude");
        assert!(claude_home.join("CLAUDE.md").exists());
        assert!(claude_home.join("scripts/chrome-devtools-mcp.sh").exists());
        assert!(claude_home.join("skills/autopilot/SKILL.md").exists());
        assert!(claude_home.join("skills/review/SKILL.md").exists());
        assert!(!claude_home.join("RTK.md").exists());

        let mcp = claude::claude_user_mcp(&home).unwrap();
        assert!(mcp["mcpServers"].get("chrome-devtools").is_some());
        fs::create_dir_all(claude_home.join("plugins/cache/omc")).unwrap();
        fs::write(claude_home.join("plugins/cache/omc/state.json"), "{}").unwrap();

        claude::uninstall(&home, &enabled, false).unwrap();
        assert!(!claude_home.join("CLAUDE.md").exists());
        assert!(!claude_home.join("scripts").exists());
        assert!(!claude_home.join("skills/autopilot").exists());
        assert!(!claude_home.join("plugins/cache/omc").exists());
        assert!(claude_home.join("backups").exists());

        fs::remove_dir_all(home).unwrap();
    }

    #[test]
    fn claude_uninstall_removes_bundle_docs_agents_and_state() {
        if !crate::runtime::command_exists("claude") {
            return;
        }

        let home = temp_home();
        let manifest = test_manifest();

        claude::install(
            &home,
            ApplyMode::Merge,
            &manifest,
            &[BaselineMcp::ChromeDevtools],
            false,
            true,
            &test_active_surfaces(),
        )
        .unwrap();

        let claude_home = home.join(".claude");
        assert!(claude_home.join("WORKFLOW.md").exists());
        assert!(claude_home.join("AUTOPILOT.md").exists());
        assert!(claude_home.join("agents/planner.md").exists());
        assert!(claude_home.join("llm-bootstrap-state.json").exists());

        claude::uninstall(&home, &[BaselineMcp::ChromeDevtools], false).unwrap();

        assert!(!claude_home.join("WORKFLOW.md").exists());
        assert!(!claude_home.join("AUTOPILOT.md").exists());
        assert!(!claude_home.join("agents/planner.md").exists());
        assert!(!claude_home.join("llm-bootstrap-state.json").exists());

        fs::remove_dir_all(home).unwrap();
    }

    #[test]
    fn claude_merge_removes_now_disabled_managed_mcp() {
        if !crate::runtime::command_exists("claude") {
            return;
        }

        let home = temp_home();
        let manifest = test_manifest();

        claude::install(
            &home,
            ApplyMode::Merge,
            &manifest,
            &[BaselineMcp::ChromeDevtools, BaselineMcp::Context7],
            false,
            true,
            &test_active_surfaces(),
        )
        .unwrap();

        claude::install(
            &home,
            ApplyMode::Merge,
            &manifest,
            &[BaselineMcp::ChromeDevtools],
            false,
            true,
            &test_active_surfaces(),
        )
        .unwrap();

        let mcp = claude::claude_user_mcp(&home).unwrap();
        assert!(mcp["mcpServers"].get("chrome-devtools").is_some());
        assert!(mcp["mcpServers"].get("context7").is_none());
        assert!(!home.join(".claude/scripts/context7-mcp.sh").exists());

        fs::remove_dir_all(home).unwrap();
    }

    #[test]
    fn claude_uninstall_preserves_unmanaged_mcp() {
        if !crate::runtime::command_exists("claude") {
            return;
        }

        let home = temp_home();
        let manifest = test_manifest();
        let enabled = vec![BaselineMcp::ChromeDevtools];

        claude::install(
            &home,
            ApplyMode::Merge,
            &manifest,
            &enabled,
            false,
            true,
            &test_active_surfaces(),
        )
        .unwrap();
        std::process::Command::new("claude")
            .env("HOME", &home)
            .args([
                "mcp",
                "add",
                "--scope",
                "user",
                "manual-tool",
                "--",
                "/bin/echo",
                "manual",
            ])
            .status()
            .unwrap();

        claude::uninstall(&home, &enabled, false).unwrap();

        let mcp = claude::claude_user_mcp(&home).unwrap();
        assert!(mcp["mcpServers"].get("chrome-devtools").is_none());
        assert!(mcp["mcpServers"].get("manual-tool").is_some());

        fs::remove_dir_all(home).unwrap();
    }

    #[test]
    fn claude_uninstall_preserves_unmanaged_skills() {
        if !crate::runtime::command_exists("claude") {
            return;
        }

        let home = temp_home();
        let manifest = test_manifest();
        let enabled = vec![BaselineMcp::ChromeDevtools];
        let unmanaged_skill = home.join(".claude/skills/omc-reference");
        fs::create_dir_all(&unmanaged_skill).unwrap();
        fs::write(
            unmanaged_skill.join("SKILL.md"),
            "---\nname: omc-reference\n---\n",
        )
        .unwrap();

        claude::install(
            &home,
            ApplyMode::Merge,
            &manifest,
            &enabled,
            false,
            true,
            &test_active_surfaces(),
        )
        .unwrap();
        assert!(home.join(".claude/skills/autopilot/SKILL.md").exists());
        assert!(unmanaged_skill.join("SKILL.md").exists());

        claude::uninstall(&home, &enabled, false).unwrap();

        assert!(unmanaged_skill.join("SKILL.md").exists());
        assert!(!home.join(".claude/skills/autopilot").exists());

        fs::remove_dir_all(home).unwrap();
    }

    #[test]
    fn claude_replace_removes_unmanaged_mcp() {
        if !crate::runtime::command_exists("claude") {
            return;
        }

        let home = temp_home();
        let manifest = test_manifest();
        let enabled = vec![BaselineMcp::ChromeDevtools];

        std::process::Command::new("claude")
            .env("HOME", &home)
            .args([
                "mcp",
                "add",
                "--scope",
                "user",
                "manual-tool",
                "--",
                "/bin/echo",
                "manual",
            ])
            .status()
            .unwrap();

        claude::install(
            &home,
            ApplyMode::Replace,
            &manifest,
            &enabled,
            false,
            true,
            &test_active_surfaces(),
        )
        .unwrap();

        let mcp = claude::claude_user_mcp(&home).unwrap();
        assert!(mcp["mcpServers"].get("manual-tool").is_none());
        assert!(mcp["mcpServers"].get("chrome-devtools").is_some());

        fs::remove_dir_all(home).unwrap();
    }

    #[test]
    fn claude_restore_recovers_latest_backup() {
        if !crate::runtime::command_exists("claude") {
            return;
        }

        let home = temp_home();
        let manifest = test_manifest();
        fs::write(
            home.join(".claude.json"),
            "{\n  \"mcpServers\": {\n    \"manual-tool\": {\"command\": \"manual-tool\"}\n  }\n}\n",
        )
        .unwrap();

        claude::install(
            &home,
            ApplyMode::Replace,
            &manifest,
            &[BaselineMcp::ChromeDevtools],
            false,
            true,
            &test_active_surfaces(),
        )
        .unwrap();

        let replaced = claude::claude_user_mcp(&home).unwrap();
        assert!(replaced["mcpServers"].get("manual-tool").is_none());

        claude::restore(&home, None).unwrap();

        let restored = claude::claude_user_mcp(&home).unwrap();
        assert!(restored["mcpServers"].get("manual-tool").is_some());

        fs::remove_dir_all(home).unwrap();
    }

    #[test]
    fn claude_install_skips_managed_skills_when_distribution_target_is_disabled() {
        if !crate::runtime::command_exists("claude") {
            return;
        }

        let home = temp_home();
        let manifest = test_manifest();

        claude::install(
            &home,
            ApplyMode::Merge,
            &manifest,
            &[BaselineMcp::ChromeDevtools],
            false,
            false,
            &test_active_surfaces(),
        )
        .unwrap();

        let claude_home = home.join(".claude");
        assert!(claude_home.join("CLAUDE.md").exists());
        assert!(!claude_home.join("skills/autopilot").exists());
        assert!(!claude_home.join("skills/review").exists());

        fs::remove_dir_all(home).unwrap();
    }
}
