---
name: deep-init
description: Map a repository before long-running implementation, migration, or cross-session work.
---

# deep-init

Use this skill before complex delivery, migration, or work that needs durable
context across sessions.

## Flow

1. Read the nearest `AGENTS.md` and project guidance first.
2. Scan README, source layout, build files, tests, CI, and likely entrypoints.
3. Identify source-of-truth config, generated artifacts, auth/session
   boundaries, external services, and risky ownership boundaries.
4. Capture the smallest credible verification commands before implementation
   begins.
5. If the work needs durable resume, use `record-work` or
   `stack-pilot internal task-state`.
6. Do not generate broad repo docs or AGENTS hierarchies unless the user
   explicitly asks for that output.

## Output

- Deep Init Summary
- Project Map
- Guardrails
- Verification Plan
- Recommended Next Command
