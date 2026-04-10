use clap::{Parser, Subcommand, ValueEnum};
use serde::Deserialize;
use std::path::PathBuf;
#[derive(Parser)]
#[command(
    name = "llm-bootstrap",
    version,
    about = "Bootstrap Codex, Gemini, and optional Claude Code dev homes"
)]
pub(crate) struct Cli {
    #[command(subcommand)]
    pub(crate) command: Option<Command>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, ValueEnum)]
#[serde(rename_all = "lowercase")]
pub(crate) enum Provider {
    Codex,
    Gemini,
    Claude,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, ValueEnum)]
#[serde(rename_all = "lowercase")]
pub(crate) enum ApplyMode {
    Merge,
    Replace,
}

impl ApplyMode {
    pub(crate) fn name(self) -> &'static str {
        match self {
            ApplyMode::Merge => "merge",
            ApplyMode::Replace => "replace",
        }
    }
}

impl Provider {
    pub(crate) fn name(self) -> &'static str {
        match self {
            Provider::Codex => "codex",
            Provider::Gemini => "gemini",
            Provider::Claude => "claude",
        }
    }
}

#[derive(Subcommand)]
pub(crate) enum Command {
    Baseline(InstallArgs),
    Install(InstallArgs),
    Sync(InstallArgs),
    Restore(RestoreArgs),
    Backups(BackupsArgs),
    Uninstall(UninstallArgs),
    Doctor(DoctorArgs),
    Probe(ProbeArgs),
    Internal(InternalArgs),
    #[command(hide = true)]
    TaskState(TaskStateArgs),
    Record(RecordArgs),
    Wizard(WizardArgs),
}

#[derive(clap::Args, Clone)]
pub(crate) struct ProviderArgs {
    #[arg(
        long,
        value_delimiter = ',',
        help = "Defaults to providers in bootstrap.toml"
    )]
    pub(crate) providers: Option<Vec<Provider>>,
}

#[derive(clap::Args, Clone)]
pub(crate) struct PackArgs {
    #[arg(
        long,
        conflicts_with = "packs",
        help = "Preset alias such as light, normal, full, or company"
    )]
    pub(crate) preset: Option<String>,
    #[arg(
        long,
        value_delimiter = ',',
        help = "Explicit pack list. Defaults to bootstrap.default_preset"
    )]
    pub(crate) packs: Option<Vec<String>>,
}

#[derive(clap::Args, Clone)]
pub(crate) struct InstallArgs {
    #[command(flatten)]
    pub(crate) provider_args: ProviderArgs,
    #[command(flatten)]
    pub(crate) pack_args: PackArgs,
    #[arg(
        long,
        value_enum,
        help = "Defaults to bootstrap.default_mode in bootstrap.toml"
    )]
    pub(crate) mode: Option<ApplyMode>,
    #[arg(
        long,
        help = "Skip RTK official init even if enabled in bootstrap.toml"
    )]
    pub(crate) without_rtk: bool,
    #[arg(
        long,
        value_enum,
        help = "Preferred operating record surface for the installed baseline"
    )]
    pub(crate) record_surface: Option<RecordSurface>,
    #[arg(long, help = "Show the planned install without writing files")]
    pub(crate) dry_run: bool,
}

#[derive(clap::Args, Clone)]
pub(crate) struct UninstallArgs {
    #[command(flatten)]
    pub(crate) provider_args: ProviderArgs,
    #[arg(long, help = "Skip RTK uninstall even if enabled in bootstrap.toml")]
    pub(crate) without_rtk: bool,
    #[arg(long, help = "Show the planned uninstall without writing files")]
    pub(crate) dry_run: bool,
}

#[derive(clap::Args, Clone)]
pub(crate) struct RestoreArgs {
    #[command(flatten)]
    pub(crate) provider_args: ProviderArgs,
    #[arg(long, help = "Optional backup directory name or absolute path")]
    pub(crate) backup: Option<String>,
    #[arg(long, help = "List available backups for the selected providers")]
    pub(crate) list: bool,
    #[arg(long, help = "Emit backup list or restore plan as JSON")]
    pub(crate) json: bool,
    #[arg(long, help = "Show the planned restore without writing files")]
    pub(crate) dry_run: bool,
}

