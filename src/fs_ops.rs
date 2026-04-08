use crate::manifest::BaselineMcp;
use anyhow::{Context, Result};
use std::fs;
#[cfg(unix)]
use std::os::unix::fs::FileTypeExt;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

pub(crate) fn create_backup_root(provider_root: &Path, timestamp: &str) -> Result<PathBuf> {
    let backups_dir = provider_root.join("backups");
    let mut backup_root = backups_dir.join(format!("llm-bootstrap-{timestamp}"));
    let mut suffix = 1usize;
    while backup_root.exists() {
        backup_root = backups_dir.join(format!("llm-bootstrap-{timestamp}-{suffix}"));
        suffix += 1;
    }
    fs::create_dir_all(&backup_root)
        .with_context(|| format!("failed to create {}", backup_root.display()))?;
    Ok(backup_root)
}

pub(crate) fn backup_relative(root: &Path, backup_root: &Path, relative: &Path) -> Result<()> {
    let source = root.join(relative);
    if !source.exists() {
        return Ok(());
    }

    let destination = backup_root.join(relative);
    if source.is_dir() {
        copy_raw_dir(&source, &destination)?;
    } else {
        copy_raw_file(&source, &destination)?;
    }

    Ok(())
}

pub(crate) fn remove_if_exists(path: &Path) -> Result<()> {
    if !path.exists() {
        return Ok(());
    }

    if path.is_dir() {
        fs::remove_dir_all(path).with_context(|| format!("failed to remove {}", path.display()))?;
    } else {
        fs::remove_file(path).with_context(|| format!("failed to remove {}", path.display()))?;
    }

    Ok(())
}

pub(crate) fn restore_relative(root: &Path, backup_root: &Path, relative: &Path) -> Result<()> {
    let source = backup_root.join(relative);
    if !source.exists() {
        return Ok(());
    }

    let destination = root.join(relative);
    copy_raw_path(&source, &destination)
}

pub(crate) fn restore_named_entry(
    source_root: &Path,
    entry_name: &str,
    destination: &Path,
) -> Result<()> {
    let source = source_root.join(entry_name);
    if !source.exists() {
        return Ok(());
    }

    copy_raw_path(&source, destination)
}

pub(crate) fn resolve_backup_root(
    provider_root: &Path,
    backup_name: Option<&str>,
) -> Result<PathBuf> {
    if let Some(name) = backup_name {
        let candidate = Path::new(name);
        let path = if candidate.is_absolute() {
            candidate.to_path_buf()
        } else {
            provider_root.join("backups").join(name)
        };
        if path.exists() {
            return Ok(path);
        }
        anyhow::bail!("backup not found: {}", path.display());
    }

    let backups_dir = provider_root.join("backups");
    let mut latest = None::<PathBuf>;
    for entry in fs::read_dir(&backups_dir)
        .with_context(|| format!("failed to read {}", backups_dir.display()))?
    {
        let entry = entry?;
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let Some(name) = path.file_name().and_then(|value| value.to_str()) else {
            continue;
        };
        if !name.starts_with("llm-bootstrap-") {
            continue;
        }
        match &latest {
            Some(current) if current.file_name() >= path.file_name() => {}
            _ => latest = Some(path),
        }
    }

    latest.with_context(|| {
        format!(
            "no llm-bootstrap backups found in {}",
            backups_dir.display()
        )
    })
}

fn copy_raw_path(source: &Path, destination: &Path) -> Result<()> {
    if source.is_dir() {
        copy_raw_dir(source, destination)
    } else {
        copy_raw_file(source, destination)
    }
}

fn copy_raw_dir(source: &Path, destination: &Path) -> Result<()> {
    fs::create_dir_all(destination)
        .with_context(|| format!("failed to create {}", destination.display()))?;

    for entry in
        fs::read_dir(source).with_context(|| format!("failed to read {}", source.display()))?
    {
        let entry = entry?;
        let src = entry.path();
        let dest = destination.join(entry.file_name());
        let file_type = entry
            .file_type()
            .with_context(|| format!("failed to stat {}", src.display()))?;

        #[cfg(unix)]
        if is_unsupported_special_file(&file_type) {
            continue;
        }

        if file_type.is_dir() {
            copy_raw_dir(&src, &dest)?;
        } else {
            copy_raw_file(&src, &dest)?;
        }
    }

    Ok(())
}

