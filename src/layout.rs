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
    "WORKFLOW.md",
    "OFFICE_HOURS.md",
    "INVESTIGATE.md",
    "AUTOPILOT.md",
    "RETRO.md",
];

pub(crate) const GEMINI_BASE_PATHS: &[&str] = &[
    "GEMINI.md",
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

pub(crate) fn codex_uninstall_paths(rtk_enabled: bool) -> Vec<&'static str> {
    managed_paths(CODEX_BASE_PATHS, "RTK.md", rtk_enabled)
}

pub(crate) fn gemini_uninstall_paths(rtk_enabled: bool) -> Vec<&'static str> {
    managed_paths(GEMINI_BASE_PATHS, "hooks/rtk-hook-gemini.sh", rtk_enabled)
}
