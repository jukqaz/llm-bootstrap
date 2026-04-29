# Orchestration and gstack Contract

This document defines what `StackPilot` adopts from `gstack`-style workflows
and what it intentionally leaves out.

The decision is narrow: `StackPilot` is not a separate worker runtime or tmux
orchestrator. It is a contract layer that renders the same execution discipline
onto each provider's native surface.

## Adopted

The useful part of `gstack` is execution discipline:

- lock one objective and acceptance target first
- create an owner map before execution
- split parallel work only when file, module, and responsibility boundaries do not overlap
- leave a 10-20 line handoff before each stage transition
- keep reviewer and verifier outside the write-owner path
- allow one targeted fix for a failed gate, then require investigation evidence after repeated failure
- close with changed files, verification, and remaining risk

These rules are delivered through `parallel-build`, `workflow-gate`, `team`,
`ultrawork`, and `record-work`.

## Not Adopted

The default product does not include:

- a long-running orchestrator daemon
- a tmux worker runtime
- a provider-agnostic session database
- a custom prompt runtime that bypasses provider-native features
- automatic multi-worker edits across overlapping file scopes
- local replication of external PM, CRM, QA, or SaaS data

External state should be linked through `record-work` or connector handoff
instead of copied into local state.

## Pack Mapping

The `gstack`-style contract is active through `team-pack`.

| Preset | Packs | Orchestration level |
| --- | --- | --- |
| `normal` | `delivery-pack`, `incident-pack` | Single-lane delivery and incident response |
| `orchestrator` | `delivery-pack`, `incident-pack`, `team-pack` | Owner, handoff, review, and QA gates |
| `all-in-one` | development, team, company, review automation | Development orchestration plus company operating records |

When `parallel-build` is active, `task-state gate` requires `ownership` before
execution and `handoff` before review, QA, or ship.

## Provider Rendering

The same contract is rendered into provider-native surfaces.

| Provider | Surface |
| --- | --- |
| Codex | `stackpilot-dev-kit` plugin, native agents, `team`, `ultrawork`, `workflow-gate` skills |
| Gemini | `stackpilot-dev` extension commands, `TEAM.md`, `gate`, `team`, `ultrawork` commands |
| Claude Code | native subagents, user-scope skills, official MCP, `team`, `workflow-gate` skills |

The shared source of truth is the harness, pack, and preset metadata in
`bootstrap.toml`, plus provider entrypoints under `addons/stackpilot-dev-kit`.

## Gate Contract

| Signal | Meaning | Required before |
| --- | --- | --- |
| `spec` | Objective and acceptance target are fixed | plan |
| `plan` | Execution order and bounded files are known | execute |
| `ownership` | Lane owners and write scopes do not overlap | execute with `parallel-build` |
| `handoff` | Stage transition record exists | review, QA, ship with `parallel-build` |
| `review` | Reviewer checked regression risk | ship |
| `qa` | Smallest credible command or runtime check ran | ship |
| `verify` | Acceptance target is proven with evidence | ship |

After repeated failure, the lane should not advance without an
`investigation-note`.

## CLI Pattern

```bash
stack-pilot install --providers codex,gemini,claude --preset orchestrator --mode replace

stack-pilot internal task-state begin \
  --title "Build auth flow" \
  --providers codex,gemini,claude \
  --preset orchestrator \
  --phase plan \
  --owner codex \
  --summary "Auth flow objective and acceptance target are being fixed." \
  --checkpoint "Resume from owner map and failing fixture inventory."

stack-pilot internal task-state advance --complete spec,plan,ownership
stack-pilot internal gate check --target-phase execute --json
stack-pilot internal gate apply --target-phase execute --json

stack-pilot internal task-state advance --complete handoff,review,qa,verify
stack-pilot internal gate check --target-phase ship --json
```

## Handoff Format

Stage handoff should stay compact.

```markdown
## Handoff
- Objective:
- Acceptance target:
- Owner map:
- Files touched:
- Decisions:
- Rejected options:
- Verification:
- Risks:
- Next action:
```

## Stop Rules

- do not parallelize when the owner map overlaps
- keep the main lane on any unresolved detail that blocks the next step
- after a second verification failure, record investigation evidence instead of blind retrying
- stop before destructive branch points or external writes that require approval
- when provider-native features are stronger, use them instead of creating a StackPilot runtime

The intended difference from `oh-my-*` and `gstack` is product boundary:
`StackPilot` aligns each LLM's baseline and execution discipline, while actual
execution remains provider-native.
