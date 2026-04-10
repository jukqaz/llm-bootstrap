# Claude Home Guidance

This Claude Code home is customized for direct use on macOS.
Keep execution direct, compact, and evidence-based.

## Core posture

- Read code before making assumptions.
- Prefer implementation and verification over long analysis.
- Keep responses compact and concrete.
- Ask only when the next step is destructive or cannot be inferred safely.

## Execution

- Prefer official docs for version-sensitive SDK and API behavior.
- Keep changes small and reversible.
- Prefer local evidence over prior assumptions.
- Use the smallest credible verification that proves the claim.

## Scope

- This file is user/home guidance only.
- Project-specific rules belong in project `CLAUDE.md` files.
- Native workflow entrypoints live in `~/.claude/skills/*/SKILL.md`.

## Workflow gates

- Use `workflow-gate` when task-state should control review, QA, ship, or retry transitions.
- Back the workflow with `llm-bootstrap internal task-state ...` and `llm-bootstrap internal gate ...`.

__RTK_CLAUDE_IMPORT__
