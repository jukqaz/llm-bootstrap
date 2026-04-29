---
name: repo-radar
description: Quickly map a repository before implementation by checking AGENTS, package/tooling files, test commands, and likely entrypoints.
---

# repo-radar

Use this skill when a new repository needs a fast, evidence-based first pass.

## Goal

- Identify stack and package manager
- Find AGENTS or equivalent local instructions
- Locate likely app entrypoints and test commands
- Surface the smallest safe next action

## Workflow

1. Read the nearest `AGENTS.md` files first.
2. Inspect top-level manifest files such as `package.json`, `Cargo.toml`, `go.mod`, `pubspec.yaml`, `pyproject.toml`.
3. Find test, lint, and build commands before editing.
4. Report:
   - stack
   - likely commands
   - important directories
   - immediate execution lane

## Output

- Keep it under 10 lines unless the caller asks for detail.
- Prefer absolute paths and concrete commands.
