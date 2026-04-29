# Workflow

Use this lane when the task needs a heavier development harness.

## Preferred sequence

1. Triage the request and isolate the user-visible outcome.
2. Read only the files that bound the behavior.
3. Implement the smallest reversible change.
4. Verify with the narrowest credible command set.
5. Report exact files changed, verification run, and remaining risk.

## Workflow gates

- Keep `sp internal task-state show --json` current when the task spans multiple phases.
- Check `sp internal gate check --target-phase plan|execute|review|qa|ship --json` before phase changes.
- Persist finished signals with `sp internal task-state advance --complete spec,plan,...`.
- Record a failed bounded retry with `sp internal task-state advance --increment-attempt --failure "..."`.
- Apply the phase move with `sp internal gate apply --target-phase ... --json`.

## Browser-heavy work

- Use `chrome-devtools` for live inspection, console, network, and layout debugging.
- Prefer direct repro and focused verification over adding browser automation by default.
