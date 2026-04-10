---
name: workflow-gate
description: Evaluate and advance the thin workflow gate contract for task-state
---

# workflow-gate

Use this skill when the task already has a `task-state` and the next move
depends on ownership, handoff, review, QA, verification, or investigation
evidence.

## Flow

1. Read the current state with `llm-bootstrap internal task-state show --json`.
2. Check the next phase with `llm-bootstrap internal gate check --target-phase review|qa|ship --json`.
3. Add missing signals with `llm-bootstrap internal task-state advance --complete ...`.
4. Advance only after the gate report is clean with `llm-bootstrap internal gate apply --target-phase ... --json`.

## Contract

- `parallel-build` requires `ownership` before execution moves forward.
- `parallel-build` requires `handoff` before review, QA, or ship.
- `review-gate` requires `review`, `qa`, and `verify` before ship.
- `incident` requires `investigate` after repeated failed attempts.

## Output

- target phase
- missing signals
- command run
- remaining risk
