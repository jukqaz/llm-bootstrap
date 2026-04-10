use crate::manifest::BaselineMcp;
use anyhow::{Context, Result};
use serde_json::{Value, json};
use std::fs;
use std::path::Path;

#[derive(Default)]
pub(crate) struct InstalledState {
    pub(crate) active_preset: Option<String>,
    pub(crate) record_surface: Option<String>,
    pub(crate) active_packs: Vec<String>,
    pub(crate) active_harnesses: Vec<String>,
    pub(crate) active_connectors: Vec<String>,
    pub(crate) active_automations: Vec<String>,
    pub(crate) active_record_templates: Vec<String>,
    pub(crate) active_surfaces: Vec<String>,
    pub(crate) managed_paths: Vec<String>,
}

pub(crate) struct RequestedState<'a> {
    pub(crate) active_preset: Option<&'a str>,
    pub(crate) record_surface: Option<&'a str>,
    pub(crate) active_packs: &'a [String],
    pub(crate) active_harnesses: &'a [String],
    pub(crate) active_connectors: &'a [String],
    pub(crate) active_automations: &'a [String],
    pub(crate) active_record_templates: &'a [String],
    pub(crate) active_surfaces: &'a [String],
    pub(crate) managed_paths: &'a [String],
}

impl InstalledState {
    pub(crate) fn mismatch(&self, requested: &RequestedState<'_>) -> bool {
        self.active_preset.as_deref() != requested.active_preset
            || requested
                .record_surface
                .is_some_and(|surface| self.record_surface.as_deref() != Some(surface))
            || self.active_packs != requested.active_packs
            || self.active_harnesses != requested.active_harnesses
            || self.active_connectors != requested.active_connectors
            || self.active_automations != requested.active_automations
            || self.active_record_templates != requested.active_record_templates
            || self.active_surfaces != requested.active_surfaces
            || self.managed_paths != requested.managed_paths
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
        record_surface: value
            .get("record_surface")
            .and_then(Value::as_str)
            .map(ToOwned::to_owned),
        active_packs: json_string_array(&value, "active_packs"),
        active_harnesses: json_string_array(&value, "active_harnesses"),
        active_connectors: json_string_array(&value, "active_connectors"),
        active_automations: json_string_array(&value, "active_automations"),
        active_record_templates: json_string_array(&value, "active_record_templates"),
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
        "record_surface": state.record_surface,
        "active_packs": state.active_packs,
        "active_harnesses": state.active_harnesses,
        "active_connectors": state.active_connectors,
        "active_automations": state.active_automations,
        "active_record_templates": state.active_record_templates,
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

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct TaskState {
    pub(crate) id: String,
    pub(crate) title: String,
    pub(crate) status: String,
    pub(crate) phase: String,
    pub(crate) owner: Option<String>,
    pub(crate) next_action: Option<String>,
    pub(crate) providers: Vec<String>,
    pub(crate) packs: Vec<String>,
    pub(crate) harnesses: Vec<String>,
    pub(crate) completed_signals: Vec<String>,
    pub(crate) attempt_count: u64,
    pub(crate) last_failure: Option<String>,
    pub(crate) updated_at: String,
}

pub(crate) fn read_task_state(home: &Path) -> Result<Option<TaskState>> {
    let path = task_state_path(home);
    if !path.exists() {
        return Ok(None);
    }

    let raw =
        fs::read_to_string(&path).with_context(|| format!("failed to read {}", path.display()))?;
    let value: Value = serde_json::from_str(&raw)
        .with_context(|| format!("failed to parse {}", path.display()))?;

    Ok(Some(TaskState {
        id: value
            .get("id")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string(),
        title: value
            .get("title")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string(),
        status: value
            .get("status")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string(),
        phase: value
            .get("phase")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string(),
        owner: value
            .get("owner")
            .and_then(Value::as_str)
            .map(ToOwned::to_owned),
        next_action: value
            .get("next_action")
            .and_then(Value::as_str)
            .map(ToOwned::to_owned),
        providers: json_string_array(&value, "providers"),
        packs: json_string_array(&value, "packs"),
        harnesses: json_string_array(&value, "harnesses"),
        completed_signals: json_string_array(&value, "completed_signals"),
        attempt_count: value
            .get("attempt_count")
            .and_then(Value::as_u64)
            .unwrap_or_default(),
        last_failure: value
            .get("last_failure")
            .and_then(Value::as_str)
            .map(ToOwned::to_owned),
        updated_at: value
            .get("updated_at")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string(),
    }))
}

pub(crate) fn write_task_state(home: &Path, state: &TaskState) -> Result<()> {
    let path = task_state_path(home);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }
    let value = json!({
        "id": state.id,
        "title": state.title,
        "status": state.status,
        "phase": state.phase,
        "owner": state.owner,
        "next_action": state.next_action,
        "providers": state.providers,
        "packs": state.packs,
        "harnesses": state.harnesses,
        "completed_signals": state.completed_signals,
        "attempt_count": state.attempt_count,
        "last_failure": state.last_failure,
        "updated_at": state.updated_at,
    });
    fs::write(
        &path,
        format!("{}\n", serde_json::to_string_pretty(&value)?),
    )
    .with_context(|| format!("failed to write {}", path.display()))?;
    Ok(())
}

pub(crate) fn clear_task_state(home: &Path) -> Result<()> {
    let path = task_state_path(home);
    if path.exists() {
        fs::remove_file(&path).with_context(|| format!("failed to remove {}", path.display()))?;
    }
    Ok(())
}

fn state_path(root: &Path) -> std::path::PathBuf {
    root.join("llm-bootstrap-state.json")
}

fn task_state_path(home: &Path) -> std::path::PathBuf {
    home.join(".llm-bootstrap/task-state.json")
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

#[cfg(test)]
mod tests {
    use super::{TaskState, clear_task_state, read_task_state, write_task_state};
    use std::fs;

    #[test]
    fn task_state_round_trip_and_clear() {
        let home = std::env::temp_dir().join(format!(
            "llm-bootstrap-task-state-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let state = TaskState {
            id: "task_1".to_string(),
            title: "Probe harness".to_string(),
            status: "in-progress".to_string(),
            phase: "review".to_string(),
            owner: Some("owner".to_string()),
            next_action: Some("run probe".to_string()),
            providers: vec!["codex".to_string(), "gemini".to_string()],
            packs: vec!["delivery-pack".to_string()],
            harnesses: vec!["ralph-loop".to_string()],
            completed_signals: vec!["ownership".to_string(), "review".to_string()],
            attempt_count: 2,
            last_failure: Some("none".to_string()),
            updated_at: "123".to_string(),
        };

        write_task_state(&home, &state).unwrap();
        let loaded = read_task_state(&home).unwrap().unwrap();
        assert_eq!(loaded, state);

        clear_task_state(&home).unwrap();
        assert!(read_task_state(&home).unwrap().is_none());

        let _ = fs::remove_dir_all(home.join(".llm-bootstrap"));
    }
}
