pub(crate) const CODEX_BASE_PATHS: &[&str] = &[
    "config.toml",
    "llm-bootstrap-state.json",
    "AGENTS.md",
    "agents",
    "scripts",
    ".agents/plugins/marketplace.json",
    "plugins/llm-dev-kit",
    "plugins/cache/llm-bootstrap/llm-dev-kit",
    "SHIP_CHECKLIST.md",
    "RALPH_PLAN.md",
    "FOUNDER_LOOP.md",
    "OPERATING_REVIEW.md",
    "CONNECTORS.md",
    "AUTOMATIONS.md",
    "WORKFLOW.md",
    "OFFICE_HOURS.md",
    "INVESTIGATE.md",
    "AUTOPILOT.md",
    "TEAM.md",
    "REVIEW_AUTOMATION.md",
    "RETRO.md",
];

pub(crate) const GEMINI_BASE_PATHS: &[&str] = &[
    "GEMINI.md",
    "llm-bootstrap-state.json",
    "RALPH_PLAN.md",
    "FOUNDER_LOOP.md",
    "OPERATING_REVIEW.md",
    "CONNECTORS.md",
    "AUTOMATIONS.md",
    "WORKFLOW.md",
    "SHIP_CHECKLIST.md",
    "TEAM.md",
    "REVIEW_AUTOMATION.md",
    "settings.json",
    "scripts",
    "extensions/llm-bootstrap-dev",
    "extensions/extension-enablement.json",
];

pub(crate) const HOME_LEGACY_CLEANUP_PATHS: &[&str] = &[
    ".omx",
    ".omc",
    ".omg",
    ".config/omx",
    ".config/omc",
    ".config/omg",
    ".config/oh-my-codex",
    ".config/oh-my-gemini",
    ".config/oh-my-claudecode",
    ".local/bin/omx",
    ".local/bin/omc",
    ".local/bin/omg",
    ".local/bin/oh-my-codex",
    ".local/bin/oh-my-gemini",
    ".local/bin/oh-my-gemini-cli",
    ".local/bin/oh-my-claudecode",
    ".local/bin/oh-my-opencode",
    "bin/omx",
    "bin/omc",
    "bin/omg",
    "bin/oh-my-codex",
    "bin/oh-my-gemini",
    "bin/oh-my-gemini-cli",
    "bin/oh-my-claudecode",
    "bin/oh-my-opencode",
    ".cargo/bin/omx",
    ".cargo/bin/omc",
    ".cargo/bin/omg",
    ".cargo/bin/oh-my-codex",
    ".cargo/bin/oh-my-gemini",
    ".cargo/bin/oh-my-gemini-cli",
    ".cargo/bin/oh-my-claudecode",
    ".cargo/bin/oh-my-opencode",
];

pub(crate) const LEGACY_ENV_KEYS: &[&str] = &[
    "OMX",
    "OMX_API_KEY",
    "OMX_HOME",
    "OMX_CONFIG_HOME",
    "OMG",
    "OMG_API_KEY",
    "OMG_HOME",
    "OMG_CONFIG_HOME",
    "OMC",
    "OMC_API_KEY",
    "OMC_HOME",
    "OMC_CONFIG_HOME",
    "OH_MY_CODEX_API_KEY",
    "OH_MY_CODEX_HOME",
    "OH_MY_CODEX_PATH",
    "OH_MY_GEMINI_API_KEY",
    "OH_MY_GEMINI_HOME",
    "OH_MY_GEMINI_PATH",
    "OH_MY_GEMINI_CLI_API_KEY",
    "OH_MY_GEMINI_CLI_HOME",
    "OH_MY_GEMINI_CLI_PATH",
    "OH_MY_CLAUDECODE_API_KEY",
    "OH_MY_CLAUDECODE_HOME",
    "OH_MY_CLAUDECODE_PATH",
    "OH_MY_OPENCODE_API_KEY",
    "OH_MY_OPENCODE_HOME",
    "OH_MY_OPENCODE_PATH",
];

