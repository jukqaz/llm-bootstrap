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
    "ENTRYPOINTS.md",
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
    "skills/deep-init/SKILL.md",
    "skills/delivery-loop/SKILL.md",
    "skills/ralph-plan/SKILL.md",
    "skills/record-work/SKILL.md",
    "skills/founder-loop/SKILL.md",
    "skills/operating-review/SKILL.md",
    "skills/investigate/SKILL.md",
    "skills/office-hours/SKILL.md",
    "skills/workflow-gate/SKILL.md",
    "skills/team/SKILL.md",
    "skills/ultrawork/SKILL.md",
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
    "extensions/stackpilot-dev/gemini-extension.json",
    "extensions/stackpilot-dev/OFFICE_HOURS.md",
    "extensions/stackpilot-dev/AUTOPILOT.md",
    "extensions/stackpilot-dev/TEAM.md",
    "extensions/stackpilot-dev/REVIEW_AUTOMATION.md",
    "extensions/stackpilot-dev/REVIEW.md",
    "extensions/stackpilot-dev/QA.md",
    "extensions/stackpilot-dev/SHIP.md",
    "extensions/stackpilot-dev/RETRO.md",
    "extensions/stackpilot-dev/agents/docs-researcher.md",
    "extensions/stackpilot-dev/agents/executor.md",
    "extensions/stackpilot-dev/agents/planner.md",
    "extensions/stackpilot-dev/agents/qa.md",
    "extensions/stackpilot-dev/agents/reviewer.md",
    "extensions/stackpilot-dev/agents/triage.md",
    "extensions/stackpilot-dev/agents/verifier.md",
    "extensions/stackpilot-dev/commands/autopilot.toml",
    "extensions/stackpilot-dev/commands/deep-init.toml",
    "extensions/stackpilot-dev/commands/doctor.toml",
    "extensions/stackpilot-dev/commands/founder-review.toml",
    "extensions/stackpilot-dev/commands/intent.toml",
    "extensions/stackpilot-dev/commands/office-hours.toml",
    "extensions/stackpilot-dev/commands/gate.toml",
    "extensions/stackpilot-dev/commands/team.toml",
    "extensions/stackpilot-dev/commands/ultrawork.toml",
    "extensions/stackpilot-dev/commands/review-automation.toml",
    "extensions/stackpilot-dev/commands/operating-review.toml",
    "extensions/stackpilot-dev/commands/qa.toml",
    "extensions/stackpilot-dev/commands/ralph-plan.toml",
    "extensions/stackpilot-dev/commands/record-work.toml",
    "extensions/stackpilot-dev/commands/retro.toml",
    "extensions/stackpilot-dev/commands/review.toml",
    "extensions/stackpilot-dev/commands/ship.toml",
];
const GEMINI_EXTENSION_ENABLEMENT_PATH: &str = "extensions/extension-enablement.json";
const CLAUDE_HARNESS_DOC_PATHS: &[&str] = &[
    "RALPH_PLAN.md",
    "OPERATING_RECORDS.md",
    "FOUNDER_LOOP.md",
    "OPERATING_REVIEW.md",
    "CONNECTORS.md",
    "AUTOMATIONS.md",
    "ENTRYPOINTS.md",
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
    "skills/deep-init",
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
    "skills/ultrawork",
    "skills/review-automation",
];

pub(crate) fn provider_surface_enabled(active_surfaces: &[String]) -> bool {
    has_any_surface(active_surfaces, DEV_SURFACES)
        || has_any_surface(active_surfaces, COMPANY_SURFACES)
}

pub(crate) fn codex_managed_paths_for(
    active_surfaces: &[String],
    plugin_enabled: bool,
    rtk_enabled: bool,
) -> Vec<String> {
    let mut paths = vec![
        "config.toml".to_string(),
        "stackpilot-state.json".to_string(),
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
        paths.push("plugins/stackpilot-dev-kit".to_string());
        paths.push("plugins/cache/stackpilot/stackpilot-dev-kit".to_string());
    }
    if rtk_enabled {
        paths.push("RTK.md".to_string());
    }
    dedupe_paths(paths)
}

pub(crate) fn gemini_managed_paths_for(
    active_surfaces: &[String],
    extension_enabled: bool,
    rtk_enabled: bool,
) -> Vec<String> {
    let mut paths = vec![
        "GEMINI.md".to_string(),
        "stackpilot-state.json".to_string(),
        "settings.json".to_string(),
        "scripts".to_string(),
    ];
    paths.extend(
        gemini_bundle_doc_paths(active_surfaces)
            .into_iter()
            .map(ToOwned::to_owned),
    );
    if extension_enabled {
        paths.push("extensions/stackpilot-dev".to_string());
        paths.push("extensions/extension-enablement.json".to_string());
    }
    if rtk_enabled {
        paths.push("hooks/rtk-hook-gemini.sh".to_string());
    }
    dedupe_paths(paths)
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
        "stackpilot-state.json".to_string(),
        "settings.json".to_string(),
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
        paths.push("RTK.md".to_string());
        paths.push("hooks/rtk-rewrite.sh".to_string());
    }
    dedupe_paths(paths)
}

pub(crate) fn codex_bundle_doc_paths(active_surfaces: &[String]) -> Vec<&'static str> {
    let mut paths = Vec::new();
    if has_any_surface(active_surfaces, &["delivery-skills", "company-skills"]) {
        paths.push("RALPH_PLAN.md");
        paths.push("OPERATING_RECORDS.md");
    }
    if provider_surface_enabled(active_surfaces) {
        paths.push("ENTRYPOINTS.md");
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
    dedupe_paths(paths)
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
        paths.push("skills/deep-init/SKILL.md");
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
        paths.push("skills/ultrawork/SKILL.md");
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

    dedupe_paths(paths)
}

