use crate::manifest::BaselineMcp;
use anyhow::{Context, Result};
use serde_json::{Value, json};
use std::fs;
use std::path::Path;

#[derive(Default)]
pub(crate) struct InstalledState {
    pub(crate) active_preset: Option<String>,
    pub(crate) active_packs: Vec<String>,
    pub(crate) active_harnesses: Vec<String>,
    pub(crate) active_connectors: Vec<String>,
    pub(crate) active_automations: Vec<String>,
    pub(crate) active_surfaces: Vec<String>,
    pub(crate) managed_paths: Vec<String>,
}

impl InstalledState {
    pub(crate) fn mismatch(
        &self,
        requested_preset: Option<&str>,
        requested_packs: &[String],
        requested_harnesses: &[String],
        requested_connectors: &[String],
        requested_automations: &[String],
        requested_surfaces: &[String],
        requested_managed_paths: &[String],
    ) -> bool {
        self.active_preset.as_deref() != requested_preset
            || self.active_packs != requested_packs
            || self.active_harnesses != requested_harnesses
            || self.active_connectors != requested_connectors
            || self.active_automations != requested_automations
            || self.active_surfaces != requested_surfaces
            || self.managed_paths != requested_managed_paths
    }
}

pub(crate) fn read_installed_state(root: &Path) -> Result<InstalledState> {
    let path = state_path(root);
    if !path.exists() {
        return Ok(InstalledState::default());
    }

    let raw =
        fs::read_to_string(&path).with_context(|| format!("failed to read {}", path.display()))?;
    let value: Value = serde_json::from_str(&raw)
        .with_context(|| format!("failed to parse {}", path.display()))?;

    Ok(InstalledState {
        active_preset: value
            .get("active_preset")
            .and_then(Value::as_str)
            .map(ToOwned::to_owned),
        active_packs: json_string_array(&value, "active_packs"),
        active_harnesses: json_string_array(&value, "active_harnesses"),
        active_connectors: json_string_array(&value, "active_connectors"),
        active_automations: json_string_array(&value, "active_automations"),
        active_surfaces: json_string_array(&value, "active_surfaces"),
        managed_paths: json_string_array(&value, "managed_paths"),
    })
}

pub(crate) fn managed_mcp_names(root: &Path) -> Result<Vec<String>> {
    let path = state_path(root);
    if !path.exists() {
        return Ok(Vec::new());
    }

    let raw =
        fs::read_to_string(&path).with_context(|| format!("failed to read {}", path.display()))?;
    let value: Value = serde_json::from_str(&raw)
        .with_context(|| format!("failed to parse {}", path.display()))?;
    Ok(json_string_array(&value, "managed_mcp"))
}

pub(crate) fn write_installed_state(
    root: &Path,
    enabled_mcp: &[BaselineMcp],
    state: &InstalledState,
) -> Result<()> {
    let path = state_path(root);
    let state = json!({
        "managed_mcp": enabled_mcp.iter().map(|mcp| mcp.name()).collect::<Vec<_>>(),
        "active_preset": state.active_preset,
        "active_packs": state.active_packs,
        "active_harnesses": state.active_harnesses,
        "active_connectors": state.active_connectors,
        "active_automations": state.active_automations,
        "active_surfaces": state.active_surfaces,
        "managed_paths": state.managed_paths,
    });
    fs::write(
        &path,
        format!("{}\n", serde_json::to_string_pretty(&state)?),
    )
    .with_context(|| format!("failed to write {}", path.display()))?;
    Ok(())
}

fn state_path(root: &Path) -> std::path::PathBuf {
    root.join("llm-bootstrap-state.json")
}

fn json_string_array(value: &Value, key: &str) -> Vec<String> {
    value
        .get(key)
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(Value::as_str)
                .map(ToOwned::to_owned)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default()
}
