mod cli;
mod fs_ops;
mod json_ops;
mod layout;
mod manifest;
mod providers;
mod repo_assets;
mod runtime;
mod state;

use anyhow::{Context, Result, bail};
use clap::Parser;
use cli::{
    BackupsArgs, Cli, Command, DoctorArgs, GateApplyArgs, GateArgs, GateCheckArgs, GateCommand,
    GateSignal, InstallArgs, InternalArgs, InternalCommand, PackArgs, ProbeArgs, Provider,
    ProviderArgs, RecordArgs, RecordSurface, RepoAutomationArgs, RepoAutomationCommand,
    RepoAutomationScaffoldArgs, RestoreArgs, TaskPhase, TaskStateAdvanceArgs, TaskStateArgs,
    TaskStateBeginArgs, TaskStateCommand, TaskStateShowArgs, TaskStatus, UninstallArgs, WizardArgs,
};
use dialoguer::{Confirm, MultiSelect, Password, Select, theme::ColorfulTheme};
use fs_ops::{
    backup_relative, copy_render_file_with_extras, list_backup_entries, remove_if_exists,
};
use indexmap::IndexSet;
use manifest::{BaselineMcp, BootstrapManifest, DistributionTarget};
use providers::{claude, codex, gemini};
use runtime::{command_exists, ensure_runtime_dependencies, home_dir, repo_root, timestamp_string};
use serde::Serialize;
use state::{
    RequestedState, TaskState, clear_task_state, read_installed_state, read_task_state,
    write_installed_state, write_task_state,
};
use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::fs;
use std::hash::Hash;
use std::path::{Path, PathBuf};
use std::process::Command as ProcessCommand;

use crate::layout::{
    HOME_LEGACY_CLEANUP_PATHS, LEGACY_ENV_KEYS, claude_managed_paths_for, codex_managed_paths_for,
    gemini_managed_paths_for,
};

const ZSHRC_MARKER_START: &str = "# >>> stackpilot env >>>";
const ZSHRC_MARKER_END: &str = "# <<< stackpilot env <<<";
const ZSHRC_ENV_RELATIVE_PATH: &str = ".zshrc.d/stackpilot-env.zsh";

fn main() -> Result<()> {
    let cli = Cli::parse();
    let manifest = load_manifest()?;

    match cli.command.unwrap_or_else(default_command) {
        Command::Baseline(args) => install(args, &manifest),
        Command::Install(args) => install(args, &manifest),
        Command::Sync(args) => install(args, &manifest),
        Command::Restore(args) => restore(args, &manifest),
        Command::Backups(args) => backups(args, &manifest),
        Command::Uninstall(args) => uninstall(args, &manifest),
        Command::Doctor(args) => doctor(args, &manifest),
        Command::Probe(args) => probe(args, &manifest),
        Command::Internal(args) => internal(args, &manifest),
        Command::TaskState(args) => task_state(args, &manifest),
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
    let record_surface = args.record_surface.unwrap_or(RecordSurface::Both);
    let resolved = resolve_plan(manifest, &args.pack_args)?;
    let home = home_dir()?;
    if args.dry_run {
        print_install_plan(
            &home,
            &providers,
            manifest,
            mode,
            rtk_enabled,
            record_surface,
            &resolved,
        );
        return Ok(());
    }
    ensure_runtime_dependencies(rtk_enabled)?;
    install_with(
        &home,
        &providers,
        manifest,
        mode,
        rtk_enabled,
        record_surface,
        &resolved,
    )
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
        args.record_surface,
        &resolved,
    )
}

fn probe(args: ProbeArgs, manifest: &BootstrapManifest) -> Result<()> {
    let home = home_dir()?;
    let providers = selected_providers(&args.provider_args, manifest);
    let resolved = resolve_plan(manifest, &args.pack_args)?;
    let mut reports = Vec::new();
    let mut ok = true;

    for provider in providers {
        let files = provider_probe_paths(&home, provider, &resolved)
            .into_iter()
            .map(|path| ProbeFileCheck {
                target: path.display().to_string(),
                status: if path.exists() { "ok" } else { "missing" }.to_string(),
            })
            .collect::<Vec<_>>();
        if files.iter().any(|check| check.status != "ok") {
            ok = false;
        }

        let runtime_check = provider_runtime_check(provider);
        let runtime = if runtime_check.status == "ok" {
            run_provider_probe(&home, provider, &args.prompt)?
        } else {
            ok = false;
            ProbeRuntimeResult {
                status: "missing".to_string(),
                command: runtime_check.target,
                stdout: None,
                stderr: runtime_check.detail,
                detail: Some("provider runtime missing".to_string()),
            }
        };
        if runtime.status != "ok" {
            ok = false;
        }
        let optimizations = if args.optimize {
            run_provider_optimization_probes(&home, provider)?
        } else {
            Vec::new()
        };
        if optimizations.iter().any(|check| check.status != "ok") {
            ok = false;
        }

        reports.push(ProbeProviderReport {
            provider: provider.name().to_string(),
            requested_surfaces: provider_surfaces(provider, &resolved.surfaces).to_vec(),
            files,
            runtime,
            optimizations,
        });
    }

    let report = ProbeReport {
        ok,
        providers: reports,
    };

    if args.json {
        println!("{}", serde_json::to_string_pretty(&report)?);
    } else {
        println!("[probe]");
        for provider in &report.providers {
            println!("[provider] {}", provider.provider);
            println!(
                "[surfaces] {}",
                if provider.requested_surfaces.is_empty() {
                    "-".to_string()
                } else {
                    provider.requested_surfaces.join(",")
                }
            );
            for file in &provider.files {
                println!("[{}] {}", file.status, file.target);
            }
            println!(
                "[runtime:{}] {}",
                provider.runtime.status, provider.runtime.command
            );
            if let Some(detail) = provider.runtime.detail.as_deref() {
                println!("[detail] {}", detail);
            }
            if let Some(stdout) = provider.runtime.stdout.as_deref() {
                println!("[stdout] {}", stdout);
            }
            if let Some(stderr) = provider.runtime.stderr.as_deref()
                && !stderr.is_empty()
            {
                println!("[stderr] {}", stderr);
            }
            for check in &provider.optimizations {
                println!("[optimize:{}] {}", check.status, check.command);
                if let Some(detail) = check.detail.as_deref() {
                    println!("[optimize-detail] {}", detail);
                }
                if let Some(stdout) = check.stdout.as_deref() {
                    println!("[optimize-stdout] {}", stdout);
                }
                if let Some(stderr) = check.stderr.as_deref()
                    && !stderr.is_empty()
                {
                    println!("[optimize-stderr] {}", stderr);
                }
            }
        }
        println!(
            "[probe] complete: {}",
            if report.ok {
                "harness probe passed"
            } else {
                "blocking failure detected"
            }
        );
    }

    if report.ok {
        Ok(())
    } else {
        bail!("probe found blocking failures")
    }
}

fn internal(args: InternalArgs, manifest: &BootstrapManifest) -> Result<()> {
    match args.command {
        InternalCommand::TaskState(args) => task_state(args, manifest),
        InternalCommand::Gate(args) => gate(args),
        InternalCommand::RepoAutomation(args) => repo_automation(args),
    }
}

#[derive(Debug, Serialize)]
struct RepoAutomationConfig {
    managed_by: String,
    default_branch: String,
    pr_review_gate: RepoAutomationPrGateConfig,
    release_readiness_gate: RepoAutomationReleaseGateConfig,
}

#[derive(Debug, Serialize)]
struct RepoAutomationPrGateConfig {
    required_checks: Vec<String>,
    required_checklist: Vec<String>,
    minimum_approvals: usize,
}

#[derive(Debug, Serialize)]
struct RepoAutomationReleaseGateConfig {
    required_checks: Vec<String>,
    require_default_branch_ancestor: bool,
}

#[derive(Debug, Serialize)]
struct RepoAutomationScaffoldReport {
    repo_root: String,
    default_branch: String,
    pr_required_checks: Vec<String>,
    release_required_checks: Vec<String>,
    minimum_approvals: usize,
    managed_files: Vec<String>,
    dry_run: bool,
    json: bool,
    next_steps: Vec<String>,
}

fn repo_automation(args: RepoAutomationArgs) -> Result<()> {
    match args.command {
        RepoAutomationCommand::Scaffold(args) => repo_automation_scaffold(args),
    }
}

fn repo_automation_scaffold(args: RepoAutomationScaffoldArgs) -> Result<()> {
    let cwd = env::current_dir().context("failed to read current directory")?;
    let target_root = if args.repo_root.is_absolute() {
        args.repo_root.clone()
    } else {
        cwd.join(&args.repo_root)
    };
    let report = repo_automation_scaffold_with(&target_root, args)?;
    print_repo_automation_scaffold_report(&report)?;
    Ok(())
}

fn repo_automation_scaffold_with(
    target_root: &Path,
    args: RepoAutomationScaffoldArgs,
) -> Result<RepoAutomationScaffoldReport> {
    let target_root = target_root.to_path_buf();
    let template_root = repo_root().join("templates/repo-automation/github");
    let pr_required_checks = normalize_repo_check_names(&args.pr_required_checks);
    let release_required_checks = if args.release_required_checks.is_empty() {
        vec!["pr-review-gate / gate".to_string()]
    } else {
        normalize_repo_check_names(&args.release_required_checks)
    };
    let minimum_approvals = args.minimum_approvals.max(1);
    let config = RepoAutomationConfig {
        managed_by: "stackpilot".to_string(),
        default_branch: args.default_branch.clone(),
        pr_review_gate: RepoAutomationPrGateConfig {
            required_checks: pr_required_checks.clone(),
            required_checklist: vec!["review".to_string(), "qa".to_string(), "verify".to_string()],
            minimum_approvals,
        },
        release_readiness_gate: RepoAutomationReleaseGateConfig {
            required_checks: release_required_checks.clone(),
            require_default_branch_ancestor: true,
        },
    };

    let config_path = target_root.join(".github/stackpilot/review-automation.json");
    let branch_doc_path = target_root.join(".github/stackpilot/BRANCH_PROTECTION.md");
    let pr_template_path = target_root.join(".github/PULL_REQUEST_TEMPLATE.md");
    let pr_workflow_path = target_root.join(".github/workflows/pr-review-gate.yml");
    let release_workflow_path = target_root.join(".github/workflows/release-readiness-gate.yml");
    let managed_files = [
        config_path.clone(),
        branch_doc_path.clone(),
        pr_template_path.clone(),
        pr_workflow_path.clone(),
        release_workflow_path.clone(),
    ];

    for path in &managed_files {
        ensure_repo_automation_path(path, args.force)?;
    }

    if !args.dry_run {
        fs::create_dir_all(target_root.join(".github/stackpilot")).with_context(|| {
            format!(
                "failed to create {}",
                target_root.join(".github/stackpilot").display()
            )
        })?;
        fs::create_dir_all(target_root.join(".github/workflows")).with_context(|| {
            format!(
                "failed to create {}",
                target_root.join(".github/workflows").display()
            )
        })?;

        let pr_check_lines = if pr_required_checks.is_empty() {
            "- no repo-specific checks configured yet; add your CI check names in `review-automation.json`".to_string()
        } else {
            pr_required_checks
                .iter()
                .map(|name| format!("- `{name}`"))
                .collect::<Vec<_>>()
                .join("\n")
        };
        let release_check_lines = release_required_checks
            .iter()
            .map(|name| format!("- `{name}`"))
            .collect::<Vec<_>>()
            .join("\n");
        let home = target_root.clone();
        let pr_required_checks_json =
            serde_json::to_string(&config.pr_review_gate.required_checks)?;
        let release_required_checks_json =
            serde_json::to_string(&config.release_readiness_gate.required_checks)?;
        copy_render_file_with_extras(
            &template_root.join("review-automation.json"),
            &config_path,
            false,
            &home,
            &[
                ("__DEFAULT_BRANCH__", &args.default_branch),
                ("__PR_REQUIRED_CHECKS_JSON__", &pr_required_checks_json),
                (
                    "__RELEASE_REQUIRED_CHECKS_JSON__",
                    &release_required_checks_json,
                ),
                ("__MINIMUM_APPROVALS__", &minimum_approvals.to_string()),
            ],
        )?;
        copy_render_file_with_extras(
            &template_root.join("BRANCH_PROTECTION.md"),
            &branch_doc_path,
            false,
            &home,
            &[
                ("__DEFAULT_BRANCH__", &args.default_branch),
                ("__PR_REQUIRED_CHECK_LINES__", &pr_check_lines),
                ("__RELEASE_REQUIRED_CHECK_LINES__", &release_check_lines),
                ("__MINIMUM_APPROVALS__", &minimum_approvals.to_string()),
            ],
        )?;
        copy_render_file_with_extras(
            &template_root.join("PULL_REQUEST_TEMPLATE.md"),
            &pr_template_path,
            false,
            &home,
            &[],
        )?;
        copy_render_file_with_extras(
            &template_root.join("pr-review-gate.yml"),
            &pr_workflow_path,
            false,
            &home,
            &[],
        )?;
        copy_render_file_with_extras(
            &template_root.join("release-readiness-gate.yml"),
            &release_workflow_path,
            false,
            &home,
            &[("__DEFAULT_BRANCH__", &args.default_branch)],
        )?;
    }

    let mut next_steps = vec![
        format!(
            "commit the generated workflow files under {}/.github",
            target_root.display()
        ),
        format!(
            "add `pr-review-gate / gate` as a required status check for `{}`",
            args.default_branch
        ),
    ];
    if !pr_required_checks.is_empty() {
        next_steps.push(
            "keep `.github/stackpilot/review-automation.json` aligned with the repo CI check names"
                .to_string(),
        );
    }
    next_steps.push(
        "keep `.github/PULL_REQUEST_TEMPLATE.md` aligned with the gate checklist when the repo review contract changes"
            .to_string(),
    );
    next_steps.push(
        "open one pull request and one workflow_dispatch release run to validate the gate end to end"
            .to_string(),
    );

    Ok(RepoAutomationScaffoldReport {
        repo_root: target_root.display().to_string(),
        default_branch: args.default_branch,
        pr_required_checks,
        release_required_checks,
        minimum_approvals,
        managed_files: managed_files
            .iter()
            .map(|path| path.display().to_string())
            .collect(),
        dry_run: args.dry_run,
        json: args.json,
        next_steps,
    })
}

fn normalize_repo_check_names(raw: &[String]) -> Vec<String> {
    let mut seen = IndexSet::new();
    for item in raw {
        let trimmed = item.trim();
        if !trimmed.is_empty() {
            seen.insert(trimmed.to_string());
        }
    }
    seen.into_iter().collect()
}

fn ensure_repo_automation_path(path: &Path, force: bool) -> Result<()> {
    if !path.exists() || force {
        return Ok(());
    }
    let raw = fs::read_to_string(path).unwrap_or_default();
    let managed =
        raw.contains("# managed by stackpilot") || raw.contains("\"managed_by\": \"stackpilot\"");
    if managed {
        return Ok(());
    }
    bail!(
        "refusing to overwrite unmanaged repo automation file {}; pass --force to replace it",
        path.display()
    )
}

fn print_repo_automation_scaffold_report(report: &RepoAutomationScaffoldReport) -> Result<()> {
    if report.json {
        println!("{}", serde_json::to_string_pretty(report)?);
        return Ok(());
    }
    if report.dry_run {
        println!("[dry-run] repo automation scaffold");
    } else {
        println!("[ok] repo automation scaffold");
    }
    println!("repo_root: {}", report.repo_root);
    println!("default_branch: {}", report.default_branch);
    println!(
        "pr_required_checks: {}",
        if report.pr_required_checks.is_empty() {
            "-".to_string()
        } else {
            report.pr_required_checks.join(",")
        }
    );
    println!(
        "release_required_checks: {}",
        report.release_required_checks.join(",")
    );
    println!("minimum_approvals: {}", report.minimum_approvals);
    println!("managed_files:");
    for path in &report.managed_files {
        println!("- {}", path);
    }
    println!("next_steps:");
    for step in &report.next_steps {
        println!("- {}", step);
    }
    Ok(())
}

fn task_state(args: TaskStateArgs, manifest: &BootstrapManifest) -> Result<()> {
    let home = home_dir()?;
    match args.command {
        TaskStateCommand::Begin(args) => task_state_begin(&home, manifest, args),
        TaskStateCommand::Advance(args) => task_state_advance(&home, args),
        TaskStateCommand::Show(args) => task_state_show(&home, args),
        TaskStateCommand::Clear => {
            clear_task_state(&home)?;
            println!("task state cleared");
            Ok(())
        }
    }
}

