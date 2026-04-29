---
name: qa
description: Run the smallest credible QA pass and summarize what was actually verified.
disable-model-invocation: true
---

Use this lane for targeted QA after implementation.

Rules:
1. Prefer the smallest credible verification set.
2. Record exact commands or checks that were run.
3. Name any path that was not validated.

For deeper reference, load `../../QA.md`.
