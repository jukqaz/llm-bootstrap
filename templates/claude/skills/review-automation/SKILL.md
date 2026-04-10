---
name: review-automation
description: Prepare repository review automation contracts for PR and release gates
---

# review-automation

Use this workflow when repository workflow should enforce `review-gate`.

## Flow

1. Decide whether the repo needs a PR gate, release gate, or both.
2. Write the contract in markdown first.
3. Register it in workflow, required checks, or branch protection.
4. Call out the remaining external registration work.