fn task_state_begin(
    home: &Path,
    manifest: &BootstrapManifest,
    args: TaskStateBeginArgs,
) -> Result<()> {
    let providers = selected_providers(&args.provider_args, manifest);
    let resolved = resolve_plan(manifest, &args.pack_args)?;
    let state = TaskState {
        id: format!("task_{}", timestamp_string()?),
        title: args.title,
        status: args.status.name().to_string(),
        phase: args.phase.name().to_string(),
        owner: args.owner,
        summary: args.summary,
        checkpoint: args.checkpoint,
        next_action: args.next_action,
        providers: providers
            .iter()
            .map(|provider| provider.name().to_string())
            .collect(),
        packs: resolved.selection.packs,
        harnesses: resolved.selection.harnesses,
        completed_signals: Vec::new(),
        attempt_count: 0,
        last_failure: None,
        investigation_note: None,
        updated_at: timestamp_string()?,
    };
    write_task_state(home, &state)?;
    print_task_state(&state, args.json)?;
    Ok(())
}

fn task_state_advance(home: &Path, args: TaskStateAdvanceArgs) -> Result<()> {
    let has_explicit_next_action = args.next_action.is_some();
    let recorded_failure = args.failure.is_some();
    let recorded_investigation = args.investigation_note.is_some();
    let mut state = read_task_state(home)?.context("no active task state")?;
    if let Some(status) = args.status {
        state.status = status.name().to_string();
    }
    if let Some(phase) = args.phase {
        state.phase = phase.name().to_string();
    }
    if let Some(summary) = args.summary {
        state.summary = Some(summary);
    }
    if args.clear_summary {
        state.summary = None;
    }
    if let Some(checkpoint) = args.checkpoint {
        state.checkpoint = Some(checkpoint);
    }
    if args.clear_checkpoint {
        state.checkpoint = None;
    }
    if let Some(next_action) = args.next_action {
        state.next_action = Some(next_action);
    }
    if let Some(failure) = args.failure {
        state.last_failure = Some(failure);
    }
    if args.clear_failure {
        state.last_failure = None;
    }
    if let Some(note) = args.investigation_note {
        state.investigation_note = Some(note);
        merge_completed_signal_names(&mut state.completed_signals, &[GateSignal::Investigate]);
    }
    if args.clear_investigation {
        state.investigation_note = None;
        remove_completed_signal_names(&mut state.completed_signals, &[GateSignal::Investigate]);
    }
    merge_completed_signal_names(&mut state.completed_signals, &args.complete);
    remove_completed_signal_names(&mut state.completed_signals, &args.clear_complete);
    if args.increment_attempt {
        state.attempt_count += 1;
    }
    if !has_explicit_next_action
        && (recorded_failure || args.increment_attempt || recorded_investigation)
    {
        state.next_action = retry_next_action(&state);
    }
    state.updated_at = timestamp_string()?;
    write_task_state(home, &state)?;
    print_task_state(&state, args.json)?;
    Ok(())
}

fn task_state_show(home: &Path, args: TaskStateShowArgs) -> Result<()> {
    let state = read_task_state(home)?;
    if let Some(state) = state {
        print_task_state(&state, args.json)?;
    } else if args.json {
        println!("null");
    } else {
        println!("no active task state");
    }
    Ok(())
}

fn print_task_state(state: &TaskState, json: bool) -> Result<()> {
    if json {
        let value = serde_json::json!({
            "id": state.id,
            "title": state.title,
            "status": state.status,
            "phase": state.phase,
            "owner": state.owner,
            "summary": state.summary,
            "checkpoint": state.checkpoint,
            "next_action": state.next_action,
            "providers": state.providers,
            "packs": state.packs,
            "harnesses": state.harnesses,
            "completed_signals": state.completed_signals,
            "attempt_count": state.attempt_count,
            "last_failure": state.last_failure,
            "investigation_note": state.investigation_note,
            "updated_at": state.updated_at,
        });
        println!("{}", serde_json::to_string_pretty(&value)?);
    } else {
        println!("id: {}", state.id);
        println!("title: {}", state.title);
        println!("status: {}", state.status);
        println!("phase: {}", state.phase);
        println!("owner: {}", state.owner.as_deref().unwrap_or(""));
        println!(
            "providers: {}",
            if state.providers.is_empty() {
                "-".to_string()
            } else {
                state.providers.join(",")
            }
        );
        println!(
            "packs: {}",
            if state.packs.is_empty() {
                "-".to_string()
            } else {
                state.packs.join(",")
            }
        );
        println!(
            "harnesses: {}",
            if state.harnesses.is_empty() {
                "-".to_string()
            } else {
                state.harnesses.join(",")
            }
        );
        println!(
            "completed_signals: {}",
            if state.completed_signals.is_empty() {
                "-".to_string()
            } else {
                state.completed_signals.join(",")
            }
        );
        println!("attempt_count: {}", state.attempt_count);
        println!("summary: {}", state.summary.as_deref().unwrap_or(""));
        println!("checkpoint: {}", state.checkpoint.as_deref().unwrap_or(""));
        println!(
            "next_action: {}",
            state.next_action.as_deref().unwrap_or("")
        );
        println!(
            "last_failure: {}",
            state.last_failure.as_deref().unwrap_or("")
        );
        println!(
            "investigation_note: {}",
            state.investigation_note.as_deref().unwrap_or("")
        );
        println!("updated_at: {}", state.updated_at);
    }
    Ok(())
}

#[derive(Serialize)]
struct GateReport {
    allowed: bool,
    task_id: String,
    title: String,
    current_phase: String,
    target_phase: String,
    status: String,
    active_harnesses: Vec<String>,
    completed_signals: Vec<String>,
    required_signals: Vec<String>,
    missing_signals: Vec<String>,
    reasons: Vec<String>,
    attempt_count: u64,
    last_failure: Option<String>,
    investigation_note: Option<String>,
    retry_status: String,
    recommended_status: String,
    recommended_next_action: Option<String>,
}

fn gate(args: GateArgs) -> Result<()> {
    let home = home_dir()?;
    match args.command {
        GateCommand::Check(args) => gate_check(&home, args),
        GateCommand::Apply(args) => gate_apply(&home, args),
    }
}

fn gate_check(home: &Path, args: GateCheckArgs) -> Result<()> {
    let state = read_task_state(home)?.context("no active task state")?;
    let target_phase = args.target_phase.unwrap_or(parse_task_phase(&state.phase)?);
    let report = evaluate_gate(&state, target_phase, &args.completed);
    print_gate_report(&report, args.json)?;
    if report.allowed {
        Ok(())
    } else {
        bail!("gate contract blocked")
    }
}

fn gate_apply(home: &Path, args: GateApplyArgs) -> Result<()> {
    let mut state = read_task_state(home)?.context("no active task state")?;
    merge_completed_signal_names(&mut state.completed_signals, &args.completed);
    let target_phase = args.target_phase.unwrap_or(parse_task_phase(&state.phase)?);
    let report = evaluate_gate(&state, target_phase, &[]);
    if report.allowed {
        if let Some(target_phase) = args.target_phase {
            state.phase = target_phase.name().to_string();
        }
        state.status = match target_phase {
            TaskPhase::Ship => TaskStatus::Ready.name().to_string(),
            _ if state.status == TaskStatus::Blocked.name()
                || state.status == TaskStatus::Draft.name() =>
            {
                TaskStatus::InProgress.name().to_string()
            }
            _ => state.status.clone(),
        };
        if state.next_action.as_deref() == report.recommended_next_action.as_deref() {
            state.next_action = None;
        }
    } else {
        state.status = TaskStatus::Blocked.name().to_string();
        state.next_action = report.recommended_next_action.clone();
    }
    state.updated_at = timestamp_string()?;
    write_task_state(home, &state)?;
    let final_report = evaluate_gate(&state, target_phase, &[]);
    print_gate_report(&final_report, args.json)?;
    if final_report.allowed {
        Ok(())
    } else {
        bail!("gate contract blocked")
    }
}

fn evaluate_gate(
    state: &TaskState,
    target_phase: TaskPhase,
    additional_completed: &[GateSignal],
) -> GateReport {
    let mut completed = BTreeSet::new();
    completed.extend(state.completed_signals.iter().cloned());
    if state.investigation_note.is_some() {
        completed.insert(GateSignal::Investigate.name().to_string());
    }
    completed.extend(
        additional_completed
            .iter()
            .map(|signal| signal.name().to_string()),
    );

    let mut required = BTreeSet::new();
    let mut reasons = Vec::new();
    let parallel_build_enabled = state.harnesses.iter().any(|name| name == "parallel-build");
    let review_gate_enabled = state.harnesses.iter().any(|name| name == "review-gate");
    let retry_escalated = state.attempt_count >= 2
        && state.last_failure.is_some()
        && matches!(
            target_phase,
            TaskPhase::Execute | TaskPhase::Review | TaskPhase::Qa | TaskPhase::Ship
        );

    if matches!(target_phase, TaskPhase::Plan) {
        required.insert(GateSignal::Spec.name().to_string());
        reasons.push(
            "phase-gate requires a spec checkpoint before planning moves forward".to_string(),
        );
    }

    if matches!(target_phase, TaskPhase::Execute) {
        required.insert(GateSignal::Plan.name().to_string());
        reasons.push("phase-gate requires a plan checkpoint before execution starts".to_string());
    }

    if parallel_build_enabled
        && matches!(
            target_phase,
            TaskPhase::Execute | TaskPhase::Review | TaskPhase::Qa | TaskPhase::Ship
        )
    {
        required.insert(GateSignal::Ownership.name().to_string());
        reasons.push(
            "parallel-build requires explicit ownership before execution moves forward".to_string(),
        );
    }

    if parallel_build_enabled
        && matches!(
            target_phase,
            TaskPhase::Review | TaskPhase::Qa | TaskPhase::Ship
        )
    {
        required.insert(GateSignal::Handoff.name().to_string());
        reasons.push("parallel-build requires a handoff record before review or ship".to_string());
    }

    if review_gate_enabled && matches!(target_phase, TaskPhase::Ship) {
        required.insert(GateSignal::Review.name().to_string());
        required.insert(GateSignal::Qa.name().to_string());
        required.insert(GateSignal::Verify.name().to_string());
        reasons.push("review-gate requires review, qa, and verification before ship".to_string());
    }

    if retry_escalated {
        required.insert(GateSignal::Investigate.name().to_string());
        reasons.push(
            "ralph-retry requires investigation evidence after repeated failed attempts"
                .to_string(),
        );
    }

    let required_signals = required.into_iter().collect::<Vec<_>>();
    let completed_signals = completed.into_iter().collect::<Vec<_>>();
    let missing_signals = required_signals
        .iter()
        .filter(|signal| !completed_signals.contains(signal))
        .cloned()
        .collect::<Vec<_>>();
    let allowed = missing_signals.is_empty();
    let retry_status = retry_status(state).to_string();
    let recommended_status = if allowed {
        if target_phase == TaskPhase::Ship {
            TaskStatus::Ready.name().to_string()
        } else if state.status == TaskStatus::Blocked.name() {
            TaskStatus::InProgress.name().to_string()
        } else {
            state.status.clone()
        }
    } else {
        TaskStatus::Blocked.name().to_string()
    };
    let recommended_next_action = if allowed
        || (retry_status == "investigate"
            && missing_signals == [GateSignal::Investigate.name().to_string()])
    {
        retry_next_action(state)
    } else {
        None
    };
    let recommended_next_action = if recommended_next_action.is_some() {
        recommended_next_action
    } else if allowed {
        None
    } else {
        Some(format!(
            "complete gate signals: {}",
            missing_signals.join(", ")
        ))
    };

    GateReport {
        allowed,
        task_id: state.id.clone(),
        title: state.title.clone(),
        current_phase: state.phase.clone(),
        target_phase: target_phase.name().to_string(),
        status: state.status.clone(),
        active_harnesses: state.harnesses.clone(),
        completed_signals,
        required_signals,
        missing_signals,
        reasons,
        attempt_count: state.attempt_count,
        last_failure: state.last_failure.clone(),
        investigation_note: state.investigation_note.clone(),
        retry_status,
        recommended_status,
        recommended_next_action,
    }
}

fn print_gate_report(report: &GateReport, json: bool) -> Result<()> {
    if json {
        println!("{}", serde_json::to_string_pretty(report)?);
        return Ok(());
    }

    println!("allowed: {}", report.allowed);
    println!("task_id: {}", report.task_id);
    println!("title: {}", report.title);
    println!("current_phase: {}", report.current_phase);
    println!("target_phase: {}", report.target_phase);
    println!("status: {}", report.status);
    println!(
        "active_harnesses: {}",
        if report.active_harnesses.is_empty() {
            "-".to_string()
        } else {
            report.active_harnesses.join(",")
        }
    );
    println!(
        "completed_signals: {}",
        if report.completed_signals.is_empty() {
            "-".to_string()
        } else {
            report.completed_signals.join(",")
        }
    );
    println!(
        "required_signals: {}",
        if report.required_signals.is_empty() {
            "-".to_string()
        } else {
            report.required_signals.join(",")
        }
    );
    println!(
        "missing_signals: {}",
        if report.missing_signals.is_empty() {
            "-".to_string()
        } else {
            report.missing_signals.join(",")
        }
    );
    println!("attempt_count: {}", report.attempt_count);
    println!("retry_status: {}", report.retry_status);
    println!(
        "last_failure: {}",
        report.last_failure.as_deref().unwrap_or("")
    );
    println!(
        "investigation_note: {}",
        report.investigation_note.as_deref().unwrap_or("")
    );
    println!("recommended_status: {}", report.recommended_status);
    println!(
        "recommended_next_action: {}",
        report.recommended_next_action.as_deref().unwrap_or("")
    );
    if !report.reasons.is_empty() {
        println!("reasons:");
        for reason in &report.reasons {
            println!("- {}", reason);
        }
    }
    Ok(())
}

fn parse_task_phase(phase: &str) -> Result<TaskPhase> {
    match phase {
        "discover" => Ok(TaskPhase::Discover),
        "plan" => Ok(TaskPhase::Plan),
        "execute" => Ok(TaskPhase::Execute),
        "review" => Ok(TaskPhase::Review),
        "qa" => Ok(TaskPhase::Qa),
        "ship" => Ok(TaskPhase::Ship),
        "operate" => Ok(TaskPhase::Operate),
        other => bail!("unknown task phase '{}'", other),
    }
}

fn retry_status(state: &TaskState) -> &'static str {
    if state.last_failure.is_none() {
        "clean"
    } else if state.attempt_count >= 2 {
        if state.investigation_note.is_some() {
            "investigated"
        } else {
            "investigate"
        }
    } else {
        "targeted-retry"
    }
}

fn retry_next_action(state: &TaskState) -> Option<String> {
    match retry_status(state) {
        "clean" => None,
        "targeted-retry" => Some(
            "apply one bounded fix, rerun verification, then clear or replace the recorded failure"
                .to_string(),
        ),
        "investigate" => Some(
            "capture investigation evidence or move the task back to plan before another retry"
                .to_string(),
        ),
        "investigated" => Some(
            "rerun verification with the investigation note attached, or move the task back to plan if the fix is no longer bounded"
                .to_string(),
        ),
        _ => None,
    }
}

fn merge_completed_signal_names(completed: &mut Vec<String>, additions: &[GateSignal]) {
    if additions.is_empty() {
        return;
    }
    let mut merged = completed.iter().cloned().collect::<BTreeSet<_>>();
    merged.extend(additions.iter().map(|signal| signal.name().to_string()));
    *completed = merged.into_iter().collect();
}

fn remove_completed_signal_names(completed: &mut Vec<String>, removals: &[GateSignal]) {
    if removals.is_empty() {
        return;
    }
    let removal_names = removals
        .iter()
        .map(|signal| signal.name())
        .collect::<BTreeSet<_>>();
    completed.retain(|name| !removal_names.contains(name.as_str()));
}

fn workflow_gate_contract_lines(harnesses: &[String]) -> Vec<String> {
    let mut lines = vec![
        "phase-gate: plan requires spec".to_string(),
        "phase-gate: execute requires plan".to_string(),
        "ralph-retry: after 2 failed attempts, execute/review/qa/ship require investigate"
            .to_string(),
    ];
    let has_parallel = harnesses.iter().any(|name| name == "parallel-build");
    let has_review = harnesses.iter().any(|name| name == "review-gate");

    if has_parallel {
        lines.push(
            "parallel-build: execute/review/qa/ship before handoff require ownership".to_string(),
        );
        lines.push("parallel-build: review/qa/ship require handoff".to_string());
    }
    if has_review {
        lines.push("review-gate: ship requires review, qa, verify".to_string());
    }

    lines
}

