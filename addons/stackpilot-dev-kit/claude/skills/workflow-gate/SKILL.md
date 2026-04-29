---
name: workflow-gate
description: Check and advance thin workflow gates for task-state
---

# workflow-gate

Use this workflow when a task should not advance until the local gate contract
is satisfied.

## Flow

1. Read the current task state.
2. Check the target phase gate.
3. Persist only the signals that are actually complete.
4. Keep the resumable lane current with `stack-pilot internal task-state advance --summary "..." --checkpoint "..."` when the stopping point changes.
5. Record a failed bounded retry with `--increment-attempt --failure "..."`.
6. After the second failed attempt, attach investigation evidence with `--investigation-note "..."`.
7. Apply the gate and move the phase only after the report is clean.

## CLI

- `stack-pilot internal task-state show --json`
- `stack-pilot internal gate check --target-phase plan|execute|review|qa|ship --json`
- `stack-pilot internal task-state advance --complete spec,plan,ownership,handoff,review,qa,verify`
- `stack-pilot internal task-state advance --summary "..." --checkpoint "..."`
- `stack-pilot internal task-state advance --increment-attempt --failure "..."`
- `stack-pilot internal task-state advance --investigation-note "..."`
- `stack-pilot internal gate apply --target-phase ship --json`

## Rules

- `phase-gate`: `spec` before plan
- `phase-gate`: `plan` before execute
- `ralph-retry`: repeated failures require investigation evidence
- `parallel-build`: `ownership`, then `handoff`
- `review-gate`: `review`, `qa`, `verify` before ship
- `incident`: investigation evidence after repeated failed attempts

## Gstack-style Lane

- Treat `gstack` as a workflow contract source, not a runtime dependency.
- Keep Claude subagents inside an explicit owner map.
- Require a compact handoff before review, QA, or ship.
- Use provider-native execution after the gate is clean.
