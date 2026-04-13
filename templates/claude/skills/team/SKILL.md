---
name: team
description: Coordinate a bounded multi-agent lane with clear ownership and verification
---

# team

Use this workflow when one agent should coordinate planner, executor, reviewer,
and verifier roles around a shared target.

## Flow

1. Define the shared objective and acceptance check.
2. Split only into independent scopes.
3. Keep reviewer and verifier outside the write owner path.
4. Merge outcomes into one final evidence report.

## Rules

- Avoid overlapping file ownership.
- Keep the final report on changed files, verification, and remaining risk.
