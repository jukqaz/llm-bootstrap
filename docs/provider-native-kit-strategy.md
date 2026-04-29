# Provider Native Kit Strategy

This document fixes the decision to avoid forcing `Codex`, `Gemini`, and
`Claude Code` into one identical UX. Each provider gets its own native kit.

## Decision

`StackPilot` is not a product that makes all three LLMs work the same way.

The product surface is split into provider-native kits:

- `codex-kit`
  - Codex config, AGENTS, MCP, plugins, skills, subagents, agent TOML
- `gemini-kit`
  - Gemini settings, extensions, commands, GEMINI.md, hooks, MCP
- `claude-kit`
  - Claude Code CLAUDE.md, subagents, official MCP CLI, hooks, skills

`StackPilot` remains the umbrella CLI and shared catalog that installs,
updates, verifies, restores, and migrates those kits.

## Why Split

The three providers have different update cycles and configuration models.

- Codex changes around plugins, skills, subagents, MCP, and config.
- Gemini changes around extensions, commands, settings, and auth shape.
- Claude changes around subagents, MCP CLI, hooks, and user/project scopes.

Hiding those differences behind one common config creates two failures:

1. The project stops using each provider's native strengths.
2. Provider updates break the common abstraction.

The target is not identical configuration. The target is identical intent.

For example:

- The intent "enable review gate" is common.
- Codex receives it as skills, plugin, and subagent contract.
- Gemini receives it as extension commands and GEMINI.md contract.
- Claude receives it as subagents, skills, and hook contract.

## Where gstack-like Features Belong

Good `gstack`-style behavior is a workflow, skill, or harness contract. It is
not a fourth LLM runtime.

Do not clone `gstack` as another provider. Render the useful contracts into
each provider kit surface:

- office-hours
- plan
- implementation
- review
- QA
- ship
- retro
- record-work
- operating review

This lets Codex, Gemini, and Claude support the same work flow in their own
native forms.

## Shared Layer

Common code should reduce repetition and improve verification. It should not
flatten provider behavior.

Shared source of truth:

- MCP catalog
- harness contracts
- pack and preset metadata
- env-gated key metadata
- backup, restore, and uninstall rules
- doctor and probe categories
- old-tool cleanup registry

Do not share:

- the full provider config shape
- plugin, extension, or skill packaging formats
- provider-specific agent and subagent details
- UI, auth, auto-update, and model preferences
- provider runtime state added by new releases

## Product Boundary

The user-facing product should be described like this:

```text
stackpilot
  ├─ codex-kit
  ├─ gemini-kit
  └─ claude-kit
```

`install --providers codex,gemini` orchestrates two kits. It does not create a
universal config.

The CLI follows the same model through the `--providers` selector. Do not add
provider subcommands such as `stack-pilot codex install` until the implementation
actually supports them.

```bash
stack-pilot install --providers codex,gemini
stack-pilot install --providers codex
stack-pilot install --providers gemini
stack-pilot install --providers claude
```

## Directory Direction

Do not split repositories yet. First make the provider kit boundary visible in
the monorepo.

Recommended direction:

```text
src/                       -> umbrella CLI, state, backup, doctor, probe
src/providers/             -> provider kit renderer implementation
kits/codex/                -> Codex kit source assets
kits/gemini/               -> Gemini kit source assets
kits/claude/               -> Claude kit source assets
addons/stackpilot-dev-kit/        -> provider-neutral workflow source
catalog/                   -> MCP, tool, reference catalog
docs/                      -> product and architecture decisions
```

The current `templates/{provider}` and provider-specific addon output should
eventually move under `kits/{provider}`. Do that as a separate migration.

## Release Direction

Keep one release artifact at first.

Consider kit-specific releases later only when:

1. provider change cadence clearly diverges,
2. one release causes too much unrelated regression verification,
3. users mostly install one provider kit at a time, and
4. the install script can reliably choose kit-specific artifacts.

Until then, one binary managing three kits is simpler.

## Implementation Order

1. Fix the docs and README around the provider-native kit model.
2. Make `doctor` show provider kit status more explicitly.
3. Design the `templates/{provider}` to `kits/{provider}` migration.
4. Rework manifest `surfaces` around kit-native surfaces.
5. Keep existing provider flags, then add kit aliases.
6. Keep `gstack`-style behavior as shared harness contracts with provider-specific renderers.
7. Split provider release notes and compatibility checks.

## Avoid

- making all providers use the same file structure,
- copying the Codex plugin model into Gemini and Claude,
- explaining Gemini extensions as if they were Codex plugins,
- hiding Claude subagent, MCP, and hook structure behind a skill pack,
- optimizing for parity tables while losing provider-native value.

## Conclusion

This project's value is not "one config for all LLMs."

Its value is:

- build the best native baseline for each provider,
- keep good workflow behavior in shared contracts,
- render those contracts into each provider's own surface,
- use one umbrella CLI for install, restore, verification, and migration.

The product definition is `provider-native kits with shared bootstrap
operations`.
