use std::path::PathBuf;

use crate::runtime::repo_root;

pub(crate) fn stackpilot_dev_kit_codex_repo_root() -> PathBuf {
    repo_root().join("addons/stackpilot-dev-kit/codex")
}

pub(crate) fn stackpilot_dev_kit_codex_bundle_root() -> PathBuf {
    repo_root().join("addons/stackpilot-dev-kit/bundles/codex")
}

pub(crate) fn stackpilot_dev_kit_gemini_repo_root() -> PathBuf {
    repo_root().join("addons/stackpilot-dev-kit/gemini")
}

#[cfg(test)]
pub(crate) fn stackpilot_dev_kit_gemini_bundle_root() -> PathBuf {
    repo_root().join("addons/stackpilot-dev-kit/bundles/gemini")
}

pub(crate) fn stackpilot_dev_kit_claude_repo_root() -> PathBuf {
    repo_root().join("addons/stackpilot-dev-kit/claude")
}
