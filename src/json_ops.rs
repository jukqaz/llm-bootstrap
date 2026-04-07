use crate::manifest::{BaselineMcp, BootstrapManifest};
use anyhow::{Context, Result};
use serde_json::{Value, json};
use std::fs;
use std::path::Path;

pub(crate) fn read_json_or_empty(path: &Path) -> Result<Value> {
    if !path.exists() {
        return Ok(json!({}));
    }

    let raw =
        fs::read_to_string(path).with_context(|| format!("failed to read {}", path.display()))?;
    let value = serde_json::from_str(&raw)
        .with_context(|| format!("failed to parse {}", path.display()))?;
    Ok(value)
}

pub(crate) fn write_json_pretty(path: &Path, value: &Value) -> Result<()> {
    let serialized = serde_json::to_string_pretty(value)?;
    fs::write(path, format!("{serialized}\n"))
        .with_context(|| format!("failed to write {}", path.display()))?;
    Ok(())
}

pub(crate) fn merge_json(target: &mut Value, patch: Value) {
    match (target, patch) {
        (Value::Object(target_map), Value::Object(patch_map)) => {
            for (key, value) in patch_map {
                match target_map.get_mut(&key) {
                    Some(existing) => merge_json(existing, value),
                    None => {
                        target_map.insert(key, value);
                    }
                }
            }
        }
        (slot, value) => *slot = value,
    }
}

pub(crate) fn prune_rtk_gemini_hooks(settings: &mut Value) {
    let Some(root) = settings.as_object_mut() else {
        return;
    };
    let Some(hooks) = root.get_mut("hooks").and_then(Value::as_object_mut) else {
        return;
    };
    let Some(before_tool) = hooks.get_mut("BeforeTool").and_then(Value::as_array_mut) else {
        return;
    };

    before_tool.retain(|entry| !is_rtk_run_shell_command_hook(entry));

    if before_tool.is_empty() {
        hooks.remove("BeforeTool");
    }

    if hooks.is_empty() {
        root.remove("hooks");
    }
}

fn is_rtk_run_shell_command_hook(entry: &Value) -> bool {
    if entry.get("matcher") != Some(&Value::String("run_shell_command".to_string())) {
        return false;
    }

    let Some(commands) = entry.get("hooks").and_then(Value::as_array) else {
        return false;
    };

    commands.iter().any(|hook| {
        hook.get("type") == Some(&Value::String("command".to_string()))
            && hook
                .get("command")
                .and_then(Value::as_str)
                .map(|command| command.ends_with("/.gemini/hooks/rtk-hook-gemini.sh"))
                .unwrap_or(false)
    })
}

pub(crate) fn cleanup_gemini_settings(
    settings_path: &Path,
    manifest: &BootstrapManifest,
    rtk_enabled: bool,
) -> Result<()> {
    if !settings_path.exists() {
        return Ok(());
    }

    let mut settings = read_json_or_empty(settings_path)?;
    remove_baseline_mcp_servers(&mut settings, manifest);
    if rtk_enabled {
        prune_rtk_gemini_hooks(&mut settings);
    }
    write_or_remove_json(settings_path, &settings)
}

pub(crate) fn remove_baseline_mcp_servers(settings: &mut Value, manifest: &BootstrapManifest) {
    let Some(root) = settings.as_object_mut() else {
        return;
    };
    let Some(mcp_servers) = root.get_mut("mcpServers").and_then(Value::as_object_mut) else {
        return;
    };

    for mcp in BaselineMcp::all() {
        if manifest.mcp.always_on.contains(mcp)
            || manifest
                .mcp
                .env_gated
                .iter()
                .any(|gated| gated.name == *mcp)
        {
            mcp_servers.remove(mcp.name());
        }
    }

    if mcp_servers.is_empty() {
        root.remove("mcpServers");
    }
}

pub(crate) fn cleanup_extension_enablement(path: &Path) -> Result<()> {
    if !path.exists() {
        return Ok(());
    }

    let mut enablement = read_json_or_empty(path)?;
    let Some(root) = enablement.as_object_mut() else {
        return Ok(());
    };
    root.remove("llm-bootstrap-dev");
    write_or_remove_json(path, &enablement)
}

fn write_or_remove_json(path: &Path, value: &Value) -> Result<()> {
    if json_effectively_empty(value) {
        if path.exists() {
            fs::remove_file(path)
                .with_context(|| format!("failed to remove {}", path.display()))?;
        }
        Ok(())
    } else {
        write_json_pretty(path, value)
    }
}

fn json_effectively_empty(value: &Value) -> bool {
    match value {
        Value::Null => true,
        Value::Object(map) => map.is_empty(),
        _ => false,
    }
}

pub(crate) fn preserved_gemini_runtime_state(existing: &Value) -> Value {
    let mut preserved = json!({});

    for key in [
        "selectedAuthType",
        "auth",
        "oauth",
        "accounts",
        "session",
        "sessions",
        "credentialStore",
        "credentials",
    ] {
        if let Some(value) = existing.get(key) {
            preserved[key] = value.clone();
        }
    }

    preserved
}
