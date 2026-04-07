mod cli;
mod fs_ops;
mod json_ops;
mod layout;
mod manifest;
mod providers;
mod runtime;

use anyhow::{Context, Result, bail};
use clap::Parser;
use cli::{Cli, Command, DoctorArgs, InstallArgs, Provider, ProviderArgs, UninstallArgs};
use manifest::{BaselineMcp, BootstrapManifest};
use providers::{codex, gemini};
use runtime::{command_exists, ensure_runtime_dependencies, home_dir, repo_root};
use std::env;
use std::fs;
use std::path::PathBuf;

fn main() -> Result<()> {
    let cli = Cli::parse();
    let manifest = load_manifest()?;

    match cli.command.unwrap_or(Command::Install(InstallArgs {
        provider_args: ProviderArgs { providers: None },
        mode: None,
        without_rtk: false,
    })) {
        Command::Install(args) => install(args, &manifest),
        Command::Uninstall(args) => uninstall(args, &manifest),
        Command::Doctor(args) => doctor(args, &manifest),
    }
}

fn load_manifest() -> Result<BootstrapManifest> {
    let path = repo_root().join("bootstrap.toml");
    let raw =
        fs::read_to_string(&path).with_context(|| format!("failed to read {}", path.display()))?;
    toml::from_str(&raw).with_context(|| format!("failed to parse {}", path.display()))
}

fn install(args: InstallArgs, manifest: &BootstrapManifest) -> Result<()> {
    let providers = selected_providers(&args.provider_args, manifest);
    let mode = args.mode.unwrap_or(manifest.bootstrap.default_mode);
    let rtk_enabled = is_rtk_enabled(args.without_rtk, manifest);
    ensure_runtime_dependencies(rtk_enabled)?;
    let home = home_dir()?;
    let enabled_mcp = enabled_mcp(manifest);

    for provider in &providers {
        match *provider {
            Provider::Codex => codex::install(&home, mode, manifest, &enabled_mcp, rtk_enabled)?,
            Provider::Gemini => gemini::install(&home, mode, manifest, &enabled_mcp, rtk_enabled)?,
        }
    }

    println!(
        "installed providers: {} (mode: {}, rtk: {})",
        provider_names(&providers),
        mode.name(),
        if rtk_enabled { "enabled" } else { "disabled" }
    );
    Ok(())
}

fn uninstall(args: UninstallArgs, manifest: &BootstrapManifest) -> Result<()> {
    let home = home_dir()?;
    let providers = selected_providers(&args.provider_args, manifest);
    let rtk_enabled = is_rtk_enabled(args.without_rtk, manifest);

    for provider in &providers {
        match *provider {
            Provider::Codex => codex::uninstall(&home, rtk_enabled)?,
            Provider::Gemini => gemini::uninstall(&home, manifest, rtk_enabled)?,
        }
    }

    println!(
        "uninstalled providers: {} (rtk: {})",
        provider_names(&providers),
        if rtk_enabled { "enabled" } else { "disabled" }
    );
    Ok(())
}