#[derive(clap::Args, Clone)]
pub(crate) struct BackupsArgs {
    #[command(flatten)]
    pub(crate) provider_args: ProviderArgs,
    #[arg(long, help = "Emit backup list as JSON")]
    pub(crate) json: bool,
}

#[derive(clap::Args, Clone)]
pub(crate) struct DoctorArgs {
    #[command(flatten)]
    pub(crate) provider_args: ProviderArgs,
    #[command(flatten)]
    pub(crate) pack_args: PackArgs,
    #[arg(long, help = "Skip RTK checks even if enabled in bootstrap.toml")]
    pub(crate) without_rtk: bool,
    #[arg(
        long,
        value_enum,
        help = "Expected operating record surface when checking installed state"
    )]
    pub(crate) record_surface: Option<RecordSurface>,
    #[arg(long, help = "Emit doctor results as JSON")]
    pub(crate) json: bool,
}

#[derive(clap::Args, Clone)]
pub(crate) struct ProbeArgs {
    #[command(flatten)]
    pub(crate) provider_args: ProviderArgs,
    #[command(flatten)]
    pub(crate) pack_args: PackArgs,
    #[arg(
        long,
        default_value = "Reply with exactly OK and nothing else.",
        help = "Prompt used for provider runtime probe"
    )]
    pub(crate) prompt: String,
    #[arg(long, help = "Emit probe results as JSON")]
    pub(crate) json: bool,
}

#[derive(Subcommand, Clone)]
pub(crate) enum TaskStateCommand {
    Begin(TaskStateBeginArgs),
    Advance(TaskStateAdvanceArgs),
    Show(TaskStateShowArgs),
    Clear,
}

#[derive(Subcommand, Clone)]
pub(crate) enum GateCommand {
    Check(GateCheckArgs),
    Apply(GateApplyArgs),
}

#[derive(Subcommand, Clone)]
pub(crate) enum InternalCommand {
    TaskState(TaskStateArgs),
    Gate(GateArgs),
    RepoAutomation(RepoAutomationArgs),
}

#[derive(clap::Args, Clone)]
pub(crate) struct InternalArgs {
    #[command(subcommand)]
    pub(crate) command: InternalCommand,
}

#[derive(clap::Args, Clone)]
pub(crate) struct GateArgs {
    #[command(subcommand)]
    pub(crate) command: GateCommand,
}

#[derive(Subcommand, Clone)]
pub(crate) enum RepoAutomationCommand {
    Scaffold(RepoAutomationScaffoldArgs),
}

#[derive(clap::Args, Clone)]
pub(crate) struct RepoAutomationArgs {
    #[command(subcommand)]
    pub(crate) command: RepoAutomationCommand,
}

#[derive(clap::Args, Clone)]
pub(crate) struct RepoAutomationScaffoldArgs {
    #[arg(
        long,
        default_value = ".",
        help = "Target repository root that receives workflow and branch protection assets"
    )]
    pub(crate) repo_root: PathBuf,
    #[arg(
        long = "pr-required-check",
        value_delimiter = ',',
        help = "Check names that must succeed before the PR review gate passes"
    )]
    pub(crate) pr_required_checks: Vec<String>,
    #[arg(
        long = "release-required-check",
        value_delimiter = ',',
        help = "Check names that must succeed before the release readiness gate passes"
    )]
    pub(crate) release_required_checks: Vec<String>,
    #[arg(
        long,
        default_value_t = 1,
        help = "Minimum approving reviews required before the PR gate passes"
    )]
    pub(crate) minimum_approvals: usize,
    #[arg(
        long,
        default_value = "main",
        help = "Default branch referenced in branch protection guidance"
    )]
    pub(crate) default_branch: String,
    #[arg(long, help = "Overwrite existing unmanaged workflow assets")]
    pub(crate) force: bool,
    #[arg(long, help = "Show the planned repo automation files without writing")]
    pub(crate) dry_run: bool,
    #[arg(long, help = "Emit scaffold results as JSON")]
    pub(crate) json: bool,
}

