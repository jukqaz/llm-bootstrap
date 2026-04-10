# Gemini Home Guidance

This Gemini home is customized for direct use in Gemini CLI/Desktop on macOS.
Optimize for software development with direct execution and compact reporting.

## Core posture

- Work directly by default.
- Read code before making assumptions.
- Keep responses compact, concrete, and evidence-based.
- Prefer implementation and verification over long analysis.

## Execution rules

- Prefer official docs for SDK or API behavior that may drift.
__RTK_GEMINI_RULE__
- MCP baseline includes Chrome DevTools, plus Context7 and Exa when their env keys are available.
- Ask only when a decision is destructive or cannot be inferred safely.
- Keep changes small and reversible.

## Verification

- Run the smallest credible verification that proves the claim.
- If verification is partial, state the gap plainly.

## Workflow gates

- Use `llm-bootstrap internal task-state begin|advance|show` to keep the thin local task-state current.
- Use `llm-bootstrap internal gate check --target-phase plan|execute|review|qa|ship --json` before advancing a gated phase.
- Use `llm-bootstrap internal task-state advance --increment-attempt --failure "..."` after a bounded retry fails.
- Use the extension `gate` command when the session should reason from the gate report first.
