---
name: team
description: Coordinate a bounded gstack-style lane with ownership, handoff, review, and verification gates
---

# team

Use this workflow when one agent should coordinate planner, executor, reviewer,
and verifier roles around a shared target.

This is a contract layer over Claude Code's native subagents and skills. Do not
introduce a separate worker runtime unless the user explicitly asks for one.

## Flow

1. `team-plan`: define the shared objective, critical lane, and acceptance
   check.
2. `team-scope`: clarify requirements only when acceptance criteria or
   boundaries are missing.
3. `team-exec`: split only into independent scopes with disjoint ownership.
4. `team-verify`: keep reviewer and verifier outside the write owner path.
5. `team-fix`: run one targeted fix pass for failed gates, then verify again.
6. Merge outcomes into one final evidence report.

## Gate Contract

- Before execution, persist `spec`, `plan`, and `ownership` when
  `parallel-build` is active.
- Before review, QA, or ship, persist `handoff`.
- Before ship, persist `review`, `qa`, and `verify`.
- After a second failed attempt, attach investigation evidence before trying
  another fix.

## Handoff

Before each stage transition, preserve a 10-20 line handoff in the
conversation or operating record:

- decisions made
- options rejected
- risks for the next stage
- touched files
- remaining work

If durable resume is needed, use `stack-pilot internal task-state` or
`record-work`.

Useful gate commands:

```bash
stack-pilot internal task-state advance --complete spec,plan,ownership
stack-pilot internal gate check --target-phase execute --json
stack-pilot internal task-state advance --complete handoff,review,qa,verify
stack-pilot internal gate check --target-phase ship --json
```

## Rules

- Avoid overlapping file ownership.
- Keep the final report on changed files, verification, and remaining risk.
- Stop on destructive branch points or repeated verification failure.
