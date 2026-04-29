# oh-my Family Comparison Report

This document compares the current `StackPilot` repository against the
public `oh-my-*` family from one practical angle:

> what is each product actually optimized for?

The comparison is intentionally split into two layers:

- local evidence from this repository
- public README claims from the `oh-my-*` projects

## Compared projects

- `StackPilot`
- `oh-my-codex`
- `oh-my-claudecode`
- `oh-my-gemini`

## Comparison premise

This is not a pure "which one is better" ranking.

The more useful question is:

> where does `StackPilot` sit in the stack, and where do the `oh-my-*`
> products sit?

Working assumption:

- `StackPilot` is primarily a bootstrap and policy renderer
- `oh-my-*` products are primarily orchestration runtimes

## Summary matrix

| Area | `StackPilot` | `oh-my-codex` | `oh-my-claudecode` | `oh-my-gemini` |
|---|---|---|---|---|
| Product center | provider-native bootstrap | Codex orchestration runtime | Claude team runtime | Gemini hook-enforced runtime |
| Primary user question | "How do I align my environment?" | "How do I run many agents?" | "How do I standardize Claude teams?" | "How do I enforce workflow and context?" |
| Source of truth | `preset -> pack -> harness -> surface` | modes, agents, skills, sessions | team workflow, skills, plugin/runtime | hooks, commands, Conductor state |
| Execution depth | medium | very strong | very strong | strong |
| Provider-native fit | very strong | medium | medium | medium |
| Install/recovery safety | very strong | medium | medium | medium |
| Set menus | strong | strong | strong | strong |
| Multi-agent execution | Codex-centered, partial | very strong | very strong | partial, roadmap still visible |
| Session persistence | install-state focused | strong | strong | strong via Conductor |
| Company operations fit | explicit and growing | weak | weak | weak |
| Connector metadata | strong | limited | limited | limited |
| Runtime opinionation | low | high | high | high |
| Maintenance cost | low to medium | high | high | high |

## Detailed assessment

### 1. Product layer

| Item | Assessment |
|---|---|
| `StackPilot` strength | keeps provider-native surfaces while sharing one pack model |
| `StackPilot` weakness | immediate "do work for me now" feel is weaker |
| `oh-my-*` strength | execution modes are the product surface itself |
| `oh-my-*` weakness | the runtime becomes thicker and more opinionated |

Judgment:

- `StackPilot` behaves like a foundation layer
- `oh-my-*` behaves like an execution layer

### 2. Multi-agent and Ralph-style execution

| Item | Assessment |
|---|---|
| `StackPilot` strength | has explicit shared harnesses such as `ralph-loop`, `ralph-plan`, `delivery`, `incident`, `founder-loop`, `operating-review` |
| `StackPilot` weakness | does not own a heavy orchestration runtime end to end |
| `oh-my-codex` strength | public README centers 32 agents, parallel execution, auto routing, session persistence |
| `oh-my-claudecode` strength | `team-plan -> team-prd -> team-exec -> team-verify -> team-fix` is a very strong execution contract |
| `oh-my-gemini` strength | hook-enforced workflow, phase gate, `ralph-retry`, and Conductor make behavior deterministic |
| `oh-my-gemini` weakness | public README still shows parts of multi-agent depth as evolving rather than fully settled |

Judgment:

- if the question is raw execution power, `oh-my-*` is stronger
- if the question is explicit harness definition plus install safety, this
  repository is cleaner

### 3. Provider-native fit

| Item | Assessment |
|---|---|
| `StackPilot` strength | Codex, Gemini, and Claude are rendered through their own native surfaces |
| `StackPilot` weakness | the shared UX is intentionally weaker |
| `oh-my-*` strength | each product creates a strong first-run experience on its home provider |
| `oh-my-*` weakness | the unified runtime can drift away from provider-native behavior |

Judgment:

- for long-term provider alignment, `StackPilot` is better positioned
- for immediate orchestration feel inside one provider, `oh-my-*` is stronger

### 4. Install, backup, doctor, and restore

| Item | Assessment |
|---|---|
| `StackPilot` strength | install, replace, uninstall, restore, and doctor are core product contracts |
| `StackPilot` weakness | it is not trying to be the live execution HUD/runtime |
| `oh-my-*` strength | setup paths are fast and user-facing |
| `oh-my-*` weakness | public positioning emphasizes runtime usability more than home-state safety and reversibility |