pub(crate) const CODEX_LEGACY_CLEANUP_PATHS: &[&str] = &["vendor_imports/skills", ".tmp/plugins"];

pub(crate) const GEMINI_LEGACY_CLEANUP_PATHS: &[&str] =
    &["extensions/oh-my-gemini-cli", "extensions/oh-my-gemini"];

pub(crate) const CLAUDE_LEGACY_CLEANUP_PATHS: &[&str] = &[
    "plugins/oh-my-claudecode",
    "plugins/cache/omc",
    "plugins/data/oh-my-claudecode-omc",
    "plugins/marketplaces/omc",
    ".omc",
    ".omc-config.json",
    "hud/omc-hud.mjs",
];

pub(crate) fn managed_paths(
    base: &[&'static str],
    rtk_path: &'static str,
    include_rtk: bool,
) -> Vec<&'static str> {
    let mut paths = base.to_vec();
    if include_rtk {
        paths.push(rtk_path);
    }
    paths
}

pub(crate) fn codex_managed_paths() -> Vec<&'static str> {
    managed_paths(CODEX_BASE_PATHS, "RTK.md", true)
}

pub(crate) fn gemini_managed_paths() -> Vec<&'static str> {
    managed_paths(GEMINI_BASE_PATHS, "hooks/rtk-hook-gemini.sh", true)
}

pub(crate) fn codex_managed_paths_for(
    active_surfaces: &[String],
    plugin_enabled: bool,
    rtk_enabled: bool,
) -> Vec<String> {
    let mut paths = vec![
        "config.toml".to_string(),
        "llm-bootstrap-state.json".to_string(),
        "AGENTS.md".to_string(),
        "agents".to_string(),
        "scripts".to_string(),
    ];
    paths.extend(
        codex_bundle_doc_paths(active_surfaces)
            .into_iter()
            .map(ToOwned::to_owned),
    );
    if plugin_enabled {
        paths.push(".agents/plugins/marketplace.json".to_string());
        paths.push("plugins/llm-dev-kit".to_string());
        paths.push("plugins/cache/llm-bootstrap/llm-dev-kit".to_string());
    }
    if rtk_enabled {
        paths.push("RTK.md".to_string());
    }
    paths
}

pub(crate) fn gemini_managed_paths_for(
    active_surfaces: &[String],
    extension_enabled: bool,
    rtk_enabled: bool,
) -> Vec<String> {
    let mut paths = vec![
        "GEMINI.md".to_string(),
        "llm-bootstrap-state.json".to_string(),
        "settings.json".to_string(),
        "scripts".to_string(),
    ];
    paths.extend(
        gemini_bundle_doc_paths(active_surfaces)
            .into_iter()
            .map(ToOwned::to_owned),
    );
    if extension_enabled {
        paths.push("extensions/llm-bootstrap-dev".to_string());
        paths.push("extensions/extension-enablement.json".to_string());
    }
    if rtk_enabled {
        paths.push("hooks/rtk-hook-gemini.sh".to_string());
    }
    paths
}

pub(crate) fn claude_managed_paths_for(
    active_surfaces: &[String],
    skills_enabled: bool,
    rtk_enabled: bool,
) -> Vec<String> {
    let mut paths = vec![
        "CLAUDE.md".to_string(),
        "agents".to_string(),
        "scripts".to_string(),
        "llm-bootstrap-state.json".to_string(),
    ];
    paths.extend(
        claude_harness_doc_paths(active_surfaces)
            .into_iter()
            .map(ToOwned::to_owned),
    );
    if skills_enabled {
        paths.extend(
            claude_skill_paths(active_surfaces)
                .into_iter()
                .map(ToOwned::to_owned),
        );
    }
    if rtk_enabled {
        paths.push("settings.json".to_string());
        paths.push("RTK.md".to_string());
        paths.push("hooks/rtk-rewrite.sh".to_string());
    }
    paths
}

