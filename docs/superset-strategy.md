# llm-bootstrap Superset Strategy

This document defines a curated superset for `llm-bootstrap`.

The goal is not to copy every good idea from every reference repo.

The goal is to:

1. combine the best ideas
2. prevent duplication and architectural drift

## Core rule

> keep exactly one source of truth and derive everything else from it.

That source of truth remains:

- `preset`
- `pack`
- `harness`
- `provider-native surface`

Everything else must derive from that layer:

- modes
- skills
- commands
- hook gates
- MCP profiles
- company loops
- review automation

## What to adopt from references

### Harness

- `gstack`
  - adopt: `office-hours -> plan -> review -> qa -> ship`
  - reject: the full runtime
  - why: we want the process contract, not the host runtime

- `oh-my-claudecode`
  - adopt: `team-plan -> team-prd -> team-exec -> team-verify -> team-fix`
  - reject: tmux worker runtime as a default
  - why: the staged pipeline is useful, but the bootstrap core should not become a worker orchestrator

- `oh-my-gemini`
  - adopt: `phase-gate`, `ralph-retry`
  - reject: the full Conductor state layer
  - why: keep gates thin and state minimal

### Multi-agent

- `oh-my-codex`
  - adopt: short execution entry points and verify/fix loop feel
  - reject: a giant agent catalog and heavy orchestration runtime
  - why: we need better entry points, not a full runtime clone

- `OpenClaw`
  - adopt: separation between control plane and worker runtime
  - reject: the multi-channel gateway runtime
  - why: useful for company-operation modeling, not for bootstrap core

### Skills and entry points

- `Roo Code`
  - adopt: short naming like `architect`, `ask`, and `build`
  - reject: a broad independent mode catalog
  - why: entry points should be clear, but not become a second source of truth

- `Aider`
  - adopt: precision-first, git-centric, small execution surface
  - reject: auto-commit by default
  - why: this is the strongest reference for keeping execution thin

### MCP, tools, and automation

- `Cline`
  - adopt: browser validation entry points and "add a tool" expansion UX
  - reject: editor snapshot runtime
  - why: validation UX matters, editor runtime does not belong in bootstrap core

- `Continue`
  - adopt: markdown-defined PR checks
  - reject: repo-level generated workflow files as a default
  - why: great for an advanced review lane, wrong for the default bootstrap layer

## Final layered architecture

### Layer 1. Core

Core is the identity of this repository.

- bootstrap installer
- provider-native renderer
- backup / restore / uninstall / doctor
- `preset -> pack -> harness -> apps/MCP/surface`
- minimal MCP baseline

No new runtime belongs here.

### Layer 2. Execution

Execution is a thin derived layer.

- `autopilot`
- `team`
- `office-hours`
- `review`
- `qa`
- `ship`
- `operating-review`

Important:

- do not create a second mode system
- every entry point must derive from `pack` and `harness`
- render only through provider-native surfaces

Examples:

- Codex: skills, subagents, AGENTS guidance
- Gemini: commands, hooks, extension docs
- Claude: subagents, hooks, workflow skills

### Layer 3. Advanced

Advanced is opt-in only.

- task-state
- repo automation lane
- company live loop
- optional team runtime
- channel and control-plane modeling

This layer must never rewrite the core identity.

### Layer 4. Never

These do not belong in the default product:

- giant mode catalogs
- tmux worker runtime by default
- multi-channel gateway runtime
- editor-extension runtimes themselves
- repo-level generated workflow files by default
- auto-commit defaults
- large session DB or telemetry systems

## Deduplication rules

### Rule 1. Modes are not a source of truth

Modes may exist only as entry point aliases.

- allowed: `autopilot` as an entry point
- forbidden: a standalone mode taxonomy outside `pack + harness`

### Rule 2. Skills, commands, and hooks are surface variants of one contract

The same feature should not be described three times.

Example:

- `review-gate`
  - Codex: skill
  - Gemini: command plus hook
  - Claude: workflow skill plus hook

But the internal contract name should remain one thing.

### Rule 3. Keep connectors as the business contract

One concept should own business meaning.

Current model:

- `connector` = business capability
- `app` = derived implementation source for connectors with `tool_source = app`

That means pack definitions should carry connectors, and user-facing app lists
should be derived instead of declared twice.

### Rule 4. Task-state must stay separate from install-state

Current state tracks install integrity:

- preset drift
- pack drift
- managed paths

Future task-state, if added, must remain separate:

- `track`
- `plan`
- `status`

Combining them would turn bootstrap into a runtime database.

### Rule 5. Repo automation stays advanced

`Continue` and `GitHub Agentic Workflows` are valuable, but
repo-level workflow generation must remain optional.

## What belongs in the backlog now

### P1

1. `entrypoint layer`
   - `autopilot`, `team`, `office-hours`, `operating-review`
   - all derived from `pack`

2. `hook gates`
   - `phase-gate`
   - `review-gate`
   - `ralph-retry`

3. `company live loop`
   - health and auth surfaces for `Linear`, `Gmail`, `Calendar`, `Drive`, `Figma`

### P2

4. `task-state`
   - a thin `track/spec/plan/status` layer

5. `review automation`
   - markdown-defined PR checks

6. `precision loop`
   - edit / verify / commit ergonomics

### P3

7. optional `team runtime`
8. channel and control-plane company modeling

## What should move out of the immediate backlog

- giant mode systems
- Conductor-scale memory layers
- tmux worker runtime by default
- OpenClaw-style gateway runtime
- Continue-style repo artifact generation by default

## Final judgment

A curated superset is possible.

But only under one condition:

> do not place every good idea in the same architectural layer.

In practice that means:

- keep `core` as bootstrap
- keep `execution` as thin entry points
- keep `advanced` opt-in
- define a clear `never` list

If that discipline holds, the result is not a messy mashup.
It becomes a structured superset.
