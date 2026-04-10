# Team

Use this lane when the task benefits from planner, executor, reviewer, and
verifier handoffs instead of one long owner chain.

## Entry criteria

- The work can be split into independent slices or clear read/write ownership.
- Review and verification can run in parallel with implementation.
- The final outcome still fits one shared plan and one acceptance target.

## Team contract

1. Planner defines the objective, slice boundaries, and stop rules.
2. Executors own disjoint write scopes and report changed files.
3. Reviewer checks regressions, unsafe assumptions, and missing tests.
4. Verifier runs the narrowest proof that the claim is true.
5. The lead agent resolves conflicts and reports the final state.

## Gate handoff

- Before review, make sure `ownership` and `handoff` are recorded in task-state.
- Before ship, check the gate and require `review`, `qa`, and `verify`.
- After repeated failures under `incident`, add `investigate` before the next gated phase.

## Stop conditions

- Two slices need the same write scope.
- A destructive choice needs operator input.
- Verification keeps failing after a targeted retry.

## Output contract

- owners and scopes
- changed files
- verification run
- remaining risk