const DEV_SURFACES: &[&str] = &[
    "delivery-skills",
    "incident-skills",
    "team-skills",
    "review-automation-skills",
    "delivery-commands",
    "incident-commands",
    "team-commands",
    "review-automation-commands",
];
const COMPANY_SURFACES: &[&str] = &["company-skills", "company-commands"];
const CODEX_BUNDLE_DOC_PATHS: &[&str] = &[
    "RALPH_PLAN.md",
    "OPERATING_RECORDS.md",
    "FOUNDER_LOOP.md",
    "OPERATING_REVIEW.md",
    "CONNECTORS.md",
    "AUTOMATIONS.md",
    "WORKFLOW.md",
    "SHIP_CHECKLIST.md",
    "OFFICE_HOURS.md",
    "INVESTIGATE.md",
    "AUTOPILOT.md",
    "TEAM.md",
    "REVIEW_AUTOMATION.md",
    "REVIEW.md",
    "QA.md",
    "SHIP.md",
    "RETRO.md",
];
const CODEX_PLUGIN_ASSET_PATHS: &[&str] = &[
    ".codex-plugin/plugin.json",
    "skills/autopilot/SKILL.md",
    "skills/delivery-loop/SKILL.md",
    "skills/ralph-plan/SKILL.md",
    "skills/record-work/SKILL.md",
    "skills/founder-loop/SKILL.md",
    "skills/operating-review/SKILL.md",
    "skills/investigate/SKILL.md",
    "skills/office-hours/SKILL.md",
    "skills/workflow-gate/SKILL.md",
    "skills/team/SKILL.md",
    "skills/review-automation/SKILL.md",
    "skills/review/SKILL.md",
    "skills/qa/SKILL.md",
    "skills/ship/SKILL.md",
    "skills/retro/SKILL.md",
    "skills/repo-radar/SKILL.md",
    "skills/ship-check/SKILL.md",
];
const CODEX_BUNDLE_PLUGIN_ASSET_PATHS: &[&str] = &["skills/qa-browser/SKILL.md"];
const GEMINI_EXTENSION_ASSET_PATHS: &[&str] = &[
    "extensions/llm-bootstrap-dev/gemini-extension.json",
    "extensions/llm-bootstrap-dev/OFFICE_HOURS.md",
    "extensions/llm-bootstrap-dev/AUTOPILOT.md",
    "extensions/llm-bootstrap-dev/TEAM.md",
    "extensions/llm-bootstrap-dev/REVIEW_AUTOMATION.md",
    "extensions/llm-bootstrap-dev/REVIEW.md",
    "extensions/llm-bootstrap-dev/QA.md",
    "extensions/llm-bootstrap-dev/SHIP.md",
    "extensions/llm-bootstrap-dev/RETRO.md",
    "extensions/llm-bootstrap-dev/agents/docs-researcher.md",
    "extensions/llm-bootstrap-dev/agents/executor.md",
    "extensions/llm-bootstrap-dev/agents/planner.md",
    "extensions/llm-bootstrap-dev/agents/qa.md",
    "extensions/llm-bootstrap-dev/agents/reviewer.md",
    "extensions/llm-bootstrap-dev/agents/triage.md",
    "extensions/llm-bootstrap-dev/agents/verifier.md",
    "extensions/llm-bootstrap-dev/commands/autopilot.toml",
    "extensions/llm-bootstrap-dev/commands/doctor.toml",
    "extensions/llm-bootstrap-dev/commands/founder-review.toml",
    "extensions/llm-bootstrap-dev/commands/intent.toml",
    "extensions/llm-bootstrap-dev/commands/office-hours.toml",
    "extensions/llm-bootstrap-dev/commands/gate.toml",
    "extensions/llm-bootstrap-dev/commands/team.toml",
    "extensions/llm-bootstrap-dev/commands/review-automation.toml",
    "extensions/llm-bootstrap-dev/commands/operating-review.toml",
    "extensions/llm-bootstrap-dev/commands/qa.toml",
    "extensions/llm-bootstrap-dev/commands/ralph-plan.toml",
    "extensions/llm-bootstrap-dev/commands/record-work.toml",
    "extensions/llm-bootstrap-dev/commands/retro.toml",
    "extensions/llm-bootstrap-dev/commands/review.toml",
    "extensions/llm-bootstrap-dev/commands/ship.toml",
];
const GEMINI_EXTENSION_ENABLEMENT_PATH: &str = "extensions/extension-enablement.json";
const CLAUDE_HARNESS_DOC_PATHS: &[&str] = &[
    "RALPH_PLAN.md",
    "OPERATING_RECORDS.md",
    "FOUNDER_LOOP.md",
    "OPERATING_REVIEW.md",
    "CONNECTORS.md",
    "AUTOMATIONS.md",
    "WORKFLOW.md",
    "SHIP_CHECKLIST.md",
    "OFFICE_HOURS.md",
    "INVESTIGATE.md",
    "AUTOPILOT.md",
    "TEAM.md",
    "REVIEW_AUTOMATION.md",
    "RETRO.md",
    "REVIEW.md",
    "QA.md",
    "SHIP.md",
];
const CLAUDE_SKILL_PATHS: &[&str] = &[
    "skills/autopilot",
    "skills/ralph-plan",
    "skills/record-work",
    "skills/founder-loop",
    "skills/investigate",
    "skills/operating-review",
    "skills/review",
    "skills/qa",
    "skills/ship",
    "skills/retro",
    "skills/office-hours",
    "skills/workflow-gate",
    "skills/team",
    "skills/review-automation",
];

