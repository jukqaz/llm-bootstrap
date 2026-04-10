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
    Install(InstallArgs),
    Restore(RestoreArgs),
    Backups(BackupsArgs),
    Uninstall(UninstallArgs),
    Doctor(DoctorArgs),
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