Judgment:

- if home state often drifts, `StackPilot` is much stronger
- if the goal is instant orchestration, `oh-my-*` is stronger

### 5. Company operations

| Item | Assessment |
|---|---|
| `StackPilot` strength | already models `founder-pack`, `ops-pack`, connectors, and company automation contracts |
| `StackPilot` weakness | connector auth and recurring scheduler registration are still runtime-managed boundaries |
| `oh-my-*` strength | excellent developer-facing orchestration |
| `oh-my-*` weakness | public READMEs remain overwhelmingly developer-orchestration focused |

Judgment:

- for "development plus company operations", `StackPilot` has the clearer direction
- for "developer orchestration right now", `oh-my-*` still feels stronger

### 6. Maintenance cost

| Item | Assessment |
|---|---|
| `StackPilot` strength | catalog and renderer separation keeps drift more manageable |
| `StackPilot` weakness | product wow-factor is lower |
| `oh-my-*` strength | modes are memorable and easy to sell |
| `oh-my-*` weakness | hooks, skills, persistence, analytics, and orchestration runtime raise maintenance cost quickly |

Judgment:

- this repository is better positioned as a long-lived operating baseline
- the `oh-my-*` family is better positioned as a high-intensity orchestration product

## Current repository strengths

1. It preserves provider-native surfaces.
2. It has a clear `preset -> pack -> harness -> apps/MCP/surface` model.
3. It treats backup, restore, uninstall, and doctor as first-class contracts.
4. It explicitly aims at both development and company operations.
5. Documentation, catalog, installer, and doctor reporting are aligned.

## Current repository weaknesses

1. The orchestration runtime feel is weaker than `oh-my-*`.
2. Session persistence and live execution memory remain intentionally thin.
3. Connector auth and recurring scheduling are still outside the bootstrap boundary.
4. Provider-native fidelity reduces the strength of a shared UX.
5. The first-run impression is less dramatic than `autopilot`, `team`, or `ralph` driven products.

## `oh-my-*` family strengths

1. Execution modes are memorable and productized.
2. Entry points like `autopilot`, `team`, `ralph`, `review`, and `ultrawork` are immediately useful.
3. Parallelism and persistence are highly visible.
4. The first user experience is strong.

## `oh-my-*` family weaknesses

1. A thicker runtime can drift away from provider-native behavior.
2. Install and recovery safety are not the center of the product story.
3. Company operations, connectors, and automation modeling are weaker.
4. Long-term maintenance cost can rise quickly.

## Final judgment

The most accurate framing is this:

- `StackPilot` is not mainly trying to replace `oh-my-*`
- it is building a lower and safer layer in the stack
- `oh-my-*` products are stronger at immediate orchestration
- this repository is stronger at provider-native alignment, safety, and company-operation extensibility

Recommended strategy:

1. Do not copy the full `oh-my-*` runtime.
2. Keep strengthening `preset`, `pack`, `ralph-loop`, `ralph-plan`, `founder-pack`, and `ops-pack`.
3. Keep provider-native rendering and install safety intact.
4. Add stronger execution entry points only where they clearly improve the current packs.

One-line summary:

> `oh-my-*` acts like a team lead that pushes work through; `StackPilot`
> acts like the operating baseline that keeps environments, policy, and rollout
> state from drifting.

## Sources

Local repository:

- [README.md](../README.md)
- [README.ko.md](../README.ko.md)
- [bootstrap.toml](../bootstrap.toml)
- [provider-surface-strategy.ko.md](provider-surface-strategy.ko.md)
- [dev-company-operating-model.ko.md](dev-company-operating-model.ko.md)

Public READMEs:

- [oh-my-codex README](https://raw.githubusercontent.com/junghwaYang/oh-my-codex/refs/heads/main/README.md)
- [oh-my-claudecode README](https://raw.githubusercontent.com/Yeachan-Heo/oh-my-claudecode/refs/heads/main/README.md)
- [oh-my-gemini README](https://raw.githubusercontent.com/richardcb/oh-my-gemini/refs/heads/main/README.md)