pub(crate) fn provider_surface_enabled(active_surfaces: &[String]) -> bool {
    has_any_surface(active_surfaces, DEV_SURFACES)
        || has_any_surface(active_surfaces, COMPANY_SURFACES)
}

pub(crate) fn codex_bundle_doc_paths(active_surfaces: &[String]) -> Vec<&'static str> {
    let mut paths = Vec::new();
    if has_any_surface(active_surfaces, &["delivery-skills", "company-skills"]) {
        paths.push("RALPH_PLAN.md");
        paths.push("OPERATING_RECORDS.md");
    }
    if has_any_surface(active_surfaces, &["delivery-skills"]) {
        paths.push("WORKFLOW.md");
    }
    if has_any_surface(active_surfaces, &["delivery-skills", "incident-skills"]) {
        paths.push("SHIP_CHECKLIST.md");
    }
    if has_any_surface(active_surfaces, &["company-skills"]) {
        paths.push("FOUNDER_LOOP.md");
        paths.push("OPERATING_REVIEW.md");
        paths.push("CONNECTORS.md");
        paths.push("AUTOMATIONS.md");
    }
    if has_any_surface(active_surfaces, &["delivery-skills"]) {
        paths.push("OFFICE_HOURS.md");
        paths.push("AUTOPILOT.md");
        paths.push("REVIEW.md");
        paths.push("QA.md");
        paths.push("SHIP.md");
    }
    if has_any_surface(active_surfaces, &["team-skills"]) {
        paths.push("TEAM.md");
    }
    if has_any_surface(active_surfaces, &["review-automation-skills"]) {
        paths.push("REVIEW_AUTOMATION.md");
    }
    if has_any_surface(active_surfaces, &["incident-skills"]) {
        paths.push("INVESTIGATE.md");
    }
    if has_any_surface(active_surfaces, &["delivery-skills"]) {
        paths.push("RETRO.md");
    }
    paths
}

pub(crate) fn all_codex_bundle_doc_paths() -> &'static [&'static str] {
    CODEX_BUNDLE_DOC_PATHS
}

