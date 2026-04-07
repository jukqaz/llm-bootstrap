use anyhow::{Context, Result, bail};
use std::env;
use std::path::{Path, PathBuf};
use std::process::Command as ProcessCommand;
use std::time::{SystemTime, UNIX_EPOCH};

pub(crate) fn ensure_runtime_dependencies(rtk_enabled: bool) -> Result<()> {
    if !command_exists("brew") {
        bail!("Homebrew is required");
    }

    ensure_brew_formula("node", "node")?;
    if rtk_enabled {
        ensure_brew_formula("rtk-ai/tap/rtk", "rtk")?;
    }
    Ok(())
}

fn ensure_brew_formula(formula: &str, binary: &str) -> Result<()> {
    if command_exists(binary) {
        return Ok(());
    }

    run_command(
        "brew",
        ["install", formula],
        &format!("installing {} with Homebrew", formula),
    )
}

pub(crate) fn command_exists(name: &str) -> bool {
    env::var_os("PATH")
        .map(|paths| {
            env::split_paths(&paths)
                .map(|path| path.join(name))
                .any(|candidate| candidate.exists())
        })
        .unwrap_or(false)
}

fn run_command<I, S>(program: &str, args: I, context: &str) -> Result<()>
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    let collected = args
        .into_iter()
        .map(|value| value.as_ref().to_string())
        .collect::<Vec<_>>();
    let status = ProcessCommand::new(program)
        .args(&collected)
        .status()
        .with_context(|| format!("failed while {}", context))?;

    if status.success() {
        Ok(())
    } else {
        bail!(
            "command failed while {}: {} {:?}",
            context,
            program,
            collected
        )
    }
}

pub(crate) fn run_command_in_home<I, S>(
    home: &Path,
    program: &str,
    args: I,
    context: &str,
) -> Result<()>
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    let collected = args
        .into_iter()
        .map(|value| value.as_ref().to_string())
        .collect::<Vec<_>>();
    let status = ProcessCommand::new(program)
        .env("HOME", home)
        .args(&collected)
        .status()
        .with_context(|| format!("failed while {}", context))?;

    if status.success() {
        Ok(())
    } else {
        bail!(
            "command failed while {}: {} {:?}",
            context,
            program,
            collected
        )
    }
}

pub(crate) fn home_dir() -> Result<PathBuf> {
    if let Some(path) = dirs::home_dir() {
        Ok(path)
    } else {
        env::var("HOME")
            .map(PathBuf::from)
            .context("HOME is not set")
    }
}

pub(crate) fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

pub(crate) fn timestamp_string() -> Result<String> {
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .context("system clock is before unix epoch")?;
    Ok(duration.as_secs().to_string())
}
