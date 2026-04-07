# QA Browser

Use this skill when a change needs repeatable browser verification beyond a quick live inspection.

## When to use

- the task touches multi-step UI flows
- you need deterministic notes about the exact UI path, console state, and network behavior
- `chrome-devtools` inspection is necessary to validate the user-visible state

## Workflow

1. Start with the smallest target page or flow.
2. Use `chrome-devtools` for live state, console, network, and DOM checks.
3. Capture the exact path exercised, what passed, and where verification remains partial.