pub(crate) fn codex_plugin_asset_paths(active_surfaces: &[String]) -> Vec<&'static str> {
    let mut paths = vec![".codex-plugin/plugin.json"];

    if has_any_surface(
        active_surfaces,
        &["delivery-skills", "incident-skills", "company-skills"],
    ) {
        paths.push("skills/repo-radar/SKILL.md");
    }
    if has_any_surface(active_surfaces, &["company-skills"]) {
        paths.push("skills/ralph-plan/SKILL.md");
    }
    if has_any_surface(
        active_surfaces,
        &["delivery-skills", "incident-skills", "company-skills"],
    ) {
        paths.push("skills/record-work/SKILL.md");
    }
    if has_any_surface(active_surfaces, &["delivery-skills"]) {
        paths.push("skills/delivery-loop/SKILL.md");
        paths.push("skills/autopilot/SKILL.md");
        paths.push("skills/office-hours/SKILL.md");
        paths.push("skills/workflow-gate/SKILL.md");
        paths.push("skills/review/SKILL.md");
        paths.push("skills/qa/SKILL.md");
        paths.push("skills/ship/SKILL.md");
        paths.push("skills/retro/SKILL.md");
    }
    if has_any_surface(active_surfaces, &["team-skills"]) {
        paths.push("skills/workflow-gate/SKILL.md");
        paths.push("skills/team/SKILL.md");
    }
    if has_any_surface(active_surfaces, &["review-automation-skills"]) {
        paths.push("skills/review-automation/SKILL.md");
    }
    if has_any_surface(active_surfaces, &["incident-skills"]) {
        paths.push("skills/workflow-gate/SKILL.md");
        paths.push("skills/investigate/SKILL.md");
    }
    if has_any_surface(active_surfaces, &["company-skills"]) {
        paths.push("skills/founder-loop/SKILL.md");
        paths.push("skills/operating-review/SKILL.md");
    }
    if has_any_surface(active_surfaces, &["delivery-skills"]) {
        paths.push("skills/ship-check/SKILL.md");
    }

    paths
}

pub(crate) fn all_codex_plugin_asset_paths() -> &'static [&'static str] {
    CODEX_PLUGIN_ASSET_PATHS
}

pub(crate) fn codex_bundle_plugin_asset_paths(active_surfaces: &[String]) -> Vec<&'static str> {
    let mut paths = Vec::new();
    if has_any_surface(active_surfaces, &["delivery-skills"]) {
        paths.push("skills/qa-browser/SKILL.md");
    }
    paths
}

pub(crate) fn all_codex_bundle_plugin_asset_paths() -> &'static [&'static str] {
    CODEX_BUNDLE_PLUGIN_ASSET_PATHS
}

pub(crate) fn gemini_bundle_doc_paths(active_surfaces: &[String]) -> Vec<&'static str> {
    let mut paths = Vec::new();
    if has_any_surface(active_surfaces, &["delivery-commands", "company-commands"]) {
        paths.push("RALPH_PLAN.md");
        paths.push("OPERATING_RECORDS.md");
    }
    if has_any_surface(active_surfaces, &["delivery-commands"]) {
        paths.push("WORKFLOW.md");
    }
    if has_any_surface(active_surfaces, &["delivery-commands", "incident-commands"]) {
        paths.push("SHIP_CHECKLIST.md");
    }
    if has_any_surface(active_surfaces, &["company-commands"]) {
        paths.push("FOUNDER_LOOP.md");
        paths.push("OPERATING_REVIEW.md");
        paths.push("CONNECTORS.md");
        paths.push("AUTOMATIONS.md");
    }
    if has_any_surface(active_surfaces, &["team-commands"]) {
        paths.push("TEAM.md");
    }
    if has_any_surface(active_surfaces, &["review-automation-commands"]) {
        paths.push("REVIEW_AUTOMATION.md");
    }
    paths
}

