# Provider Surface Strategy

This document explains how `StackPilot` should deliver multi-agent harnesses,
Ralph-loop style control flow, and `gstack`-inspired workflow contracts across
different LLM runtimes.

The top-level rule is simple:

> keep catalog and workflow intent shared, but split the product surface into
> provider-native kits

That means we should not force one abstraction across `plugin`, `extension`,
`skill`, `command`, and `mcp`. Each provider should receive the same intent as
`codex-kit`, `gemini-kit`, or `claude-kit` through the surface it handles best.

`StackPilot` is the umbrella installer, doctor, probe, and migration tool for
those three kits.

## Why This Direction

The current implementation already shows that each provider has a different
natural shape.

- `Codex`: strongest with plugins, agent config files, and multi-agent runtime
- `Gemini`: strongest with extensions, settings merge, and native commands
- `Claude Code`: safest when it prioritizes subagents, official MCP CLI, and
  hook or settings paths

The codebase already follows that pattern:

- Codex installs `.agents/plugins/marketplace.json`, local plugins, and agent
  TOML files
- Gemini installs `extensions/stackpilot-dev`, patches `settings.json`, and
  adds command TOML files
- Claude installs `agents/*`, workflow skills, and registers MCP through
  official CLI flows

So the right move is to formalize provider-native distribution instead of
flattening everything into one fake abstraction.

## Official Guidance Comes First

Before adopting community patterns, we should align with the surfaces each
provider officially supports and documents.

Priority order:

1. provider-native surfaces explicitly documented by the vendor
2. officially supported install and configuration paths
3. community-proven patterns
4. our own abstraction layer

`StackPilot` should compose and automate native surfaces, not replace them.

## Official Docs Summary

### Codex / OpenAI

OpenAI’s official Codex docs treat these as first-class surfaces:

- `config.toml`
- `AGENTS.md`
- hooks
- MCP
- plugins
- skills
- subagents

OpenAI’s Docs MCP guide also states that MCP configuration is shared between the
Codex CLI and IDE extension.

Implication:

- Codex is not just a plugin surface
- it is a combined config, rules, MCP, plugin, and subagent system
- our Codex renderer should treat `config + AGENTS + MCP` as the base, then add
  plugins, skills, and subagents on top

### Gemini CLI / Google

Gemini CLI’s official docs emphasize:

- extensions
- `settings.json`
- custom commands
- `GEMINI.md`
- hooks
- MCP servers
- subagents
- agent skills

The extension reference also formalizes commands, hooks, skills, policies, and
themes as extension-level capabilities.

Implication:

- Gemini should remain extension-first
- command and settings merge are official paths, not just our preference
- skills and hooks should naturally live inside extensions where possible

### Claude Code / Anthropic

Anthropic’s official Claude Code docs strongly emphasize:

- subagents
- MCP
- hooks
- settings
- scope hierarchy

They also document `claude mcp add`, `.mcp.json` environment expansion, and
scope precedence explicitly.

Implication:

- Claude should follow `subagents + MCP + hooks/settings` before community
  skill-pack patterns
- skills can remain a packaging layer, but they are not the main official
  surface
- our Claude renderer should prioritize subagents, MCP, hooks, and settings,
  with skills as a secondary workflow layer

## Faster And Lighter Implementation Rules

If we want the same capability with less weight and less latency, we should
remove duplication instead of adding more machinery.

Priority rules:

- define shared intent only once
- do not build a new runtime per provider
- do not add new persistent state layers
- keep default install limited to core MCP and core harnesses
- make heavy lanes opt-in only

Execution rules:

- keep one shared role taxonomy source
- keep one shared harness definition source
- treat provider plugin, extension, and skill outputs as thin render targets
- use `planner`, `triage`, `executor`, `reviewer`, and `verifier` as the core
  baseline
- make specialist roles additive, not default

Avoid:

- duplicating harness docs per provider until they drift
- rewriting the same workflow contract across plugins, extensions, and skills
- adding taskboards, session databases, or runtime caches
- shipping heavy MCP in the default install

In one line: one shared intent and verification layer, three provider-native
kits.

## Top-Level Structure

The repository should separate five concepts:

- `mcp catalog`
- `plugin catalog`
- `harness catalog`
- `pack catalog`
- `provider renderer`

Meaning:

- `mcp catalog`: what MCP entries exist and what env or permission they need
- `plugin catalog`: plugin or extension style distribution units
- `harness catalog`: team topology, workflow, and artifact contracts
- `pack catalog`: install compositions
- `provider renderer`: the layer that translates shared catalogs into native
  provider surfaces

