# Team

Use this Claude Code lane when planner, executor, reviewer, and verifier roles
should coordinate one bounded delivery target.

## Sequence

1. Lock the target, acceptance rule, and stop rule.
2. Assign disjoint ownership before implementation begins.
3. Keep reviewer and verifier separate from the write owner.
4. Merge the results into one final report with evidence.

## Gate handoff

- Before review, record `ownership` and `handoff`.
- Before ship, require `review`, `qa`, and `verify`.
- After repeated failure under `incident`, require `investigate`.

## Guardrails

- No overlapping write scopes.
- No blind retries after repeated verification failure.
- Escalate when the next decision is destructive.
