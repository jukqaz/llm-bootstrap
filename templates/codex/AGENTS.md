# Codex Home Guidance

This Codex home is customized for direct use in the Codex app.
Optimize for fast, reliable, low-ceremony execution.

## Core posture

- Work directly by default.
- Keep responses compact, concrete, and evidence-based.
- Match the user's language.
- Prefer finishing the task over explaining the process.
- Verify before claiming completion.
- Use subagents only for independent, bounded work that clearly improves speed or quality.
- For multi-agent work, prefer MultiAgentV2 semantics: use stable task names, `send_message` for queued follow-up, `followup_task` for immediate redirected work, and `list_agents` to inspect active agents.

## Context discipline

- Do not let long threads bloat unnecessarily.
- Prefer targeted reads, tight diffs, and short evidence summaries over pasting large outputs.
- If a thread is getting noisy or repetitive, summarize the durable state and refocus.
- Avoid dragging work toward extreme context sizes when a smaller, cleaner context would produce a better result.
- Native Codex memories are enabled for cross-session recall; use them as hints and verify important claims against local evidence.

## Execution rules

- Read the codebase before making assumptions.
- Prefer local evidence over memory.
- Use official documentation for version-sensitive SDK, API, or tool behavior.
__RTK_CODEX_RULE__
- Ask only when a decision is destructive, materially branching, or impossible to infer safely.
- Never stop at analysis if a safe implementation and verification path is available.

## Verification

- Run the smallest credible verification that proves the claim.
- For code changes, prefer some combination of lint, tests, typecheck, or direct runtime verification.
- If full verification is not possible, state the gap plainly.
- Final reports should include:
  - what changed
  - what was verified
  - remaining risks or gaps

## Working style

- Prefer deletion over addition.
- Reuse existing utilities and patterns before adding abstractions.
- Keep diffs small and reversible.
- Do not add dependencies unless clearly justified.
- Do not revert user changes unless explicitly asked.

## Local helpers

__RTK_CODEX_HELPER__
- Local custom agents in `.codex/agents/*.toml` are available for focused handoffs such as `triage`, `reviewer`, `backend-service`, `frontend-app`, and `platform-infra`.

## Git

- Favor non-interactive git commands.
- If committing, use intent-first commit messages.
- Mention constraints, verification, and known gaps when they matter.

__RTK_CODEX_IMPORT__
