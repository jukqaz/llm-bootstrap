# Office Hours

Use this lane before implementation when the task is vague, too large, or drifting.

## Forcing questions

1. What exact user-visible outcome should change?
2. What is explicitly out of scope for this pass?
3. Which files, routes, or jobs define the current behavior?
4. What can fail in production even if the change looks locally correct?
5. What evidence is enough to ship this slice?
6. What is the smallest reversible slice we can complete first?

## Output contract

- `Outcome:` one sentence
- `Boundaries:` exact files or systems
- `Risks:` one or two real failure modes
- `Verification:` smallest credible command or runtime check
- `Not now:` one thing intentionally excluded
