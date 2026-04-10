# Team

Use this lane when planner, executor, reviewer, and verifier roles should
coordinate one bounded objective with explicit ownership.

## Contract

1. Lock the target and acceptance rule.
2. Split only into independent scopes.
3. Keep review and verification outside the write owner path.
4. Merge the evidence into one final report.

## Gate handoff

- `parallel-build` lanes should record `ownership` and `handoff` before review.
- `review-gate` lanes should not ship without `review`, `qa`, and `verify`.
- `incident` lanes add `investigate` after repeated failed attempts.

## Stop conditions

- overlapping ownership
- destructive branch point
- repeated verification failure
