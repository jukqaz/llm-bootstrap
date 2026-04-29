---
name: team
description: Run a bounded gstack-style team lane with ownership, handoff, review, and verification gates
---

# team

Use this skill when the task is large enough to benefit from a planner,
executor, reviewer, and verifier split.

This is a contract layer, not a separate worker runtime. Use provider-native
agents when they are available, but keep the same owner map, handoff, and gate
rules.

## Flow

1. Run `deep-init` first when repository shape or validation commands are
   unclear.
2. `team-plan`: lock one objective, the critical lane, and the smallest
   acceptance target.
3. `team-scope`: clarify requirements only when acceptance criteria or
   boundaries are missing.
4. `team-exec`: split work only into independent slices with disjoint
   ownership.
5. `team-verify`: keep reviewer and verifier outside the write-owner path.
6. `team-fix`: run one targeted fix pass for failed gates, then verify again.
7. Finish with one merged report: changed files, verification, and residual
   risk.

## Gate Contract

- Before execution, record `spec`, `plan`, and `ownership` when
  `parallel-build` is active.
- Before review, QA, or ship, record `handoff`.
- Before ship, record `review`, `qa`, and `verify`.
- After a second failed attempt, add an investigation note instead of retrying
  blindly.

## Handoff

Before each stage transition, preserve a 10-20 line handoff in the
conversation or operating record:

- decisions made
- options rejected
- risks for the next stage
- touched files
- remaining work

If durable resume is needed, use `record-work` or
`stack-pilot internal task-state`.

Useful gate commands:

```bash
stack-pilot internal task-state advance --complete spec,plan,ownership
stack-pilot internal gate check --target-phase execute --json
stack-pilot internal task-state advance --complete handoff,review,qa,verify
stack-pilot internal gate check --target-phase ship --json
```

## Rules

- Do not fan out when the next step depends on one unresolved detail.
- Do not let two workers edit the same file set.
- Reviewer findings lead the report.
- Verifier proves the critical path; do not pad with broad checks.
- Stop on destructive branch points or repeated verification failure.
