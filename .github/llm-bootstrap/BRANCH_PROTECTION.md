# managed by llm-bootstrap

# Review Automation

This repository uses `llm-bootstrap` review automation contracts.

Default branch: `main`

## Required status checks

Add these checks to the branch protection rule for `main`.

Always require:

- `pr-review-gate / gate`

Repo-specific checks currently configured:

- `check`

## Release readiness checks

Before tagging or shipping, run `release-readiness-gate` and keep these checks green:

- `check`
- `pr-review-gate / gate`

## Pull request checklist

Keep this checklist in the PR body or PR template:

```md
- [x] review
- [x] qa
- [x] verify
```

Approval requirement:

- minimum approvals: `1`

## Operator notes

- `pr-review-gate` blocks draft PRs.
- `pr-review-gate` also requires a non-author approval; GitHub does not allow the PR author to self-approve.
- `release-readiness-gate` verifies the target ref is still reachable from `main`.
- The first `workflow_dispatch` validation for `release-readiness-gate` can run only after this workflow file exists on the default branch.
- Update `.github/llm-bootstrap/review-automation.json` when CI check names change.
