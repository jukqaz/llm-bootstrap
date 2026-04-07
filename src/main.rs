mod cli;
mod json_ops;
mod layout;
mod manifest;
mod runtime;

use anyhow::{Context, Result, bail};
use clap::Parser;
use cli::{
    ApplyMode, Cli, Command, DoctorArgs, InstallArgs, Provider, ProviderArgs, UninstallArgs,
};
use json_ops::{
    cleanup_extension_enablement, cleanup_gemini_settings, merge_json,
    preserved_gemini_runtime_state, prune_rtk_gemini_hooks, read_json_or_empty, write_json_pretty,
};
use layout::{
    codex_managed_paths, codex_uninstall_paths, gemini_managed_paths, gemini_uninstall_paths,
};
use manifest::{BaselineMcp, BootstrapManifest};
use runtime::{
    command_exists, ensure_runtime_dependencies, home_dir, repo_root, run_command_in_home,
    timestamp_string,
};
use serde_json::{Map, Value, json};
use std::env;
use std::fs;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

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

    for provider in &providers {
        match *provider {
            Provider::Codex => install_codex(&home, mode, manifest, rtk_enabled)?,
            Provider::Gemini => install_gemini(&home, mode, manifest, rtk_enabled)?,
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
            Provider::Codex => uninstall_codex(&home, rtk_enabled)?,
            Provider::Gemini => uninstall_gemini(&home, manifest, rtk_enabled)?,
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
            Provider::Codex => codex_doctor_checks(&home, manifest, rtk_enabled),
            Provider::Gemini => gemini_doctor_checks(&home, manifest, rtk_enabled),
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

fn codex_doctor_checks(
    home: &Path,
    manifest: &BootstrapManifest,
    rtk_enabled: bool,
) -> Vec<PathBuf> {
    let root = Provider::Codex.home_dir(home);
    let mut checks = vec![
        root.join("config.toml"),
        root.join("AGENTS.md"),
        root.join("agents/planner.toml"),
        root.join(".agents/plugins/marketplace.json"),
        root.join("plugins/llm-dev-kit/.codex-plugin/plugin.json"),
        root.join("plugins/cache/llm-bootstrap/llm-dev-kit/local/.codex-plugin/plugin.json"),
        root.join("SHIP_CHECKLIST.md"),
        root.join("WORKFLOW.md"),
        root.join("plugins/llm-dev-kit/skills/qa-browser/SKILL.md"),
    ];
    if rtk_enabled {
        checks.push(root.join("RTK.md"));
    }

    checks.extend(
        enabled_mcp(manifest)
            .into_iter()
            .map(|mcp| root.join("scripts").join(mcp.script_name())),
    );
    checks
}

fn gemini_doctor_checks(
    home: &Path,
    manifest: &BootstrapManifest,
    rtk_enabled: bool,
) -> Vec<PathBuf> {
    let root = Provider::Gemini.home_dir(home);
    let mut checks = vec![
        root.join("GEMINI.md"),
        root.join("settings.json"),
        root.join("extensions/llm-bootstrap-dev/gemini-extension.json"),
        root.join("extensions/extension-enablement.json"),
        root.join("extensions/llm-bootstrap-dev/agents/qa.md"),
    ];
    if rtk_enabled {
        checks.push(root.join("hooks/rtk-hook-gemini.sh"));
    }

    checks.extend(
        enabled_mcp(manifest)
            .into_iter()
            .map(|mcp| root.join("scripts").join(mcp.script_name())),
    );
    checks
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

fn install_codex(
    home: &Path,
    mode: ApplyMode,
    manifest: &BootstrapManifest,
    rtk_enabled: bool,
) -> Result<()> {
    let root = Provider::Codex.home_dir(home);
    let template_root = repo_root().join("templates/codex");
    let plugin_root = repo_root().join("plugins/llm-dev-kit");
    let marketplace_path = repo_root().join(".agents/plugins/marketplace.json");
    let installed_plugin_root = root.join("plugins/cache/llm-bootstrap/llm-dev-kit/local");
    let bundle_root = repo_root().join("bundles/full/codex");
    let bundle_plugin_root = repo_root().join("bundles/full/plugins/llm-dev-kit");

    fs::create_dir_all(&root)?;
    let backup_root = create_backup_root(&root)?;
    println!("[codex] backup {}", backup_root.display());

    for relative in codex_managed_paths() {
        backup_relative(&root, &backup_root, Path::new(relative))?;
    }

    if mode == ApplyMode::Replace {
        for relative in codex_managed_paths() {
            remove_if_exists(&root.join(relative))?;
        }
    }

    if rtk_enabled {
        run_rtk_codex_init(home)?;
    } else {
        remove_if_exists(&root.join("RTK.md"))?;
    }

    let codex_mcp_blocks = codex_mcp_blocks(home, manifest);
    let codex_plugin_blocks = codex_plugin_blocks();
    copy_render_file_with_extras(
        &template_root.join("config.toml"),
        &root.join("config.toml"),
        false,
        home,
        &[
            ("__CODEX_MCP_BLOCKS__", codex_mcp_blocks.as_str()),
            ("__CODEX_PLUGIN_BLOCKS__", codex_plugin_blocks.as_str()),
        ],
    )?;
    copy_render_file_with_extras(
        &template_root.join("AGENTS.md"),
        &root.join("AGENTS.md"),
        false,
        home,
        &codex_rtk_tokens(rtk_enabled),
    )?;
    copy_render_dir(&template_root.join("agents"), &root.join("agents"), home)?;
    copy_selected_scripts(
        &template_root.join("scripts"),
        &root.join("scripts"),
        home,
        &enabled_mcp(manifest),
    )?;
    copy_render_file(
        &marketplace_path,
        &root.join(".agents/plugins/marketplace.json"),
        false,
        home,
    )?;
    copy_render_dir(&plugin_root, &root.join("plugins/llm-dev-kit"), home)?;
    copy_render_dir(&plugin_root, &installed_plugin_root, home)?;
    copy_render_dir(&bundle_root, &root, home)?;
    copy_render_dir(&bundle_plugin_root, &root.join("plugins/llm-dev-kit"), home)?;
    copy_render_dir(&bundle_plugin_root, &installed_plugin_root, home)?;

    println!("[codex] installed {} ({})", root.display(), mode.name());
    Ok(())
}

fn install_gemini(
    home: &Path,
    mode: ApplyMode,
    manifest: &BootstrapManifest,
    rtk_enabled: bool,
) -> Result<()> {
    let root = Provider::Gemini.home_dir(home);
    let template_root = repo_root().join("templates/gemini");
    let bundle_root = repo_root().join("bundles/full/gemini");

    fs::create_dir_all(&root)?;
    fs::create_dir_all(root.join("hooks"))?;
    fs::create_dir_all(root.join("scripts"))?;
    fs::create_dir_all(root.join("extensions"))?;
    let backup_root = create_backup_root(&root)?;
    println!("[gemini] backup {}", backup_root.display());

    for relative in gemini_managed_paths() {
        backup_relative(&root, &backup_root, Path::new(relative))?;
    }

    if mode == ApplyMode::Replace {
        for relative in gemini_managed_paths() {
            remove_if_exists(&root.join(relative))?;
        }
        fs::create_dir_all(root.join("hooks"))?;
        fs::create_dir_all(root.join("scripts"))?;
        fs::create_dir_all(root.join("extensions"))?;
    }

    if rtk_enabled {
        run_rtk_gemini_init(home)?;
    } else {
        remove_if_exists(&root.join("hooks/rtk-hook-gemini.sh"))?;
    }

    let settings_path = root.join("settings.json");
    let existing_settings = read_json_or_empty(&settings_path)?;
    let enablement_path = root.join("extensions/extension-enablement.json");
    let existing_enablement = read_json_or_empty(&enablement_path)?;

    copy_render_file_with_extras(
        &template_root.join("GEMINI.md"),
        &root.join("GEMINI.md"),
        false,
        home,
        &gemini_rtk_tokens(rtk_enabled),
    )?;
    copy_selected_scripts(
        &template_root.join("scripts"),
        &root.join("scripts"),
        home,
        &enabled_mcp(manifest),
    )?;
    copy_render_dir(
        &template_root.join("extensions/llm-bootstrap-dev"),
        &root.join("extensions/llm-bootstrap-dev"),
        home,
    )?;
    copy_render_dir(&bundle_root, &root, home)?;

    let mut current_settings = match mode {
        ApplyMode::Merge => existing_settings.clone(),
        ApplyMode::Replace => preserved_gemini_runtime_state(&existing_settings),
    };
    merge_json(
        &mut current_settings,
        gemini_settings_patch(home, rtk_enabled),
    );
    if !rtk_enabled {
        prune_rtk_gemini_hooks(&mut current_settings);
    }
    current_settings["mcpServers"] = gemini_mcp_servers(home, manifest);
    write_json_pretty(&settings_path, &current_settings)?;

    let mut enablement = match mode {
        ApplyMode::Merge => existing_enablement,
        ApplyMode::Replace => json!({}),
    };
    let override_path = format!("{}/{}", home.display(), "*");
    enablement["llm-bootstrap-dev"] = json!({
        "overrides": [override_path]
    });
    write_json_pretty(&enablement_path, &enablement)?;

    println!("[gemini] installed {} ({})", root.display(), mode.name());
    Ok(())
}

fn uninstall_codex(home: &Path, rtk_enabled: bool) -> Result<()> {
    let root = Provider::Codex.home_dir(home);
    if !root.exists() {
        println!("[codex] skipped uninstall: {} not found", root.display());
        return Ok(());
    }

    let backup_root = create_backup_root(&root)?;
    println!("[codex] backup {}", backup_root.display());

    for relative in codex_uninstall_paths(rtk_enabled) {
        backup_relative(&root, &backup_root, Path::new(relative))?;
    }

    if rtk_enabled {
        run_rtk_codex_uninstall(home)?;
    }

    for relative in codex_uninstall_paths(rtk_enabled) {
        remove_if_exists(&root.join(relative))?;
    }

    println!("[codex] uninstalled {}", root.display());
    Ok(())
}

fn uninstall_gemini(home: &Path, manifest: &BootstrapManifest, rtk_enabled: bool) -> Result<()> {
    let root = Provider::Gemini.home_dir(home);
    if !root.exists() {
        println!("[gemini] skipped uninstall: {} not found", root.display());
        return Ok(());
    }

    let backup_root = create_backup_root(&root)?;
    println!("[gemini] backup {}", backup_root.display());

    for relative in gemini_uninstall_paths(rtk_enabled) {
        backup_relative(&root, &backup_root, Path::new(relative))?;
    }

    if rtk_enabled {
        run_rtk_gemini_uninstall(home)?;
    }

    remove_if_exists(&root.join("GEMINI.md"))?;
    remove_if_exists(&root.join("scripts"))?;
    remove_if_exists(&root.join("extensions/llm-bootstrap-dev"))?;
    if rtk_enabled {
        remove_if_exists(&root.join("hooks/rtk-hook-gemini.sh"))?;
    }

    cleanup_gemini_settings(&root.join("settings.json"), manifest, rtk_enabled)?;
    cleanup_extension_enablement(&root.join("extensions/extension-enablement.json"))?;

    println!("[gemini] uninstalled {}", root.display());
    Ok(())
}

fn gemini_settings_patch(home: &Path, rtk_enabled: bool) -> Value {
    let hook_path = home.join(".gemini/hooks/rtk-hook-gemini.sh");
    let hooks = if rtk_enabled {
        json!({
            "BeforeTool": [
                {
                    "hooks": [
                        {
                            "command": hook_path.to_string_lossy(),
                            "type": "command"
                        }
                    ],
                    "matcher": "run_shell_command"
                }
            ]
        })
    } else {
        json!({})
    };

    json!({
        "general": {
            "defaultApprovalMode": "plan",
            "enableAutoUpdate": false,
            "enableNotifications": true,
            "plan": {
                "directory": "",
                "modelRouting": true
            },
            "vimMode": false
        },
        "hooks": hooks,
        "ideMode": true,
        "output": {
            "format": "text"
        },
        "showLineNumbers": false,
        "showMemoryUsage": true,
        "ui": {
            "autoThemeSwitching": true,
            "errorVerbosity": "full",
            "hideTips": true,
            "hideWindowTitle": false,
            "inlineThinkingMode": "full",
            "loadingPhrases": "all",
            "showMemoryUsage": true,
            "showStatusInTitle": true
        }
    })
}

fn run_rtk_codex_init(home: &Path) -> Result<()> {
    run_command_in_home(
        home,
        "rtk",
        ["init", "-g", "--codex"],
        "initializing RTK for Codex",
    )
}

fn run_rtk_gemini_init(home: &Path) -> Result<()> {
    run_command_in_home(
        home,
        "rtk",
        ["init", "-g", "--gemini", "--auto-patch"],
        "initializing RTK for Gemini",
    )
}

fn run_rtk_codex_uninstall(home: &Path) -> Result<()> {
    if !command_exists("rtk") {
        println!("[warn] command rtk missing; skipping official Codex RTK uninstall");
        return Ok(());
    }

    run_command_in_home(
        home,
        "rtk",
        ["init", "-g", "--codex", "--uninstall"],
        "uninstalling RTK for Codex",
    )
}

fn run_rtk_gemini_uninstall(home: &Path) -> Result<()> {
    if !command_exists("rtk") {
        println!("[warn] command rtk missing; skipping official Gemini RTK uninstall");
        return Ok(());
    }

    run_command_in_home(
        home,
        "rtk",
        ["init", "-g", "--gemini", "--uninstall", "--auto-patch"],
        "uninstalling RTK for Gemini",
    )
}

fn gemini_mcp_servers(home: &Path, manifest: &BootstrapManifest) -> Value {
    let gemini_home = home.join(".gemini");
    let mut servers = Map::new();

    for mcp in enabled_mcp(manifest) {
        servers.insert(
            mcp.name().to_string(),
            json!({
                "command": gemini_home.join("scripts").join(mcp.script_name()).to_string_lossy().to_string()
            }),
        );
    }

    Value::Object(servers)
}

fn codex_mcp_blocks(home: &Path, manifest: &BootstrapManifest) -> String {
    let codex_home = home.join(".codex");
    enabled_mcp(manifest)
        .into_iter()
        .map(|mcp| {
            format!(
                "[mcp_servers.{name}]\ncommand = \"{command}\"\nenabled = true",
                name = toml_table_key(mcp.name()),
                command = codex_home.join("scripts").join(mcp.script_name()).display()
            )
        })
        .collect::<Vec<_>>()
        .join("\n\n")
}

fn codex_plugin_blocks() -> String {
    "[plugins.\"llm-dev-kit@llm-bootstrap\"]\nenabled = true".to_string()
}

fn create_backup_root(provider_root: &Path) -> Result<PathBuf> {
    let timestamp = timestamp_string()?;
    let backup_root = provider_root
        .join("backups")
        .join(format!("llm-bootstrap-{timestamp}"));
    fs::create_dir_all(&backup_root)
        .with_context(|| format!("failed to create {}", backup_root.display()))?;
    Ok(backup_root)
}

fn backup_relative(root: &Path, backup_root: &Path, relative: &Path) -> Result<()> {
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

fn remove_if_exists(path: &Path) -> Result<()> {
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

fn copy_raw_dir(source: &Path, destination: &Path) -> Result<()> {
    fs::create_dir_all(destination)
        .with_context(|| format!("failed to create {}", destination.display()))?;

    for entry in
        fs::read_dir(source).with_context(|| format!("failed to read {}", source.display()))?
    {
        let entry = entry?;
        let src = entry.path();
        let dest = destination.join(entry.file_name());

        if src.is_dir() {
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

fn copy_render_dir(source: &Path, destination: &Path, home: &Path) -> Result<()> {
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

fn copy_selected_scripts(
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

fn copy_render_file(
    source: &Path,
    destination: &Path,
    executable: bool,
    home: &Path,
) -> Result<()> {
    copy_render_file_with_extras(source, destination, executable, home, &[])
}

fn copy_render_file_with_extras(
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
fn render_tokens(contents: &str, home: &Path) -> String {
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

fn toml_table_key(name: &str) -> String {
    if name.contains('-') {
        format!("\"{name}\"")
    } else {
        name.to_string()
    }
}

fn codex_rtk_tokens(rtk_enabled: bool) -> Vec<(&'static str, &'static str)> {
    if rtk_enabled {
        vec![
            (
                "__RTK_CODEX_RULE__",
                "- Prefer `rtk <command>` for shell commands that can generate large or noisy output, especially `git`, `curl`, test, build, and diff workflows.",
            ),
            (
                "__RTK_CODEX_HELPER__",
                "- `RTK.md` describes the preferred `rtk` shell wrapper workflow.",
            ),
            ("__RTK_CODEX_IMPORT__", "@RTK.md"),
        ]
    } else {
        vec![
            ("__RTK_CODEX_RULE__", ""),
            ("__RTK_CODEX_HELPER__", ""),
            ("__RTK_CODEX_IMPORT__", ""),
        ]
    }
}

fn gemini_rtk_tokens(rtk_enabled: bool) -> Vec<(&'static str, &'static str)> {
    if rtk_enabled {
        vec![(
            "__RTK_GEMINI_RULE__",
            "- Prefer `rtk <command>` for large or noisy shell commands.",
        )]
    } else {
        vec![("__RTK_GEMINI_RULE__", "")]
    }
}

fn is_executable_script(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|value| value.to_str()),
        Some("sh")
    )
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
    use super::{
        ApplyMode, cleanup_extension_enablement, codex_mcp_blocks, codex_plugin_blocks,
        install_codex, install_gemini, merge_json, preserved_gemini_runtime_state, render_tokens,
        uninstall_codex, uninstall_gemini,
    };
    use crate::json_ops::remove_baseline_mcp_servers;
    use crate::manifest::{
        BaselineMcp, BootstrapManifest, BootstrapSection, EnvGatedMcp, ExternalSection, McpSection,
        RtkSection,
    };
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
        let blocks = codex_mcp_blocks(Path::new("/tmp/home"), &test_manifest());
        assert!(blocks.contains("chrome-devtools-mcp.sh"));
        assert!(!blocks.contains("context7-mcp.sh"));
        assert!(!blocks.contains("exa-mcp.sh"));
        assert!(!blocks.contains("playwright-mcp.sh"));
        assert!(!blocks.contains("github-mcp.sh"));
    }

    #[test]
    fn codex_plugin_blocks_are_always_enabled() {
        assert!(codex_plugin_blocks().contains("llm-dev-kit@llm-bootstrap"));
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

        super::prune_rtk_gemini_hooks(&mut settings);

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

        install_codex(&home, ApplyMode::Merge, &manifest, false).unwrap();
        let codex_home = home.join(".codex");
        assert!(codex_home.join("config.toml").exists());
        assert!(codex_home.join("AGENTS.md").exists());
        assert!(!codex_home.join("RTK.md").exists());

        uninstall_codex(&home, false).unwrap();
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

        install_gemini(&home, ApplyMode::Merge, &manifest, false).unwrap();
        let installed = fs::read_to_string(gemini_home.join("settings.json")).unwrap();
        assert!(installed.contains("/tmp/custom-run-shell.sh"));
        assert!(gemini_home.join("GEMINI.md").exists());
        assert!(!gemini_home.join("hooks/rtk-hook-gemini.sh").exists());

        uninstall_gemini(&home, &manifest, false).unwrap();
        let uninstalled = fs::read_to_string(gemini_home.join("settings.json")).unwrap();
        assert!(uninstalled.contains("/tmp/custom-run-shell.sh"));
        assert!(!gemini_home.join("GEMINI.md").exists());
        assert!(!gemini_home.join("extensions/llm-bootstrap-dev").exists());
        assert!(gemini_home.join("backups").exists());

        fs::remove_dir_all(home).unwrap();
    }
}
