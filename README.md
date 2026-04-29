# StackPilot

`StackPilot` is not a tool that forces `Codex`, `Gemini`, and optional
`Claude Code` into one common UX. It is a bootstrap umbrella that installs,
updates, verifies, and restores provider-native kits.

This repository stays a monorepo, but the product surface is split into
`codex-kit`, `gemini-kit`, and `claude-kit`. Workflow and company capabilities
are addon contracts rendered on top of those kits, not the core product
contract.

This repository configures user-scope LLM tooling on macOS without touching
provider auth tokens or project-level files. It applies a small, reproducible
baseline for MCP wiring, workflow docs, native skills or commands, and RTK
integration.

## Install

Current release: `v0.3.3`

Default path: run the wizard first.

```bash
curl -fsSL https://github.com/jukqaz/stack-pilot/releases/latest/download/install-release.sh | bash
```

Release assets:
- [GitHub Releases](https://github.com/jukqaz/stack-pilot/releases)

Direct non-interactive install is still available:

```bash
curl -fsSL https://github.com/jukqaz/stack-pilot/releases/latest/download/install-release.sh | bash -s -- --providers codex,gemini
```

## Documentation

Start here:

- [docs/README.md](docs/README.md)
- [docs/README.ko.md](docs/README.ko.md)
- [docs/product-goal.md](docs/product-goal.md)
- [docs/monorepo-boundary.md](docs/monorepo-boundary.md)
- [docs/provider-native-kit-strategy.md](docs/provider-native-kit-strategy.md)
- [docs/provider-config-ownership.md](docs/provider-config-ownership.md)

Reference data:
- [catalog/sources/README.md](catalog/sources/README.md)
- [catalog/sources/index.toml](catalog/sources/index.toml)
- [docs/reference-coverage.md](docs/reference-coverage.md)

## Scope

`StackPilot` manages only user-home state.

- It does not create or modify repo-level bootstrap files in application repos.
- It does not manage provider login state or personal auth tokens.
- It always backs up managed files before writing or removing them.
- It supports `merge` and `replace`, with `merge` as the default preserving
  provider and user preferences.
- It uses official tool init paths when they exist, especially for RTK.

The current provider priority is:

1. `Codex`
2. `Gemini`
3. `Claude Code`

The default provider set from `bootstrap.toml` is currently:

- `codex`
- `gemini`

`claude` is supported, but opt-in unless selected explicitly.

## Provider Layout

The default baseline is intentionally small.

- Always on:
  - `chrome-devtools`
- Enabled only when env exists:
  - `context7` if `CONTEXT7_API_KEY` is set
  - `exa` if `EXA_API_KEY` is set

| Provider | baseline kit | addon surface | install target |
| --- | --- | --- | --- |
| `Codex` | `codex-kit` | `stackpilot-dev-kit` plugin | `~/.codex/config.toml`, `AGENTS.md`, `agents/*.toml`, plugin skills |
| `Gemini` | `gemini-kit` | `stackpilot-dev` extension | `~/.gemini/settings.json`, `GEMINI.md`, extension commands |
| `Claude Code` | `claude-kit` | native skills/subagents | `~/.claude/CLAUDE.md`, `agents/*.md`, official MCP, skills |

The same packs render through provider-native surfaces. StackPilot does not copy
the Codex plugin model into Gemini or Claude.

This repository does not ship project-specific MCP such as payment, internal,
or app-specific tools. In `merge` mode, unmanaged MCP already present in a
user's local home stays intact.

## Safety model

- backups are created before `install`, `replace`, `restore`, and `uninstall`
- `merge` preserves unmanaged assets
- `replace` resets managed assets and removes known old `omx`, `omc`, `omg`,
  and `oh-my-*` user-level artifacts after backing them up
- `replace` and `uninstall` also remove known old `launchctl` env keys and
  old keys from `~/.zshrc.d/stackpilot-env.zsh`
- `restore` replays a selected backup after creating a fresh backup first
- env-gated MCP stay disabled until the required env is available

## Public repository policy

This repository is safe to publish as long as these rules remain true:

- no secrets are committed
- generated local state stays in user home, not in the repo
- examples use env variable names, not live values
- tests use generic unmanaged MCP fixture names

The wizard can persist keys for both GUI and CLI use:

- GUI apps via `launchctl setenv`
- CLI shells via `~/.zshrc.d/stackpilot-env.zsh`

The repo never stores the actual secret values.

## Provider surfaces

`Codex`
- agent TOML files
- local plugin skills
- workflow artifacts

`Gemini`
- extension agents
- extension custom commands in `commands/*.toml`
- workflow artifacts

`Claude Code`
- user-scope skills in `skills/*/SKILL.md`
- user-scope workflow artifacts
- lightweight subagent docs

## RTK

RTK is enabled by default but can be disabled.

- Codex: `rtk init -g --codex`
- Gemini: `rtk init -g --gemini --auto-patch`
- Claude: `rtk init -g --auto-patch`

Disable RTK for any command with:

```bash
cargo run -- install --without-rtk
```

## Install usage

Fastest path on macOS or Linux:

```bash
curl -fsSL https://github.com/jukqaz/stack-pilot/releases/latest/download/install-release.sh | bash
```

Recommended for end users: use the release archive and run the bundled binary or
wrapper scripts. This path does not require Rust.

1. Download the latest archive from
   [GitHub Releases](https://github.com/jukqaz/stack-pilot/releases).
2. Extract it.
3. Run either:

```bash
./stack-pilot install --providers codex,gemini
```

The same binary is also shipped as the short `sp` alias.

or:

```bash
./install.sh --providers codex,gemini
```

To pin a specific release with the curl installer:

```bash
curl -fsSL https://github.com/jukqaz/stack-pilot/releases/latest/download/install-release.sh | \
  STACKPILOT_VERSION=v0.3.3 bash -s -- --providers codex,gemini
```

For source-based development, clone the repo and run from source:

```bash
git clone https://github.com/jukqaz/stack-pilot.git
cd stack-pilot
./install.sh
```

Common install examples:

```bash
cargo run -- install --providers codex
cargo run -- install --providers gemini
cargo run -- install --providers claude
cargo run -- install --providers codex,gemini
cargo run -- baseline --providers codex,gemini
cargo run -- install --providers codex,gemini,claude
cargo run -- install --providers codex,gemini --preset light
cargo run -- sync --providers codex,gemini --preset full
cargo run -- install --providers codex,gemini,claude --preset full
cargo run -- install --providers codex,gemini,claude --preset orchestrator
cargo run -- probe --providers codex,gemini,claude --preset normal
cargo run -- probe --providers codex --preset normal --optimize
```

## Command boundary

Default user-facing core commands:

- `baseline`
- `install`
- `sync`
- `restore`
- `backups`
- `uninstall`
- `doctor`
- `probe`
- `wizard`

Addon command:

- `record`

Hidden internal lanes:

- `internal ...`
- `task-state ...`

The orchestrator lane and workflow gate commands stay available, but they are
not part of the bootstrap core contract:

```bash
stack-pilot internal task-state begin --title "Review auth flow" --providers codex,gemini,claude --preset orchestrator --phase execute
stack-pilot internal gate check --target-phase plan|execute|review|qa|ship --json
stack-pilot internal task-state advance --summary "review gate is blocked on the flaky fixture repro" --checkpoint "resume from the oauth fixture repro and inspect the failing trace"
stack-pilot internal task-state advance --complete spec,plan,ownership,handoff,review,qa,verify
stack-pilot internal task-state advance --increment-attempt --failure "verification still failing"
stack-pilot internal task-state advance --investigation-note "isolated flaky fixture and captured failing trace"
stack-pilot internal gate apply --target-phase ship --json
```

`doctor --json` now exposes both the requested preset state and the last
installed home state through `installed_preset`, `installed_packs`,
`installed_record_surface`, `requested_record_surface`, and `state_mismatch`.
That makes preset drift visible before you reinstall. Provider runtime checks
now also distinguish shared command prerequisites from provider runtime
requirements:

- Codex: `codex` CLI or `/Applications/Codex.app`
- Gemini: `gemini` CLI
- Claude Code: `claude` CLI

`probe --optimize` keeps the default probe cheap, then adds high-cost optimized
runtime checks where they matter. Today that means the Codex `gpt-5.5` 1M agent
override path and Claude `opus[1m]` / `sonnet[1m]` aliases.

## Preset menus

Like the `oh-my` style set menus, `StackPilot` now exposes user-facing
presets. The internal source of truth remains `pack`, and presets are only
aliases over pack groups.

- `light`
  - `delivery-pack`
- `normal`
  - `delivery-pack`, `incident-pack`
- `full`
  - `delivery-pack`, `incident-pack`, `founder-pack`, `ops-pack`
- `orchestrator`
  - `delivery-pack`, `incident-pack`, `team-pack`
- `company`
  - `founder-pack`, `ops-pack`
- `review-automation`
  - `review-automation-pack`
- `all-in-one`
  - `delivery-pack`, `incident-pack`, `team-pack`, `founder-pack`, `ops-pack`, `review-automation-pack`

`company` and `full` now render actual company-operation assets into the
provider-native surfaces, not just metadata.

- `RALPH_PLAN.md`
- `FOUNDER_LOOP.md`
- `OPERATING_REVIEW.md`
- `OPERATING_RECORDS.md`
- `CONNECTORS.md`
- `AUTOMATIONS.md`
- `ENTRYPOINTS.md`
- Codex skills, Gemini commands, and Claude skills for the company lanes
- `record-work` Codex/Claude skill and Gemini command

`all-in-one` is the strongest preset for users who want an `oh-my` style
all-in-one surface. It turns on development, multi-agent, company, and review
automation packs together.

Examples:

```bash
cargo run -- install --providers codex,gemini --preset normal
cargo run -- install --providers codex,gemini,claude --preset full
cargo run -- install --providers codex,gemini,claude --preset orchestrator
cargo run -- install --providers codex,gemini,claude --preset all-in-one
cargo run -- doctor --providers codex,gemini --preset company --json
```

Addon record examples:

```bash
cargo run -- record --type project --title "MVP scope" --next-action "create first issue"
cargo run -- internal task-state begin --title "Build auth flow" --phase execute --owner codex --summary "Auth flow is wired and waiting on review." --checkpoint "Resume from the oauth fixture repro and capture the failing output." --next-action "capture resumable record"
cargo run -- record --type task --title "Build auth flow" --from-task-state
cargo run -- record --type task --title "Build auth flow" --surface both --github-repo owner/repo
```

If you need exact control, continue to use `--packs delivery-pack,incident-pack`.
Do not combine `--preset` and `--packs`.

### Preset capability mapping

Each preset now resolves as a concrete
`pack -> connectors -> connector apps -> MCP -> provider surface`
composition, not just a document bundle.

- `light`
  - packs: `delivery-pack`
  - connector apps: `github`, `linear`
  - MCP: `chrome-devtools`, `context7`
  - surfaces:
    - Codex: `stackpilot-dev-kit`, `delivery-skills`
    - Gemini: `stackpilot-dev`, `delivery-commands`
    - Claude: `claude-skills`, `delivery-skills`
- `normal`
  - packs: `delivery-pack`, `incident-pack`
  - connector apps: `github`, `linear`
  - MCP: `chrome-devtools`, `context7`
  - surfaces:
    - Codex: `delivery-skills`, `incident-skills`
    - Gemini: `delivery-commands`, `incident-commands`
    - Claude: `delivery-skills`, `incident-skills`
- `full`
  - packs: `delivery-pack`, `incident-pack`, `founder-pack`, `ops-pack`
  - connector apps: `github`, `linear`, `gmail`, `calendar`, `drive`, `figma`, `stitch`
  - MCP: `chrome-devtools`, `context7`, `exa`
  - surfaces:
    - Codex: development and company skills
    - Gemini: development and company commands
    - Claude: development and company skills
- `orchestrator`
  - packs: `delivery-pack`, `incident-pack`, `team-pack`
  - connector apps: `github`, `linear`
  - MCP: `chrome-devtools`, `context7`
  - surfaces:
    - Codex: delivery, incident, and team skills
    - Gemini: delivery, incident, and team commands
    - Claude: delivery, incident, and team skills
- `company`
  - packs: `founder-pack`, `ops-pack`
  - connector apps: `linear`, `gmail`, `calendar`, `drive`, `figma`, `stitch`
  - MCP: `exa`
  - surfaces:
    - Codex: company skills
    - Gemini: company commands
    - Claude: company skills
- `review-automation`
  - packs: `review-automation-pack`
  - connector apps: `github`, `linear`
  - MCP: `chrome-devtools`, `context7`
  - automations: `pr-review-gate`, `release-readiness-gate`
  - surfaces:
    - Codex: `review-automation-skills`
    - Gemini: `review-automation-commands`
    - Claude: `review-automation-skills`
- `all-in-one`
  - packs: `delivery-pack`, `incident-pack`, `team-pack`, `founder-pack`, `ops-pack`, `review-automation-pack`
  - connector apps: `github`, `linear`, `gmail`, `calendar`, `drive`, `figma`, `stitch`
  - MCP: `chrome-devtools`, `context7`, `exa`
  - automations: `pr-review-gate`, `release-readiness-gate`
  - surfaces:
    - Codex: delivery, incident, team, company, and review automation skills
    - Gemini: delivery, incident, team, company, and review automation commands
    - Claude: delivery, incident, team, company, and review automation skills

`doctor --json` exposes the same pack mapping directly. It now also records the
installed preset state per provider, including connectors, automations,
surfaces, and pack-projected managed paths.

Runtime boundaries:

- app connector auth is owned by the provider runtime and reported as `runtime-managed`
- runtime scheduler automation contracts are rendered into installed state, while recurring scheduler registration remains runtime-managed
- repo automation contracts are rendered into installed state, while repository workflow and branch protection registration remain repo-managed

`doctor --json` now also exposes runtime handoff hints for active connectors and
automations:

- connectors: `runtime_owner`, `verification_mode`, `connection_status`, `next_step`
- automations: `lane`, `scheduler_owner`, `registration_status`, `next_step`
- runtime queue: `runtime_handoff.connector_queue`, `runtime_handoff.automation_queue`, `runtime_handoff.next_steps`
- repo automation queue: `runtime_handoff.repo_automation_queue`, `runtime_handoff.pending_repo_registration_count`
- records: `active_record_templates`, `record_templates`, `record_readiness`

Optional repo automation scaffolding:

```bash
cargo run -- internal repo-automation scaffold --repo-root /path/to/repo
cargo run -- internal repo-automation scaffold --repo-root /path/to/repo --pr-required-check check --release-required-check "check,pr-review-gate / gate"
```

This writes `.github/workflows/pr-review-gate.yml`,
`.github/workflows/release-readiness-gate.yml`,
`.github/stackpilot/BRANCH_PROTECTION.md`, and
`.github/PULL_REQUEST_TEMPLATE.md` into the target repository without making
repo-level workflow generation part of the default home bootstrap path.

Mode examples:

```bash
cargo run -- install --providers codex --mode replace
cargo run -- install --providers gemini --mode merge --without-rtk
cargo run -- install --providers codex,gemini --mode replace --dry-run
```

Status and removal:

```bash
cargo run -- doctor --providers codex,gemini,claude --json
cargo run -- uninstall --providers codex
cargo run -- uninstall --providers codex,gemini --dry-run
```

## Wizard

Run:

```bash
cargo run -- wizard
```

The wizard asks for:

- target providers
- preset
- record surface
- `merge` or `replace`
- RTK on or off
- `EXA_API_KEY`
- `CONTEXT7_API_KEY`
- where to persist keys
  - GUI apps
  - CLI shells
- whether to run `install` and `doctor` immediately

Wizard env reuse order:

1. current process env
2. `launchctl getenv`
3. managed CLI env file

## Modes

`merge`
- preserves unmanaged MCP
- refreshes bootstrap-managed files

`replace`
- removes managed bootstrap files first
- keeps only the current baseline MCP set
- preserves known auth or session state where supported
- removes known old `omx`, `omc`, `omg`, and `oh-my-*` user-level artifacts
  after backing them up
- removes known old env keys from `launchctl` and the managed CLI env file

## Backup and restore

Every `install`, `replace`, and `uninstall` creates provider-level backups first.
Home-level old-tool cleanup creates `~/.stackpilot-legacy-backups/*` backups.

Restore the latest backup for selected providers:

```bash
cargo run -- restore --providers codex,gemini,claude
```

Restore a specific backup directory:

```bash
cargo run -- restore --providers codex --backup stackpilot-1712550000
```

List available backups:

```bash
cargo run -- backups --providers codex,gemini,claude
cargo run -- restore --providers codex,gemini --list --json
```

Preview a restore without changing files:

```bash
cargo run -- restore --providers codex,gemini --backup stackpilot-1712550000 --dry-run
```

The restore command first backs up the current state again, then restores the
selected backup for bootstrap-managed files.

## Repository layout

- [bootstrap.toml](bootstrap.toml): shared manifest
- [src/main.rs](src/main.rs): CLI orchestration
- [src/providers/](src/providers): provider installers
- [src/layout/](src/layout): core/addon layout rules
- [src/repo_assets.rs](src/repo_assets.rs): repo addon asset roots
- [templates/](templates): bootstrap core renderer assets
- [addons/stackpilot-dev-kit/](addons/stackpilot-dev-kit): addon bundle assets for Codex, Gemini, and Claude

## Verification

Local verification:

```bash
bash -n install.sh
bash -n uninstall.sh
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
```

## Release model

CI runs on pull requests and on pushes to `main`.

Tagged releases publish GitHub Release assets on tags that match `v*`.
When the patch number would exceed `10`, roll the next release to the next
minor version and reset patch to `0` instead of publishing `x.y.11+`.

## References

This repository borrows ideas, not full runtime behavior, from:

- `oh-my-codex`
- `oh-my-gemini-cli`
- `oh-my-claudecode`
- `gstack`
- `OpenHarness`
- `oh-my-openagent`

## License

This repository is currently prepared with the MIT license.
