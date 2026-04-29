---
name: autopilot
description: Run a staged delivery loop from scoping through implementation, QA, and validation
---

# autopilot

Use this skill when the task should move through a full delivery loop instead of a single direct edit.

## Flow

1. Start from `OFFICE_HOURS.md` if scope is still fuzzy.
2. Lock a buildable plan using `WORKFLOW.md`.
3. Execute the smallest independent slices first.
4. Use `INVESTIGATE.md` before repeated blind fixes.
5. Finish against `SHIP_CHECKLIST.md`.

## Rules

- Prefer one owner unless parallel slices are genuinely independent.
- Do not skip QA and validation just because implementation finished.
- Stop when repeated failures indicate a fundamental issue.