pub(crate) fn all_gemini_bundle_doc_paths() -> &'static [&'static str] {
    const ALL: &[&str] = &[
        "WORKFLOW.md",
        "SHIP_CHECKLIST.md",
        "TEAM.md",
        "REVIEW_AUTOMATION.md",
        "RALPH_PLAN.md",
        "OPERATING_RECORDS.md",
        "FOUNDER_LOOP.md",
        "OPERATING_REVIEW.md",
        "CONNECTORS.md",
        "AUTOMATIONS.md",
    ];
    ALL
}

pub(crate) fn gemini_extension_asset_paths(active_surfaces: &[String]) -> Vec<&'static str> {
    let mut paths = vec![
        "extensions/llm-bootstrap-dev/gemini-extension.json",
        "extensions/llm-bootstrap-dev/commands/doctor.toml",
    ];

    if has_any_surface(active_surfaces, &["delivery-commands", "company-commands"]) {
        paths.push("extensions/llm-bootstrap-dev/agents/planner.md");
        paths.push("extensions/llm-bootstrap-dev/agents/executor.md");
        paths.push("extensions/llm-bootstrap-dev/commands/intent.toml");
    }
    if has_any_surface(active_surfaces, &["company-commands"]) {
        paths.push("extensions/llm-bootstrap-dev/commands/ralph-plan.toml");
    }
    if has_any_surface(
        active_surfaces,
        &["delivery-commands", "incident-commands", "company-commands"],
    ) {
        paths.push("extensions/llm-bootstrap-dev/commands/record-work.toml");
    }
    if has_any_surface(active_surfaces, &["delivery-commands"]) {
        paths.push("extensions/llm-bootstrap-dev/OFFICE_HOURS.md");
        paths.push("extensions/llm-bootstrap-dev/AUTOPILOT.md");
        paths.push("extensions/llm-bootstrap-dev/commands/autopilot.toml");
        paths.push("extensions/llm-bootstrap-dev/commands/gate.toml");
        paths.push("extensions/llm-bootstrap-dev/commands/office-hours.toml");
    }
    if has_any_surface(active_surfaces, &["team-commands"]) {
        paths.push("extensions/llm-bootstrap-dev/TEAM.md");
        paths.push("extensions/llm-bootstrap-dev/commands/gate.toml");
        paths.push("extensions/llm-bootstrap-dev/commands/team.toml");
    }
    if has_any_surface(active_surfaces, &["review-automation-commands"]) {
        paths.push("extensions/llm-bootstrap-dev/REVIEW_AUTOMATION.md");
        paths.push("extensions/llm-bootstrap-dev/commands/review-automation.toml");
    }
    if has_any_surface(active_surfaces, &["incident-commands"]) {
        paths.push("extensions/llm-bootstrap-dev/commands/gate.toml");
        paths.push("extensions/llm-bootstrap-dev/agents/triage.md");
    }
    if has_any_surface(active_surfaces, &["delivery-commands", "incident-commands"]) {
        paths.push("extensions/llm-bootstrap-dev/agents/docs-researcher.md");
    }
    if has_any_surface(active_surfaces, &["company-commands"]) {
        paths.push("extensions/llm-bootstrap-dev/commands/founder-review.toml");
        paths.push("extensions/llm-bootstrap-dev/commands/operating-review.toml");
    }
    if has_any_surface(active_surfaces, &["delivery-commands"]) {
        paths.push("extensions/llm-bootstrap-dev/REVIEW.md");
        paths.push("extensions/llm-bootstrap-dev/QA.md");
        paths.push("extensions/llm-bootstrap-dev/SHIP.md");
        paths.push("extensions/llm-bootstrap-dev/RETRO.md");
        paths.push("extensions/llm-bootstrap-dev/agents/qa.md");
        paths.push("extensions/llm-bootstrap-dev/agents/reviewer.md");
        paths.push("extensions/llm-bootstrap-dev/agents/verifier.md");
        paths.push("extensions/llm-bootstrap-dev/commands/qa.toml");
        paths.push("extensions/llm-bootstrap-dev/commands/retro.toml");
        paths.push("extensions/llm-bootstrap-dev/commands/review.toml");
        paths.push("extensions/llm-bootstrap-dev/commands/ship.toml");
    }

    paths
}

