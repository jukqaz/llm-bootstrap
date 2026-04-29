# Reference Coverage

> Product scope is defined by [product-goal.md](product-goal.md),
> monorepo boundaries are defined by [monorepo-boundary.md](monorepo-boundary.md),
> and the source catalog schema lives in
> [../catalog/sources/README.md](../catalog/sources/README.md).

This document defines what it means to "cover all the reference repos" for
`StackPilot`.

Here, `coverage` does not mean copying every repo.

- important sources should be tracked in the catalog
- each source should land in exactly one boundary bucket
- adoption and explicit non-adoption should both be recorded

This is a triage document, not a feature-wishlist document.

## Coverage Rules

Every tracked source should end in one of these states:

- `implemented`
  - already reflected in repo structure, renderers, or install flows
- `partial`
  - some ideas are absorbed, but full parity is not a goal
- `reference-only`
  - worth tracking, but intentionally not installed or copied
- `retired`
  - historical signal only, not an active adoption target

Every tracked source also belongs to one of these layers:

- `core`
  - directly shapes the bootstrap baseline
- `addon`
  - opt-in capability layer such as `stackpilot-dev-kit`
- `advanced`
  - higher-level runtime, automation, or remote-execution reference
- `archive-watch`
  - archived or replaced material

## Current Coverage

### 1. Official provider sources

| id | state | layer | current decision |
| --- | --- | --- | --- |
| `openai_codex` | `implemented` | `core` | Primary source for the Codex renderer and home baseline. |
| `gemini_cli` | `implemented` | `core` | Primary source for Gemini extension surfaces and settings merge behavior. |
| `claude_code` | `implemented` | `core` | Primary source for Claude user-scope skills and MCP registration. |

### 2. Workflow and harness references

| id | state | layer | current decision |
| --- | --- | --- | --- |
| `gstack` | `partial` | `addon` | Explicit plan or review loops matter, but full parity is not the target. |
| `oh_my_codex` | `partial` | `addon` | Entry-point naming, skill packs, and mode aliases are the useful parts. |
| `github_agentic_workflows` | `partial` | `advanced` | Workflow-contract and repo-automation ideas matter more than runtime parity. |
| `spec_kit` | `partial` | `addon` | Spec-first and task-state ideas are useful, but not as core bootstrap defaults. |
| `bmad_method` | `partial` | `addon` | Planning and role decomposition remain lightweight references only. |
| `superclaude` | `partial` | `addon` | Claude skill and catalog patterns matter; a cloned runtime does not. |
| `openclaw` | `partial` | `advanced` | Company or inbox control-plane thinking matters; gateway runtime does not. |
| `openai_agentkit` | `reference-only` | `advanced` | Tracked for orchestration ideas only. |
| `n8n_ai_workflows` | `reference-only` | `advanced` | Tracked for external automation lanes, not for bootstrap install. |

### 3. Tools and runtime references

| id | state | layer | current decision |
| --- | --- | --- | --- |
| `rtk` | `implemented` | `core` | Near-core shell output compression and direct install target. |
| `cline` | `partial` | `addon` | Browser validation and tool-onboarding UX are the useful parts. |
| `roo_code` | `partial` | `addon` | Mode naming and packaging patterns are worth borrowing. |
| `continue` | `partial` | `advanced` | Repo automation contracts matter; default bootstrap generation does not. |
| `aider` | `partial` | `addon` | Thin precision loops and git ergonomics are the key takeaways. |
| `repomix` | `reference-only` | `advanced` | Useful for large-repo ingest lanes, not as a default dependency. |
| `gitingest` | `reference-only` | `advanced` | Useful for fast remote-repo triage, not as a default dependency. |
| `coding_context_cli` | `partial` | `addon` | Rules-vs-task separation fits the current renderer direction. |
| `mcpm` | `reference-only` | `advanced` | Useful for MCP profile-management ideas only. |
| `goose` | `reference-only` | `advanced` | A local runtime reference, not a core bootstrap dependency. |
| `github_copilot_cli` | `reference-only` | `advanced` | A GitHub-native agent CLI reference, not a universal baseline. |
| `openhands` | `reference-only` | `advanced` | A heavy remote platform reference, not a local baseline target. |
| `crush` | `reference-only` | `advanced` | A maintained terminal runtime reference, not a bootstrap replacement. |
| `opencode` | `retired` | `archive-watch` | Archived source kept for historical context; follow-on tracking moves to `crush`. |
| `caveman` | `reference-only` | `addon` | Keep as a terse-output reference for future common capabilities, not as a separate install pack. |
| `agentapi` | `reference-only` | `advanced` | Tracked as a universal adapter reference only. |
| `mini_swe_agent` | `reference-only` | `advanced` | Tracked for minimal-loop lessons only. |
| `kelos` | `reference-only` | `advanced` | Tracked for remote sandbox orchestration ideas only. |
| `mcp_universe` | `reference-only` | `advanced` | Tracked for MCP-heavy evaluation patterns only. |

## What Was Missing

This pass adds the major GitHub sources that were already mentioned in repo
docs but were missing from the catalog:

- `caveman`
- `gitingest`
- `github_copilot_cli`
- `openhands`
- `crush`
- `opencode`

That closes the main gap between the written reference docs and the structured
catalog.

## Rule for Future Additions

From here on, a new source should only be added if it changes one of these:

1. product boundary
2. core versus addon versus reference-only classification
3. implementation decisions backed by official docs or a primary README

If it does not cross one of those thresholds, it should stay out of the
catalog.

## Bottom Line

For this repo, "cover all the repos" means:

- track the important sources
- classify them cleanly
- keep core, addon, advanced, and retired-source boundaries explicit

It does not mean cloning every runtime into `StackPilot`.