fn print_workflow_gate_summary(harnesses: &[String]) {
    let lines = workflow_gate_contract_lines(harnesses);
    if lines.is_empty() {
        return;
    }

    println!("workflow_gates:");
    for line in &lines {
        println!("- {}", line);
    }
    println!("gate_commands:");
    println!("- stack-pilot internal gate check --target-phase plan|execute|review|qa|ship --json");
    println!(
        "- stack-pilot internal task-state advance --complete spec,plan,ownership,handoff,review,qa,verify"
    );
    println!("- stack-pilot internal task-state advance --increment-attempt --failure \"...\"");
    println!("- stack-pilot internal gate apply --target-phase ship --json");
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
    record_surface: Option<RecordSurface>,
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
        &resolved.surfaces,
        record_surface.unwrap_or(RecordSurface::Both),
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
        let runtime_check = provider_runtime_check(*provider);
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
        let requested_record_surface = record_surface.map(RecordSurface::name);
        let requested_state = RequestedState {
            active_preset: resolved.selection.preset.as_deref(),
            record_surface: requested_record_surface,
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
                    "[warn] installed state differs: preset={}, record_surface={}, packs={}",
                    installed_state
                        .active_preset
                        .as_deref()
                        .unwrap_or("unknown"),
                    installed_state
                        .record_surface
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
        if !json {
            let label = match runtime_check.status.as_str() {
                "ok" => "ok",
                _ => "missing",
            };
            if let Some(detail) = runtime_check.detail.as_deref() {
                println!("[{}] runtime {} ({})", label, runtime_check.target, detail);
            } else {
                println!("[{}] runtime {}", label, runtime_check.target);
            }
        }
        if runtime_check.status != "ok" {
            failures.push(PathBuf::from(runtime_check.target.clone()));
        }
        provider_checks.push(runtime_check);

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
            installed_record_surface: installed_state.record_surface,
            installed_packs: installed_state.active_packs,
            installed_harnesses: installed_state.active_harnesses,
            installed_connectors: installed_state.active_connectors,
            installed_automations: installed_state.active_automations,
            installed_record_templates: installed_state.active_record_templates,
            installed_surfaces: installed_state.active_surfaces,
            installed_managed_paths: installed_state.managed_paths,
            requested_record_surface: requested_record_surface.map(str::to_string),
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
        for surface in &catalog_report.surfaces {
            println!(
                "[ok] surface {} (kind: {}, owner: {}, lane: {}, target: {}, active: {}, packs: {})",
                surface.name,
                surface.kind,
                surface.runtime_owner,
                surface.default_lane,
                surface.distribution_target,
                if surface.active { "yes" } else { "no" },
                if surface.selected_by_packs.is_empty() {
                    "-".to_string()
                } else {
                    surface.selected_by_packs.join(",")
                }
            );
        }
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

    println!("stack-pilot wizard");
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
        "CLI shells (~/.zshrc + ~/.zshrc.d/stackpilot-env.zsh)",
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
    print_workflow_gate_summary(&resolved.selection.harnesses);

    if apply_now {
        ensure_runtime_dependencies(rtk_enabled)?;
        let home = home_dir()?;
        install_with(
            &home,
            &providers,
            manifest,
            mode,
            rtk_enabled,
            record_surface,
            &resolved,
        )?;
        doctor_with(
            &home,
            &providers,
            manifest,
            rtk_enabled,
            false,
            Some(record_surface),
            &resolved,
        )?;
    }

    Ok(())
}

fn install_with(
    home: &std::path::Path,
    providers: &[Provider],
    manifest: &BootstrapManifest,
    mode: cli::ApplyMode,
    rtk_enabled: bool,
    record_surface: RecordSurface,
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
                record_surface: Some(record_surface.name().to_string()),
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
        "installed providers: {} (mode: {}, rtk: {}, preset: {}, packs: {}, record_surface: {}, requested_targets: {}, targets: {})",
        provider_names(providers),
        mode.name(),
        if rtk_enabled { "enabled" } else { "disabled" },
        resolved.selection.preset.as_deref().unwrap_or("custom"),
        resolved.selection.packs.join(","),
        record_surface.name(),
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

    let backups_dir = home.join(".stackpilot-legacy-backups");
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
    record_surface: RecordSurface,
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
    print_workflow_gate_summary(&resolved.selection.harnesses);
    println!("record_surface: {}", record_surface.name());
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
    installed_record_surface: Option<String>,
    installed_packs: Vec<String>,
    installed_harnesses: Vec<String>,
    installed_connectors: Vec<String>,
    installed_automations: Vec<String>,
    installed_record_templates: Vec<String>,
    installed_surfaces: Vec<String>,
    installed_managed_paths: Vec<String>,
    requested_record_surface: Option<String>,
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
struct ProbeReport {
    ok: bool,
    providers: Vec<ProbeProviderReport>,
}

#[derive(Serialize)]
struct ProbeProviderReport {
    provider: String,
    requested_surfaces: Vec<String>,
    files: Vec<ProbeFileCheck>,
    runtime: ProbeRuntimeResult,
    optimizations: Vec<ProbeRuntimeResult>,
}

#[derive(Serialize)]
struct ProbeFileCheck {
    target: String,
    status: String,
}

#[derive(Serialize)]
struct ProbeRuntimeResult {
    status: String,
    command: String,
    stdout: Option<String>,
    stderr: Option<String>,
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
    surfaces: Vec<DoctorSurfaceReport>,
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
    active_runtime_automation_count: usize,
    active_repo_automation_count: usize,
    pending_scheduler_registration_count: usize,
    pending_repo_registration_count: usize,
    connector_queue: Vec<String>,
    automation_queue: Vec<String>,
    repo_automation_queue: Vec<String>,
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
    lane: String,
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
struct DoctorSurfaceReport {
    name: String,
    kind: String,
    runtime_owner: String,
    default_lane: String,
    distribution_target: String,
    active: bool,
    selected_by_packs: Vec<String>,
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
    surfaces: &ProviderSurfaces,
    record_surface: RecordSurface,
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
        surfaces: manifest
            .surfaces
            .iter()
            .map(|surface| DoctorSurfaceReport {
                name: surface.name.clone(),
                kind: surface.kind.name().to_string(),
                runtime_owner: surface.runtime_owner.name().to_string(),
                default_lane: surface.default_lane.name().to_string(),
                distribution_target: surface.distribution_target.name().to_string(),
                active: provider_surfaces_for_target(surface.distribution_target, surfaces)
                    .iter()
                    .any(|active| active == &surface.name),
                selected_by_packs: selected_packs_for_surface(
                    manifest,
                    &selection.packs,
                    surface.distribution_target,
                    &surface.name,
                ),
                description: surface.description.clone(),
            })
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
                lane: automation.lane.name().to_string(),
                packs: automation.packs.clone(),
                connectors: automation.connectors.clone(),
                artifact: automation.artifact.clone(),
                active: active_automations.contains(&automation.name),
                status: automation_status(automation, &active_automations).to_string(),
                scheduler_owner: automation_scheduler_owner(automation).to_string(),
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
            record_surface,
            &active_connectors,
            &active_record_templates,
        ),
    }
}

fn doctor_record_readiness_report(
    record_surface: RecordSurface,
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
        record_system: record_surface.name().to_string(),
        runtime_owner: "bootstrap-contracts+external-tools".to_string(),
        active_templates: active_record_templates.to_vec(),
        missing_handoffs,
        next_action,
    }
}

fn provider_runtime_check(provider: Provider) -> DoctorCheck {
    match provider {
        Provider::Codex => {
            if command_exists("codex") {
                DoctorCheck {
                    target: "codex".to_string(),
                    status: "ok".to_string(),
                    detail: Some("runtime=cli".to_string()),
                }
            } else if codex_app_exists() {
                DoctorCheck {
                    target: "/Applications/Codex.app".to_string(),
                    status: "ok".to_string(),
                    detail: Some("runtime=app".to_string()),
                }
            } else {
                DoctorCheck {
                    target: "codex or /Applications/Codex.app".to_string(),
                    status: "missing".to_string(),
                    detail: Some("install Codex CLI or Codex app".to_string()),
                }
            }
        }
        Provider::Gemini => {
            required_runtime_check("gemini", "Gemini CLI is required", Some("runtime=cli"))
        }
        Provider::Claude => {
            required_runtime_check("claude", "Claude Code CLI is required", Some("runtime=cli"))
        }
    }
}

fn provider_probe_paths(home: &Path, provider: Provider, resolved: &ResolvedPlan) -> Vec<PathBuf> {
    let root = provider_root(home, provider);
    let paths = match provider {
        Provider::Codex => {
            let mut paths = vec![root.join("AGENTS.md"), root.join("config.toml")];
            if resolved
                .distribution_state
                .enabled(DistributionTarget::CodexPlugin)
            {
                paths.push(root.join("plugins/stackpilot-dev-kit/.codex-plugin/plugin.json"));
            }
            if resolved
                .surfaces
                .codex
                .iter()
                .any(|surface| surface == "delivery-skills")
            {
                paths.push(root.join("WORKFLOW.md"));
                paths.push(root.join("plugins/stackpilot-dev-kit/skills/autopilot/SKILL.md"));
                paths.push(root.join("plugins/stackpilot-dev-kit/skills/deep-init/SKILL.md"));
                paths.push(root.join("plugins/stackpilot-dev-kit/skills/workflow-gate/SKILL.md"));
            }
            if resolved
                .surfaces
                .codex
                .iter()
                .any(|surface| surface == "team-skills")
            {
                paths.push(root.join("TEAM.md"));
                paths.push(root.join("plugins/stackpilot-dev-kit/skills/team/SKILL.md"));
                paths.push(root.join("plugins/stackpilot-dev-kit/skills/ultrawork/SKILL.md"));
                paths.push(root.join("plugins/stackpilot-dev-kit/skills/workflow-gate/SKILL.md"));
            }
            if resolved
                .surfaces
                .codex
                .iter()
                .any(|surface| surface == "review-automation-skills")
            {
                paths.push(root.join("REVIEW_AUTOMATION.md"));
                paths.push(
                    root.join("plugins/stackpilot-dev-kit/skills/review-automation/SKILL.md"),
                );
            }
            if resolved
                .surfaces
                .codex
                .iter()
                .any(|surface| surface == "incident-skills")
            {
                paths.push(root.join("plugins/stackpilot-dev-kit/skills/investigate/SKILL.md"));
                paths.push(root.join("plugins/stackpilot-dev-kit/skills/workflow-gate/SKILL.md"));
            }
            if resolved
                .surfaces
                .codex
                .iter()
                .any(|surface| surface == "company-skills")
            {
                paths.push(root.join("FOUNDER_LOOP.md"));
            }
            paths
        }
        Provider::Gemini => {
            let mut paths = vec![root.join("GEMINI.md"), root.join("settings.json")];
            if resolved
                .distribution_state
                .enabled(DistributionTarget::GeminiExtension)
            {
                paths.push(root.join("extensions/stackpilot-dev/gemini-extension.json"));
            }
            if resolved
                .surfaces
                .gemini
                .iter()
                .any(|surface| surface == "delivery-commands")
            {
                paths.push(root.join("WORKFLOW.md"));
                paths.push(root.join("extensions/stackpilot-dev/commands/autopilot.toml"));
                paths.push(root.join("extensions/stackpilot-dev/commands/deep-init.toml"));
                paths.push(root.join("extensions/stackpilot-dev/commands/gate.toml"));
            }
            if resolved
                .surfaces
                .gemini
                .iter()
                .any(|surface| surface == "team-commands")
            {
                paths.push(root.join("TEAM.md"));
                paths.push(root.join("extensions/stackpilot-dev/commands/team.toml"));
                paths.push(root.join("extensions/stackpilot-dev/commands/ultrawork.toml"));
                paths.push(root.join("extensions/stackpilot-dev/commands/gate.toml"));
            }
            if resolved
                .surfaces
                .gemini
                .iter()
                .any(|surface| surface == "review-automation-commands")
            {
                paths.push(root.join("REVIEW_AUTOMATION.md"));
                paths.push(root.join("extensions/stackpilot-dev/commands/review-automation.toml"));
            }
            if resolved
                .surfaces
                .gemini
                .iter()
                .any(|surface| surface == "incident-commands")
            {
                paths.push(root.join("extensions/stackpilot-dev/agents/triage.md"));
                paths.push(root.join("extensions/stackpilot-dev/commands/gate.toml"));
            }
            if resolved
                .surfaces
                .gemini
                .iter()
                .any(|surface| surface == "company-commands")
            {
                paths.push(root.join("extensions/stackpilot-dev/commands/operating-review.toml"));
            }
            paths
        }
        Provider::Claude => {
            let mut paths = vec![root.join("CLAUDE.md"), root.join("agents/planner.md")];
            if resolved
                .surfaces
                .claude
                .iter()
                .any(|surface| surface == "delivery-skills")
            {
                paths.push(root.join("WORKFLOW.md"));
                paths.push(root.join("skills/autopilot/SKILL.md"));
                paths.push(root.join("skills/deep-init/SKILL.md"));
                paths.push(root.join("skills/workflow-gate/SKILL.md"));
            }
            if resolved
                .surfaces
                .claude
                .iter()
                .any(|surface| surface == "team-skills")
            {
                paths.push(root.join("TEAM.md"));
                paths.push(root.join("skills/team/SKILL.md"));
                paths.push(root.join("skills/ultrawork/SKILL.md"));
                paths.push(root.join("skills/workflow-gate/SKILL.md"));
            }
            if resolved
                .surfaces
                .claude
                .iter()
                .any(|surface| surface == "review-automation-skills")
            {
                paths.push(root.join("REVIEW_AUTOMATION.md"));
                paths.push(root.join("skills/review-automation/SKILL.md"));
            }
            if resolved
                .surfaces
                .claude
                .iter()
                .any(|surface| surface == "incident-skills")
            {
                paths.push(root.join("skills/investigate/SKILL.md"));
                paths.push(root.join("skills/workflow-gate/SKILL.md"));
            }
            if resolved
                .surfaces
                .claude
                .iter()
                .any(|surface| surface == "company-skills")
            {
                paths.push(root.join("FOUNDER_LOOP.md"));
            }
            paths
        }
    };
    dedupe_ordered(paths)
}

fn dedupe_ordered<T>(items: Vec<T>) -> Vec<T>
where
    T: Eq + Hash + Clone,
{
    let mut seen = IndexSet::new();
    let mut unique = Vec::new();
    for item in items {
        if seen.insert(item.clone()) {
            unique.push(item);
        }
    }
    unique
}

fn run_provider_probe(home: &Path, provider: Provider, prompt: &str) -> Result<ProbeRuntimeResult> {
    run_provider_probe_attempts(home, provider, provider_probe_attempts(provider, prompt))
}

fn run_provider_optimization_probes(
    home: &Path,
    provider: Provider,
) -> Result<Vec<ProbeRuntimeResult>> {
    match provider {
        Provider::Codex => Ok(vec![run_provider_probe_attempts(
            home,
            provider,
            codex_long_context_probe_attempts(),
        )?]),
        Provider::Claude => {
            let mut results = Vec::new();
            for attempt in claude_1m_probe_attempts() {
                results.push(run_provider_probe_attempts(home, provider, vec![attempt])?);
            }
            Ok(results)
        }
        Provider::Gemini => Ok(Vec::new()),
    }
}

fn run_provider_probe_attempts(
    home: &Path,
    provider: Provider,
    attempts: Vec<(String, Vec<String>)>,
) -> Result<ProbeRuntimeResult> {
    let mut last_failure = None;

    for (command, args) in attempts {
        let output = ProcessCommand::new(&command)
            .env("HOME", home)
            .args(&args)
            .output()
            .with_context(|| format!("failed while probing {}", provider.name()))?;
        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        let display_command = format!("{} {}", command, args.join(" "));

        if output.status.success() {
            if stdout == "OK" {
                return Ok(ProbeRuntimeResult {
                    status: "ok".to_string(),
                    command: display_command,
                    stdout: Some(stdout),
                    stderr: None,
                    detail: None,
                });
            }
            last_failure = Some(ProbeRuntimeResult {
                status: "unexpected-output".to_string(),
                command: display_command,
                stdout: Some(stdout),
                stderr: if stderr.is_empty() {
                    None
                } else {
                    Some(stderr)
                },
                detail: Some("runtime responded but did not return exact OK".to_string()),
            });
            continue;
        }

        last_failure = Some(ProbeRuntimeResult {
            status: "failed".to_string(),
            command: display_command,
            stdout: if stdout.is_empty() {
                None
            } else {
                Some(stdout)
            },
            stderr: if stderr.is_empty() {
                None
            } else {
                Some(stderr)
            },
            detail: Some("runtime command exited non-zero".to_string()),
        });
    }

    Ok(last_failure.unwrap_or(ProbeRuntimeResult {
        status: "failed".to_string(),
        command: provider.name().to_string(),
        stdout: None,
        stderr: None,
        detail: Some("no runtime probe attempt available".to_string()),
    }))
}

