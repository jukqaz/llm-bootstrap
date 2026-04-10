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
4. Apply the gate and move the phase only after the report is clean.

## CLI

- `llm-bootstrap internal task-state show --json`
- `llm-bootstrap internal gate check --target-phase review|qa|ship --json`
- `llm-bootstrap internal task-state advance --complete ownership,handoff,review,qa,verify`
- `llm-bootstrap internal gate apply --target-phase ship --json`

## Rules

- `parallel-build`: `ownership`, then `handoff`
- `review-gate`: `review`, `qa`, `verify` before ship
- `incident`: `investigate` after repeated failed attempts
