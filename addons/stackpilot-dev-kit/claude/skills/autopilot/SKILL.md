---
name: autopilot
description: Run a staged implementation loop from scoped plan through verification. Use when the task should move from clear scope to verified output with minimal steering.
disable-model-invocation: true
---

Use this workflow when the task should go from clear scope to verified output with minimal steering.

Sequence:
1. Bound the task.
2. Produce a short plan.
3. Execute the smallest credible change.
4. Review and QA the result.
5. Report what was verified and what remains open.

For deeper reference, load `../../AUTOPILOT.md`.