fn codex_long_context_probe_attempts() -> Vec<(String, Vec<String>)> {
    vec![(
        "codex".to_string(),
        vec![
            "exec".to_string(),
            "-c".to_string(),
            "model=\"gpt-5.5\"".to_string(),
            "-c".to_string(),
            "model_context_window=1000000".to_string(),
            "-c".to_string(),
            "model_auto_compact_token_limit=900000".to_string(),
            "--skip-git-repo-check".to_string(),
            "--ephemeral".to_string(),
            "Reply with exactly OK and nothing else.".to_string(),
        ],
    )]
}

fn claude_1m_probe_attempts() -> Vec<(String, Vec<String>)> {
    [("opus[1m]", "max"), ("sonnet[1m]", "high")]
        .into_iter()
        .map(|(model, effort)| {
            (
                "claude".to_string(),
                vec![
                    "-p".to_string(),
                    "--model".to_string(),
                    model.to_string(),
                    "--effort".to_string(),
                    effort.to_string(),
                    "Reply with exactly OK and nothing else.".to_string(),
                ],
            )
        })
        .collect()
}

fn provider_probe_attempts(provider: Provider, prompt: &str) -> Vec<(String, Vec<String>)> {
    match provider {
        Provider::Codex => vec![
            (
                "codex".to_string(),
                vec![
                    "exec".to_string(),
                    "--skip-git-repo-check".to_string(),
                    "--ephemeral".to_string(),
                    prompt.to_string(),
                ],
            ),
            (
                "codex".to_string(),
                vec![
                    "exec".to_string(),
                    "--ephemeral".to_string(),
                    prompt.to_string(),
                ],
            ),
            (
                "codex".to_string(),
                vec!["exec".to_string(), prompt.to_string()],
            ),
        ],
        Provider::Gemini => vec![(
            "gemini".to_string(),
            vec!["-p".to_string(), prompt.to_string()],
        )],
        Provider::Claude => vec![(
            "claude".to_string(),
            vec!["-p".to_string(), prompt.to_string()],
        )],
    }
}

fn required_runtime_check(
    target: &str,
    missing_detail: &str,
    ok_detail: Option<&str>,
) -> DoctorCheck {
    if command_exists(target) {
        DoctorCheck {
            target: target.to_string(),
            status: "ok".to_string(),
            detail: ok_detail.map(str::to_string),
        }
    } else {
        DoctorCheck {
            target: target.to_string(),
            status: "missing".to_string(),
            detail: Some(missing_detail.to_string()),
        }
    }
}

fn codex_app_exists() -> bool {
    Path::new("/Applications/Codex.app").exists()
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
        .filter(|automation| automation.lane == manifest::AutomationLane::RuntimeScheduler)
        .map(|automation| automation.name.clone())
        .collect::<Vec<_>>();
    let repo_automation_queue = manifest
        .automations
        .iter()
        .filter(|automation| active_automations.contains(&automation.name))
        .filter(|automation| automation.lane == manifest::AutomationLane::RepoAutomation)
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
    if !repo_automation_queue.is_empty() {
        next_steps.push(
            "register active review automation contracts in the target repository workflow or branch protection lane before expecting PR gates"
                .to_string(),
        );
    }
    if next_steps.is_empty() {
        next_steps.push("no runtime handoff work is pending for the active preset".to_string());
    }

    DoctorRuntimeHandoffReport {
        active_app_connector_count: connector_queue.len(),
        pending_app_verification_count: connector_queue.len(),
        active_automation_count: automation_queue.len() + repo_automation_queue.len(),
        active_runtime_automation_count: automation_queue.len(),
        active_repo_automation_count: repo_automation_queue.len(),
        pending_scheduler_registration_count: automation_queue.len(),
        pending_repo_registration_count: repo_automation_queue.len(),
        connector_queue,
        automation_queue,
        repo_automation_queue,
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

fn automation_scheduler_owner(automation: &manifest::AutomationDefinition) -> &'static str {
    match automation.lane {
        manifest::AutomationLane::RuntimeScheduler => "runtime-managed",
        manifest::AutomationLane::RepoAutomation => "repo-managed",
    }
}

fn automation_registration_status(
    automation: &manifest::AutomationDefinition,
    active_automations: &[String],
) -> &'static str {
    if !active_automations.contains(&automation.name) {
        return "not-requested";
    }

    match automation.lane {
        manifest::AutomationLane::RuntimeScheduler => "not-registered",
        manifest::AutomationLane::RepoAutomation => "not-configured",
    }
}

fn automation_next_step(
    automation: &manifest::AutomationDefinition,
    active_automations: &[String],
) -> Option<String> {
    if !active_automations.contains(&automation.name) {
        return None;
    }

    Some(match automation.lane {
        manifest::AutomationLane::RuntimeScheduler => format!(
            "register {} in the target runtime scheduler if you want recurring execution",
            automation.name
        ),
        manifest::AutomationLane::RepoAutomation => format!(
            "register {} in the target repository workflow or branch protection lane before expecting review gates",
            automation.name
        ),
    })
}

fn automation_detail(
    automation: &manifest::AutomationDefinition,
    active_automations: &[String],
) -> Option<String> {
    if !active_automations.contains(&automation.name) {
        return None;
    }

    Some(match automation.lane {
        manifest::AutomationLane::RuntimeScheduler => {
            "automation contract is rendered into the installed runtime state; recurring scheduler registration stays runtime-managed"
                .to_string()
        }
        manifest::AutomationLane::RepoAutomation => {
            "automation contract is rendered into the installed runtime state; repository workflow or branch protection registration stays external"
                .to_string()
        }
    })
}

fn record_with(args: &RecordArgs, manifest: &BootstrapManifest) -> Result<()> {
    let record_template = manifest
        .record_templates
        .iter()
        .find(|record| record.record_type == args.record_type.record_type());
    let task_state = if args.from_task_state {
        let home = home_dir()?;
        Some(read_task_state(&home)?.context(
            "no active task state to attach; start one with `stack-pilot internal task-state begin ...`",
        )?)
    } else {
        None
    };
    let body = render_record_body(args, record_template, task_state.as_ref())?;
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
    task_state: Option<&TaskState>,
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
    let owner = args
        .owner
        .as_deref()
        .or_else(|| task_state.and_then(|state| state.owner.as_deref()))
        .unwrap_or("");
    let context_summary = task_state
        .and_then(|state| state.summary.as_deref())
        .unwrap_or("");
    let next_action = args
        .next_action
        .as_deref()
        .or_else(|| task_state.and_then(|state| state.next_action.as_deref()))
        .unwrap_or("");
    let task_state_source = if task_state.is_some() {
        "active-task-state"
    } else {
        ""
    };
    let task_state_id = task_state.map(|state| state.id.as_str()).unwrap_or("");
    let task_state_phase = task_state.map(|state| state.phase.as_str()).unwrap_or("");
    let task_state_status = task_state.map(|state| state.status.as_str()).unwrap_or("");
    let task_state_summary = task_state
        .and_then(|state| state.summary.as_deref())
        .unwrap_or("");
    let task_state_checkpoint = task_state
        .and_then(|state| state.checkpoint.as_deref())
        .unwrap_or("");
    let task_state_providers = task_state
        .map(|state| yaml_inline_string_array(&state.providers))
        .unwrap_or_else(|| "[]".to_string());
    let task_state_packs = task_state
        .map(|state| yaml_inline_string_array(&state.packs))
        .unwrap_or_else(|| "[]".to_string());
    let task_state_harnesses = task_state
        .map(|state| yaml_inline_string_array(&state.harnesses))
        .unwrap_or_else(|| "[]".to_string());
    let task_state_completed = task_state
        .map(|state| yaml_inline_string_array(&state.completed_signals))
        .unwrap_or_else(|| "[]".to_string());
    let task_state_attempts = task_state.map(|state| state.attempt_count).unwrap_or(0);
    let task_state_failure = task_state
        .and_then(|state| state.last_failure.as_deref())
        .unwrap_or("");
    let task_state_investigation = task_state
        .and_then(|state| state.investigation_note.as_deref())
        .unwrap_or("");

    Ok(format!(
        "# {title}\n\n```yaml\nid: \"{id}\"\ntype: \"{record_type}\"\ntemplate: \"{template_name}\"\nstage: \"{stage}\"\ntitle: \"{title}\"\nstatus: \"{status}\"\nsource: \"stack-pilot record\"\nowner: \"{owner}\"\nupdated_at: \"{updated_at}\"\nnext_action: \"{next_action}\"\nlinked_tools:\n  github: \"\"\n  linear: \"\"\n  figma: \"\"\n  docs: \"\"\n  calendar: \"\"\n  crm: \"\"\n  helpdesk: \"\"\n  analytics: \"\"\ncontext:\n  summary: \"{context_summary}\"\n  assumptions: []\n  task_state:\n    source: \"{task_state_source}\"\n    id: \"{task_state_id}\"\n    phase: \"{task_state_phase}\"\n    status: \"{task_state_status}\"\n    summary: \"{task_state_summary}\"\n    checkpoint: \"{task_state_checkpoint}\"\n    providers: {task_state_providers}\n    packs: {task_state_packs}\n    harnesses: {task_state_harnesses}\n    completed_signals: {task_state_completed}\n    attempt_count: {task_state_attempts}\n    last_failure: \"{task_state_failure}\"\n    investigation_note: \"{task_state_investigation}\"\ndecision:\n  chosen: \"\"\n  alternatives: []\n  rationale: \"\"\nevidence:\n  links: []\n  notes: []\napprovals:\n  required: false\n  reason: \"\"\n  approver: \"\"\nhandoff:\n  runtime_owner: \"\"\n  external_object_id: \"\"\n  next_step: \"\"\n```\n\n## Description\n\n{description}\n\n## Notes\n\n- Keep this record compact.\n- Link to external source-of-truth systems instead of copying their data.\n- If task-state is attached, keep this record aligned with the active lane before closing the loop.\n- Require approval before customer sends, legal/finance decisions, or external writes.\n",
        title = yaml_string(&args.title),
        id = id,
        record_type = args.record_type.record_type(),
        template_name = yaml_string(template_name),
        stage = yaml_string(stage),
        status = yaml_string(&args.status),
        owner = yaml_string(owner),
        updated_at = updated_at,
        next_action = yaml_string(next_action),
        context_summary = yaml_string(context_summary),
        task_state_source = yaml_string(task_state_source),
        task_state_id = yaml_string(task_state_id),
        task_state_phase = yaml_string(task_state_phase),
        task_state_status = yaml_string(task_state_status),
        task_state_summary = yaml_string(task_state_summary),
        task_state_checkpoint = yaml_string(task_state_checkpoint),
        task_state_providers = task_state_providers,
        task_state_packs = task_state_packs,
        task_state_harnesses = task_state_harnesses,
        task_state_completed = task_state_completed,
        task_state_attempts = task_state_attempts,
        task_state_failure = yaml_string(task_state_failure),
        task_state_investigation = yaml_string(task_state_investigation),
        description = description,
    ))
}