fn copy_raw_file(source: &Path, destination: &Path) -> Result<()> {
    if let Some(parent) = destination.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }

    fs::copy(source, destination).with_context(|| {
        format!(
            "failed to copy {} -> {}",
            source.display(),
            destination.display()
        )
    })?;

    #[cfg(unix)]
    {
        let permissions = fs::metadata(source)
            .with_context(|| format!("failed to stat {}", source.display()))?
            .permissions();
        fs::set_permissions(destination, permissions).with_context(|| {
            format!(
                "failed to copy permissions {} -> {}",
                source.display(),
                destination.display()
            )
        })?;
    }

    Ok(())
}

pub(crate) fn copy_render_dir(source: &Path, destination: &Path, home: &Path) -> Result<()> {
    fs::create_dir_all(destination)
        .with_context(|| format!("failed to create {}", destination.display()))?;

    for entry in
        fs::read_dir(source).with_context(|| format!("failed to read {}", source.display()))?
    {
        let entry = entry?;
        let src = entry.path();
        let dest = destination.join(entry.file_name());

        if src.is_dir() {
            copy_render_dir(&src, &dest, home)?;
        } else {
            copy_render_file(&src, &dest, is_executable_script(&src), home)?;
        }
    }

    Ok(())
}

pub(crate) fn copy_selected_scripts(
    source: &Path,
    destination: &Path,
    home: &Path,
    baseline: &[BaselineMcp],
) -> Result<()> {
    fs::create_dir_all(destination)
        .with_context(|| format!("failed to create {}", destination.display()))?;

    for mcp in BaselineMcp::all() {
        if !baseline.contains(mcp) {
            remove_if_exists(&destination.join(mcp.script_name()))?;
        }
    }

    for mcp in baseline {
        let name = mcp.script_name();
        copy_render_file(&source.join(name), &destination.join(name), true, home)?;
    }

    Ok(())
}

pub(crate) fn copy_render_file(
    source: &Path,
    destination: &Path,
    executable: bool,
    home: &Path,
) -> Result<()> {
    copy_render_file_with_extras(source, destination, executable, home, &[])
}

pub(crate) fn copy_render_file_with_extras(
    source: &Path,
    destination: &Path,
    executable: bool,
    home: &Path,
    extra_tokens: &[(&str, &str)],
) -> Result<()> {
    if let Some(parent) = destination.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }

    let rendered = render_tokens_with_extras(
        &fs::read_to_string(source)
            .with_context(|| format!("failed to read {}", source.display()))?,
        home,
        extra_tokens,
    );
    fs::write(destination, rendered).with_context(|| {
        format!(
            "failed to write {} -> {}",
            source.display(),
            destination.display()
        )
    })?;

    #[cfg(unix)]
    if executable {
        let permissions = fs::Permissions::from_mode(0o755);
        fs::set_permissions(destination, permissions).with_context(|| {
            format!("failed to set executable bit on {}", destination.display())
        })?;
    }

    Ok(())
}

#[cfg(test)]
pub(crate) fn render_tokens(contents: &str, home: &Path) -> String {
    render_tokens_with_extras(contents, home, &[])
}

fn render_tokens_with_extras(contents: &str, home: &Path, extra_tokens: &[(&str, &str)]) -> String {
    let home_str = home.to_string_lossy();
    let codex_home = home.join(".codex");
    let gemini_home = home.join(".gemini");
    let mut rendered = contents
        .replace("__HOME__", &home_str)
        .replace("__CODEX_HOME__", &codex_home.to_string_lossy())
        .replace("__GEMINI_HOME__", &gemini_home.to_string_lossy());

    for (token, value) in extra_tokens {
        rendered = rendered.replace(token, value);
    }

    rendered
}

pub(crate) fn toml_table_key(name: &str) -> String {
    if name.contains('-') {
        format!("\"{name}\"")
    } else {
        name.to_string()
    }
}

fn is_executable_script(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|value| value.to_str()),
        Some("sh")
    )
}

#[cfg(unix)]
fn is_unsupported_special_file(file_type: &fs::FileType) -> bool {
    file_type.is_socket()
        || file_type.is_fifo()
        || file_type.is_block_device()
        || file_type.is_char_device()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::os::unix::net::UnixListener;

    #[test]
    fn backup_relative_skips_unix_sockets() {
        let root = std::env::temp_dir().join(format!("llm-bootstrap-fsops-{}", std::process::id()));
        let source_dir = root.join("source");
        let backup_dir = root.join("backup");
        fs::create_dir_all(&source_dir).unwrap();
        let socket_path = source_dir.join("test.sock");
        let _listener = UnixListener::bind(&socket_path).unwrap();

        backup_relative(&root, &backup_dir, Path::new("source")).unwrap();

        assert!(!backup_dir.join("source/test.sock").exists());
        let _ = fs::remove_dir_all(root);
    }
}
