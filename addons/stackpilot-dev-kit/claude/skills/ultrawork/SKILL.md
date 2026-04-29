---
name: ultrawork
description: Run a high-throughput lane for independent work with periodic review gates.
---

# ultrawork

Use this skill only when the task can be split into independent, reversible
shards.

## Flow

1. Ground intent: implementation, investigation, review, QA, docs, or research.
2. Split the work into dependency-aware shards with disjoint file or module
   ownership.
3. Run independent reads, searches, and safe implementation slices in parallel
   where the environment supports it.
4. Keep shared contracts, generated files, and risky migrations on the critical
   lane.
5. Run a review or verification gate after each batch, not only at the end.
6. Stop when shard ownership overlaps, verification fails repeatedly, or the
   next decision is destructive.

## Report

```markdown
## Mode
- ultrawork

## Shard Plan
| Shard | Scope | Owner | Risk | Verify Gate |
| --- | --- | --- | --- | --- |

## Throughput Log
- ...

## Quality Gate
- ...

## Remaining Batch
- ...
```

## Rules

- Do not split work just to create activity.
- Do not let two agents edit the same file set.
- Do not merge an unverified critical shard.
- Prefer a blocked report over blind retry loops.