The catalog can be shared while rendering remains provider-specific.

To stay lightweight, keep the catalog count minimal at first.

Recommended minimum structure:

- `mcp catalog`
- `harness catalog`
- `pack catalog`
- `provider renderer`

Only promote `plugin catalog` to a first-class layer when there are multiple
independent plugin sources to manage. Early on, plugin metadata can live inside
pack metadata and still stay simpler.

## Shared Across Providers

These should stay common:

- role taxonomy
- harness definitions
- workflow contracts
- artifact schemas
- MCP metadata
- permission metadata
- doctor categories

Examples:

- role taxonomy: `triage`, `planner`, `executor`, `reviewer`, `verifier`
- harnesses: `delivery`, `parallel-build`, `incident`, `review-gate`
- workflow gates: `office-hours`, `autopilot`, `review`, `qa`, `ship`, `retro`

## Codex Strategy

### Preferred surfaces

- `config.toml`
- `AGENTS.md`
- MCP
- plugin
- agent TOML
- hooks
- subagents
- skills
- workflow docs

### Why

Codex has the strongest multi-agent and plugin surface.

Today it already uses:

- `.agents/plugins/marketplace.json`
- `plugins/stackpilot-dev-kit`
- `agents/*.toml`
- `config.toml` features such as `enable_fanout` and `multi_agent_v2`

The official docs also expose `Config`, `AGENTS.md`, `Hooks`, `MCP`,
`Plugins`, `Skills`, and `Subagents` as core Codex surfaces.

That makes Codex the right reference runtime for harness behavior.

### Direction

- use Codex as the reference provider for multi-agent harness behavior
- define squad topology and handoff contracts here first
- ship `gstack`-style workflow gates through rules, skills, workflow docs, and
  subagent contracts
- keep MCP managed via scripts and rendered `config.toml` blocks
- treat plugins as rich distribution units, not as replacements for config,
  AGENTS, or MCP

### Good fits in Codex

- planner-driven staffing
- keeping `reviewer`, `verifier`, and `test-engineer` distinct
- preserving `*-1m` lanes as opt-in only
- using plugins as workflow and harness distribution units

### Avoid in Codex

- one giant plugin that owns everything
- expanding role count without harness definitions
- mixing MCP, plugins, and harnesses into one block of config

## Gemini Strategy

### Preferred surfaces

- extension
- `settings.json`
- native custom commands
- `GEMINI.md`
- hooks
- MCP entries in `settings.json`
- subagents
- skills
- lightweight agent markdown

### Why

Gemini is more naturally extension-first than plugin-first.

The current implementation centers on:

- `extensions/stackpilot-dev`
- `commands/*.toml`
- `settings.json` merge
- `extension-enablement.json`

The official docs also place `Extensions`, `Settings`, `Custom commands`,
`GEMINI.md`, `Hooks`, `MCP servers`, `Subagents`, and `Agent Skills` in the
main feature surface.

Gemini should receive the harness through commands, extension docs, and role
prompts rather than through a fake plugin abstraction.

### Direction

- make Gemini workflow-extension-first
- keep role files prompt-contract focused
- translate Codex role taxonomy into Gemini commands and agent notes
- keep MCP managed through `settings.json` patches
- avoid forcing the word `plugin` where `extension` is the real surface
- keep hooks, skills, and policies inside extensions when possible

### Good fits in Gemini

- explicit command lanes such as `intent`, `doctor`, `autopilot`, `review`,
  and `ship`
- extension docs that expose handoff contracts
- settings merge that preserves user runtime state

### Avoid in Gemini

- pretending provider-native per-agent model pinning exists
- cloning the Codex plugin model directly
- projecting an overly heavy subagent system onto the Gemini surface

## Claude Code Strategy

### Preferred surfaces

- subagents
- official `claude mcp add/remove --scope user`
- `.mcp.json`
- hooks
- settings
- official `claude mcp add/remove --scope user`
- lightweight subagent docs
- workflow skill pack
- workflow markdown
- minimal settings patching

### Why

Claude Code is most stable when it stays close to official subagent, MCP, hook,
and settings flows.

The current implementation already follows this:

- `agents/*.md`
- official MCP registration
- user-scope `CLAUDE.md`
- `skills/*/SKILL.md`

Claude should stay a compatibility lane, but its official base is
`subagents + MCP + hooks/settings`.

### Direction

- prioritize `subagents + MCP + hooks/settings`
- preserve `autopilot`, `investigate`, `review`, `qa`, `ship`, and `retro`
  as workflow skills and docs, but as a secondary layer
