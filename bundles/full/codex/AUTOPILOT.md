# Autopilot

Use this lane when the task should run end to end with minimal operator steering.

## Entry criteria

- The outcome is clear enough to build.
- The task spans planning, implementation, QA, and validation.
- The next steps are mostly reversible and low-risk.

## Execution contract

1. Clarify only the missing constraints that materially change the build.
2. Write or refresh the plan before broad execution.
3. Execute the smallest parallelizable slices.
4. Run QA until the critical path is proven or a repeated failure indicates a fundamental issue.
5. Finish with review, security, and acceptance validation.

## Stop conditions

- The same failure repeats three times.
- The remaining decision is destructive or preference-driven.
- Validation still fails after targeted fixes.

## Output contract

- What was built
- What was verified
- What still needs a human decision
