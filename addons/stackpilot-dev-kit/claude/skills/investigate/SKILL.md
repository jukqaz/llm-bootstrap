---
name: investigate
description: Debug a bug, regression, or runtime failure by separating environment, config, and code causes before patching.
disable-model-invocation: true
---

Use this workflow when the task is a bug, regression, or unclear runtime failure.

Rules:
1. Name the exact symptom first.
2. Separate environment, config, and code causes.
3. Patch only after the failure boundary is clear.
4. Re-run the narrowest proof after each fix.

For deeper reference, load `../../INVESTIGATE.md`.
