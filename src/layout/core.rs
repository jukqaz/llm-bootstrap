pub(crate) const CODEX_BASE_PATHS: &[&str] = &[
    "config.toml",
    "stackpilot-state.json",
    "AGENTS.md",
    "agents",
    "scripts",
    ".agents/plugins/marketplace.json",
    "plugins/stackpilot-dev-kit",
    "plugins/cache/stackpilot/stackpilot-dev-kit",
    "SHIP_CHECKLIST.md",
    "RALPH_PLAN.md",
    "FOUNDER_LOOP.md",
    "OPERATING_REVIEW.md",
    "CONNECTORS.md",
    "AUTOMATIONS.md",
    "ENTRYPOINTS.md",
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
    "stackpilot-state.json",
    "RALPH_PLAN.md",
    "FOUNDER_LOOP.md",
    "OPERATING_REVIEW.md",
    "CONNECTORS.md",
    "AUTOMATIONS.md",
    "ENTRYPOINTS.md",
    "WORKFLOW.md",
    "SHIP_CHECKLIST.md",
    "TEAM.md",
    "REVIEW_AUTOMATION.md",
    "settings.json",
    "scripts",
    "extensions/stackpilot-dev",
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
