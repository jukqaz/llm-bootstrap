---
name: delivery-loop
description: Run a gstack-like plan, review, QA, and ship loop without expanding scope.
---

# delivery-loop

Use this skill when the task is large enough to need a deliberate execution loop, but not a giant harness.

## Goal

- Keep planning, review, QA, and ship decisions explicit
- Avoid scope drift between implementation and signoff
- Leave a compact handoff trail in the final report

## Workflow

1. Plan the smallest deliverable slice and name the bounded files.
2. Review the intended change before or immediately after implementation.
3. Run the narrowest credible QA or verification set.
4. Ship only after gaps are stated plainly.

## Output

- `Plan:` requested outcome and bounded files
- `Review:` main risk or regression check
- `QA:` command or runtime check
- `Ship:` ready or blocked, with reason
