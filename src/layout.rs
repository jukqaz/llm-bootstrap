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
    "settings.json",
    "scripts",
    "extensions/llm-bootstrap-dev",
    "extensions/extension-enablement.json",
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
    "delivery-commands",
    "incident-commands",
];
const COMPANY_SURFACES: &[&str] = &["company-skills", "company-commands"];
const CODEX_BUNDLE_DOC_PATHS: &[&str] = &[
    "RALPH_PLAN.md",
    "FOUNDER_LOOP.md",
    "OPERATING_REVIEW.md",
    "CONNECTORS.md",
    "AUTOMATIONS.md",
    "WORKFLOW.md",
    "SHIP_CHECKLIST.md",
    "OFFICE_HOURS.md",
    "INVESTIGATE.md",
    "AUTOPILOT.md",
    "RETRO.md",
];
const CODEX_PLUGIN_ASSET_PATHS: &[&str] = &[
    ".codex-plugin/plugin.json",
    "skills/autopilot/SKILL.md",
    "skills/delivery-loop/SKILL.md",
    "skills/ralph-plan/SKILL.md",
    "skills/founder-loop/SKILL.md",
    "skills/operating-review/SKILL.md",
    "skills/investigate/SKILL.md",
    "skills/repo-radar/SKILL.md",
    "skills/ship-check/SKILL.md",
];
const CODEX_BUNDLE_PLUGIN_ASSET_PATHS: &[&str] = &["skills/qa-browser/SKILL.md"];
const GEMINI_EXTENSION_ASSET_PATHS: &[&str] = &[
    "extensions/llm-bootstrap-dev/gemini-extension.json",
    "extensions/llm-bootstrap-dev/OFFICE_HOURS.md",
    "extensions/llm-bootstrap-dev/AUTOPILOT.md",
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
    "extensions/llm-bootstrap-dev/commands/operating-review.toml",
    "extensions/llm-bootstrap-dev/commands/ralph-plan.toml",
    "extensions/llm-bootstrap-dev/commands/review.toml",
    "extensions/llm-bootstrap-dev/commands/ship.toml",
];
const GEMINI_EXTENSION_ENABLEMENT_PATH: &str = "extensions/extension-enablement.json";
const CLAUDE_HARNESS_DOC_PATHS: &[&str] = &[
    "RALPH_PLAN.md",
    "FOUNDER_LOOP.md",
    "OPERATING_REVIEW.md",
    "CONNECTORS.md",
    "AUTOMATIONS.md",
    "WORKFLOW.md",
    "SHIP_CHECKLIST.md",
    "OFFICE_HOURS.md",
    "INVESTIGATE.md",
    "AUTOPILOT.md",
    "RETRO.md",
    "REVIEW.md",
    "QA.md",
    "SHIP.md",
];
const CLAUDE_SKILL_PATHS: &[&str] = &[
    "skills/autopilot",
    "skills/ralph-plan",
    "skills/founder-loop",
    "skills/investigate",
    "skills/operating-review",
    "skills/review",
    "skills/qa",
    "skills/ship",
    "skills/retro",
    "skills/office-hours",
];

pub(crate) fn provider_surface_enabled(active_surfaces: &[String]) -> bool {
    has_any_surface(active_surfaces, DEV_SURFACES)
        || has_any_surface(active_surfaces, COMPANY_SURFACES)
}

pub(crate) fn codex_bundle_doc_paths(active_surfaces: &[String]) -> Vec<&'static str> {
    let mut paths = Vec::new();
    if has_any_surface(active_surfaces, &["delivery-skills", "company-skills"]) {
        paths.push("RALPH_PLAN.md");
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
    if has_any_surface(active_surfaces, &["delivery-skills"]) {
        paths.push("skills/delivery-loop/SKILL.md");
        paths.push("skills/autopilot/SKILL.md");
    }
    if has_any_surface(active_surfaces, &["incident-skills"]) {
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
    paths
}

pub(crate) fn all_gemini_bundle_doc_paths() -> &'static [&'static str] {
    const ALL: &[&str] = &[
        "WORKFLOW.md",
        "SHIP_CHECKLIST.md",
        "RALPH_PLAN.md",
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
    if has_any_surface(active_surfaces, &["delivery-commands"]) {
        paths.push("extensions/llm-bootstrap-dev/OFFICE_HOURS.md");
        paths.push("extensions/llm-bootstrap-dev/AUTOPILOT.md");
        paths.push("extensions/llm-bootstrap-dev/commands/autopilot.toml");
    }
    if has_any_surface(active_surfaces, &["incident-commands"]) {
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
        paths.push("extensions/llm-bootstrap-dev/RETRO.md");
        paths.push("extensions/llm-bootstrap-dev/agents/qa.md");
        paths.push("extensions/llm-bootstrap-dev/agents/reviewer.md");
        paths.push("extensions/llm-bootstrap-dev/agents/verifier.md");
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
    if has_any_surface(active_surfaces, &["delivery-skills"]) {
        paths.push("skills/autopilot");
        paths.push("skills/office-hours");
        paths.push("skills/review");
        paths.push("skills/qa");
        paths.push("skills/ship");
        paths.push("skills/retro");
    }
    if has_any_surface(active_surfaces, &["incident-skills"]) {
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
