# Codex-First Blueprint

`StackPilot` combines ideas from several LLM home bootstrap projects without
copying any of them wholesale.

The target support order is:

1. Codex
2. Gemini
3. Claude Code

Codex is the mainline. Gemini and Claude Code are bridge layers that preserve
the Codex-first design rather than redefining it.

## Borrowed patterns

### oh-my-codex

- Codex-first home baseline
- practical agent roster
- `AGENTS.md` driven operating rules
- explicit end-to-end workflow entrypoints

### oh-my-gemini-cli and oh-my-gemini

- extension-first layout
- hook and docs separation
- `settings.json` merge instead of full replacement

### oh-my-claudecode

- Claude compatibility lane
- lightweight reuse of Claude-native surfaces such as skills and subagent docs

### oh-my-openagent

- routing work by role
- specialist fanout
- hierarchical instruction layering

### oh-my-agent

- opinionated workflow pack
- specialist lane separation

### OpenHarness

- modular harness mindset
- plugin, role, and tool grouping as distinct layers

### gstack

- strong delivery loop:
  - plan
  - review
  - qa
  - ship
- separate artifacts for investigation, office hours, and retro

### Harness-style patterns

- verification discipline
- release discipline
- artifact-first operational thinking

## What we intentionally do not import

- giant state machines
- persistent taskboard or workspace memory
- provider-specific interaction models that diverge too far
- large default MCP bundles
- giant agent catalogs
- telemetry or auto-learning hooks by default

## Core repository rules

- Rust installer and doctor
- manifest-driven shared intent in `bootstrap.toml`
- idempotent `install`, `uninstall`, and `doctor`
- backup before write or delete
- `merge` by default, `replace` as opt-in
- official tool init paths when possible
- home-scope only, not project-scope
- env-gated MCP are disabled, not half-enabled
- secret managers are not integrated directly; the repo consumes env only

## Provider lanes

### Codex

- richest workflow pack
- strongest agent roster
- local plugin skills
- long-context opt-in lanes only

### Gemini

- extension pack
- native custom commands
- workflow docs and lightweight agents
- global model surface only, no per-agent model pin

### Claude Code

- optional compatibility lane
- user-scope docs and skills
- lightweight subagent docs
- official skill and user-scope MCP flow

## Module layout

This section describes the logical architecture, not literal top-level paths.
The actual implementation lives under `src/`, `templates/`, and `plugins/`.

- `src/main.rs`
  - CLI orchestration
- `src/runtime.rs`, `src/fs_ops.rs`, `src/json_ops.rs`
  - runtime dependency bootstrap, backup, file and JSON operations
- `src/providers/codex.rs`
  - Codex config and plugin renderer
- `src/providers/gemini.rs`
  - Gemini settings merge and extension renderer
- `src/providers/claude.rs`
  - Claude user-scope skill and MCP renderer
- `templates/*` and `addons/stackpilot-dev-kit/`
  - provider templates and addon assets

## Current baseline

- `chrome-devtools`
- `context7` when `CONTEXT7_API_KEY` exists
- `exa` when `EXA_API_KEY` exists
- Codex plugin and workflow pack
- Gemini extension and workflow pack
- Claude compatibility pack
