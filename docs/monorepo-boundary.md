# Monorepo Boundary

This document fixes how the `StackPilot` repository should be split in
practice.

The decision is simple:

- keep one monorepo for now
- split the product contract into `provider kit`, `bootstrap umbrella`, and
  `addon`
- keep the name `StackPilot` attached to the umbrella CLI that manages
  provider kits

## Decision

The urgent problem is not repo splitting. It is boundary clarity.

This repository currently contains three different layers:

1. `provider-native kits`
2. `bootstrap umbrella`
3. `workflow/company addons`

They should not be flattened into the same product surface.

- provider kits change at provider update cadence
- the umbrella should change slowly and optimize for safety
- addons may iterate quickly and tolerate more churn

So the repository stays monorepo for now, while the docs, commands, and
ownership model are split into three layers.

## Provider Kits

Provider kits are baseline products that follow each LLM's native surface.

- `codex-kit`
  - config, AGENTS, MCP, plugins, skills, subagents, agent TOML
- `gemini-kit`
  - settings, extensions, commands, GEMINI.md, hooks, MCP
- `claude-kit`
  - CLAUDE.md, subagents, official MCP CLI, hooks, skills

Kit success criteria:

- keep provider-native strengths
- keep setting ownership clear across provider updates
- avoid tying release and verification too tightly to unrelated providers

## Umbrella

The `StackPilot` umbrella installs and verifies provider kits.

It owns:

- `install`
- `replace`
- `restore`
- `uninstall`
- `backup`
- `doctor`
- `probe`
- provider-kit rendering
- auth/session/history preservation
- old-tool cleanup
- compatibility checks against official provider surfaces

The umbrella succeeds when:

- it can reproduce a clean machine
- it can be re-applied to an existing machine
- it does not break auth or history
- drift is visible through `doctor`
- rollback and restore remain predictable

## Addons

Addons are optional capabilities layered on top of provider kits.

Examples:

- `stackpilot-dev-kit`
- workflow gate
- task-state
- record-work
- review / qa / ship / retro
- repo automation
- founder / company / operating review
- orchestration, team, and multi-agent helpers

Addons should:

- improve execution flow
- stay provider-native
- remain opt-in
- avoid polluting the umbrella contract

## Monorepo Rules

Inside the monorepo, keep these rules:

1. each provider kit must be explainable and verifiable independently
2. the umbrella installs, restores, and verifies kits without hiding native surfaces
3. addons may build on kits, but they cannot define umbrella success
4. the first README paragraph and release language must describe provider-native kits
5. addon docs should live in their own document set and directory boundary
6. preview/nightly provider churn is a compatibility-check problem, not a core feature race

If a stronger all-in-one product surface is needed, bundle it through a preset
such as `all-in-one`. Even then, keep the kit contract intact and preserve the
rule that `all-in-one = provider kits + addon bundle`, not a monolithic runtime.

## Documentation Boundary

From now on, read the docs like this:

- `README*`
  - product description and installation contract
- `product-goal*`
  - top-level goals centered on provider kits
- `monorepo-boundary*`
  - kit / umbrella / addon boundary
- `provider-native-kit-strategy*`
  - provider-specific kit product model
- capability, company, and workflow docs
  - addon design docs

The existence of capability docs does not automatically make them part of the
umbrella product contract.

## Directory Boundary

Do not split repos yet.
Split responsibility first.

Recommended direction:

```text
src/                       -> umbrella CLI, state, backup, doctor, probe
src/providers/             -> provider kit renderer implementation
kits/codex/                -> Codex kit source assets
kits/gemini/               -> Gemini kit source assets
kits/claude/               -> Claude kit source assets
templates/                 -> transitional provider renderer assets during migration
docs/monorepo-boundary*    -> repo-boundary docs
addons/stackpilot-dev-kit/        -> addon bundle source
addons/stackpilot-dev-kit/bundles -> addon bundle output
future addons/*            -> future addon-only space
```

Addon source lives under `addons/stackpilot-dev-kit/`.
Provider baseline assets should eventually move under `kits/{provider}`.

## Command Boundary

Describe user-facing default commands as umbrella commands.

Umbrella:

- `install`
- `baseline`
- `sync`
- `restore`
- `uninstall`
- `doctor`
- `probe`
- `wizard`

Candidate provider kit aliases:

- `codex install`
- `gemini install`
- `claude install`

Addon or internal lanes:

- `task-state`
- `record`
- `internal gate`
- `internal repo-automation`
- workflow/company capability entrypoints

Do not delete these commands yet.
Just stop documenting them as part of the default product contract.

## When to Extract Later

Move to a separate repo or package only when:

1. a provider kit's version cadence clearly diverges from the other kits
2. shipping a provider kit separately makes verification and release simpler
3. an addon can ship independently from the umbrella
4. the addon interface is stable enough
5. separate release, test, and docs pipelines are simpler than one monorepo

Before that point, the monorepo is cheaper.

## Current Operating Conclusion

Right now `StackPilot` can look like one UX for three providers plus addon
lab. The new framing should be:

- `StackPilot` = provider kit umbrella
- `codex-kit`, `gemini-kit`, `claude-kit` = provider-native baseline surfaces
- `stackpilot-dev-kit` = provider-neutral workflow addon bundle
- capability/company/orchestration docs = addon design layer

Keep one repo, but narrow the product identity around provider-native kits.
