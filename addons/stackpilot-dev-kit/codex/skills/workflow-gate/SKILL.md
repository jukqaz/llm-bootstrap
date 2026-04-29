---
name: workflow-gate
description: Evaluate and advance the thin workflow gate contract for task-state
---

# workflow-gate

Use this skill when the task already has a `task-state` and the next move
depends on ownership, handoff, review, QA, verification, or investigation
evidence.

## Flow

1. Read the current state with `stack-pilot internal task-state show --json`.
2. Check the next phase with `stack-pilot internal gate check --target-phase plan|execute|review|qa|ship --json`.
3. Add missing signals with `stack-pilot internal task-state advance --complete ...`.
4. Keep the resumable lane current with `stack-pilot internal task-state advance --summary "..." --checkpoint "..."` when the stopping point changes.
5. Advance only after the gate report is clean with `stack-pilot internal gate apply --target-phase ... --json`.
6. When verification fails, record the failure with `stack-pilot internal task-state advance --increment-attempt --failure "..."`.
7. After the second failed attempt, attach investigation evidence with `stack-pilot internal task-state advance --investigation-note "..."`.

## Contract

- `phase-gate` requires `spec` before plan.
- `phase-gate` requires `plan` before execute.
- `ralph-retry` requires investigation evidence after repeated failed attempts.
- `parallel-build` requires `ownership` before execution moves forward.
- `parallel-build` requires `handoff` before review, QA, or ship.
- `review-gate` requires `review`, `qa`, and `verify` before ship.
- `incident` requires investigation evidence after repeated failed attempts.

## Gstack-style Lane

- Treat `gstack` as a workflow contract source, not as a runtime dependency.
- Owner maps must be explicit before delegated or parallel execution.
- Handoffs must capture objective, acceptance target, owner map, touched files,
  verification, risks, and next action.
- Prefer provider-native agents, skills, or commands for execution after the
  gate is clean.

## Output

- target phase
- missing signals
- command run
- remaining risk
