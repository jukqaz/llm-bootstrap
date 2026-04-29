---
name: ship-check
description: Run the smallest credible verification set before reporting completion.
---

# ship-check

Use this skill before claiming a change is done.

## Goal

- Choose the smallest command set that proves the claim
- Avoid redundant or vanity checks
- State gaps plainly when full verification is not possible

## Workflow

1. Identify what changed: code, config, docs, runtime wiring, or tests.
2. Prefer targeted verification in this order:
   - syntax / parse
   - typecheck / lint
   - focused tests
   - runtime smoke
3. If a stronger check is skipped, say why.

## Output

- `Verified:` command and result
- `Not verified:` missing command and reason, only if needed
