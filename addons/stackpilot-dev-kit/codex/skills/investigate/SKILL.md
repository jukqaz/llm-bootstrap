---
name: investigate
description: Debug systematically by separating environment, config, and code causes before patching.
---

# investigate

Use this skill for runtime failures, regressions, and unclear breakage.

## Goal

- Avoid blind fixes
- Narrow the failure path before editing
- Keep proof and patch tightly connected

## Workflow

1. Name the observed symptom exactly.
2. Separate likely environment, config, and code causes.
3. Reproduce with the smallest command or runtime step.
4. Patch only after the failure boundary is clear.
5. Re-run the proof command and compare results.

## Output

- `Symptom:`
- `Cause class:`
- `Proof:`
- `Fix:`
- `Residual risk:`
