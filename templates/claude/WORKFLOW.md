# Workflow

Use this Claude Code lane for a compact plan -> review -> QA -> ship loop.

## Preferred sequence

1. Clarify the exact requested outcome.
2. Plan the smallest safe slice.
3. Execute against bounded files only.
4. Review for regression and missing verification.
5. Ship only with explicit evidence and known gaps.

## Workflow gates

- Keep `stack-pilot internal task-state begin|advance|show` current for multi-phase work.
- Check `stack-pilot internal gate check --target-phase plan|execute|review|qa|ship --json` before advancing.
- Record a failed bounded retry with `stack-pilot internal task-state advance --increment-attempt --failure "..."`.
- After the second failed attempt, attach investigation evidence with `stack-pilot internal task-state advance --investigation-note "..."`.
- Use the `workflow-gate` skill when the session should reason from gate evidence first.
