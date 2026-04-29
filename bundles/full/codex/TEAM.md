# Team

Use this lane when the task benefits from planner, executor, reviewer, and
verifier handoffs instead of one long owner chain.

This is a gstack-style workflow contract over Codex-native agents and skills,
not a separate worker runtime.

## Entry criteria

- The work can be split into independent slices or clear read/write ownership.
- Review and verification can run in parallel with implementation.
- The final outcome still fits one shared plan and one acceptance target.

## Team contract

1. Run `deep-init` first when repository shape or validation commands are
   unclear.
2. Planner defines the objective, critical lane, slice boundaries, and stop
   rules.
3. Use `team-scope` only when requirements or boundaries are still ambiguous.
4. Executors own disjoint write scopes and report changed files.
5. Reviewer checks regressions, unsafe assumptions, and missing tests.
6. Verifier runs the narrowest proof that the claim is true.
7. Use `ultrawork` only for independent, reversible shards.
8. The lead agent resolves conflicts and reports the final state.

## Gate handoff

- Before execution, make sure `spec`, `plan`, and `ownership` are recorded when
  `parallel-build` is active.
- Before review, QA, or ship, make sure `handoff` is recorded in task-state.
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
