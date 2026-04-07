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
use providers::{claude, codex, gemini};
use runtime::{command_exists, ensure_runtime_dependencies, home_dir, repo_root};
use serde::Serialize;
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
            Provider::Claude => claude::install(&home, mode, manifest, &enabled_mcp, rtk_enabled)?,
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
    let enabled_mcp = enabled_mcp(manifest);

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

fn doctor(args: DoctorArgs, manifest: &BootstrapManifest) -> Result<()> {
    let home = home_dir()?;
    let providers = selected_providers(&args.provider_args, manifest);
    let mut failures = Vec::new();
    let mut warnings = Vec::new();
    let rtk_enabled = is_rtk_enabled(args.without_rtk, manifest);
    let enabled_mcp = enabled_mcp(manifest);
    let mut command_checks = Vec::new();
    let mut env_checks = Vec::new();
    let mut provider_reports = Vec::new();

    if !args.json {
        println!("[doctor] commands");
    }
    let mut commands = vec!["node", "npx"];
    if rtk_enabled {
        commands.insert(0, "rtk");
    }
    if providers.contains(&Provider::Claude) {
        commands.insert(0, "claude");
    }
    for command in commands {
        if command_exists(command) {
            if !args.json {
                println!("[ok] command {}", command);
            }
            command_checks.push(DoctorCheck {
                target: command.to_string(),
                status: "ok".to_string(),
                detail: None,
            });
        } else {
            if !args.json {
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

    if !args.json {
        println!("[doctor] api");
    }
    for gated in &manifest.mcp.env_gated {
        if env_is_set(&gated.env) {
            if !args.json {
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
            if !args.json {
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

    for provider in &providers {
        if !args.json {
            println!("[doctor] provider {}", provider.name());
        }
        let checks = match provider {
            Provider::Codex => codex::doctor_checks(&home, manifest, &enabled_mcp, rtk_enabled),
            Provider::Gemini => gemini::doctor_checks(&home, &enabled_mcp, rtk_enabled),
            Provider::Claude => claude::doctor_checks(&home, &enabled_mcp, rtk_enabled),
        };
        let mut provider_checks = Vec::new();

        for path in checks {
            if path.exists() {
                if !args.json {
                    println!("[ok] {}", path.display());
                }
                provider_checks.push(DoctorCheck {
                    target: path.display().to_string(),
                    status: "ok".to_string(),
                    detail: None,
                });
            } else {
                if !args.json {
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
            checks: provider_checks,
        });
    }

    let report = DoctorReport {
        ok: failures.is_empty(),
        warning_count: warnings.len(),
        command_checks,
        env_checks,
        providers: provider_reports,
    };

    if args.json {
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

#[derive(Serialize)]
struct DoctorReport {
    ok: bool,
    warning_count: usize,
    command_checks: Vec<DoctorCheck>,
    env_checks: Vec<DoctorEnvCheck>,
    providers: Vec<DoctorProviderReport>,
}

#[derive(Serialize)]
struct DoctorProviderReport {
    provider: String,
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
        prune_rtk_claude_hooks, prune_rtk_gemini_hooks, remove_baseline_mcp_servers,
    };
    use crate::manifest::{
        BaselineMcp, BootstrapManifest, BootstrapSection, EnvGatedMcp, ExternalSection, McpSection,
        RtkSection,
    };
    use crate::providers::{claude, codex, gemini};
    use serde_json::json;
    use std::{
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
        let counter = TEMP_HOME_COUNTER.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir()
            .join(format!(
                "llm-bootstrap-test-{}-{}-{}",
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
        let root = temp_home().join(".codex");
        fs::create_dir_all(&root).unwrap();
        let blocks =
            codex::mcp_blocks(Path::new("/tmp/home"), &root, &enabled, ApplyMode::Merge).unwrap();
        assert!(blocks.contains("chrome-devtools-mcp.sh"));
        assert!(!blocks.contains("context7-mcp.sh"));
        assert!(!blocks.contains("exa-mcp.sh"));
        assert!(!blocks.contains("playwright-mcp.sh"));
        assert!(!blocks.contains("github-mcp.sh"));
        fs::remove_dir_all(root.parent().unwrap()).unwrap();
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
            assert!(
                parsed.get("model_reasoning_effort").is_some(),
                "missing effort in {}",
                name
            );
            if parsed.get("model_context_window").is_some() {
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
            }
        }

        pinned.sort();
        assert_eq!(pinned, vec!["architect-1m", "planner-1m", "reviewer-1m"]);
    }

    #[test]
    fn claude_agent_templates_use_official_frontmatter_model_fields() {
        let agents_dir = crate::runtime::repo_root().join("templates/claude/agents");
        let expected = [
            ("executor.md", "model: inherit"),
            ("planner.md", "model: inherit"),
            ("reviewer.md", "model: sonnet"),
            ("triage.md", "model: haiku"),
            ("verifier.md", "model: sonnet"),
        ];

        for (file, needle) in expected {
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
            assert!(raw.contains(needle), "{file} missing expected model");
        }
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
    fn codex_merge_preserves_unmanaged_mcp_blocks() {
        let home = temp_home();
        let codex_home = home.join(".codex");
        fs::create_dir_all(&codex_home).unwrap();
        fs::write(
            codex_home.join("config.toml"),
            "[mcp_servers.bootpay]\ncommand = \"bootpay\"\nenabled = true\n",
        )
        .unwrap();

        let blocks = codex::mcp_blocks(
            &home,
            &codex_home,
            &[BaselineMcp::ChromeDevtools],
            ApplyMode::Merge,
        )
        .unwrap();

        assert!(blocks.contains("[mcp_servers.bootpay]"));
        assert!(blocks.contains("[mcp_servers.\"chrome-devtools\"]"));

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

    #[test]
    fn gemini_merge_preserves_unmanaged_mcp_servers() {
        let home = temp_home();
        let manifest = test_manifest();
        let gemini_home = home.join(".gemini");
        fs::create_dir_all(&gemini_home).unwrap();
        fs::write(
            gemini_home.join("settings.json"),
            "{\n  \"mcpServers\": {\n    \"icm\": {\"command\": \"icm\"},\n    \"bootpay\": {\"command\": \"bootpay\"}\n  },\n  \"selectedAuthType\": \"oauth-personal\"\n}\n",
        )
        .unwrap();

        gemini::install(
            &home,
            ApplyMode::Merge,
            &manifest,
            &[BaselineMcp::ChromeDevtools],
            false,
        )
        .unwrap();

        let after: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(gemini_home.join("settings.json")).unwrap())
                .unwrap();
        assert!(after["mcpServers"].get("icm").is_some());
        assert!(after["mcpServers"].get("bootpay").is_some());
        assert!(after["mcpServers"].get("chrome-devtools").is_some());

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
            "{\n  \"mcpServers\": {\n    \"icm\": {\"command\": \"icm\"},\n    \"bootpay\": {\"command\": \"bootpay\"}\n  },\n  \"selectedAuthType\": \"oauth-personal\"\n}\n",
        )
        .unwrap();

        gemini::install(
            &home,
            ApplyMode::Replace,
            &manifest,
            &[BaselineMcp::ChromeDevtools],
            false,
        )
        .unwrap();

        let after: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(gemini_home.join("settings.json")).unwrap())
                .unwrap();
        assert!(after["mcpServers"].get("icm").is_none());
        assert!(after["mcpServers"].get("bootpay").is_none());
        assert!(after["mcpServers"].get("chrome-devtools").is_some());

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

        claude::install(&home, ApplyMode::Merge, &manifest, &enabled, false).unwrap();
        let claude_home = home.join(".claude");
        assert!(claude_home.join("CLAUDE.md").exists());
        assert!(claude_home.join("scripts/chrome-devtools-mcp.sh").exists());
        assert!(!claude_home.join("RTK.md").exists());

        let mcp = claude::claude_user_mcp(&home).unwrap();
        assert!(mcp["mcpServers"].get("chrome-devtools").is_some());

        claude::uninstall(&home, &enabled, false).unwrap();
        assert!(!claude_home.join("CLAUDE.md").exists());
        assert!(!claude_home.join("scripts").exists());
        assert!(claude_home.join("backups").exists());

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
        )
        .unwrap();

        claude::install(
            &home,
            ApplyMode::Merge,
            &manifest,
            &[BaselineMcp::ChromeDevtools],
            false,
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

        claude::install(&home, ApplyMode::Merge, &manifest, &enabled, false).unwrap();
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

        claude::install(&home, ApplyMode::Replace, &manifest, &enabled, false).unwrap();

        let mcp = claude::claude_user_mcp(&home).unwrap();
        assert!(mcp["mcpServers"].get("manual-tool").is_none());
        assert!(mcp["mcpServers"].get("chrome-devtools").is_some());

        fs::remove_dir_all(home).unwrap();
    }
}
