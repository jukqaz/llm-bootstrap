# Workflow

Use this lane when a Gemini session needs more than a single prompt-response loop.

## Preferred sequence

1. Clarify the exact user-visible change.
2. Plan the smallest safe slice.
3. Execute against bounded files only.
4. Review for regressions and missing verification.
5. Ship only after QA evidence is written down.

## Notes

- Prefer extension agents for planning, execution, review, and verification instead of one long mixed prompt.
- Keep the final report short and evidence-based.
