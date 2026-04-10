# Workflow

Use this Claude Code lane for a compact plan -> review -> QA -> ship loop.

## Preferred sequence

1. Clarify the exact requested outcome.
2. Plan the smallest safe slice.
3. Execute against bounded files only.
4. Review for regression and missing verification.
5. Ship only with explicit evidence and known gaps.

## Workflow gates

- Keep `llm-bootstrap internal task-state begin|advance|show` current for multi-phase work.
- Check `llm-bootstrap internal gate check --target-phase plan|execute|review|qa|ship --json` before advancing.
- Use the `workflow-gate` skill when the session should reason from gate evidence first.
