---
name: review-automation
description: Prepare repository review and release gate contracts without forcing repo file generation
---

# review-automation

Use this skill when `review-gate` should be pushed into a repository automation
lane.

## Flow

1. Identify whether the contract is for PR review, release readiness, or both.
2. Keep the contract in markdown first.
3. Keep `.github/PULL_REQUEST_TEMPLATE.md` aligned with the PR checklist.
4. Register the contract in repository workflow, required checks, or branch protection.
5. Report what still needs to be configured outside bootstrap.

## Outputs

- contract type
- required checks
- PR template state
- repository registration target
- remaining external setup
