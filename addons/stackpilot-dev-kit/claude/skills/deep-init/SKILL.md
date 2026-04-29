---
name: deep-init
description: Map a repository before long-running implementation, migration, or cross-session work.
---

# deep-init

Use this skill before complex delivery, migration, or work that needs durable
context across sessions.

## Flow

1. Scan repository instructions, README, source layout, build files, tests, CI,
   and likely entrypoints.
2. Identify source-of-truth config, generated artifacts, auth/session
   boundaries, and external services.
3. Build a compact project map: modules, responsibilities, dependency
   hotspots, and risky ownership boundaries.
4. Capture the smallest credible verification commands before implementation
   begins.
5. If the work needs a durable record, prefer `record-work` or
   `stack-pilot internal task-state` over inventing provider-specific state
   files.
6. Do not create AGENTS.md files or broad generated docs unless the user
   explicitly asks for that output.

## Output

- Deep Init Summary
- Project Map
- Guardrails
- Verification Plan
- Recommended Next Command