- express squad composition first through subagent contracts, then reinforce it
  through skills and docs
- keep MCP strictly on official CLI registration paths
- minimize settings mutation

### Good fits in Claude

- clear subagent separation
- official MCP add/remove flow
- hooks and scope hierarchy
- clear workflow skill packs
- readable lightweight agent docs

### Avoid in Claude

- imitating the Codex plugin structure
- inventing a heavy runtime abstraction Claude does not natively support
- creating new hidden state layers outside user home

## MCP Strategy

MCP should use a shared catalog with provider-specific installation.

### Shared rules

- split MCP into `core`, `optional`, and `specialized`
- keep env-gating and permission scope as metadata
- separate read-only from write-capable integrations
- install via provider-native paths

### Suggested categories

- `core`
  - `chrome-devtools`
  - `context7`
  - `exa`
- `orchestration-support`
  - docs, browser QA, repo search style tools
- `domain-specific`
  - payment, CRM, analytics, design, project tools

### Provider-specific application

- Codex: `[mcp_servers.*]` in `config.toml`
- Gemini: `mcpServers` in `settings.json`
- Claude: official CLI registration results

## Plugin and Extension Strategy

Plugins or extensions should be treated as distribution units, not runtime
units.

Recommended mapping:

- Codex: config + MCP + plugin
- Gemini: extension + commands + settings
- Claude: subagents + MCP + hooks, with skill packs as secondary packaging

So the shared catalog should speak in terms of `pack` or `harness`, while each
provider renderer turns that into provider-native packaging.

Example:

- `delivery-harness`
  - Codex: rules plus MCP plus plugin skills plus subagent contracts
  - Gemini: extension commands plus settings plus docs plus skills
  - Claude: subagents plus MCP plus hooks plus workflow skills

For a lighter design, plugin, extension, and skill outputs should not become
the source of truth.

Recommended rule:

- source of truth lives in shared `harness` and `pack` metadata
- plugin, extension, and skill outputs are rendered artifacts at install time
- do not manually maintain the same contract in multiple provider folders

## Harness Strategy

Harnesses should be provider-agnostic at definition time.

Recommended harnesses:

- `delivery`
- `parallel-build`
- `incident`
- `review-gate`
- `ralph-loop`

Each harness should include:

- team topology
- role ownership
- handoff schema
- stop rule
- QA gate
- final artifact contract

This is where `gstack` belongs as a contract source:

- `office-hours`
- `review`
- `qa`
- `ship`
- `retro`

In other words, `gstack` is not the runtime. It is the workflow contract layer
on top of our runtime.

## Recommended Install Compositions

### Codex-first

- core MCP
- `stackpilot-dev-kit` plugin
- `delivery-harness`
- `parallel-build-harness`

### Gemini-first

- core MCP
- `stackpilot-dev` extension
- `delivery-harness`
- `review-gate-harness`

### Claude compatibility

- core MCP
- skill pack
- `delivery-harness`
- `incident-harness`

### Full orchestrator

- core MCP
- provider-native pack
- `delivery`
- `parallel-build`
- `incident`
- `review-gate`
- `ralph-loop`

## Doctor Direction

`doctor` should move beyond file existence checks.

It should report at least:

- `runtime`
- `mcp`
- `plugins/extensions/skills`
- `harnesses`
- `agent parity`
- `workflow gates`

Examples:

- `ok mcp chrome-devtools`
- `missing harness parallel-build on Gemini`
- `role security-reviewer has no Claude equivalent`

This should also roll out incrementally.

Minimum doctor priority:

1. `runtime`
2. `mcp`
3. `harness`
4. `agent parity`

`plugins/extensions/skills` can initially remain implementation details under
the broader question of whether the intended harness is installed.

## Implementation Order

1. add `harnesses` and `packs` to the manifest
2. keep the current MCP structure and enrich it with metadata
3. make Codex, Gemini, and Claude render the same harnesses through different
   native surfaces
4. define `delivery`, `parallel-build`, `incident`, `review-gate`, and
   `ralph-loop`
5. upgrade `doctor` to be minimally category-aware
6. add a standalone `plugins` catalog and domain-specific MCP only when needed

## One-Line Summary

- Codex should be config/MCP/plugin-first
- Gemini should be extension/settings-first
- Claude should be subagent/MCP/hook-first
- MCP should use a shared catalog with provider-native install
- harnesses should be shared in definition and provider-specific in rendering
- `gstack` should act as a workflow contract source, not the base runtime
- the lightest implementation is one shared harness spec plus provider renderers