fn yaml_inline_string_array(values: &[String]) -> String {
    if values.is_empty() {
        "[]".to_string()
    } else {
        format!(
            "[{}]",
            values
                .iter()
                .map(|value| format!("\"{}\"", yaml_string(value)))
                .collect::<Vec<_>>()
                .join(", ")
        )
    }
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
                "stackpilot-record-{}-{}.md",
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

fn provider_surfaces_for_target(
    target: DistributionTarget,
    surfaces: &ProviderSurfaces,
) -> &[String] {
    match target {
        DistributionTarget::CodexPlugin => &surfaces.codex,
        DistributionTarget::GeminiExtension => &surfaces.gemini,
        DistributionTarget::ClaudeSkills => &surfaces.claude,
    }
}

fn selected_packs_for_surface(
    manifest: &BootstrapManifest,
    active_packs: &[String],
    target: DistributionTarget,
    surface_name: &str,
) -> Vec<String> {
    manifest
        .packs
        .iter()
        .filter(|pack| active_packs.contains(&pack.name))
        .filter(|pack| match target {
            DistributionTarget::CodexPlugin => {
                pack.codex_surfaces.iter().any(|s| s == surface_name)
            }
            DistributionTarget::GeminiExtension => {
                pack.gemini_surfaces.iter().any(|s| s == surface_name)
            }
            DistributionTarget::ClaudeSkills => {
                pack.claude_surfaces.iter().any(|s| s == surface_name)
            }
        })
        .map(|pack| pack.name.clone())
        .collect()
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
    let mut surface_keys = IndexSet::new();
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
        for surface in &pack.codex_surfaces {
            validate_pack_surface_reference(
                manifest,
                &pack.name,
                DistributionTarget::CodexPlugin,
                surface,
                &mut errors,
            );
        }
        for surface in &pack.gemini_surfaces {
            validate_pack_surface_reference(
                manifest,
                &pack.name,
                DistributionTarget::GeminiExtension,
                surface,
                &mut errors,
            );
        }
        for surface in &pack.claude_surfaces {
            validate_pack_surface_reference(
                manifest,
                &pack.name,
                DistributionTarget::ClaudeSkills,
                surface,
                &mut errors,
            );
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

    for surface in &manifest.surfaces {
        let key = format!("{}:{}", surface.distribution_target.name(), surface.name);
        if !surface_keys.insert(key.clone()) {
            errors.push(format!("duplicate surface: {}", key));
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

fn validate_pack_surface_reference(
    manifest: &BootstrapManifest,
    pack_name: &str,
    target: DistributionTarget,
    surface_name: &str,
    errors: &mut Vec<String>,
) {
    if !manifest
        .surfaces
        .iter()
        .any(|surface| surface.distribution_target == target && surface.name == surface_name)
    {
        errors.push(format!(
            "pack {} references unknown surface {} for {}",
            pack_name,
            surface_name,
            target.name()
        ));
    }
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
    let mut body = String::from("# managed by stackpilot\n");
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

    if existing.contains("stackpilot-env.zsh") || zshrc_has_zshrc_d_loader(&existing) {
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
    use crate::cli::{ApplyMode, GateApplyArgs, GateCheckArgs, GateSignal, TaskPhase};
    use crate::fs_ops::render_tokens;
    use crate::json_ops::{
        cleanup_extension_enablement, merge_json, preserved_gemini_runtime_state,
        prune_rtk_claude_hooks, prune_rtk_gemini_hooks, remove_baseline_mcp_servers,
    };
    use crate::manifest::{
        AutomationCadence, AutomationDefinition, AutomationLane, BaselineMcp, BootstrapManifest,
        BootstrapSection, ConnectorAccess, ConnectorApproval, ConnectorCategory,
        ConnectorDefinition, ConnectorToolSource, DistributionTarget, EnvGatedMcp, ExternalSection,
        HarnessCategory, HarnessDefinition, McpSection, PackDefinition, PackLane, PackScope,
        PresetDefinition, RecordTemplateDefinition, RtkSection, SurfaceDefinition, SurfaceKind,
        SurfaceRuntimeOwner,
    };
    use crate::providers::{claude, codex, gemini};
    use serde_json::json;
    use std::{
        collections::BTreeSet,
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
                    name: "parallel-build".to_string(),
                    category: HarnessCategory::Development,
                    default_enabled: false,
                    description: "Gstack-style parallel build harness with ownership, handoff, review, QA, and verification gates."
                        .to_string(),
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
                    codex_surfaces: vec!["stackpilot-dev-kit".to_string(), "delivery-skills".to_string()],
                    gemini_surfaces: vec![
                        "stackpilot-dev".to_string(),
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
                    codex_surfaces: vec!["stackpilot-dev-kit".to_string(), "incident-skills".to_string()],
                    gemini_surfaces: vec![
                        "stackpilot-dev".to_string(),
                        "incident-commands".to_string(),
                    ],
                    claude_surfaces: vec![
                        "claude-skills".to_string(),
                        "incident-skills".to_string(),
                    ],
                    description: "Incident response pack.".to_string(),
                },
                PackDefinition {
                    name: "team-pack".to_string(),
                    scope: PackScope::Development,
                    lane: PackLane::Advanced,
                    harnesses: vec![
                        "ralph-loop".to_string(),
                        "ralph-plan".to_string(),
                        "delivery".to_string(),
                        "parallel-build".to_string(),
                        "review-gate".to_string(),
                    ],
                    mcp_servers: vec![BaselineMcp::ChromeDevtools, BaselineMcp::Context7],
                    connectors: vec!["github".to_string(), "linear".to_string()],
                    codex_surfaces: vec![
                        "stackpilot-dev-kit".to_string(),
                        "delivery-skills".to_string(),
                        "team-skills".to_string(),
                    ],
                    gemini_surfaces: vec![
                        "stackpilot-dev".to_string(),
                        "delivery-commands".to_string(),
                        "team-commands".to_string(),
                    ],
                    claude_surfaces: vec![
                        "claude-skills".to_string(),
                        "delivery-skills".to_string(),
                        "team-skills".to_string(),
                    ],
                    description: "Advanced gstack-style team delivery pack with decomposition, ownership, handoff, and verification lanes.".to_string(),
                },
                PackDefinition {
                    name: "review-automation-pack".to_string(),
                    scope: PackScope::Development,
                    lane: PackLane::Advanced,
                    harnesses: vec!["delivery".to_string(), "review-gate".to_string()],
                    mcp_servers: vec![BaselineMcp::ChromeDevtools, BaselineMcp::Context7],
                    connectors: vec!["github".to_string(), "linear".to_string()],
                    codex_surfaces: vec![
                        "stackpilot-dev-kit".to_string(),
                        "review-automation-skills".to_string(),
                    ],
                    gemini_surfaces: vec![
                        "stackpilot-dev".to_string(),
                        "review-automation-commands".to_string(),
                    ],
                    claude_surfaces: vec![
                        "claude-skills".to_string(),
                        "review-automation-skills".to_string(),
                    ],
                    description: "Repository automation pack.".to_string(),
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
                    codex_surfaces: vec!["stackpilot-dev-kit".to_string(), "company-skills".to_string()],
                    gemini_surfaces: vec![
                        "stackpilot-dev".to_string(),
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
                    codex_surfaces: vec!["stackpilot-dev-kit".to_string(), "company-skills".to_string()],
                    gemini_surfaces: vec![
                        "stackpilot-dev".to_string(),
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
                    name: "orchestrator".to_string(),
                    packs: vec![
                        "delivery-pack".to_string(),
                        "incident-pack".to_string(),
                        "team-pack".to_string(),
                    ],
                    description: "Development baseline with gstack-style orchestration gates and provider-native team surfaces."
                        .to_string(),
                },
                PresetDefinition {
                    name: "company".to_string(),
                    packs: vec!["founder-pack".to_string(), "ops-pack".to_string()],
                    description: "Company operating packs.".to_string(),
                },
                PresetDefinition {
                    name: "review-automation".to_string(),
                    packs: vec!["review-automation-pack".to_string()],
                    description: "Repository review automation lane.".to_string(),
                },
                PresetDefinition {
                    name: "all-in-one".to_string(),
                    packs: vec![
                        "delivery-pack".to_string(),
                        "incident-pack".to_string(),
                        "team-pack".to_string(),
                        "founder-pack".to_string(),
                        "ops-pack".to_string(),
                        "review-automation-pack".to_string(),
                    ],
                    description:
                        "All-in-one lane with development, orchestration, company, and review automation."
                            .to_string(),
                },
            ],
            surfaces: vec![
                SurfaceDefinition {
                    name: "stackpilot-dev-kit".to_string(),
                    kind: SurfaceKind::Baseline,
                    runtime_owner: SurfaceRuntimeOwner::Bootstrap,
                    default_lane: PackLane::Core,
                    distribution_target: DistributionTarget::CodexPlugin,
                    description: "Codex baseline plugin bundle.".to_string(),
                },
                SurfaceDefinition {
                    name: "delivery-skills".to_string(),
                    kind: SurfaceKind::Entrypoint,
                    runtime_owner: SurfaceRuntimeOwner::ProviderNative,
                    default_lane: PackLane::Core,
                    distribution_target: DistributionTarget::CodexPlugin,
                    description: "Codex delivery entrypoints.".to_string(),
                },
                SurfaceDefinition {
                    name: "incident-skills".to_string(),
                    kind: SurfaceKind::Entrypoint,
                    runtime_owner: SurfaceRuntimeOwner::ProviderNative,
                    default_lane: PackLane::Core,
                    distribution_target: DistributionTarget::CodexPlugin,
                    description: "Codex incident entrypoints.".to_string(),
                },
                SurfaceDefinition {
                    name: "company-skills".to_string(),
                    kind: SurfaceKind::Company,
                    runtime_owner: SurfaceRuntimeOwner::ProviderNative,
                    default_lane: PackLane::Optional,
                    distribution_target: DistributionTarget::CodexPlugin,
                    description: "Codex company entrypoints.".to_string(),
                },
                SurfaceDefinition {
                    name: "team-skills".to_string(),
                    kind: SurfaceKind::Team,
                    runtime_owner: SurfaceRuntimeOwner::ProviderNative,
                    default_lane: PackLane::Advanced,
                    distribution_target: DistributionTarget::CodexPlugin,
                    description: "Codex multi-agent team entrypoints.".to_string(),
                },
                SurfaceDefinition {
                    name: "review-automation-skills".to_string(),
                    kind: SurfaceKind::ReviewAutomation,
                    runtime_owner: SurfaceRuntimeOwner::ExternalRuntime,
                    default_lane: PackLane::Advanced,
                    distribution_target: DistributionTarget::CodexPlugin,
                    description: "Codex review automation entrypoints.".to_string(),
                },
                SurfaceDefinition {
                    name: "stackpilot-dev".to_string(),
                    kind: SurfaceKind::Baseline,
                    runtime_owner: SurfaceRuntimeOwner::Bootstrap,
                    default_lane: PackLane::Core,
                    distribution_target: DistributionTarget::GeminiExtension,
                    description: "Gemini extension baseline.".to_string(),
                },
                SurfaceDefinition {
                    name: "delivery-commands".to_string(),
                    kind: SurfaceKind::Entrypoint,
                    runtime_owner: SurfaceRuntimeOwner::ProviderNative,
                    default_lane: PackLane::Core,
                    distribution_target: DistributionTarget::GeminiExtension,
                    description: "Gemini delivery commands.".to_string(),
                },
                SurfaceDefinition {
                    name: "incident-commands".to_string(),
                    kind: SurfaceKind::Entrypoint,
                    runtime_owner: SurfaceRuntimeOwner::ProviderNative,
                    default_lane: PackLane::Core,
                    distribution_target: DistributionTarget::GeminiExtension,
                    description: "Gemini incident commands.".to_string(),
                },
                SurfaceDefinition {
                    name: "company-commands".to_string(),
                    kind: SurfaceKind::Company,
                    runtime_owner: SurfaceRuntimeOwner::ProviderNative,
                    default_lane: PackLane::Optional,
                    distribution_target: DistributionTarget::GeminiExtension,
                    description: "Gemini company commands.".to_string(),
                },
                SurfaceDefinition {
                    name: "team-commands".to_string(),
                    kind: SurfaceKind::Team,
                    runtime_owner: SurfaceRuntimeOwner::ProviderNative,
                    default_lane: PackLane::Advanced,
                    distribution_target: DistributionTarget::GeminiExtension,
                    description: "Gemini multi-agent team commands.".to_string(),
                },
                SurfaceDefinition {
                    name: "review-automation-commands".to_string(),
                    kind: SurfaceKind::ReviewAutomation,
                    runtime_owner: SurfaceRuntimeOwner::ExternalRuntime,
                    default_lane: PackLane::Advanced,
                    distribution_target: DistributionTarget::GeminiExtension,
                    description: "Gemini review automation commands.".to_string(),
                },
                SurfaceDefinition {
                    name: "claude-skills".to_string(),
                    kind: SurfaceKind::Baseline,
                    runtime_owner: SurfaceRuntimeOwner::Bootstrap,
                    default_lane: PackLane::Core,
                    distribution_target: DistributionTarget::ClaudeSkills,
                    description: "Claude baseline skills.".to_string(),
                },
                SurfaceDefinition {
                    name: "delivery-skills".to_string(),
                    kind: SurfaceKind::Entrypoint,
                    runtime_owner: SurfaceRuntimeOwner::ProviderNative,
                    default_lane: PackLane::Core,
                    distribution_target: DistributionTarget::ClaudeSkills,
                    description: "Claude delivery entrypoints.".to_string(),
                },
                SurfaceDefinition {
                    name: "incident-skills".to_string(),
                    kind: SurfaceKind::Entrypoint,
                    runtime_owner: SurfaceRuntimeOwner::ProviderNative,
                    default_lane: PackLane::Core,
                    distribution_target: DistributionTarget::ClaudeSkills,
                    description: "Claude incident entrypoints.".to_string(),
                },
                SurfaceDefinition {
                    name: "company-skills".to_string(),
                    kind: SurfaceKind::Company,
                    runtime_owner: SurfaceRuntimeOwner::ProviderNative,
                    default_lane: PackLane::Optional,
                    distribution_target: DistributionTarget::ClaudeSkills,
                    description: "Claude company entrypoints.".to_string(),
                },
                SurfaceDefinition {
                    name: "team-skills".to_string(),
                    kind: SurfaceKind::Team,
                    runtime_owner: SurfaceRuntimeOwner::ProviderNative,
                    default_lane: PackLane::Advanced,
                    distribution_target: DistributionTarget::ClaudeSkills,
                    description: "Claude multi-agent team entrypoints.".to_string(),
                },
                SurfaceDefinition {
                    name: "review-automation-skills".to_string(),
                    kind: SurfaceKind::ReviewAutomation,
                    runtime_owner: SurfaceRuntimeOwner::ExternalRuntime,
                    default_lane: PackLane::Advanced,
                    distribution_target: DistributionTarget::ClaudeSkills,
                    description: "Claude review automation entrypoints.".to_string(),
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
                    lane: AutomationLane::RuntimeScheduler,
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
                    lane: AutomationLane::RuntimeScheduler,
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
                AutomationDefinition {
                    name: "pr-review-gate".to_string(),
                    cadence: AutomationCadence::OnDemand,
                    lane: AutomationLane::RepoAutomation,
                    packs: vec!["review-automation-pack".to_string()],
                    connectors: vec!["github".to_string(), "linear".to_string()],
                    artifact: "PR Review Gate".to_string(),
                    description: "PR review gate.".to_string(),
                },
                AutomationDefinition {
                    name: "release-readiness-gate".to_string(),
                    cadence: AutomationCadence::OnDemand,
                    lane: AutomationLane::RepoAutomation,
                    packs: vec!["review-automation-pack".to_string()],
                    connectors: vec!["github".to_string(), "linear".to_string()],
                    artifact: "Release Readiness Gate".to_string(),
                    description: "Release readiness gate.".to_string(),
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
        let surfaces = super::ProviderSurfaces {
            codex: super::selected_codex_surfaces(&manifest, &active_packs),
            gemini: super::selected_gemini_surfaces(&manifest, &active_packs),
            claude: super::selected_claude_surfaces(&manifest, &active_packs),
        };
        let report = super::doctor_catalog_report(
            &manifest,
            &selection,
            &requested_mcp,
            &requested_mcp,
            &distribution_state,
            &surfaces,
            crate::cli::RecordSurface::Both,
        );

        assert_eq!(report.harnesses.len(), 8);
        assert_eq!(report.packs.len(), 6);
        assert_eq!(report.surfaces.len(), 18);
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
        assert_eq!(report.automations.len(), 4);
        assert_eq!(report.runtime_handoff.active_app_connector_count, 2);
        assert_eq!(report.runtime_handoff.pending_app_verification_count, 2);
        assert_eq!(report.runtime_handoff.active_automation_count, 0);
        assert_eq!(report.runtime_handoff.active_runtime_automation_count, 0);
        assert_eq!(report.runtime_handoff.active_repo_automation_count, 0);
        assert_eq!(
            report.runtime_handoff.pending_scheduler_registration_count,
            0
        );
        assert_eq!(report.runtime_handoff.pending_repo_registration_count, 0);
        assert_eq!(
            report.runtime_handoff.connector_queue,
            vec!["github".to_string(), "linear".to_string()]
        );
        assert!(report.runtime_handoff.automation_queue.is_empty());
        assert!(report.runtime_handoff.repo_automation_queue.is_empty());
        assert_eq!(
            report.active_record_templates,
            vec!["project-record".to_string()]
        );
        assert_eq!(report.record_templates.len(), 2);
        assert!(report.record_templates[0].active);
        assert_eq!(report.record_templates[0].runtime_owner, "external-tools");
        assert_eq!(report.record_readiness.record_system, "both");
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
            record_surface: Some("local-docs".to_string()),
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
            active_surfaces: vec![
                "stackpilot-dev-kit".to_string(),
                "delivery-skills".to_string(),
            ],
            managed_paths: vec!["config.toml".to_string(), "AGENTS.md".to_string()],
        };

        let requested_state = crate::state::RequestedState {
            active_preset: Some("normal"),
            record_surface: Some("github-issue"),
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
    fn installed_state_ignores_record_surface_when_not_requested() {
        let installed = crate::state::InstalledState {
            active_preset: Some("normal".to_string()),
            record_surface: Some("local-docs".to_string()),
            active_packs: vec!["delivery-pack".to_string()],
            active_harnesses: vec!["delivery".to_string()],
            active_connectors: vec!["github".to_string()],
            active_automations: Vec::new(),
            active_record_templates: vec!["project-record".to_string()],
            active_surfaces: vec!["delivery-skills".to_string()],
            managed_paths: vec!["config.toml".to_string()],
        };

        let requested_state = crate::state::RequestedState {
            active_preset: Some("normal"),
            record_surface: None,
            active_packs: &installed.active_packs,
            active_harnesses: &installed.active_harnesses,
            active_connectors: &installed.active_connectors,
            active_automations: &installed.active_automations,
            active_record_templates: &installed.active_record_templates,
            active_surfaces: &installed.active_surfaces,
            managed_paths: &installed.managed_paths,
        };

        assert!(!installed.mismatch(&requested_state));
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
        let surfaces = super::ProviderSurfaces {
            codex: super::selected_codex_surfaces(&manifest, &active_packs),
            gemini: super::selected_gemini_surfaces(&manifest, &active_packs),
            claude: super::selected_claude_surfaces(&manifest, &active_packs),
        };
        let report = super::doctor_catalog_report(
            &manifest,
            &selection,
            &requested_mcp,
            &active_mcp,
            &distribution_state,
            &surfaces,
            crate::cli::RecordSurface::GithubIssue,
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
        assert_eq!(report.record_readiness.record_system, "github-issue");
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
        assert!(manifest.packs.iter().any(|pack| {
            pack.name == "team-pack"
                && pack.lane == PackLane::Advanced
                && pack.harnesses.contains(&"parallel-build".to_string())
                && pack.codex_surfaces.contains(&"team-skills".to_string())
                && pack.gemini_surfaces.contains(&"team-commands".to_string())
        }));
        assert!(
            manifest
                .presets
                .iter()
                .any(|preset| preset.name == "full"
                    && preset.packs.contains(&"ops-pack".to_string()))
        );
        assert!(manifest.presets.iter().any(|preset| {
            preset.name == "orchestrator" && preset.packs.contains(&"team-pack".to_string())
        }));
        assert!(manifest.presets.iter().any(|preset| {
            preset.name == "review-automation"
                && preset.packs.contains(&"review-automation-pack".to_string())
        }));
        assert!(manifest.presets.iter().any(|preset| {
            preset.name == "all-in-one"
                && preset.packs.contains(&"team-pack".to_string())
                && preset.packs.contains(&"founder-pack".to_string())
                && preset.packs.contains(&"review-automation-pack".to_string())
        }));
        assert!(manifest.surfaces.iter().any(|surface| {
            surface.name == "delivery-commands"
                && surface.distribution_target == DistributionTarget::GeminiExtension
                && surface.kind == SurfaceKind::Entrypoint
        }));
        assert!(manifest.surfaces.iter().any(|surface| {
            surface.name == "team-commands"
                && surface.distribution_target == DistributionTarget::GeminiExtension
                && surface.kind == SurfaceKind::Team
        }));
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
        let path = crate::repo_assets::stackpilot_dev_kit_codex_bundle_root()
            .join("skills/qa-browser/SKILL.md");
        let raw = fs::read_to_string(path).unwrap();
        assert!(raw.starts_with("---\n"));
    }

    fn temp_home() -> PathBuf {
        let counter = TEMP_HOME_COUNTER.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir()
            .join(format!(
                "stackpilot-test-{}-{}-{}",
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

    fn assert_claude_opus_1m_model(value: &serde_json::Value) {
        let model = value.as_str().unwrap_or_default();
        assert_eq!(
            model, "opus[1m]",
            "expected Claude Opus 1M model, got {model:?}"
        );
    }

    fn test_task_state(harnesses: &[&str]) -> crate::state::TaskState {
        crate::state::TaskState {
            id: "task_1".to_string(),
            title: "Gate smoke".to_string(),
            status: "in-progress".to_string(),
            phase: "execute".to_string(),
            owner: Some("codex".to_string()),
            summary: Some("Auth flow is wired and blocked on the review lane.".to_string()),
            checkpoint: Some(
                "Resume from the failing oauth fixture and re-run the review probes.".to_string(),
            ),
            next_action: Some("continue".to_string()),
            providers: vec!["codex".to_string(), "gemini".to_string()],
            packs: vec!["delivery-pack".to_string(), "team-pack".to_string()],
            harnesses: harnesses.iter().map(|value| value.to_string()).collect(),
            completed_signals: Vec::new(),
            attempt_count: 0,
            last_failure: None,
            investigation_note: None,
            updated_at: "123".to_string(),
        }
    }

    #[test]
    fn gate_check_requires_parallel_build_contract_signals() {
        let state = test_task_state(&["parallel-build", "review-gate"]);
        let report = super::evaluate_gate(&state, TaskPhase::Review, &[]);

        assert!(!report.allowed);
        assert_eq!(
            report.missing_signals,
            vec!["handoff".to_string(), "ownership".to_string()]
        );
        assert_eq!(report.recommended_status, "blocked");
    }

    #[test]
    fn gate_check_requires_spec_before_plan() {
        let mut state = test_task_state(&[]);
        state.phase = "discover".to_string();
        let report = super::evaluate_gate(&state, TaskPhase::Plan, &[]);

        assert!(!report.allowed);
        assert_eq!(report.missing_signals, vec!["spec".to_string()]);
    }

    #[test]
    fn gate_check_requires_plan_before_execute() {
        let mut state = test_task_state(&[]);
        state.phase = "plan".to_string();
        let report = super::evaluate_gate(&state, TaskPhase::Execute, &[]);

        assert!(!report.allowed);
        assert_eq!(report.missing_signals, vec!["plan".to_string()]);
    }

    #[test]
    fn gate_check_requires_ship_review_signals() {
        let mut state = test_task_state(&["parallel-build", "review-gate"]);
        state.completed_signals = vec!["ownership".to_string(), "handoff".to_string()];
        let report = super::evaluate_gate(&state, TaskPhase::Ship, &[]);

        assert!(!report.allowed);
        assert_eq!(
            report.missing_signals,
            vec!["qa".to_string(), "review".to_string(), "verify".to_string()]
        );
    }

    #[test]
    fn gate_check_requires_investigation_after_recorded_retries() {
        let mut state = test_task_state(&["review-gate"]);
        state.phase = "review".to_string();
        state.attempt_count = 2;
        state.last_failure = Some("verification still failing".to_string());
        let report = super::evaluate_gate(&state, TaskPhase::Qa, &[]);

        assert!(!report.allowed);
        assert_eq!(report.missing_signals, vec!["investigate".to_string()]);
        assert_eq!(report.retry_status, "investigate");
    }

    #[test]
    fn gate_check_accepts_investigation_note_after_escalated_retry() {
        let mut state = test_task_state(&[]);
        state.phase = "review".to_string();
        state.attempt_count = 2;
        state.last_failure = Some("verification still failing".to_string());
        state.investigation_note = Some("isolated flaky fixture".to_string());

        let report = super::evaluate_gate(&state, TaskPhase::Qa, &[]);

        assert!(report.allowed);
        assert_eq!(report.retry_status, "investigated");
        assert_eq!(
            report.recommended_next_action.as_deref(),
            Some(
                "rerun verification with the investigation note attached, or move the task back to plan if the fix is no longer bounded"
            )
        );
    }

    #[test]
    fn gate_apply_persists_signals_and_advances_phase() {
        let home = temp_home();
        let state = test_task_state(&["parallel-build", "review-gate"]);
        crate::state::write_task_state(&home, &state).unwrap();

        let blocked = super::gate_check(
            &home,
            GateCheckArgs {
                target_phase: Some(TaskPhase::Ship),
                completed: Vec::new(),
                json: false,
            },
        );
        assert!(blocked.is_err());

        let applied = super::gate_apply(
            &home,
            GateApplyArgs {
                target_phase: Some(TaskPhase::Ship),
                completed: vec![
                    GateSignal::Ownership,
                    GateSignal::Handoff,
                    GateSignal::Review,
                    GateSignal::Qa,
                    GateSignal::Verify,
                ],
                json: false,
            },
        );
        assert!(applied.is_ok());

        let loaded = crate::state::read_task_state(&home).unwrap().unwrap();
        assert_eq!(loaded.phase, "ship");
        assert_eq!(loaded.status, "ready");
        assert_eq!(
            loaded.completed_signals,
            vec![
                "handoff".to_string(),
                "ownership".to_string(),
                "qa".to_string(),
                "review".to_string(),
                "verify".to_string()
            ]
        );

        crate::state::clear_task_state(&home).unwrap();
        fs::remove_dir_all(home).unwrap();
    }

    #[test]
    fn gate_apply_advances_execute_after_plan_signal() {
        let home = temp_home();
        let mut state = test_task_state(&[]);
        state.phase = "plan".to_string();
        crate::state::write_task_state(&home, &state).unwrap();

        let applied = super::gate_apply(
            &home,
            GateApplyArgs {
                target_phase: Some(TaskPhase::Execute),
                completed: vec![GateSignal::Plan],
                json: false,
            },
        );
        assert!(applied.is_ok());

        let loaded = crate::state::read_task_state(&home).unwrap().unwrap();
        assert_eq!(loaded.phase, "execute");
        assert_eq!(loaded.completed_signals, vec!["plan".to_string()]);

        fs::remove_dir_all(home).unwrap();
    }

    #[test]
    fn gate_report_marks_targeted_retry_after_first_failure() {
        let mut state = test_task_state(&[]);
        state.completed_signals = vec!["spec".to_string()];
        state.last_failure = Some("unit test still failing".to_string());
        state.attempt_count = 1;
        let report = super::evaluate_gate(&state, TaskPhase::Plan, &[]);

        assert_eq!(report.retry_status, "targeted-retry");
        assert_eq!(
            report.recommended_next_action.as_deref(),
            Some(
                "apply one bounded fix, rerun verification, then clear or replace the recorded failure"
            )
        );
    }

    #[test]
    fn task_state_advance_records_investigation_note_and_signal() {
        let home = temp_home();
        let mut state = test_task_state(&[]);
        state.attempt_count = 2;
        state.last_failure = Some("verification still failing".to_string());
        crate::state::write_task_state(&home, &state).unwrap();

        super::task_state_advance(
            &home,
            crate::cli::TaskStateAdvanceArgs {
                status: None,
                phase: None,
                summary: None,
                clear_summary: false,
                checkpoint: None,
                clear_checkpoint: false,
                next_action: None,
                failure: None,
                clear_failure: false,
                investigation_note: Some("isolated flaky fixture".to_string()),
                clear_investigation: false,
                complete: Vec::new(),
                clear_complete: Vec::new(),
                increment_attempt: false,
                json: false,
            },
        )
        .unwrap();

        let loaded = crate::state::read_task_state(&home).unwrap().unwrap();
        assert_eq!(
            loaded.investigation_note.as_deref(),
            Some("isolated flaky fixture")
        );
        assert_eq!(loaded.completed_signals, vec!["investigate".to_string()]);
        assert_eq!(
            loaded.next_action.as_deref(),
            Some(
                "rerun verification with the investigation note attached, or move the task back to plan if the fix is no longer bounded"
            )
        );

        fs::remove_dir_all(home).unwrap();
    }

    #[test]
    fn task_state_advance_updates_summary_and_checkpoint() {
        let home = temp_home();
        let state = test_task_state(&[]);
        crate::state::write_task_state(&home, &state).unwrap();

        super::task_state_advance(
            &home,
            crate::cli::TaskStateAdvanceArgs {
                status: None,
                phase: None,
                summary: Some("Review lane is waiting on a flaky fixture repro.".to_string()),
                clear_summary: false,
                checkpoint: Some(
                    "Open the oauth fixture, rerun the repro, and capture output.".to_string(),
                ),
                clear_checkpoint: false,
                next_action: Some("rerun the flaky review repro".to_string()),
                failure: None,
                clear_failure: false,
                investigation_note: None,
                clear_investigation: false,
                complete: Vec::new(),
                clear_complete: Vec::new(),
                increment_attempt: false,
                json: false,
            },
        )
        .unwrap();

        let loaded = crate::state::read_task_state(&home).unwrap().unwrap();
        assert_eq!(
            loaded.summary.as_deref(),
            Some("Review lane is waiting on a flaky fixture repro.")
        );
        assert_eq!(
            loaded.checkpoint.as_deref(),
            Some("Open the oauth fixture, rerun the repro, and capture output.")
        );
        assert_eq!(
            loaded.next_action.as_deref(),
            Some("rerun the flaky review repro")
        );

        fs::remove_dir_all(home).unwrap();
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
        let backup_root = fs::read_dir(home.join(".stackpilot-legacy-backups"))
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
        let env_path = home.join(".zshrc.d/stackpilot-env.zsh");
        fs::create_dir_all(env_path.parent().unwrap()).unwrap();
        fs::write(
            &env_path,
            "# managed by stackpilot\nexport EXA_API_KEY='exa-key'\nexport OMX_API_KEY='old-omx'\nexport OH_MY_OPENCODE_API_KEY='old-oh-my'\n",
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
        let enabled = vec![BaselineMcp::ChromeDevtools, BaselineMcp::Exa];
        let root = temp_home().join(".codex");
        fs::create_dir_all(&root).unwrap();
        let blocks =
            codex::mcp_blocks(Path::new("/tmp/home"), &root, &enabled, ApplyMode::Merge).unwrap();
        assert!(blocks.contains("chrome-devtools-mcp.sh"));
        assert!(blocks.contains("exa-mcp.sh"));
        assert!(blocks.contains("startup_timeout_sec = 20"));
        assert!(blocks.contains("tool_timeout_sec = 120"));
        assert!(blocks.contains("env_vars = [\"EXA_API_KEY\"]"));
        assert!(!blocks.contains("context7-mcp.sh"));
        assert!(!blocks.contains("playwright-mcp.sh"));
        assert!(!blocks.contains("github-mcp.sh"));
        fs::remove_dir_all(root.parent().unwrap()).unwrap();
    }

    #[test]
    fn codex_plugin_blocks_follow_distribution_target_selection() {
        assert!(codex::plugin_blocks(true).contains("stackpilot-dev-kit@stackpilot"));
        assert!(codex::plugin_blocks(false).is_empty());
    }

    #[test]
    fn apply_mode_names_match_cli_values() {
        assert_eq!(ApplyMode::Merge.name(), "merge");
        assert_eq!(ApplyMode::Replace.name(), "replace");
    }

    #[test]
    fn package_patch_version_stays_within_release_policy() {
        let patch = env!("CARGO_PKG_VERSION")
            .split('.')
            .nth(2)
            .expect("package version must include a patch component")
            .parse::<u64>()
            .expect("package patch version must be numeric");

        assert!(patch <= 10, "roll patch > 10 to the next minor version");
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
    fn selected_pack_names_resolve_all_in_one_preset() {
        let selection = super::selected_pack_names(
            &super::PackArgs {
                preset: Some("all-in-one".to_string()),
                packs: None,
            },
            &test_manifest(),
        )
        .unwrap();
        assert_eq!(selection.preset, Some("all-in-one".to_string()));
        assert_eq!(
            selection.packs,
            vec![
                "delivery-pack".to_string(),
                "incident-pack".to_string(),
                "team-pack".to_string(),
                "founder-pack".to_string(),
                "ops-pack".to_string(),
                "review-automation-pack".to_string()
            ]
        );
    }

    #[test]
    fn provider_runtime_check_requires_gemini_cli() {
        let check = super::provider_runtime_check(super::Provider::Gemini);

        if crate::runtime::command_exists("gemini") {
            assert_eq!(check.status, "ok");
            assert_eq!(check.detail.as_deref(), Some("runtime=cli"));
        } else {
            assert_eq!(check.status, "missing");
            assert_eq!(check.target, "gemini");
            assert_eq!(check.detail.as_deref(), Some("Gemini CLI is required"));
        }
    }

    #[test]
    fn provider_runtime_check_accepts_codex_cli_or_app() {
        let check = super::provider_runtime_check(super::Provider::Codex);

        if crate::runtime::command_exists("codex") {
            assert_eq!(check.status, "ok");
            assert_eq!(check.target, "codex");
            assert_eq!(check.detail.as_deref(), Some("runtime=cli"));
        } else if Path::new("/Applications/Codex.app").exists() {
            assert_eq!(check.status, "ok");
            assert_eq!(check.target, "/Applications/Codex.app");
            assert_eq!(check.detail.as_deref(), Some("runtime=app"));
        } else {
            assert_eq!(check.status, "missing");
            assert_eq!(check.target, "codex or /Applications/Codex.app");
            assert_eq!(
                check.detail.as_deref(),
                Some("install Codex CLI or Codex app")
            );
        }
    }

    #[test]
    fn provider_probe_attempts_cover_all_runtimes() {
        let codex = super::provider_probe_attempts(
            super::Provider::Codex,
            "Reply with exactly OK and nothing else.",
        );
        assert_eq!(codex.len(), 3);
        assert_eq!(codex[0].0, "codex");
        assert!(codex[0].1.contains(&"--ephemeral".to_string()));

        let gemini = super::provider_probe_attempts(super::Provider::Gemini, "ok");
        assert_eq!(
            gemini,
            vec![(
                "gemini".to_string(),
                vec!["-p".to_string(), "ok".to_string()]
            )]
        );

        let claude = super::provider_probe_attempts(super::Provider::Claude, "ok");
        assert_eq!(
            claude,
            vec![(
                "claude".to_string(),
                vec!["-p".to_string(), "ok".to_string()]
            )]
        );
    }

    #[test]
    fn codex_optimization_probe_attempt_uses_gpt55_1m_override() {
        let attempts = super::codex_long_context_probe_attempts();
        assert_eq!(attempts.len(), 1);
        assert_eq!(attempts[0].0, "codex");
        let args = &attempts[0].1;
        assert!(args.contains(&"model=\"gpt-5.5\"".to_string()));
        assert!(args.contains(&"model_context_window=1000000".to_string()));
        assert!(args.contains(&"model_auto_compact_token_limit=900000".to_string()));
        assert!(args.contains(&"--skip-git-repo-check".to_string()));
        assert!(args.contains(&"--ephemeral".to_string()));
    }

    #[test]
    fn claude_optimization_probe_attempts_use_1m_aliases() {
        let attempts = super::claude_1m_probe_attempts();
        assert_eq!(attempts.len(), 2);
        assert_eq!(attempts[0].0, "claude");
        assert!(attempts[0].1.contains(&"opus[1m]".to_string()));
        assert!(attempts[0].1.contains(&"max".to_string()));
        assert_eq!(attempts[1].0, "claude");
        assert!(attempts[1].1.contains(&"sonnet[1m]".to_string()));
        assert!(attempts[1].1.contains(&"high".to_string()));
    }

    #[test]
    fn provider_probe_paths_follow_selected_surfaces() {
        let home = temp_home();
        let resolved = super::resolve_plan(
            &test_manifest(),
            &super::PackArgs {
                preset: Some("normal".to_string()),
                packs: None,
            },
        )
        .unwrap();

        let codex_paths = super::provider_probe_paths(&home, super::Provider::Codex, &resolved);
        assert_unique_paths(&codex_paths);
        assert!(codex_paths.iter().any(|path| path.ends_with("AGENTS.md")));
        assert!(codex_paths.iter().any(|path| path.ends_with("WORKFLOW.md")));
        assert!(
            codex_paths
                .iter()
                .any(|path| path.ends_with("plugins/stackpilot-dev-kit/skills/autopilot/SKILL.md"))
        );

        let gemini_paths = super::provider_probe_paths(&home, super::Provider::Gemini, &resolved);
        assert_unique_paths(&gemini_paths);
        assert!(gemini_paths.iter().any(|path| path.ends_with("GEMINI.md")));
        assert!(
            gemini_paths
                .iter()
                .any(|path| path.ends_with("extensions/stackpilot-dev/commands/autopilot.toml"))
        );

        fs::remove_dir_all(home).unwrap();
    }

    #[test]
    fn provider_probe_paths_include_team_surfaces_when_requested() {
        let home = temp_home();
        let resolved = super::resolve_plan(
            &test_manifest(),
            &super::PackArgs {
                preset: Some("orchestrator".to_string()),
                packs: None,
            },
        )
        .unwrap();

        let codex_paths = super::provider_probe_paths(&home, super::Provider::Codex, &resolved);
        assert_unique_paths(&codex_paths);
        assert!(codex_paths.iter().any(|path| path.ends_with("TEAM.md")));
        assert!(
            codex_paths
                .iter()
                .any(|path| path.ends_with("plugins/stackpilot-dev-kit/skills/team/SKILL.md"))
        );
        assert!(
            codex_paths
                .iter()
                .any(|path| path.ends_with("plugins/stackpilot-dev-kit/skills/ultrawork/SKILL.md"))
        );
        assert!(codex_paths.iter().any(|path| {
            path.ends_with("plugins/stackpilot-dev-kit/skills/workflow-gate/SKILL.md")
        }));

        let gemini_paths = super::provider_probe_paths(&home, super::Provider::Gemini, &resolved);
        assert_unique_paths(&gemini_paths);
        assert!(gemini_paths.iter().any(|path| path.ends_with("TEAM.md")));
        assert!(
            gemini_paths
                .iter()
                .any(|path| path.ends_with("extensions/stackpilot-dev/commands/team.toml"))
        );
        assert!(
            gemini_paths
                .iter()
                .any(|path| path.ends_with("extensions/stackpilot-dev/commands/ultrawork.toml"))
        );
        assert!(
            gemini_paths
                .iter()
                .any(|path| path.ends_with("extensions/stackpilot-dev/commands/gate.toml"))
        );

        let claude_paths = super::provider_probe_paths(&home, super::Provider::Claude, &resolved);
        assert_unique_paths(&claude_paths);
        assert!(claude_paths.iter().any(|path| path.ends_with("TEAM.md")));
        assert!(
            claude_paths
                .iter()
                .any(|path| path.ends_with("skills/team/SKILL.md"))
        );
        assert!(
            claude_paths
                .iter()
                .any(|path| path.ends_with("skills/ultrawork/SKILL.md"))
        );
        assert!(
            claude_paths
                .iter()
                .any(|path| path.ends_with("skills/workflow-gate/SKILL.md"))
        );

        fs::remove_dir_all(home).unwrap();
    }

    #[test]
    fn selected_provider_asset_paths_are_unique() {
        let codex_surfaces = vec![
            "delivery-skills".to_string(),
            "incident-skills".to_string(),
            "team-skills".to_string(),
            "review-automation-skills".to_string(),
            "company-skills".to_string(),
        ];
        let gemini_surfaces = vec![
            "delivery-commands".to_string(),
            "incident-commands".to_string(),
            "team-commands".to_string(),
            "review-automation-commands".to_string(),
            "company-commands".to_string(),
        ];

        assert_unique_static_paths(crate::layout::codex_managed_paths_for(
            &codex_surfaces,
            true,
            true,
        ));
        assert_unique_static_paths(crate::layout::codex_plugin_asset_paths(&codex_surfaces));
        assert_unique_static_paths(crate::layout::gemini_managed_paths_for(
            &gemini_surfaces,
            true,
            true,
        ));
        assert_unique_static_paths(crate::layout::gemini_extension_asset_paths(
            &gemini_surfaces,
        ));
        assert_unique_static_paths(crate::layout::claude_managed_paths_for(
            &codex_surfaces,
            true,
            true,
        ));
        assert_unique_static_paths(crate::layout::claude_skill_paths(&codex_surfaces));
    }

    fn assert_unique_paths(paths: &[PathBuf]) {
        let mut seen = BTreeSet::new();
        for path in paths {
            let rendered = path.display().to_string();
            assert!(seen.insert(rendered.clone()), "duplicate path: {rendered}");
        }
    }

    fn assert_unique_static_paths<T: ToString>(paths: Vec<T>) {
        let mut seen = BTreeSet::new();
        for path in paths {
            let rendered = path.to_string();
            assert!(seen.insert(rendered.clone()), "duplicate path: {rendered}");
        }
    }

    #[test]
    fn workflow_gate_contract_lines_reflect_active_harnesses() {
        let lines = super::workflow_gate_contract_lines(&[
            "parallel-build".to_string(),
            "review-gate".to_string(),
            "incident".to_string(),
        ]);

        assert_eq!(lines.len(), 6);
        assert!(lines.iter().any(|line| line.contains("plan requires spec")));
        assert!(
            lines
                .iter()
                .any(|line| line.contains("execute requires plan"))
        );
        assert!(lines.iter().any(|line| line.contains("ownership")));
        assert!(lines.iter().any(|line| line.contains("handoff")));
        assert!(lines.iter().any(|line| line.contains("review, qa, verify")));
        assert!(lines.iter().any(|line| line.contains("investigate")));
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
    fn review_automation_pack_routes_repo_automation_queue() {
        let manifest = test_manifest();
        let active_packs = vec!["review-automation-pack".to_string()];
        let active_connectors = super::selected_connector_names(&manifest, &active_packs);
        let active_automations = super::selected_automation_names(&manifest, &active_packs);
        let handoff = super::doctor_runtime_handoff_report(
            &manifest,
            &active_connectors,
            &active_automations,
        );

        assert_eq!(
            active_automations,
            vec![
                "pr-review-gate".to_string(),
                "release-readiness-gate".to_string()
            ]
        );
        assert_eq!(handoff.active_runtime_automation_count, 0);
        assert_eq!(handoff.active_repo_automation_count, 2);
        assert_eq!(handoff.pending_repo_registration_count, 2);
        assert_eq!(
            handoff.repo_automation_queue,
            vec![
                "pr-review-gate".to_string(),
                "release-readiness-gate".to_string()
            ]
        );
        assert!(
            handoff
                .next_steps
                .iter()
                .any(|step| step.contains("repository workflow"))
        );
    }

    #[test]
    fn repo_automation_scaffold_writes_managed_files() {
        let repo = temp_home();
        let args = crate::cli::RepoAutomationScaffoldArgs {
            repo_root: repo.clone(),
            pr_required_checks: vec!["check".to_string()],
            release_required_checks: vec!["check".to_string(), "pr-review-gate / gate".to_string()],
            minimum_approvals: 1,
            default_branch: "main".to_string(),
            force: false,
            dry_run: false,
            json: false,
        };

        let report = super::repo_automation_scaffold_with(&repo, args).unwrap();

        assert_eq!(report.pr_required_checks, vec!["check".to_string()]);
        assert_eq!(
            report.release_required_checks,
            vec!["check".to_string(), "pr-review-gate / gate".to_string()]
        );
        assert!(
            repo.join(".github/stackpilot/review-automation.json")
                .exists()
        );
        assert!(repo.join(".github/workflows/pr-review-gate.yml").exists());
        assert!(
            repo.join(".github/workflows/release-readiness-gate.yml")
                .exists()
        );
        assert!(repo.join(".github/PULL_REQUEST_TEMPLATE.md").exists());
        let config =
            fs::read_to_string(repo.join(".github/stackpilot/review-automation.json")).unwrap();
        assert!(config.contains("\"managed_by\": \"stackpilot\""));
        assert!(config.contains("\"check\""));
        let pr_template =
            fs::read_to_string(repo.join(".github/PULL_REQUEST_TEMPLATE.md")).unwrap();
        assert!(pr_template.contains("- [ ] review"));

        fs::remove_dir_all(repo).unwrap();
    }

    #[test]
    fn repo_automation_scaffold_refuses_unmanaged_overwrite() {
        let repo = temp_home();
        let workflow = repo.join(".github/workflows/pr-review-gate.yml");
        fs::create_dir_all(workflow.parent().unwrap()).unwrap();
        fs::write(&workflow, "name: custom\n").unwrap();

        let args = crate::cli::RepoAutomationScaffoldArgs {
            repo_root: repo.clone(),
            pr_required_checks: vec![],
            release_required_checks: vec![],
            minimum_approvals: 1,
            default_branch: "main".to_string(),
            force: false,
            dry_run: false,
            json: false,
        };

        let err = super::repo_automation_scaffold_with(&repo, args).unwrap_err();
        assert!(err.to_string().contains("refusing to overwrite unmanaged"));

        fs::remove_dir_all(repo).unwrap();
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
            from_task_state: false,
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
    fn render_record_body_attaches_task_state_context() {
        let args = super::RecordArgs {
            record_type: crate::cli::RecordKind::Task,
            title: "Attach task state".to_string(),
            status: "draft".to_string(),
            owner: None,
            next_action: None,
            from_task_state: true,
            surface: crate::cli::RecordSurface::LocalDocs,
            output_dir: PathBuf::from(".stackpilot/records"),
            github_repo: None,
            dry_run: true,
        };
        let task_state = test_task_state(&["parallel-build", "review-gate"]);
        let mut task_state = task_state;
        task_state.investigation_note = Some("isolated flaky fixture".to_string());
        let body = super::render_record_body(
            &args,
            test_manifest()
                .record_templates
                .iter()
                .find(|record| record.record_type == "TaskRecord"),
            Some(&task_state),
        )
        .unwrap();

        assert!(body.contains("source: \"active-task-state\""));
        assert!(body.contains("id: \"task_1\""));
        assert!(body.contains("harnesses: [\"parallel-build\", \"review-gate\"]"));
        assert!(body.contains("owner: \"codex\""));
        assert!(body.contains("next_action: \"continue\""));
        assert!(body.contains(
            "context:\n  summary: \"Auth flow is wired and blocked on the review lane.\""
        ));
        assert!(body.contains("summary: \"Auth flow is wired and blocked on the review lane.\""));
        assert!(body.contains(
            "checkpoint: \"Resume from the failing oauth fixture and re-run the review probes.\""
        ));
        assert!(body.contains("investigation_note: \"isolated flaky fixture\""));
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
    fn validate_manifest_rejects_unknown_pack_surface() {
        let mut manifest = test_manifest();
        manifest.packs[0]
            .gemini_surfaces
            .push("missing-surface".to_string());

        let err = super::validate_manifest(&manifest).unwrap_err();
        assert!(err.to_string().contains(
            "pack delivery-pack references unknown surface missing-surface for gemini-extension"
        ));
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
            assert_ne!(
                parsed["model"].as_str(),
                Some("gpt-5-codex"),
                "stale Codex model alias in {}",
                name
            );
            assert!(
                parsed.get("model_reasoning_effort").is_some(),
                "missing effort in {}",
                name
            );
            if parsed.get("model_context_window").is_some() {
                assert_eq!(
                    parsed["model"].as_str(),
                    Some("gpt-5.5"),
                    "long-context Codex role should request the latest model in {}",
                    name
                );
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
            } else {
                let (expected_model, expected_effort) = match name.as_str() {
                    "triage" => ("gpt-5.4-mini", "low"),
                    "explore" | "git-master" => ("gpt-5.4-mini", "medium"),
                    "docs-researcher" | "verifier" => ("gpt-5.4-mini", "high"),
                    "backend-service" | "executor" | "frontend-app" | "mobile-app"
                    | "test-engineer" => ("gpt-5.5", "high"),
                    _ => ("gpt-5.5", "xhigh"),
                };
                assert_eq!(
                    parsed["model"].as_str(),
                    Some(expected_model),
                    "unexpected Codex model tier in {}",
                    name
                );
                assert_eq!(
                    parsed["model_reasoning_effort"].as_str(),
                    Some(expected_effort),
                    "unexpected Codex reasoning effort in {}",
                    name
                );
            }
        }

        pinned.sort();
        assert_eq!(pinned, vec!["architect-1m", "planner-1m", "reviewer-1m"]);
    }

    #[test]
    fn claude_agent_templates_use_official_frontmatter_model_fields() {
        let agents_dir = crate::runtime::repo_root().join("templates/claude/agents");
        let expected = [
            ("executor.md", "model: sonnet[1m]", "effort: high"),
            ("planner.md", "model: opus[1m]", "effort: max"),
            ("reviewer.md", "model: opus[1m]", "effort: max"),
            ("triage.md", "model: haiku", "effort: low"),
            ("verifier.md", "model: sonnet[1m]", "effort: high"),
        ];

        for (file, model, effort) in expected {
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
            assert!(raw.contains(model), "{file} missing expected model");
            assert!(raw.contains(effort), "{file} missing expected effort");
        }
    }

    #[test]
    fn gemini_agent_templates_include_required_frontmatter() {
        let agents_dir = crate::repo_assets::stackpilot_dev_kit_gemini_repo_root()
            .join("extensions/stackpilot-dev/agents");
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

        let bundled_qa = crate::repo_assets::stackpilot_dev_kit_gemini_bundle_root()
            .join("extensions/stackpilot-dev/agents/qa.md");
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
        let raw = "# managed by stackpilot\nexport EXA_API_KEY='exa-key'\nexport CONTEXT7_API_KEY='ctx-key'\n";
        assert_eq!(
            super::parse_managed_env_content(raw, "CONTEXT7_API_KEY"),
            Some("ctx-key".to_string())
        );
    }

    #[test]
    fn upsert_managed_block_appends_when_missing() {
        let existing = "export PATH=\"$HOME/.local/bin:$PATH\"\n";
        let block = "# >>> stackpilot env >>>\nsource test\n# <<< stackpilot env <<<\n";
        let updated = super::upsert_managed_block(existing, block);
        assert!(updated.contains("stackpilot env"));
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
    fn cleanup_extension_enablement_removes_only_stackpilot_entry() {
        let temp = temp_home();
        fs::create_dir_all(&temp).unwrap();
        let path = temp.join("extension-enablement.json");
        fs::write(
            &path,
            "{\n  \"stackpilot-dev\": {\"overrides\": [\"/tmp/*\"]},\n  \"other\": {\"overrides\": [\"/keep/*\"]}\n}\n",
        )
        .unwrap();

        cleanup_extension_enablement(&path).unwrap();

        let after = fs::read_to_string(&path).unwrap();
        assert!(!after.contains("stackpilot-dev"));
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
        assert!(codex_home.join("REVIEW.md").exists());
        assert!(
            codex_home
                .join("plugins/stackpilot-dev-kit/.codex-plugin/plugin.json")
                .exists()
        );
        assert!(
            codex_home
                .join("plugins/stackpilot-dev-kit/skills/review/SKILL.md")
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
        fs::create_dir_all(codex_home.join(".tmp/plugins-backup-old/repo")).unwrap();
        fs::write(
            codex_home.join(".tmp/plugins-backup-old/repo/README.md"),
            "old",
        )
        .unwrap();
        fs::write(codex_home.join(".tmp/plugins.sha"), "old-sha").unwrap();

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
        assert!(!codex_home.join(".tmp/plugins-backup-old").exists());
        assert!(!codex_home.join(".tmp/plugins.sha").exists());
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
    fn codex_merge_preserves_unmanaged_top_level_config() {
        let home = temp_home();
        let manifest = test_manifest();
        let codex_home = home.join(".codex");
        fs::create_dir_all(&codex_home).unwrap();
        fs::write(
            codex_home.join("config.toml"),
            "model = \"legacy-model\"\n\n[agents]\nmax_threads = 99\n\n[features]\nmulti_agent_v2 = true\ntool_search = true\nchronicle = true\nmemories = false\n\n[memories]\nno_memories_if_mcp_or_web_search = true\n\n[projects.\"/tmp/demo\"]\ntrust_level = \"trusted\"\n\n[mcp_servers.manual-tool]\ncommand = \"manual-tool\"\nenabled = true\n",
        )
        .unwrap();

        codex::install(
            &home,
            ApplyMode::Merge,
            &manifest,
            &[BaselineMcp::ChromeDevtools],
            false,
            true,
            &test_active_surfaces(),
        )
        .unwrap();

        let config = fs::read_to_string(codex_home.join("config.toml")).unwrap();
        let parsed: toml::Value = config.parse().unwrap();
        assert_eq!(
            parsed.get("model").and_then(toml::Value::as_str),
            Some("gpt-5.5")
        );
        assert_eq!(
            parsed
                .get("model_reasoning_effort")
                .and_then(toml::Value::as_str),
            Some("xhigh")
        );
        assert_eq!(
            parsed
                .get("plan_mode_reasoning_effort")
                .and_then(toml::Value::as_str),
            Some("xhigh")
        );
        assert_eq!(
            parsed.get("service_tier").and_then(toml::Value::as_str),
            Some("fast")
        );
        assert!(
            parsed
                .get("agents")
                .and_then(toml::Value::as_table)
                .and_then(|agents| agents.get("max_threads"))
                .is_none()
        );
        let features = parsed
            .get("features")
            .and_then(toml::Value::as_table)
            .unwrap();
        assert!(features.get("multi_agent").is_some());
        assert!(features.get("multi_agent_v2").is_none());
        assert!(features.get("tool_search").is_none());
        assert!(features.get("chronicle").is_some());
        assert_eq!(
            features.get("fast_mode").and_then(toml::Value::as_bool),
            Some(true)
        );
        assert_eq!(
            features.get("memories").and_then(toml::Value::as_bool),
            Some(true)
        );
        let memories = parsed
            .get("memories")
            .and_then(toml::Value::as_table)
            .unwrap();
        assert_eq!(
            memories
                .get("generate_memories")
                .and_then(toml::Value::as_bool),
            Some(true)
        );
        assert_eq!(
            memories.get("use_memories").and_then(toml::Value::as_bool),
            Some(true)
        );
        assert_eq!(
            memories
                .get("disable_on_external_context")
                .and_then(toml::Value::as_bool),
            Some(false)
        );
        assert!(memories.get("no_memories_if_mcp_or_web_search").is_none());
        assert_eq!(
            parsed
                .get("projects")
                .and_then(toml::Value::as_table)
                .and_then(|projects| projects.get("/tmp/demo"))
                .and_then(toml::Value::as_table)
                .and_then(|project| project.get("trust_level"))
                .and_then(toml::Value::as_str),
            Some("trusted")
        );
        let mcp_servers = parsed
            .get("mcp_servers")
            .and_then(toml::Value::as_table)
            .unwrap();
        assert!(mcp_servers.contains_key("manual-tool"));
        assert!(mcp_servers.contains_key("chrome-devtools"));

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
        assert!(!config.contains("stackpilot-dev-kit@stackpilot"));
        assert!(!codex_home.join(".agents/plugins/marketplace.json").exists());
        assert!(!codex_home.join("plugins/stackpilot-dev-kit").exists());

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
                .join("plugins/stackpilot-dev-kit/.codex-plugin/plugin.json")
                .exists()
        );
        assert!(
            codex_home
                .join("plugins/stackpilot-dev-kit/skills/investigate/SKILL.md")
                .exists()
        );
        assert!(
            codex_home
                .join("plugins/stackpilot-dev-kit/skills/repo-radar/SKILL.md")
                .exists()
        );
        assert!(
            !codex_home
                .join("plugins/stackpilot-dev-kit/skills/autopilot/SKILL.md")
                .exists()
        );
        assert!(
            !codex_home
                .join("plugins/stackpilot-dev-kit/skills/delivery-loop/SKILL.md")
                .exists()
        );
        assert!(
            !codex_home
                .join("plugins/stackpilot-dev-kit/skills/qa-browser/SKILL.md")
                .exists()
        );

        fs::remove_dir_all(home).unwrap();
    }

    #[test]
    fn codex_install_renders_team_surface_assets() {
        let home = temp_home();
        let manifest = test_manifest();
        let enabled = vec![BaselineMcp::ChromeDevtools];
        let team_surfaces = vec!["delivery-skills".to_string(), "team-skills".to_string()];

        codex::install(
            &home,
            ApplyMode::Merge,
            &manifest,
            &enabled,
            false,
            true,
            &team_surfaces,
        )
        .unwrap();

        let codex_home = home.join(".codex");
        assert!(codex_home.join("TEAM.md").exists());
        assert!(codex_home.join("ENTRYPOINTS.md").exists());
        assert!(
            codex_home
                .join("plugins/stackpilot-dev-kit/skills/team/SKILL.md")
                .exists()
        );
        assert!(
            codex_home
                .join("plugins/stackpilot-dev-kit/skills/deep-init/SKILL.md")
                .exists()
        );
        assert!(
            codex_home
                .join("plugins/stackpilot-dev-kit/skills/ultrawork/SKILL.md")
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
                .join("extensions/stackpilot-dev/commands/doctor.toml")
                .exists()
        );
        assert!(
            gemini_home
                .join("extensions/stackpilot-dev/commands/autopilot.toml")
                .exists()
        );
        assert!(
            gemini_home
                .join("extensions/stackpilot-dev/commands/office-hours.toml")
                .exists()
        );
        assert!(
            gemini_home
                .join("extensions/stackpilot-dev/commands/qa.toml")
                .exists()
        );
        assert!(
            gemini_home
                .join("extensions/stackpilot-dev/commands/retro.toml")
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
        assert!(!gemini_home.join("extensions/stackpilot-dev").exists());
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
            &[BaselineMcp::ChromeDevtools, BaselineMcp::Exa],
            false,
            true,
            &test_active_surfaces(),
        )
        .unwrap();
        fs::write(
            gemini_home.join("stackpilot-state.json"),
            "{\n  \"managed_mcp\": [\"chrome-devtools\"]\n}\n",
        )
        .unwrap();
        fs::write(
            gemini_home.join("extensions/extension-enablement.json"),
            "{\n  \"stackpilot-dev\": {\"overrides\": [\"/tmp/*\"]},\n  \"other-extension\": {\"overrides\": [\"/opt/*\"]}\n}\n",
        )
        .unwrap();

        gemini::uninstall(&home, &manifest, false).unwrap();

        let enablement: serde_json::Value = serde_json::from_str(
            &fs::read_to_string(gemini_home.join("extensions/extension-enablement.json")).unwrap(),
        )
        .unwrap();
        assert!(enablement.get("stackpilot-dev").is_none());
        assert!(enablement.get("other-extension").is_some());
        assert!(!gemini_home.join("WORKFLOW.md").exists());
        assert!(!gemini_home.join("SHIP_CHECKLIST.md").exists());
        assert!(!gemini_home.join("stackpilot-state.json").exists());

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
            &[BaselineMcp::ChromeDevtools, BaselineMcp::Exa],
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
        assert_eq!(
            after["mcpServers"]["chrome-devtools"]["timeout"],
            json!(30000)
        );
        assert_eq!(
            after["mcpServers"]["chrome-devtools"]["trust"],
            json!(false)
        );
        assert_eq!(
            after["mcpServers"]["exa"]["env"]["EXA_API_KEY"],
            json!("$EXA_API_KEY")
        );

        fs::remove_dir_all(home).unwrap();
    }

    #[test]
    fn gemini_merge_preserves_user_preference_settings() {
        let home = temp_home();
        let manifest = test_manifest();
        let gemini_home = home.join(".gemini");
        fs::create_dir_all(&gemini_home).unwrap();
        fs::write(
            gemini_home.join("settings.json"),
            "{\n  \"general\": {\n    \"defaultApprovalMode\": \"auto_edit\",\n    \"enableAutoUpdate\": true,\n    \"checkpointing\": {\n      \"enabled\": false\n    },\n    \"plan\": {\n      \"directory\": \"\",\n      \"enabled\": false,\n      \"modelRouting\": false\n    },\n    \"retryFetchErrors\": false,\n    \"sessionRetention\": {\n      \"enabled\": true\n    },\n    \"topicUpdateNarration\": false\n  },\n  \"ideMode\": true,\n  \"showLineNumbers\": false,\n  \"showMemoryUsage\": false,\n  \"ui\": {\n    \"inlineThinkingMode\": \"off\",\n    \"hideTips\": false\n  },\n  \"output\": {\n    \"format\": \"json\"\n  }\n}\n",
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
        assert_eq!(after["general"]["defaultApprovalMode"], json!("plan"));
        assert_eq!(after["general"]["enableAutoUpdate"], json!(false));
        assert_eq!(
            after["general"]["enableAutoUpdateNotification"],
            json!(false)
        );
        assert_eq!(after["general"]["plan"]["modelRouting"], json!(true));
        assert_eq!(after["general"]["checkpointing"]["enabled"], json!(true));
        assert_eq!(after["general"]["plan"]["enabled"], json!(true));
        assert!(after["general"]["plan"].get("directory").is_none());
        assert_eq!(after["general"]["retryFetchErrors"], json!(true));
        assert_eq!(
            after["general"]["sessionRetention"]["enabled"],
            json!(false)
        );
        assert!(after["general"].get("topicUpdateNarration").is_none());
        assert_eq!(after["experimental"]["contextManagement"], json!(true));
        assert_eq!(after["experimental"]["autoMemory"], json!(true));
        assert_eq!(after["experimental"]["memoryV2"], json!(true));
        assert_eq!(after["experimental"]["modelSteering"], json!(true));
        assert_eq!(after["experimental"]["topicUpdateNarration"], json!(true));
        assert_eq!(
            after["contextManagement"]["historyWindow"]["maxTokens"],
            json!(150000)
        );
        assert_eq!(after["hooksConfig"]["enabled"], json!(true));
        assert_eq!(after["skills"]["enabled"], json!(true));
        assert_eq!(after["model"]["maxSessionTurns"], json!(-1));
        assert_eq!(after["model"]["name"], json!("auto"));
        assert!(after.get("ideMode").is_none());
        assert!(after.get("showLineNumbers").is_none());
        assert!(after.get("showMemoryUsage").is_none());
        assert_eq!(after["ui"]["inlineThinkingMode"], json!("off"));
        assert_eq!(after["ui"]["hideTips"], json!(false));
        assert_eq!(after["ui"]["showLineNumbers"], json!(true));
        assert_eq!(after["ui"]["showMemoryUsage"], json!(true));
        assert_eq!(after["ui"]["showCitations"], json!(true));
        assert_eq!(after["ui"]["footer"]["hideContextPercentage"], json!(false));
        assert_eq!(after["output"]["format"], json!("json"));

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
                .join("extensions/stackpilot-dev/gemini-extension.json")
                .exists()
        );
        assert!(
            !gemini_home
                .join("extensions/stackpilot-dev/commands/autopilot.toml")
                .exists()
        );
        let enablement = gemini_home.join("extensions/extension-enablement.json");
        if enablement.exists() {
            let raw = fs::read_to_string(enablement).unwrap();
            assert!(!raw.contains("stackpilot-dev"));
        }

        fs::remove_dir_all(home).unwrap();
    }

    #[test]
    fn gemini_install_renders_team_surface_assets() {
        let home = temp_home();
        let manifest = test_manifest();
        let team_surfaces = vec!["delivery-commands".to_string(), "team-commands".to_string()];

        gemini::install(
            &home,
            ApplyMode::Merge,
            &manifest,
            &[BaselineMcp::ChromeDevtools],
            false,
            true,
            &team_surfaces,
        )
        .unwrap();

        let gemini_home = home.join(".gemini");
        assert!(gemini_home.join("TEAM.md").exists());
        assert!(gemini_home.join("ENTRYPOINTS.md").exists());
        assert!(
            gemini_home
                .join("extensions/stackpilot-dev/commands/team.toml")
                .exists()
        );
        assert!(
            gemini_home
                .join("extensions/stackpilot-dev/commands/deep-init.toml")
                .exists()
        );
        assert!(
            gemini_home
                .join("extensions/stackpilot-dev/commands/ultrawork.toml")
                .exists()
        );

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
        let settings: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(claude_home.join("settings.json")).unwrap())
                .unwrap();
        assert_eq!(settings["autoUpdatesChannel"], json!("stable"));
        assert_eq!(settings["autoMemoryEnabled"], json!(true));
        assert_eq!(settings["awaySummaryEnabled"], json!(true));
        assert_eq!(settings["cleanupPeriodDays"], json!(365));
        assert!(settings.get("effortLevel").is_none());
        assert_eq!(settings["env"]["CLAUDE_CODE_EFFORT_LEVEL"], json!("max"));
        assert_eq!(
            settings["env"]["CLAUDE_CODE_EXPERIMENTAL_AGENT_TEAMS"],
            json!("1")
        );
        assert_eq!(settings["fastModePerSessionOptIn"], json!(true));
        assert_eq!(settings["includeGitInstructions"], json!(true));
        assert_claude_opus_1m_model(&settings["model"]);
        assert_eq!(settings["permissions"]["defaultMode"], json!("auto"));
        assert_eq!(settings["showThinkingSummaries"], json!(true));
        assert_eq!(settings["useAutoModeDuringPlan"], json!(false));
        assert!(
            settings["permissions"]["deny"]
                .as_array()
                .unwrap()
                .iter()
                .any(|rule| rule == "Read(./.env)")
        );
        assert!(
            settings["permissions"]["ask"]
                .as_array()
                .unwrap()
                .iter()
                .any(|rule| rule == "Bash(git push *)")
        );

        let mcp = claude::claude_user_mcp(&home).unwrap();
        assert!(mcp["mcpServers"].get("chrome-devtools").is_some());
        fs::create_dir_all(claude_home.join("plugins/cache/omc")).unwrap();
        fs::write(claude_home.join("plugins/cache/omc/state.json"), "{}").unwrap();

        claude::uninstall(&home, &enabled, false).unwrap();
        assert!(!claude_home.join("CLAUDE.md").exists());
        assert!(!claude_home.join("scripts").exists());
        assert!(!claude_home.join("skills/autopilot").exists());
        assert!(!claude_home.join("plugins/cache/omc").exists());
        assert!(!claude_home.join("settings.json").exists());
        assert!(claude_home.join("backups").exists());

        fs::remove_dir_all(home).unwrap();
    }

    #[test]
    fn claude_merge_appends_baseline_permissions_without_dropping_custom_rules() {
        if !crate::runtime::command_exists("claude") {
            return;
        }

        let home = temp_home();
        let manifest = test_manifest();
        let claude_home = home.join(".claude");
        fs::create_dir_all(&claude_home).unwrap();
        fs::write(
            claude_home.join("settings.json"),
            "{\n  \"model\": \"sonnet\",\n  \"env\": {\"CUSTOM_FLAG\": \"1\", \"CLAUDE_CODE_DISABLE_AUTO_MEMORY\": \"1\"},\n  \"permissions\": {\n    \"ask\": [\"Bash(make deploy *)\"],\n    \"deny\": [\"Read(./private/**)\"]\n  }\n}\n",
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

        let settings: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(claude_home.join("settings.json")).unwrap())
                .unwrap();
        assert_eq!(settings["model"], json!("sonnet"));
        assert_eq!(settings["autoMemoryEnabled"], json!(true));
        assert!(settings.get("effortLevel").is_none());
        assert_eq!(settings["env"]["CUSTOM_FLAG"], json!("1"));
        assert!(
            settings["env"]
                .get("CLAUDE_CODE_DISABLE_AUTO_MEMORY")
                .is_none()
        );
        assert_eq!(settings["env"]["CLAUDE_CODE_EFFORT_LEVEL"], json!("max"));
        assert_eq!(
            settings["env"]["CLAUDE_CODE_EXPERIMENTAL_AGENT_TEAMS"],
            json!("1")
        );
        let deny = settings["permissions"]["deny"].as_array().unwrap();
        assert!(deny.iter().any(|rule| rule == "Read(./private/**)"));
        assert!(deny.iter().any(|rule| rule == "Read(./.env)"));
        let ask = settings["permissions"]["ask"].as_array().unwrap();
        assert!(ask.iter().any(|rule| rule == "Bash(make deploy *)"));
        assert!(ask.iter().any(|rule| rule == "Bash(git push *)"));

        fs::remove_dir_all(home).unwrap();
    }

    #[test]
    fn claude_merge_preserves_user_opt_down_for_model_effort_and_fast_mode() {
        if !crate::runtime::command_exists("claude") {
            return;
        }

        let home = temp_home();
        let manifest = test_manifest();
        let claude_home = home.join(".claude");
        fs::create_dir_all(&claude_home).unwrap();
        fs::write(
            claude_home.join("settings.json"),
            "{\n  \"model\": \"sonnet\",\n  \"env\": {\"CLAUDE_CODE_EFFORT_LEVEL\": \"xhigh\"},\n  \"fastModePerSessionOptIn\": false\n}\n",
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

        let settings: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(claude_home.join("settings.json")).unwrap())
                .unwrap();
        assert_eq!(settings["env"]["CLAUDE_CODE_EFFORT_LEVEL"], json!("xhigh"));
        assert_eq!(settings["fastModePerSessionOptIn"], json!(false));
        assert_eq!(settings["model"], json!("sonnet"));

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
        assert!(claude_home.join("ENTRYPOINTS.md").exists());
        assert!(claude_home.join("agents/planner.md").exists());
        assert!(claude_home.join("stackpilot-state.json").exists());

        claude::uninstall(&home, &[BaselineMcp::ChromeDevtools], false).unwrap();

        assert!(!claude_home.join("WORKFLOW.md").exists());
        assert!(!claude_home.join("AUTOPILOT.md").exists());
        assert!(!claude_home.join("ENTRYPOINTS.md").exists());
        assert!(!claude_home.join("agents/planner.md").exists());
        assert!(!claude_home.join("stackpilot-state.json").exists());

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

    #[test]
    fn claude_install_renders_team_surface_assets() {
        if !crate::runtime::command_exists("claude") {
            return;
        }

        let home = temp_home();
        let manifest = test_manifest();
        let team_surfaces = vec!["delivery-skills".to_string(), "team-skills".to_string()];

        claude::install(
            &home,
            ApplyMode::Merge,
            &manifest,
            &[BaselineMcp::ChromeDevtools],
            false,
            true,
            &team_surfaces,
        )
        .unwrap();

        let claude_home = home.join(".claude");
        assert!(claude_home.join("TEAM.md").exists());
        assert!(claude_home.join("ENTRYPOINTS.md").exists());
        assert!(claude_home.join("skills/deep-init/SKILL.md").exists());
        assert!(claude_home.join("skills/team/SKILL.md").exists());
        assert!(claude_home.join("skills/ultrawork/SKILL.md").exists());

        fs::remove_dir_all(home).unwrap();
    }
}
