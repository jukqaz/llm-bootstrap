# Provider Config Ownership

`Codex`, `Gemini`, and `Claude Code` change at different speeds and use
different configuration shapes. `StackPilot` must not treat every provider
settings file as a single blob that it owns.

This document defines the ownership model used to prevent provider updates from
turning into repeated local config drift.

## Principles

- `merge` preserves user and provider-runtime preferences.
- `replace` is an explicit rebaseline operation.
- Auth, sessions, history, and credentials are not bootstrap-owned.
- MCP scripts, bootstrap docs, plugins, extensions, skills, and commands are
  bootstrap-owned.
- Unknown settings added by newer provider releases are preserved by default.

## Ownership

`runtime-owned`:

- login, auth, OAuth, accounts, credential stores
- sessions, conversation history, history files
- provider-generated marketplace, project, cache, or telemetry state
- provider UI, output, and approval-mode preferences

`bootstrap-owned`:

- managed MCP scripts and managed MCP registrations
- `stackpilot-state.json`
- workflow docs
- Codex plugin, Gemini extension, and Claude skills
- RTK hooks and RTK docs
- known old-tool artifact cleanup

`user-owned`:

- provider UI preferences
- project trust, manual MCP servers, custom hooks

## Provider Rules

### Codex

Even in `merge`, bootstrap owns the settings needed to keep the current
baseline aligned: `model`, `model_reasoning_effort`,
`plan_mode_reasoning_effort`, and native memories. Existing verbosity, history,
tools, most feature toggles, project trust, and marketplace settings win.

Bootstrap manages `AGENTS.md`, agents, managed MCP, plugin surfaces, and
workflow docs.

For `replace`, `merge`, or a fresh install, the Codex baseline follows this
model split:

- default model: `gpt-5.5`
- default reasoning effort: `xhigh`
- Fast mode is explicitly enabled with `[features].fast_mode = true` and
  `service_tier = "fast"` on fresh installs and `replace`; `merge` preserves an
  explicit user `service_tier` opt-down.
- custom agents use role-specific model tiers:
  - high-risk reasoning and review: `planner`, `architect`, `reviewer`,
    `security-reviewer`, and `platform-infra` use `gpt-5.5` with `xhigh`
  - implementation lanes: `executor`, `backend-service`, `frontend-app`,
    `mobile-app`, and `test-engineer` use `gpt-5.5` with `high`
  - repeatable support lanes: `triage`, `explore`, `docs-researcher`,
    `git-master`, and `verifier` use `gpt-5.4-mini` with role-specific
    `low`/`medium`/`high` effort. The base `gpt-5.4` model is not used in the
    default agent templates.
- `*-1m` long-context agents use `gpt-5.5` with `xhigh`,
  `model_context_window = 1000000`, and
  `model_auto_compact_token_limit = 900000`. The local CLI smoke accepts this
  combination, but the local Codex catalog still reports `gpt-5.5` as
  `272000`, so the global default stays on `gpt-5.5` without forcing 1M
  metadata
- native memory is enabled through `[features].memories = true`,
  `[memories].generate_memories = true`, `[memories].use_memories = true`, and
  `[memories].disable_on_external_context = false`
- the old `gpt-5-codex` alias is not used in new Codex model surfaces
- `deep-init`, `team`, and `ultrawork` are provided as Codex plugin skills

### Gemini

In `merge`, UI display choices, output format, and similar user preferences
mostly win. The following settings are owned by the Gemini bootstrap stability
baseline:

- `general.defaultApprovalMode = "plan"`
- `general.checkpointing.enabled = true`
- `general.plan.enabled = true`
- `general.plan.modelRouting = true`
- `general.retryFetchErrors = true`
- `general.maxAttempts = 10`
- `general.sessionRetention.enabled = false` is kept to prevent automatic chat
  history cleanup
- `general.enableAutoUpdate = false`
- `general.enableAutoUpdateNotification = false`
- `model.name = "auto"` because Gemini CLI model routing changes quickly and the
  bootstrap baseline should not pin a preview model by default
- `hooksConfig.enabled = true`
- `skills.enabled = true`
- `experimental.memoryV2 = true`
- `experimental.autoMemory = true`
- `experimental.contextManagement = true`
- `experimental.modelSteering = true`
- `experimental.topicUpdateNarration = true`

Bootstrap manages managed MCP, extension surfaces, RTK hooks, and workflow docs.
Auth shape is preserved even during `replace`.

Gemini does not get forced per-agent model routing in the same way as Claude or
Codex. The official Gemini CLI extension surface packages prompts, MCP servers,
and custom commands, so `extensions/stackpilot-dev/agents/*.md` are role prompt
assets and execution is driven through extension command contracts such as
`/team`, `/review`, `/qa`, and `/ship`.

### Claude Code

Claude relies more on `CLAUDE.md`, agents, skills, and MCP registration than on
a large settings file.

Bootstrap manages skills, docs, RTK hooks, and official MCP registration.
Conversation history and project logs are left alone.

Even in `merge`, the following values are owned by the Claude baseline
stability policy:

- `model = "opus[1m]"` so Claude uses 1M context where the account and plan
  support it. Claude Code model configuration accepts `opus[1m]` and
  `sonnet[1m]` aliases for extended context.
- file-level `effortLevel` is removed and max reasoning is applied through
  `env.CLAUDE_CODE_EFFORT_LEVEL = "max"` by default
- `autoMemoryEnabled = true`
- `autoUpdatesChannel = "stable"`
- `cleanupPeriodDays = 365`
- `includeGitInstructions = true`
- `awaySummaryEnabled = true`
- `fastModePerSessionOptIn = true`
- Claude subagents use official `~/.claude/agents/*.md` frontmatter. `planner`
  and `reviewer` use `opus[1m]`, `executor` and `verifier` use `sonnet[1m]`, and
  `triage` uses `haiku`.
- `showThinkingSummaries = true`
- `useAutoModeDuringPlan = false`
- `env.CLAUDE_CODE_EXPERIMENTAL_AGENT_TEAMS = "1"`
- `permissions.defaultMode = "auto"`
- sensitive file reads and destructive git/shell commands are appended to
  `permissions.deny`
- git push, curl/wget, ssh, kubectl, and terraform apply are appended to
  `permissions.ask`

In `merge`, explicit user opt-downs for `model`,
`env.CLAUDE_CODE_EFFORT_LEVEL`, and `fastModePerSessionOptIn` are preserved.
Existing `opus[1m]` and `sonnet[1m]` values are kept as explicit user choices,
while the old non-1M `opus` baseline is migrated to `opus[1m]`. `replace`
reapplies the full max baseline.

## Operating Rule

Do not reach for `replace` first when settings look tangled.

1. Run `doctor --json` to separate missing files from state mismatch.
2. Repair missing provider CLIs or auth inside the provider runtime first.
3. Re-render missing surfaces with `install --mode merge`.
4. Use `--mode replace` only when the provider baseline itself should be reset.

With this model, fast provider updates can add runtime settings without forcing
bootstrap to fight those settings on every install.
