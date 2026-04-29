# managed by stackpilot

# Review Automation

This repository uses `StackPilot` review automation contracts.

Default branch: `__DEFAULT_BRANCH__`

## Required status checks

Add these checks to the branch protection rule for `__DEFAULT_BRANCH__`.

Always require:

- `pr-review-gate / gate`

Repo-specific checks currently configured:

__PR_REQUIRED_CHECK_LINES__

## Release readiness checks

Before tagging or shipping, run `release-readiness-gate` and keep these checks green:

__RELEASE_REQUIRED_CHECK_LINES__

## Pull request checklist

Keep this checklist in the PR body or PR template:

```md
- [x] review
- [x] qa
- [x] verify
```

Approval requirement:

- minimum approvals: `__MINIMUM_APPROVALS__`

## Operator notes

- `pr-review-gate` blocks draft PRs.
- `pr-review-gate` also requires a non-author approval; GitHub does not allow the PR author to self-approve.
- `release-readiness-gate` verifies the target ref is still reachable from `__DEFAULT_BRANCH__`.
- The first `workflow_dispatch` validation for `release-readiness-gate` can run only after this workflow file exists on the default branch.
- Update `.github/stackpilot/review-automation.json` when CI check names change.
