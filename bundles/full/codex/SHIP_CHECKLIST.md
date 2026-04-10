# Ship Checklist

- Re-read the changed files and confirm the diff matches the requested scope.
- Run the smallest credible verification set for runtime, types, lint, and tests.
- Run `llm-bootstrap internal gate check --target-phase ship --json` when task-state is active.
- Check documentation or examples when external behavior or commands changed.
- Call out known gaps, skipped checks, or environment-dependent risks in the final report.
- Avoid broad follow-up work unless it is required to keep the change safe.
