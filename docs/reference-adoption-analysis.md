# Reference Adoption Analysis

As of: `2026-04-13`

> Product scope is defined in [product-goal.md](product-goal.md),
> core/addon boundaries are defined in [monorepo-boundary.md](monorepo-boundary.md),
> and source classification lives in [reference-coverage.md](reference-coverage.md).

This document answers one question:

What should `StackPilot` actually adopt from the other reference repos if it
wants to become more useful without losing its product boundary?

This is not a collection document.
It is an adoption-priority document.

## Bottom Line

To reach `oh-my`-level usefulness, the next step is not "track even more repos."
It is to adopt eight concrete capability areas well:

1. strong entrypoint aliases
2. hook and gate enforcement
3. resumable context and checkpoints
4. precision-first execution loops
5. repo automation contracts
6. browser QA and tool onboarding
7. company inbox and control-plane thinking
8. context, rules, and MCP profile assembly

## Sources Reviewed

This analysis was refreshed against the current README or official page for:

- [`oh-my-codex`](https://github.com/junghwaYang/oh-my-codex)
- [`oh-my-gemini`](https://github.com/richardcb/oh-my-gemini)
- [`oh-my-claudecode`](https://github.com/Yeachan-Heo/oh-my-claudecode)
- [`gstack`](https://github.com/garrytan/gstack)
- [`GitHub Spec Kit`](https://github.com/github/spec-kit)
- [`GitHub Agentic Workflows`](https://github.github.com/gh-aw/)
- [`githubnext/agentics`](https://github.com/githubnext/agentics)
- [`Cline`](https://github.com/cline/cline)
- [`Roo Code`](https://github.com/RooCodeInc/Roo-Code)
- [`Continue`](https://github.com/continuedev/continue)
- [`Aider`](https://github.com/Aider-AI/aider)
- [`OpenClaw`](https://github.com/openclaw/openclaw)
- [`Repomix`](https://github.com/yamadashy/repomix)
- [`Coding Context CLI`](https://github.com/kitproj/coding-context-cli)
- [`MCPM`](https://github.com/pathintegral-institute/mcpm.sh)
- [`GitHub Copilot CLI`](https://github.com/github/copilot-cli)
- [`OpenHands`](https://github.com/OpenHands/OpenHands)
- [`Crush`](https://github.com/charmbracelet/crush)
- [`AgentAPI`](https://github.com/coder/agentapi)
- [`Caveman`](https://github.com/JuliusBrussee/caveman)

## P0: Adopt Next

### 1. Strong entrypoint alias layer

Primary sources:

- `oh-my-codex`
- `oh-my-claudecode`
- `Roo Code`
- `gstack`

Why it matters:

- `oh-my` feels strong because users see memorable entrypoints immediately.
- `StackPilot` has strong packs but still weak first-touch aliases.

Adopt:

- short, memorable execution aliases
- intent-first mode naming
- provider-consistent entrypoint names

Put it in:

- [`bootstrap.toml`](../bootstrap.toml)
- [`addons/stackpilot-dev-kit/`](../addons/stackpilot-dev-kit)
- [`README.md`](../README.md)

Do not copy:

- standalone `omx` / `omc` / `omg` style runtimes
- giant mode catalogs

### 2. Hook and gate enforcement

Primary sources:

- `oh-my-gemini`
- `gstack`
- `GitHub Agentic Workflows`

Adopt:

- deterministic phase gates
- retry semantics
- clear approval boundaries
- review/qa/verify requirements before ship

Put it in:

- [`src/main.rs`](../src/main.rs)
- [`src/state.rs`](../src/state.rs)
- provider addon entrypoints under [`addons/stackpilot-dev-kit/`](../addons/stackpilot-dev-kit)

Do not copy:

- GitHub Actions runtime into local core
- the full Gemini hook stack everywhere

### 3. Resumable context and checkpoints

Primary sources:

- `oh-my-gemini`
- `Spec Kit`
- `Cline`
- `Roo Code`

Adopt:

- spec -> plan -> tasks flow
- task checkpoints
- resumable summaries
- compact session re-entry state

Put it in:

- [`src/state.rs`](../src/state.rs)
- [`src/main.rs`](../src/main.rs)
- [`operating-record-model.ko.md`](operating-record-model.ko.md)

Do not copy:

- the full Conductor runtime
- full editor snapshot systems

### 4. Precision-first execution loop

Primary sources:

- `Aider`
- `gstack`

Adopt:

- short edit -> test -> review -> ship loops
- git-centric verification discipline
- evidence before commit and ship

Put it in:

- `delivery-pack`
- `incident-pack`
- `review-gate`
- `ship-check`

Do not copy:

- auto-commit defaults
- one universal runtime replacing provider-native tools

### 5. Repo automation contracts

Primary sources:

- `Continue`
- `GitHub Agentic Workflows`
- `githubnext/agentics`

Adopt:

- markdown or frontmatter workflow contracts
- data-driven PR and release gates
- repo-specific check registration

Put it in:

- `review-automation-pack`
- `internal repo-automation scaffold`
- `.github/stackpilot/*` generated contracts

Do not copy:

- forced repo generation by default
- Actions-centric runtime into bootstrap core

### 6. Browser QA and tool onboarding

Primary source:

- `Cline`

Adopt:

- browser QA entrypoints
- clearer tool-onboarding surfaces
- verification timeline thinking

Put it in:

- `delivery-pack`
- `qa-browser`
- docs and wizard guidance

Do not copy:

- the editor runtime
- the full GUI workflow

## P1: Adopt After P0 Lands

### 7. Company inbox and control-plane thinking

Primary source:

- `OpenClaw`

Adopt:

- treat connectors as inbox or channel surfaces, not just app names
- account routing and channel-health thinking

Put it in:

- `founder-pack`
- `ops-pack`
- `CONNECTORS.md`
- runtime handoff docs

### 8. Context and rule assembly

Primary sources:

- `Coding Context CLI`
- `Repomix`
- `Gitingest`

Adopt:

- rules versus task separation
- repo ingest lanes
- lightweight remote triage digests

Put it in:

- optional docs or context packs
- a future `repo-intake` lane

### 9. MCP profile management

Primary source:

- `MCPM`

Adopt:

- profile-based MCP thinking
- separate client integration from global catalog thinking

Put it in:

- `doctor`
- source catalog
- optional MCP management lane

### 10. Optional terse mode

Primary source:

- `Caveman`

Adopt:

- terse output constraint ideas
- terse commit and review helper ideas

Put it in:

- the source catalog as a reference-only item for now
- a shared common capability option later, only if repeated use proves the need

Do not make it default behavior or install Caveman as a separate pack.

## P2: Watch Only

Track these, but do not prioritize direct adoption now:

- `GitHub Copilot CLI`
- `OpenHands`
- `Goose`
- `Crush`
- `AgentAPI`
- `mini-swe-agent`
- `Kelos`
- `MCP-Universe`

These remain useful as boundary or architecture references, not immediate
product slices.

## Never Copy

Keep these hard boundaries:

1. full editor-extension runtimes
2. full tmux worker orchestration
3. full gateway daemons
4. auth/session replacement runtimes
5. forced project-repo scaffolding by default
6. giant specialist catalogs inside bootstrap core

## Recommended Implementation Order

1. strengthen `all-in-one` with real alias and entrypoint surfaces
2. upgrade `task-state` into a conductor-lite layer
3. push `review-automation-pack` toward markdown-contract workflows
4. raise `qa-browser` and tool onboarding in user-facing surfaces
5. add inbox/channel thinking to `founder-pack` and `ops-pack`

## Final Judgment

The correct move is not to copy every repo a little bit.
The correct move is to adopt the P0 capabilities strongly enough that the
`all-in-one` surface becomes real.

That means:

- take entrypoint feel, gates, and persistence from `oh-my-*`
- take execution discipline from `gstack`, `Spec Kit`, `Aider`, and `Continue`
- take higher-level ops models from `OpenClaw`, `MCPM`, and `Coding Context CLI`

That is the shortest path to a stronger `StackPilot`.
