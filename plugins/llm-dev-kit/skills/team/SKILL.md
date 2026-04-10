---
name: team
description: Run a bounded multi-agent delivery lane with clear ownership, review, and verification
---

# team

Use this skill when the task is large enough to benefit from a planner,
executor, reviewer, and verifier split.

## Flow

1. Lock one objective and one acceptance target.
2. Split work only into slices with disjoint ownership.
3. Keep the planner on coordination and unblock decisions.
4. Run reviewer and verifier in parallel only when they do not block the next
   implementation step.
5. Finish with one merged report: changed files, verification, and residual
   risk.

## Rules

- Do not fan out when the next step depends on one unresolved detail.
- Do not let two workers edit the same file set.
- Reviewer findings lead the report.
- Verifier proves the critical path; do not pad with broad checks.
