# Reference Repo Backlog

This document captures how additional reference repositories should influence
the next planning layer for `StackPilot`.

The goal is not to collect repos for their own sake.

It is to separate:

- what should enter the plan
- what should remain reference only
- what should stay out of the core product

## Additional repos reviewed

- `OpenClaw`
- `Cline`
- `Roo Code`
- `Continue`
- `Aider`

## Main judgment

The current repository is already strong at
`preset -> pack -> harness -> apps/MCP/surface`.

These additional references mainly reinforce five gaps:

1. stronger execution entry points
2. browser or runtime validation
3. task-state or snapshots
4. workflow-as-code review gates
5. channel and control-plane thinking for company operations

## Repo-by-repo assessment

### 1. OpenClaw

- Link: [openclaw/openclaw](https://github.com/openclaw/openclaw)
- Adopt:
  - multi-channel inbox thinking
  - agent routing by channel or account
  - clear separation between control plane and worker runtime
- Later:
  - when founder or ops packs grow into real external channel workflows
- Do not copy:
  - the messaging gateway itself into bootstrap core

Judgment:

OpenClaw matters less as a coding runtime and more as a future company-ops
reference for account, channel, and inbox routing.

### 2. Cline

- Link: [cline/cline](https://github.com/cline/cline)
- Adopt:
  - browser validation entry points
  - task snapshot and restore ideas
  - "add a tool" MCP-style expansion UX
- Later:
  - editor-bound runtime details
- Do not copy:
  - workspace snapshot machinery as default home-bootstrap state

Judgment:

The current repository has install-state recovery, not task-state recovery.
Cline is a good reference for a thin future task-state layer.

### 3. Roo Code

- Link: [RooCodeInc/Roo-Code](https://github.com/RooCodeInc/Roo-Code)
- Adopt:
  - mode naming ideas
  - short entry points like architect, ask, and build
  - custom mode packaging patterns
- Later:
  - editor-extension-heavy UX
- Do not copy:
  - broad mode catalogs that live outside pack and harness mapping

Judgment:

`StackPilot` has strong presets, but weak execution entry points.
Roo Code is useful as a naming and packaging reference.

### 4. Continue

- Link: [continuedev/continue](https://github.com/continuedev/continue)
- Adopt:
  - markdown-defined PR checks
  - review-gate as repository automation
  - security and QA gate contracts as files
- Later:
  - repo-level workflow generation as a default path
- Do not copy:
  - project file generation into the bootstrap default flow

Judgment:

This is the clearest reference for turning `review-gate` into an advanced
automation lane without pulling repo-level behavior into the default bootstrap.

### 5. Aider

- Link: [Aider-AI/aider](https://github.com/Aider-AI/aider)
- Adopt:
  - precision-first loop
  - git-centric commit ergonomics
  - minimal execution surface
- Later:
  - auto-commit behavior
- Do not copy:
  - a single runtime replacing provider-native lanes

Judgment:

Aider is a good reminder that stronger execution does not require a huge
runtime. It is the best reference here for keeping the surface small.

## Plan insertion

### P1

1. Add an `entrypoint layer`
   - candidates: `autopilot`, `team`, `office-hours`, `operating-review`
   - references: `oh-my-*`, `Roo Code`

2. Add thin `hook gates`
   - candidates: `phase-gate`, `review-gate`, `ralph-retry`
   - reference: `oh-my-gemini`

3. Strengthen the `company live loop`
   - candidates: health or auth surfaces for `Linear`, `Gmail`, `Calendar`, `Drive`, `Figma`
   - reference: `OpenClaw`

### P2

4. Add a thin `task-state` layer
   - candidates: `track/spec/plan/status`
   - references: `Cline`, `oh-my-gemini`, `spec-kit`

5. Push `review-gate` into a repo automation lane
   - candidate: PR check contracts
   - references: `Continue`, `GitHub Agentic Workflows`

6. Tighten the `precision loop`
   - candidate: lighter edit/verify/commit flow
   - reference: `Aider`

### P3

7. Reframe company connectors around inbox or channel models
   - reference: `OpenClaw`

8. Add an optional `team runtime` lane
   - references: `oh-my-claudecode`, `oh-my-codex`

## Keep out of the core

- multi-channel gateway runtime
- editor-extension runtimes themselves
- giant mode catalogs
- repo-level generated workflow files by default
- auto-commit as a default behavior

## One-line conclusion

These additional references reinforce the same product direction:

- keep the core as `bootstrap + provider-native renderer`
- add execution depth through `entrypoints + hook gates + task-state + review automation`
- evolve company operations from a connector list toward channel and control-plane thinking
