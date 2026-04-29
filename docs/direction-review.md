# Direction Review

This document records the reviewed materials, compares them against the current
implementation, and answers whether the current direction for
`StackPilot` is sound.

Its purpose is simple:

- keep the reviewed direction from fragmenting across multiple docs
- lock the sequencing so implementation priorities stay correct

## Review Scope

This checkpoint considers all of the following together:

- [docs/codex-first-blueprint.md](docs/codex-first-blueprint.md)
- [docs/provider-surface-strategy.md](docs/provider-surface-strategy.md)
- [docs/external-tool-landscape.md](docs/external-tool-landscape.md)
- [docs/business-ops-blueprint.md](docs/business-ops-blueprint.md)
- [bootstrap.toml](../bootstrap.toml)
- [src/manifest.rs](../src/manifest.rs)
- [src/providers/codex.rs](../src/providers/codex.rs)
- [src/providers/gemini.rs](../src/providers/gemini.rs)
- [src/providers/claude.rs](../src/providers/claude.rs)

So this is not just a collection of ideas. It is a judgment about whether the
documents and the code are pointing in the same direction.

## Current Verdict

The short version is:

> The direction is mostly correct, but the product priority and implementation
> order need to be locked down more tightly.

The right sequence is:

1. keep the bootstrap core small and safe
2. define multi-agent harnesses and the Ralph-loop as shared specs
3. render them through provider-native surfaces
4. absorb `gstack` as a workflow contract source, not as the runtime
5. use external tools to enrich capability without raising default cost
6. move business ops into higher-level packs only after the above is stable

That means the correct center of gravity right now is
`harness + provider-native distribution + optional external tooling`, not
`business ops`.

## What Is Already Correct

### 1. Bootstrap core positioning

The repository is still correctly positioned as a bootstrap core.

- `bootstrap.toml` only knows providers, default mode, and baseline MCP
- [src/manifest.rs](../src/manifest.rs) still only models `bootstrap`,
  `external`, and `mcp`
- provider renderers are still focused on user-home state, backup or restore,
  and official init paths

This center should remain intact.

### 2. Provider-native distribution

This is also correct.

- Codex is strongest through `config.toml`, `AGENTS.md`, plugins, agent TOML,
  and MCP
- Gemini is most natural through extensions, `settings.json`, and native
  command TOML
- Claude should prioritize official MCP CLI plus subagent, hook, and settings
  paths

The lowest-maintenance approach is not to force one file format across all
providers. It is to render the same intent into each provider's native surface.

### 3. Treating `gstack` as a contract source

This is the right interpretation.

`gstack` is most valuable for workflow discipline, not for runtime structure.

What is worth absorbing:

- `office-hours`
- `review`
- `qa`
- `ship`
- `retro`

What should not be copied directly:

- an entire runtime model
- a new taskboard or state layer
- abstractions that ignore provider-native surfaces

### 4. Keeping external tools in a separate landscape doc

This is also a good call.

The current grouping is practical:

- `RTK`: shell output compression
- `Repomix`: repo or context packing
- `Coding Context CLI`: rules and task separation
- `MCPM`: MCP profile management

That means token efficiency is being treated as an output-path and context-path
problem, not just as a terse-prompt problem.

## What Is Ahead Of The Code

### 1. Harness and pack catalogs

The docs already describe shared `harness`, `pack`, `ralph-loop`,
`parallel-build`, and `review-gate` structures. The code does not yet model
them.

Current implementation state:

- `bootstrap.toml` has no `harnesses` or `packs`
- [src/manifest.rs](../src/manifest.rs) does not deserialize them
- provider renderers do not yet read shared harness metadata and render from it

So the direction is correct, but it is not yet implemented structure.

### 2. Some Claude wording still drifts

The final direction for Claude is `subagent/MCP/hook-first`.

But a few document summaries and README phrases still carry older
skill-centered wording. That is not a conceptual error so much as an unfinished
documentation pass.

### 3. Category-aware doctor

The docs propose `runtime`, `mcp`, `harness`, `agent parity`, and
`workflow gates` as diagnostic categories, but the current doctor behavior is
still mostly file-presence based.

Again, the direction is right. The implementation is simply earlier.

## What Should Not Become The Immediate Priority

### 1. Implementing business ops first

[docs/business-ops-blueprint.md](docs/business-ops-blueprint.md) is still valid
as a longer-term expansion. It should not become the immediate implementation
center.

Why:

- there is no shared harness spec yet
- provider renderers do not consume shared harness metadata
- doctor cannot yet report pack or harness parity

So the business layer is not wrong. It is just one layer too early.

### 2. Overwriting native surfaces with one abstraction

This should be avoided.

If `plugin`, `extension`, `skill`, `subagent`, and `command` are forced into a
single install abstraction, maintenance cost will grow and provider-native
behavior will drift.

The correct pattern is:

- shared `harness` and `pack` as source of truth
- provider-native renderer output as the installed surface

### 3. Treating "lighter" as "fewer capabilities"

This was clarified during review and should stay explicit.

The goal is not capability reduction.

The goal is:

- rich capability
- lean baseline
- fast activation
- small context path

Heavy capability can still exist. It just should not be always-on.

## Recommended Priority Order

The safest next sequence is:

1. normalize the wording in `provider-surface-strategy`
2. add `harnesses` and `packs` design to `bootstrap.toml` and
   [src/manifest.rs](../src/manifest.rs)
3. define shared `delivery`, `parallel-build`, `incident`, `review-gate`, and
   `ralph-loop` harnesses
4. make Codex, Gemini, and Claude render those harnesses through their native
   surfaces
5. upgrade doctor in this order:
   `runtime -> mcp -> harness -> agent parity`
6. wire external tools into `core / optional / advanced` lanes
7. only then move business ops into higher-level packs

## Final Judgment

The current direction is not wrong. It is too broad unless the sequencing is
kept tight.

The correct direction is:

- keep the bootstrap core intact
- promote multi-agent harnesses into the default execution model
- define Ralph-loop as the baseline control flow
- absorb `gstack` as workflow contracts only
- connect external tools under a rich-but-lean principle
- follow provider official docs first

What should stay deferred:

- implementing business-wide packs first
- building a provider-agnostic runtime abstraction over native surfaces
- adding new persistent state layers or heavy gateway defaults

In one sentence:

> `StackPilot` should become the installer that places shared harness intent
> onto provider-native surfaces, not a new runtime that replaces them.