fn doctor(args: DoctorArgs, manifest: &BootstrapManifest) -> Result<()> {
    let home = home_dir()?;
    let providers = selected_providers(&args.provider_args, manifest);
    let mut failures = Vec::new();
    let mut warnings = Vec::new();
    let rtk_enabled = is_rtk_enabled(args.without_rtk, manifest);
    let enabled_mcp = enabled_mcp(manifest);

    println!("[doctor] commands");
    let mut commands = vec!["node", "npx"];
    if rtk_enabled {
        commands.insert(0, "rtk");
    }
    for command in commands {
        if command_exists(command) {
            println!("[ok] command {}", command);
        } else {
            println!("[missing] command {}", command);
            failures.push(PathBuf::from(command));
        }
    }

    println!("[doctor] api");
    for gated in &manifest.mcp.env_gated {
        if env_is_set(&gated.env) {
            println!("[ok] env {} enables {}", gated.env, gated.name.name());
        } else {
            println!(
                "[warn] mcp {} disabled: env {} not set; {}",
                gated.name.name(),
                gated.env,
                env_warning(&gated.env)
            );
            warnings.push(format!("{} disabled", gated.name.name()));
        }
    }

    for provider in &providers {
        println!("[doctor] provider {}", provider.name());
        let checks = match provider {
            Provider::Codex => codex::doctor_checks(&home, manifest, &enabled_mcp, rtk_enabled),
            Provider::Gemini => gemini::doctor_checks(&home, &enabled_mcp, rtk_enabled),
        };

        for path in checks {
            if path.exists() {
                println!("[ok] {}", path.display());
            } else {
                println!("[missing] {}", path.display());
                failures.push(path);
            }
        }
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

fn env_warning(name: &str) -> &'static str {
    match name {
        "EXA_API_KEY" => "Exa stays disabled until EXA_API_KEY is exported",
        "CONTEXT7_API_KEY" => "Context7 stays disabled until CONTEXT7_API_KEY is exported",
        _ => "recommended runtime env is missing",
    }
}

fn env_is_set(name: &str) -> bool {
    env::var(name)
        .map(|value| !value.trim().is_empty())
        .unwrap_or(false)
}

fn selected_providers(args: &ProviderArgs, manifest: &BootstrapManifest) -> Vec<Provider> {
    args.providers
        .clone()
        .unwrap_or_else(|| manifest.bootstrap.providers.clone())
}

fn is_rtk_enabled(without_rtk: bool, manifest: &BootstrapManifest) -> bool {
    manifest.external.rtk.enabled && !without_rtk
}

fn enabled_mcp(manifest: &BootstrapManifest) -> Vec<BaselineMcp> {
    resolve_enabled_mcp_with(manifest, env_is_set)
}

fn resolve_enabled_mcp_with<F>(manifest: &BootstrapManifest, is_enabled: F) -> Vec<BaselineMcp>
where
    F: Fn(&str) -> bool,
{
    let mut enabled = manifest.mcp.always_on.clone();
    enabled.extend(
        manifest
            .mcp
            .env_gated
            .iter()
            .filter(|gated| is_enabled(&gated.env))
            .map(|gated| gated.name),
    );
    enabled
}

fn provider_names(providers: &[Provider]) -> String {
    providers
        .iter()
        .map(|provider| provider.name())
        .collect::<Vec<_>>()
        .join(",")
}

#[cfg(test)]
mod tests {
    use crate::cli::ApplyMode;
    use crate::fs_ops::render_tokens;
    use crate::json_ops::{
        cleanup_extension_enablement, merge_json, preserved_gemini_runtime_state,
        prune_rtk_gemini_hooks, remove_baseline_mcp_servers,
    };
    use crate::manifest::{
        BaselineMcp, BootstrapManifest, BootstrapSection, EnvGatedMcp, ExternalSection, McpSection,
        RtkSection,
    };
    use crate::providers::{codex, gemini};
    use serde_json::json;
    use std::{
        fs,
        path::{Path, PathBuf},
    };

    fn test_manifest() -> BootstrapManifest {
        BootstrapManifest {
            bootstrap: BootstrapSection {
                providers: vec![super::Provider::Codex, super::Provider::Gemini],
                default_mode: ApplyMode::Merge,
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
        }
    }

    fn temp_home() -> PathBuf {
        let path = std::env::temp_dir().join(format!(
            "llm-bootstrap-test-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(&path).unwrap();
        path
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
        let blocks = codex::mcp_blocks(Path::new("/tmp/home"), &enabled);
        assert!(blocks.contains("chrome-devtools-mcp.sh"));
        assert!(!blocks.contains("context7-mcp.sh"));
        assert!(!blocks.contains("exa-mcp.sh"));
        assert!(!blocks.contains("playwright-mcp.sh"));
        assert!(!blocks.contains("github-mcp.sh"));
    }

    #[test]
    fn codex_plugin_blocks_are_always_enabled() {
        assert!(codex::plugin_blocks().contains("llm-dev-kit@llm-bootstrap"));
    }

    #[test]
    fn apply_mode_names_match_cli_values() {
        assert_eq!(ApplyMode::Merge.name(), "merge");
        assert_eq!(ApplyMode::Replace.name(), "replace");
    }

    #[test]
    fn preserved_gemini_runtime_state_keeps_auth_shape_only() {
        let existing = json!({
            "selectedAuthType": "oauth-personal",
            "accounts": [{"email": "dev@example.com"}],
            "general": {"defaultApprovalMode": "plan"},
            "mcpServers": {"legacy": {"command": "noop"}}
        });

        let preserved = preserved_gemini_runtime_state(&existing);

        assert_eq!(preserved["selectedAuthType"], json!("oauth-personal"));
        assert_eq!(preserved["accounts"][0]["email"], json!("dev@example.com"));
        assert!(preserved.get("general").is_none());
        assert!(preserved.get("mcpServers").is_none());
    }

    #[test]
    fn enabled_mcp_turns_on_env_gated_entries_only_when_keys_exist() {
        let manifest = test_manifest();
        let enabled = super::resolve_enabled_mcp_with(&manifest, |name| name == "EXA_API_KEY");

        assert!(enabled.contains(&BaselineMcp::ChromeDevtools));
        assert!(enabled.contains(&BaselineMcp::Exa));
        assert!(!enabled.contains(&BaselineMcp::Context7));
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
    fn remove_baseline_mcp_servers_keeps_unmanaged_entries() {
        let manifest = test_manifest();
        let mut settings = json!({
            "mcpServers": {
                "chrome-devtools": {"command": "a"},
                "context7": {"command": "b"},
                "exa": {"command": "c"},
                "bootpay": {"command": "keep"}
            }
        });

        remove_baseline_mcp_servers(&mut settings, &manifest);

        assert!(settings["mcpServers"].get("chrome-devtools").is_none());
        assert!(settings["mcpServers"].get("context7").is_none());
        assert!(settings["mcpServers"].get("exa").is_none());
        assert_eq!(settings["mcpServers"]["bootpay"]["command"], json!("keep"));
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
        codex::install(&home, ApplyMode::Merge, &manifest, &enabled, false).unwrap();
        let codex_home = home.join(".codex");
        assert!(codex_home.join("config.toml").exists());
        assert!(codex_home.join("AGENTS.md").exists());
        assert!(!codex_home.join("RTK.md").exists());

        codex::uninstall(&home, false).unwrap();
        assert!(!codex_home.join("config.toml").exists());
        assert!(!codex_home.join("AGENTS.md").exists());
        assert!(codex_home.join("backups").exists());

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
        gemini::install(&home, ApplyMode::Merge, &manifest, &enabled, false).unwrap();
        let installed = fs::read_to_string(gemini_home.join("settings.json")).unwrap();
        assert!(installed.contains("/tmp/custom-run-shell.sh"));
        assert!(gemini_home.join("GEMINI.md").exists());
        assert!(!gemini_home.join("hooks/rtk-hook-gemini.sh").exists());

        gemini::uninstall(&home, &manifest, false).unwrap();
        let uninstalled = fs::read_to_string(gemini_home.join("settings.json")).unwrap();
        assert!(uninstalled.contains("/tmp/custom-run-shell.sh"));
        assert!(!gemini_home.join("GEMINI.md").exists());
        assert!(!gemini_home.join("extensions/llm-bootstrap-dev").exists());
        assert!(gemini_home.join("backups").exists());

        fs::remove_dir_all(home).unwrap();
    }
}