#[derive(clap::Args, Clone)]
pub(crate) struct TaskStateArgs {
    #[command(subcommand)]
    pub(crate) command: TaskStateCommand,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
#[value(rename_all = "kebab-case")]
pub(crate) enum TaskStatus {
    Draft,
    InProgress,
    Blocked,
    Ready,
    Done,
}

impl TaskStatus {
    pub(crate) fn name(self) -> &'static str {
        match self {
            TaskStatus::Draft => "draft",
            TaskStatus::InProgress => "in-progress",
            TaskStatus::Blocked => "blocked",
            TaskStatus::Ready => "ready",
            TaskStatus::Done => "done",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
#[value(rename_all = "kebab-case")]
pub(crate) enum TaskPhase {
    Discover,
    Plan,
    Execute,
    Review,
    Qa,
    Ship,
    Operate,
}

impl TaskPhase {
    pub(crate) fn name(self) -> &'static str {
        match self {
            TaskPhase::Discover => "discover",
            TaskPhase::Plan => "plan",
            TaskPhase::Execute => "execute",
            TaskPhase::Review => "review",
            TaskPhase::Qa => "qa",
            TaskPhase::Ship => "ship",
            TaskPhase::Operate => "operate",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
#[value(rename_all = "kebab-case")]
pub(crate) enum GateSignal {
    Spec,
    Plan,
    Ownership,
    Handoff,
    Review,
    Qa,
    Verify,
    Investigate,
}

impl GateSignal {
    pub(crate) fn name(self) -> &'static str {
        match self {
            GateSignal::Spec => "spec",
            GateSignal::Plan => "plan",
            GateSignal::Ownership => "ownership",
            GateSignal::Handoff => "handoff",
            GateSignal::Review => "review",
            GateSignal::Qa => "qa",
            GateSignal::Verify => "verify",
            GateSignal::Investigate => "investigate",
        }
    }
}

#[derive(clap::Args, Clone)]
pub(crate) struct TaskStateBeginArgs {
    #[command(flatten)]
    pub(crate) provider_args: ProviderArgs,
    #[command(flatten)]
    pub(crate) pack_args: PackArgs,
    #[arg(long, help = "Task title")]
    pub(crate) title: String,
    #[arg(long, value_enum, default_value = "in-progress", help = "Task status")]
    pub(crate) status: TaskStatus,
    #[arg(long, value_enum, default_value = "plan", help = "Current task phase")]
    pub(crate) phase: TaskPhase,
    #[arg(long, help = "Task owner")]
    pub(crate) owner: Option<String>,
    #[arg(long = "next-action", help = "Next action to resume work")]
    pub(crate) next_action: Option<String>,
    #[arg(long, help = "Emit task state as JSON")]
    pub(crate) json: bool,
}

#[derive(clap::Args, Clone)]
pub(crate) struct TaskStateAdvanceArgs {
    #[arg(long, value_enum, help = "New task status")]
    pub(crate) status: Option<TaskStatus>,
    #[arg(long, value_enum, help = "New task phase")]
    pub(crate) phase: Option<TaskPhase>,
    #[arg(long = "next-action", help = "Next action to resume work")]
    pub(crate) next_action: Option<String>,
    #[arg(long, help = "Failure summary")]
    pub(crate) failure: Option<String>,
    #[arg(long, help = "Clear stored failure summary")]
    pub(crate) clear_failure: bool,
    #[arg(
        long,
        value_enum,
        value_delimiter = ',',
        help = "Mark completed gate signals"
    )]
    pub(crate) complete: Vec<GateSignal>,
    #[arg(
        long = "clear-complete",
        value_enum,
        value_delimiter = ',',
        help = "Clear completed gate signals"
    )]
    pub(crate) clear_complete: Vec<GateSignal>,
    #[arg(long, help = "Increment attempt counter")]
    pub(crate) increment_attempt: bool,
    #[arg(long, help = "Emit task state as JSON")]
    pub(crate) json: bool,
}

#[derive(clap::Args, Clone)]
pub(crate) struct TaskStateShowArgs {
    #[arg(long, help = "Emit task state as JSON")]
    pub(crate) json: bool,
}

