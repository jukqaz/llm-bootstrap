use crate::cli::{ApplyMode, Provider};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub(crate) struct BootstrapManifest {
    pub(crate) bootstrap: BootstrapSection,
    pub(crate) external: ExternalSection,
    pub(crate) mcp: McpSection,
    #[serde(default)]
    pub(crate) harnesses: Vec<HarnessDefinition>,
    #[serde(default)]
    pub(crate) packs: Vec<PackDefinition>,
    #[serde(default)]
    pub(crate) presets: Vec<PresetDefinition>,
    #[serde(default)]
    pub(crate) connectors: Vec<ConnectorDefinition>,
    #[serde(default)]
    pub(crate) automations: Vec<AutomationDefinition>,
    #[serde(default)]
    pub(crate) record_templates: Vec<RecordTemplateDefinition>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct BootstrapSection {
    pub(crate) providers: Vec<Provider>,
    pub(crate) default_mode: ApplyMode,
    pub(crate) default_preset: String,
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

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct HarnessDefinition {
    pub(crate) name: String,
    pub(crate) category: HarnessCategory,
    #[serde(default)]
    pub(crate) default_enabled: bool,
    pub(crate) description: String,
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct PackDefinition {
    pub(crate) name: String,
    pub(crate) scope: PackScope,
    pub(crate) lane: PackLane,
    #[serde(default)]
    pub(crate) harnesses: Vec<String>,
    #[serde(default)]
    pub(crate) mcp_servers: Vec<BaselineMcp>,
    #[serde(default)]
    pub(crate) connectors: Vec<String>,
    #[serde(default)]
    pub(crate) codex_surfaces: Vec<String>,
    #[serde(default)]
    pub(crate) gemini_surfaces: Vec<String>,
    #[serde(default)]
    pub(crate) claude_surfaces: Vec<String>,
    pub(crate) description: String,
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct PresetDefinition {
    pub(crate) name: String,
    #[serde(default)]
    pub(crate) packs: Vec<String>,
    pub(crate) description: String,
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct ConnectorDefinition {
    pub(crate) name: String,
    pub(crate) category: ConnectorCategory,
    pub(crate) tool_source: ConnectorToolSource,
    pub(crate) access: ConnectorAccess,
    pub(crate) approval: ConnectorApproval,
    pub(crate) automation_allowed: bool,
    pub(crate) description: String,
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct AutomationDefinition {
    pub(crate) name: String,
    pub(crate) cadence: AutomationCadence,
    #[serde(default)]
    pub(crate) packs: Vec<String>,
    #[serde(default)]
    pub(crate) connectors: Vec<String>,
    pub(crate) artifact: String,
    pub(crate) description: String,
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct RecordTemplateDefinition {
    pub(crate) name: String,
    pub(crate) record_type: String,
    pub(crate) stage: String,
    #[serde(default)]
    pub(crate) packs: Vec<String>,
    #[serde(default)]
    pub(crate) surfaces: Vec<String>,
    pub(crate) description: String,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum DistributionTarget {
    CodexPlugin,
    GeminiExtension,
    ClaudeSkills,
}

impl DistributionTarget {
    pub(crate) fn name(self) -> &'static str {
        match self {
            DistributionTarget::CodexPlugin => "codex-plugin",
            DistributionTarget::GeminiExtension => "gemini-extension",
            DistributionTarget::ClaudeSkills => "claude-skills",
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum HarnessCategory {
    Core,
    Development,
    Company,
    Quality,
}

impl HarnessCategory {
    pub(crate) fn name(self) -> &'static str {
        match self {
            HarnessCategory::Core => "core",
            HarnessCategory::Development => "development",
            HarnessCategory::Company => "company",
            HarnessCategory::Quality => "quality",
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum PackScope {
    Development,
    Company,
}

impl PackScope {
    pub(crate) fn name(self) -> &'static str {
        match self {
            PackScope::Development => "development",
            PackScope::Company => "company",
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum PackLane {
    Core,
    Optional,
    Advanced,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum ConnectorCategory {
    Delivery,
    Communication,
    Knowledge,
    Design,
}

impl ConnectorCategory {
    pub(crate) fn name(self) -> &'static str {
        match self {
            ConnectorCategory::Delivery => "delivery",
            ConnectorCategory::Communication => "communication",
            ConnectorCategory::Knowledge => "knowledge",
            ConnectorCategory::Design => "design",
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum ConnectorToolSource {
    App,
    Mcp,
    Native,
}

impl ConnectorToolSource {
    pub(crate) fn name(self) -> &'static str {
        match self {
            ConnectorToolSource::App => "app",
            ConnectorToolSource::Mcp => "mcp",
            ConnectorToolSource::Native => "native",
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum ConnectorAccess {
    ReadOnly,
    ReadWrite,
}

impl ConnectorAccess {
    pub(crate) fn name(self) -> &'static str {
        match self {
            ConnectorAccess::ReadOnly => "read-only",
            ConnectorAccess::ReadWrite => "read-write",
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum ConnectorApproval {
    None,
    OnWrite,
    Always,
}

impl ConnectorApproval {
    pub(crate) fn name(self) -> &'static str {
        match self {
            ConnectorApproval::None => "none",
            ConnectorApproval::OnWrite => "on-write",
            ConnectorApproval::Always => "always",
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum AutomationCadence {
    Daily,
    Weekly,
    OnDemand,
}

impl AutomationCadence {
    pub(crate) fn name(self) -> &'static str {
        match self {
            AutomationCadence::Daily => "daily",
            AutomationCadence::Weekly => "weekly",
            AutomationCadence::OnDemand => "on-demand",
        }
    }
}

impl PackLane {
    pub(crate) fn name(self) -> &'static str {
        match self {
            PackLane::Core => "core",
            PackLane::Optional => "optional",
            PackLane::Advanced => "advanced",
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq)]
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
