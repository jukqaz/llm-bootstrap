use clap::{Parser, Subcommand, ValueEnum};
use serde::Deserialize;
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
    #[arg(long, help = "Emit doctor results as JSON")]
    pub(crate) json: bool,
}

#[derive(clap::Args, Clone, Default)]
pub(crate) struct WizardArgs {}