pub(crate) fn all_codex_plugin_asset_paths() -> &'static [&'static str] {
    CODEX_PLUGIN_ASSET_PATHS
}

pub(crate) fn codex_bundle_plugin_asset_paths(active_surfaces: &[String]) -> Vec<&'static str> {
    let mut paths = Vec::new();
    if has_any_surface(active_surfaces, &["delivery-skills"]) {
        paths.push("skills/qa-browser/SKILL.md");
    }
    dedupe_paths(paths)
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
    if provider_surface_enabled(active_surfaces) {
        paths.push("ENTRYPOINTS.md");
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
    dedupe_paths(paths)
}

pub(crate) fn all_gemini_bundle_doc_paths() -> &'static [&'static str] {
    const ALL: &[&str] = &[
        "ENTRYPOINTS.md",
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
        "extensions/stackpilot-dev/gemini-extension.json",
        "extensions/stackpilot-dev/commands/doctor.toml",
    ];

    if has_any_surface(active_surfaces, &["delivery-commands", "company-commands"]) {
        paths.push("extensions/stackpilot-dev/agents/planner.md");
        paths.push("extensions/stackpilot-dev/agents/executor.md");
        paths.push("extensions/stackpilot-dev/commands/intent.toml");
    }
    if has_any_surface(active_surfaces, &["company-commands"]) {
        paths.push("extensions/stackpilot-dev/commands/ralph-plan.toml");
    }
    if has_any_surface(
        active_surfaces,
        &["delivery-commands", "incident-commands", "company-commands"],
    ) {
        paths.push("extensions/stackpilot-dev/commands/record-work.toml");
    }
    if has_any_surface(active_surfaces, &["delivery-commands"]) {
        paths.push("extensions/stackpilot-dev/OFFICE_HOURS.md");
        paths.push("extensions/stackpilot-dev/AUTOPILOT.md");
        paths.push("extensions/stackpilot-dev/commands/autopilot.toml");
        paths.push("extensions/stackpilot-dev/commands/deep-init.toml");
        paths.push("extensions/stackpilot-dev/commands/gate.toml");
        paths.push("extensions/stackpilot-dev/commands/office-hours.toml");
    }
    if has_any_surface(active_surfaces, &["team-commands"]) {
        paths.push("extensions/stackpilot-dev/TEAM.md");
        paths.push("extensions/stackpilot-dev/commands/gate.toml");
        paths.push("extensions/stackpilot-dev/commands/team.toml");
        paths.push("extensions/stackpilot-dev/commands/ultrawork.toml");
    }
    if has_any_surface(active_surfaces, &["review-automation-commands"]) {
        paths.push("extensions/stackpilot-dev/REVIEW_AUTOMATION.md");
        paths.push("extensions/stackpilot-dev/commands/review-automation.toml");
    }
    if has_any_surface(active_surfaces, &["incident-commands"]) {
        paths.push("extensions/stackpilot-dev/commands/gate.toml");
        paths.push("extensions/stackpilot-dev/agents/triage.md");
    }
    if has_any_surface(active_surfaces, &["delivery-commands", "incident-commands"]) {
        paths.push("extensions/stackpilot-dev/agents/docs-researcher.md");
    }
    if has_any_surface(active_surfaces, &["company-commands"]) {
        paths.push("extensions/stackpilot-dev/commands/founder-review.toml");
        paths.push("extensions/stackpilot-dev/commands/operating-review.toml");
    }
    if has_any_surface(active_surfaces, &["delivery-commands"]) {
        paths.push("extensions/stackpilot-dev/REVIEW.md");
        paths.push("extensions/stackpilot-dev/QA.md");
        paths.push("extensions/stackpilot-dev/SHIP.md");
        paths.push("extensions/stackpilot-dev/RETRO.md");
        paths.push("extensions/stackpilot-dev/agents/qa.md");
        paths.push("extensions/stackpilot-dev/agents/reviewer.md");
        paths.push("extensions/stackpilot-dev/agents/verifier.md");
        paths.push("extensions/stackpilot-dev/commands/qa.toml");
        paths.push("extensions/stackpilot-dev/commands/retro.toml");
        paths.push("extensions/stackpilot-dev/commands/review.toml");
        paths.push("extensions/stackpilot-dev/commands/ship.toml");
    }

    dedupe_paths(paths)
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
    if provider_surface_enabled(active_surfaces) {
        paths.push("ENTRYPOINTS.md");
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
    dedupe_paths(paths)
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
        paths.push("skills/deep-init");
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
        paths.push("skills/ultrawork");
    }
    if has_any_surface(active_surfaces, &["review-automation-skills"]) {
        paths.push("skills/review-automation");
    }
    if has_any_surface(active_surfaces, &["incident-skills"]) {
        paths.push("skills/workflow-gate");
        paths.push("skills/investigate");
    }
    dedupe_paths(paths)
}

pub(crate) fn all_claude_skill_paths() -> &'static [&'static str] {
    CLAUDE_SKILL_PATHS
}

fn has_any_surface(active_surfaces: &[String], expected: &[&str]) -> bool {
    expected
        .iter()
        .any(|expected_name| active_surfaces.iter().any(|active| active == expected_name))
}

fn dedupe_paths<T: Eq + Clone>(paths: Vec<T>) -> Vec<T> {
    let mut unique = Vec::new();
    for path in paths {
        if !unique.contains(&path) {
            unique.push(path);
        }
    }
    unique
}
