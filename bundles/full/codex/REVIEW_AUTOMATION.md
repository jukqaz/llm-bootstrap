# Review Automation

Use this lane when `review-gate` should move from home guidance into repository
checks.

## Contract

1. Define the PR or release gate contract in markdown first.
2. Register the contract in repository workflow, required checks, or branch
   protection.
3. Keep the task-state gate and the repo gate aligned on `review`, `qa`,
   `verify`, and approval boundaries.

## Expected outputs

- PR review gate contract
- release readiness gate contract
- repository registration target
- missing external registration work