pub(crate) fn all_gemini_extension_asset_paths() -> &'static [&'static str] {
    GEMINI_EXTENSION_ASSET_PATHS
}

pub(crate) fn gemini_extension_enablement_path() -> &'static str {
    GEMINI_EXTENSION_ENABLEMENT_PATH
}

pub(crate) fn claude_harness_doc_paths(active_surfaces: &[String]) -> Vec<&'static str> {
    let mut paths = Vec::new();
    if has_any_surface(active_surfaces, &["delivery-skills", "company-skills"]) {
        paths.push("RALPH_PLAN.md");
        paths.push("OPERATING_RECORDS.md");
    }
    if has_any_surface(active_surfaces, &["delivery-skills"]) {
        paths.push("WORKFLOW.md");
    }
    if has_any_surface(active_surfaces, &["delivery-skills", "incident-skills"]) {
        paths.push("SHIP_CHECKLIST.md");
    }
    if has_any_surface(active_surfaces, &["delivery-skills"]) {
        paths.push("OFFICE_HOURS.md");
        paths.push("AUTOPILOT.md");
    }
    if has_any_surface(active_surfaces, &["team-skills"]) {
        paths.push("TEAM.md");
    }
    if has_any_surface(active_surfaces, &["review-automation-skills"]) {
        paths.push("REVIEW_AUTOMATION.md");
    }
    if has_any_surface(active_surfaces, &["incident-skills"]) {
        paths.push("INVESTIGATE.md");
    }
    if has_any_surface(active_surfaces, &["company-skills"]) {
        paths.push("FOUNDER_LOOP.md");
        paths.push("OPERATING_REVIEW.md");
        paths.push("CONNECTORS.md");
        paths.push("AUTOMATIONS.md");
    }
    if has_any_surface(active_surfaces, &["delivery-skills"]) {
        paths.push("RETRO.md");
        paths.push("REVIEW.md");
        paths.push("QA.md");
        paths.push("SHIP.md");
    }
    paths
}

pub(crate) fn all_claude_harness_doc_paths() -> &'static [&'static str] {
    CLAUDE_HARNESS_DOC_PATHS
}

pub(crate) fn claude_skill_paths(active_surfaces: &[String]) -> Vec<&'static str> {
    let mut paths = Vec::new();
    if has_any_surface(active_surfaces, &["company-skills"]) {
        paths.push("skills/ralph-plan");
        paths.push("skills/founder-loop");
        paths.push("skills/operating-review");
    }
    if has_any_surface(
        active_surfaces,
        &["delivery-skills", "incident-skills", "company-skills"],
    ) {
        paths.push("skills/record-work");
    }
    if has_any_surface(active_surfaces, &["delivery-skills"]) {
        paths.push("skills/autopilot");
        paths.push("skills/office-hours");
        paths.push("skills/workflow-gate");
        paths.push("skills/review");
        paths.push("skills/qa");
        paths.push("skills/ship");
        paths.push("skills/retro");
    }
    if has_any_surface(active_surfaces, &["team-skills"]) {
        paths.push("skills/workflow-gate");
        paths.push("skills/team");
    }
    if has_any_surface(active_surfaces, &["review-automation-skills"]) {
        paths.push("skills/review-automation");
    }
    if has_any_surface(active_surfaces, &["incident-skills"]) {
        paths.push("skills/workflow-gate");
        paths.push("skills/investigate");
    }
    paths
}

fn has_any_surface(active_surfaces: &[String], expected: &[&str]) -> bool {
    expected
        .iter()
        .any(|expected_name| active_surfaces.iter().any(|active| active == expected_name))
}

pub(crate) fn all_claude_skill_paths() -> &'static [&'static str] {
    CLAUDE_SKILL_PATHS
}
