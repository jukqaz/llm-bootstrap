# Ralph Loop Program Plan

This document turns the current `llm-bootstrap` backlog into a small set of
delivery harness lanes that can run through the same Ralph loop repeatedly.

The intent is simple:

- stop re-sorting the backlog every turn
- keep the next slice explicit
- finish one bounded lane at a time

## Program Goal

Grow the enablement layer without weakening baseline safety.

## Completion Rule

The current program is "done" only when every active harness lane has:

1. a stable contract
2. matching provider-facing surfaces
3. passing narrow verification
4. no unresolved blocker that belongs in the same lane

## Ralph Loop

Every lane follows the same loop:

1. lock a bounded goal
2. implement the smallest useful slice
3. review for drift or overlap
4. run the smallest credible QA
5. rewrite the next slice before leaving the lane

## Harness Lanes

### 1. `workflow-control-plane`

Status: `done`

Goal:
- make the thin local workflow contract strong enough to control phase movement

Scope:
- `phase-gate`
- `task-state`
- `review-gate`
- `ralph-retry`
- `record` handoff alignment

Done already:
- `phase-gate` now covers `plan -> execute -> review -> qa -> ship`
- ship-side review gate is already enforced
- `record --from-task-state` can now attach the active task-state to record context and supply owner/next action fallbacks
- `ralph-retry` now requires investigation evidence after repeated failures and keeps gate output resumable

Verification:
- `cargo test`
- targeted CLI smoke for `task-state` and `gate`
- install plus provider surface refresh

Next slice:
- close the first live PR and release validation in `repo-automation-lane`

### 2. `repo-automation-lane`

Status: `in-progress`

Goal:
- move review and release gate behavior into an optional repository automation lane

Scope:
- `pr-review-gate`
- `release-readiness-gate`
- branch protection contract
- optional scaffold command

Done already:
- scaffold command exists
- workflow templates exist
- this repo already dogfoods the generated gate files
- live PR validation now proves the gate waits for `check` and then stops only on the external approval requirement

Still inside this lane:
- tighten PR checklist and repo contract ergonomics
- document required manual GitHub settings more sharply
- rerun PR gate with a non-author approval
- run the first live `workflow_dispatch` release validation after the workflow lands on `main`

Verification:
- `cargo test`
- YAML parse check
- scaffold dry-run and real-run checks
- one real GitHub Actions validation pass when available

Next slice:
- land the repo automation workflows on `main`, then validate one approved PR run and one release-dispatch run

### 3. `entrypoint-layer`

Status: `next`

Goal:
- make the product feel actionable immediately through short, obvious entrypoints

Scope:
- `autopilot`
- `team`
- `office-hours`
- `operating-review`
- mode and lane naming cleanup

Why this matters:
- the current repository has strong packs and harnesses
- it still feels weaker at "how do I start this lane right now?"

Verification:
- provider surface diff review
- install and doctor
- minimal command help and usage smoke

Next slice:
- define the exact entrypoint contract and trim overlap between entrypoints

### 4. `precision-loop`

Status: `next`

Goal:
- improve the edit -> verify -> commit loop without building a heavy runtime

Scope:
- tighter verification guidance
- retry behavior
- smaller result summaries
- commit ergonomics

Verification:
- direct CLI or shell smoke
- docs and command examples staying aligned

Next slice:
- define the smallest acceptable precision contract before adding more commands

### 5. `company-live-loop`

Status: `later`

Goal:
- turn company-operation connectors into live operating surfaces, not just listed apps

Scope:
- health or auth surfaces for `Linear`, `Gmail`, `Calendar`, `Drive`, `Figma`
- founder and ops handoff quality
- channel or inbox model preparation

Verification:
- `doctor --json`
- connector-specific runtime handoff checks
- record-first evidence that points to external source-of-truth systems

Next slice:
- define connector health surface expectations before adding more operating loops

## Sequencing

Run the program in this order:

1. `workflow-control-plane`
2. `repo-automation-lane`
3. `entrypoint-layer`
4. `precision-loop`
5. `company-live-loop`

The practical reason is dependency shape:

- `workflow-control-plane` is the control contract
- `repo-automation-lane` depends on that contract
- `entrypoint-layer` should expose stable underlying behavior
- `precision-loop` should refine a stable execution path
- `company-live-loop` should sit on top of all of the above

## Keep Out

Do not pull these into the current program scope:

- heavy runtime orchestration
- provider-specific wow defaults
- giant mode catalogs
- repo workflow generation as a default install path
- auto-commit as baseline behavior
