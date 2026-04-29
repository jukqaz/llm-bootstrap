# Team

Use this lane when a Gemini session should coordinate planner, executor,
reviewer, and verifier roles against one bounded objective.

This is a gstack-style workflow contract over Gemini-native execution. It does
not introduce a separate worker runtime.

## Sequence

1. Run `deep-init` first when repository shape or validation commands are
   unclear.
2. Run `team-plan` to define the target, acceptance target, critical lane, and
   stop rule.
3. Run `team-scope` only when requirements or boundaries are still ambiguous.
4. Assign read or write ownership per slice before implementation.
5. Let `team-exec` run only on independent scopes.
6. Run `team-verify` against the same acceptance target.
7. Run `team-fix` only for targeted failed-gate repairs, then verify again.
8. Use `ultrawork` only for independent, reversible shards.

## Gate Contract

- Before execution, record `spec`, `plan`, and `ownership` when
  `parallel-build` is active.
- Before review, QA, or ship, record `handoff`.
- Before ship, record `review`, `qa`, and `verify`.
- After a second failed attempt, add investigation evidence instead of retrying
  blindly.

## Notes

- Prefer extension agents for planning, execution, review, and verification.
- Do not split work when file ownership overlaps.
- For cross-session work, persist state with `record-work` or
  `stack-pilot internal task-state`.