#[derive(clap::Args, Clone)]
pub(crate) struct GateCheckArgs {
    #[arg(long, value_enum, help = "Evaluate the contract for a target phase")]
    pub(crate) target_phase: Option<TaskPhase>,
    #[arg(
        long,
        value_enum,
        value_delimiter = ',',
        help = "Additional completed gate signals to include in the evaluation"
    )]
    pub(crate) completed: Vec<GateSignal>,
    #[arg(long, help = "Emit gate report as JSON")]
    pub(crate) json: bool,
}

#[derive(clap::Args, Clone)]
pub(crate) struct GateApplyArgs {
    #[arg(long, value_enum, help = "Advance the contract to a target phase")]
    pub(crate) target_phase: Option<TaskPhase>,
    #[arg(
        long,
        value_enum,
        value_delimiter = ',',
        help = "Completed gate signals to persist before applying the gate"
    )]
    pub(crate) completed: Vec<GateSignal>,
    #[arg(long, help = "Emit gate report as JSON")]
    pub(crate) json: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
#[value(rename_all = "kebab-case")]
pub(crate) enum RecordKind {
    Opportunity,
    Decision,
    Project,
    Task,
    Support,
    Growth,
    Ops,
    Risk,
    Handoff,
}

impl RecordKind {
    pub(crate) fn name(self) -> &'static str {
        match self {
            RecordKind::Opportunity => "opportunity",
            RecordKind::Decision => "decision",
            RecordKind::Project => "project",
            RecordKind::Task => "task",
            RecordKind::Support => "support",
            RecordKind::Growth => "growth",
            RecordKind::Ops => "ops",
            RecordKind::Risk => "risk",
            RecordKind::Handoff => "handoff",
        }
    }

    pub(crate) fn record_type(self) -> &'static str {
        match self {
            RecordKind::Opportunity => "OpportunityRecord",
            RecordKind::Decision => "DecisionRecord",
            RecordKind::Project => "ProjectRecord",
            RecordKind::Task => "TaskRecord",
            RecordKind::Support => "SupportRecord",
            RecordKind::Growth => "GrowthRecord",
            RecordKind::Ops => "OpsRecord",
            RecordKind::Risk => "RiskRecord",
            RecordKind::Handoff => "HandoffRecord",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
#[value(rename_all = "kebab-case")]
pub(crate) enum RecordSurface {
    LocalDocs,
    GithubIssue,
    Both,
}

impl RecordSurface {
    pub(crate) fn name(self) -> &'static str {
        match self {
            RecordSurface::LocalDocs => "local-docs",
            RecordSurface::GithubIssue => "github-issue",
            RecordSurface::Both => "both",
        }
    }

    pub(crate) fn includes_local_docs(self) -> bool {
        matches!(self, RecordSurface::LocalDocs | RecordSurface::Both)
    }

    pub(crate) fn includes_github_issue(self) -> bool {
        matches!(self, RecordSurface::GithubIssue | RecordSurface::Both)
    }
}

#[derive(clap::Args, Clone)]
pub(crate) struct RecordArgs {
    #[arg(long = "type", value_enum, help = "Record type to create")]
    pub(crate) record_type: RecordKind,
    #[arg(long, help = "Record title")]
    pub(crate) title: String,
    #[arg(long, default_value = "draft", help = "Record status")]
    pub(crate) status: String,
    #[arg(long, help = "Record owner")]
    pub(crate) owner: Option<String>,
    #[arg(long = "next-action", help = "Next action to keep the work resumable")]
    pub(crate) next_action: Option<String>,
    #[arg(
        long,
        help = "Attach the active local task-state to the record and use its owner or next action when missing"
    )]
    pub(crate) from_task_state: bool,
    #[arg(
        long,
        value_enum,
        default_value = "local-docs",
        help = "Where to write the record"
    )]
    pub(crate) surface: RecordSurface,
    #[arg(
        long = "output-dir",
        default_value = ".llm-bootstrap/records",
        help = "Directory for local record markdown files"
    )]
    pub(crate) output_dir: PathBuf,
    #[arg(
        long = "github-repo",
        help = "Optional GitHub repo for issue creation, such as owner/repo"
    )]
    pub(crate) github_repo: Option<String>,
    #[arg(
        long,
        help = "Show the record without writing local files or GitHub issues"
    )]
    pub(crate) dry_run: bool,
}

#[derive(clap::Args, Clone, Default)]
pub(crate) struct WizardArgs {}
