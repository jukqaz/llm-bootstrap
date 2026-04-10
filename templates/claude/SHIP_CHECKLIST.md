# Ship Checklist

- Confirm the diff still matches the requested scope.
- Run the smallest credible verification set.
- Run `llm-bootstrap internal gate check --target-phase ship --json` when task-state is active.
- Recheck docs when commands, SDKs, or MCP behavior changed.
- State skipped checks and environment-dependent risks explicitly.
