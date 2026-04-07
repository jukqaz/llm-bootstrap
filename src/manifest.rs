use crate::cli::{ApplyMode, Provider};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub(crate) struct BootstrapManifest {
    pub(crate) bootstrap: BootstrapSection,
    pub(crate) external: ExternalSection,
    pub(crate) mcp: McpSection,
}

#[derive(Debug, Deserialize)]
pub(crate) struct BootstrapSection {
    pub(crate) providers: Vec<Provider>,
    pub(crate) default_mode: ApplyMode,
}

#[derive(Debug, Deserialize)]
pub(crate) struct ExternalSection {
    pub(crate) rtk: RtkSection,
}

#[derive(Debug, Deserialize)]
pub(crate) struct RtkSection {
    pub(crate) enabled: bool,
}

#[derive(Debug, Deserialize)]
pub(crate) struct McpSection {
    #[serde(default)]
    pub(crate) always_on: Vec<BaselineMcp>,
    #[serde(default)]
    pub(crate) env_gated: Vec<EnvGatedMcp>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct EnvGatedMcp {
    pub(crate) name: BaselineMcp,
    pub(crate) env: String,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum BaselineMcp {
    Context7,
    Exa,
    ChromeDevtools,
}

impl BaselineMcp {
    pub(crate) fn all() -> &'static [BaselineMcp] {
        &[
            BaselineMcp::Context7,
            BaselineMcp::Exa,
            BaselineMcp::ChromeDevtools,
        ]
    }

    pub(crate) fn name(self) -> &'static str {
        match self {
            BaselineMcp::Context7 => "context7",
            BaselineMcp::Exa => "exa",
            BaselineMcp::ChromeDevtools => "chrome-devtools",
        }
    }

    pub(crate) fn script_name(self) -> &'static str {
        match self {
            BaselineMcp::Context7 => "context7-mcp.sh",
            BaselineMcp::Exa => "exa-mcp.sh",
            BaselineMcp::ChromeDevtools => "chrome-devtools-mcp.sh",
        }
    }
}
