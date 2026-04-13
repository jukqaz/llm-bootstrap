# Review Automation

Use this lane when review and ship gates should be enforced by repository
workflow, not only by the local session.

## Contract

1. Write the gate contract.
2. Register it in the repository workflow or branch protection lane.
3. Keep the local task-state gate aligned with the repo gate.

## Outputs

- PR gate contract
- release gate contract
- repository registration target
