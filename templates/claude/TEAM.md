# Team

Use this Claude Code lane when planner, executor, reviewer, and verifier roles
should coordinate one bounded delivery target.

This is a gstack-style workflow contract over Claude Code native subagents and
skills, not a separate worker runtime.

## Sequence

1. Run `deep-init` first when the repository shape or validation commands are
   unclear.
2. Run `team-plan` to lock the target, acceptance rule, critical lane, and stop
   rule.
3. Run `team-scope` only when requirements or boundaries are still ambiguous.
4. Run `team-exec` with disjoint ownership before implementation begins.
5. Run `team-verify` with reviewer and verifier separate from the write owner.
6. Run `team-fix` only for a targeted failed-gate pass, then verify again.
7. Use `ultrawork` only for independent, reversible shards.

## Gate handoff

- Before execution, record `spec`, `plan`, and `ownership` when
  `parallel-build` is active.
- Before review, QA, or ship, record `handoff`.
- Before ship, require `review`, `qa`, and `verify`.
- After repeated failure under `incident`, require `investigate`.
- For cross-session work, persist state with `record-work` or
  `stack-pilot internal task-state`.

## Guardrails

- No overlapping write scopes.
- No blind retries after repeated verification failure.
- Escalate when the next decision is destructive.
