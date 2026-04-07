# Investigate

Use this lane when the task is a bug, regression, or unclear runtime failure.

## Rules

1. Do not patch blindly before naming the failure path.
2. Separate environment, config, and code causes.
3. Prefer a reproduction and a narrowed code path before editing.
4. If the same fix fails repeatedly, stop and restate the root issue.

## Output contract

- `Symptom:` exact failure
- `Boundary:` where the failure enters
- `Likely cause:` environment, config, or code
- `Next proof:` smallest command or repro step
