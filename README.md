# llm-bootstrap

Developer home bootstrap for `Codex`, `Gemini`, and optional `Claude Code`.

This repository configures user-scope LLM tooling on macOS without touching
provider auth tokens or project-level files. It applies a small, reproducible
baseline for MCP wiring, workflow docs, native skills or commands, and RTK
integration.

## Install

Current release: `v0.2.0`

Default path: run the wizard first.

```bash
curl -fsSL https://github.com/jukqaz/llm-bootstrap/releases/latest/download/install-release.sh | bash
```

Release assets:
- [GitHub Releases](https://github.com/jukqaz/llm-bootstrap/releases)

Direct non-interactive install is still available:

```bash
curl -fsSL https://github.com/jukqaz/llm-bootstrap/releases/latest/download/install-release.sh | bash -s -- --providers codex,gemini
```

## Documentation

English:
- [README.md](README.md)
- [docs/codex-first-blueprint.md](docs/codex-first-blueprint.md)
- [docs/direction-review.md](docs/direction-review.md)
- [docs/business-ops-blueprint.md](docs/business-ops-blueprint.md)
- [docs/dev-company-operating-model.md](docs/dev-company-operating-model.md)
- [docs/external-tool-landscape.md](docs/external-tool-landscape.md)
- [docs/official-best-practices.md](docs/official-best-practices.md)
- [docs/recent-signal-scan.md](docs/recent-signal-scan.md)
- [docs/provider-surface-strategy.md](docs/provider-surface-strategy.md)
- [docs/oh-my-comparison-report.md](docs/oh-my-comparison-report.md)
- [docs/reference-repo-backlog.md](docs/reference-repo-backlog.md)
- [docs/superset-strategy.md](docs/superset-strategy.md)
- [docs/runtime-handoff.md](docs/runtime-handoff.md)
- [docs/runtime-risk-register.md](docs/runtime-risk-register.md)

Korean:
- [README.ko.md](README.ko.md)
- [docs/codex-first-blueprint.ko.md](docs/codex-first-blueprint.ko.md)
- [docs/direction-review.ko.md](docs/direction-review.ko.md)
- [docs/business-ops-blueprint.ko.md](docs/business-ops-blueprint.ko.md)
- [docs/external-tool-landscape.ko.md](docs/external-tool-landscape.ko.md)
- [docs/provider-surface-strategy.ko.md](docs/provider-surface-strategy.ko.md)
- [docs/oh-my-comparison-report.ko.md](docs/oh-my-comparison-report.ko.md)
- [docs/reference-repo-backlog.ko.md](docs/reference-repo-backlog.ko.md)
- [docs/superset-strategy.ko.md](docs/superset-strategy.ko.md)

Reference data:
- [catalog/sources/README.md](catalog/sources/README.md)
- [catalog/sources/index.toml](catalog/sources/index.toml)

## Scope

`llm-bootstrap` manages only user-home state.

- It does not create or modify repo-level bootstrap files in application repos.
- It does not manage provider login state or personal auth tokens.
- It always backs up managed files before writing or removing them.
- It supports `merge` and `replace`, with `merge` as the default.
- It uses official tool init paths when they exist, especially for RTK.

The current provider priority is:

1. `Codex`
2. `Gemini`
3. `Claude Code`

The default provider set from `bootstrap.toml` is currently:

- `codex`
- `gemini`

`claude` is supported, but opt-in unless selected explicitly.

## Default baseline

The default baseline is intentionally small.

- Always on:
  - `chrome-devtools`
- Enabled only when env exists:
  - `context7` if `CONTEXT7_API_KEY` is set
  - `exa` if `EXA_API_KEY` is set
- Codex:
  - local `llm-dev-kit` plugin
  - workflow docs and skills
- Gemini:
  - `llm-bootstrap-dev` extension
  - native custom commands
  - workflow docs and lightweight agent pack
- Claude Code:
  - official MCP registration
  - lightweight subagent docs
  - workflow skill pack

This repository does not ship project-specific MCP such as payment, internal,
or app-specific tools. In `merge` mode, unmanaged MCP already present in a
user's local home stays intact.

## Safety model

- backups are created before `install`, `replace`, `restore`, and `uninstall`
- `merge` preserves unmanaged assets
- `replace` resets managed assets and removes known legacy `omx`, `omc`, `omg`,
  and `oh-my-*` user-level artifacts after backing them up
- `replace` and `uninstall` also remove known legacy `launchctl` env keys and
  legacy keys from `~/.zshrc.d/llm-bootstrap-env.zsh`
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
- CLI shells via `~/.zshrc.d/llm-bootstrap-env.zsh`

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
curl -fsSL https://github.com/jukqaz/llm-bootstrap/releases/latest/download/install-release.sh | bash
```

Recommended for end users: use the release archive and run the bundled binary or
wrapper scripts. This path does not require Rust.

1. Download the latest archive from
   [GitHub Releases](https://github.com/jukqaz/llm-bootstrap/releases).
2. Extract it.
3. Run either:

```bash
./llm-bootstrap install --providers codex,gemini
```

or:

```bash
./install.sh --providers codex,gemini
```

To pin a specific release with the curl installer:

```bash
curl -fsSL https://github.com/jukqaz/llm-bootstrap/releases/latest/download/install-release.sh | \
  LLM_BOOTSTRAP_VERSION=v0.2.0 bash -s -- --providers codex,gemini
```

For source-based development, clone the repo and run from source:

```bash
git clone https://github.com/jukqaz/llm-bootstrap.git
cd llm-bootstrap
./install.sh
```

Common install examples:

```bash
cargo run -- install --providers codex
cargo run -- install --providers gemini
cargo run -- install --providers claude
cargo run -- install --providers codex,gemini
cargo run -- install --providers codex,gemini,claude
cargo run -- install --providers codex,gemini --preset light
cargo run -- install --providers codex,gemini,claude --preset full
```

`doctor --json` now exposes both the requested preset state and the last
installed home state through `installed_preset`, `installed_packs`, and
`state_mismatch`. That makes preset drift visible before you reinstall.

## Preset menus

Like the `oh-my` style set menus, `llm-bootstrap` now exposes user-facing
presets. The internal source of truth remains `pack`, and presets are only
aliases over pack groups.

- `light`
  - `delivery-pack`
- `normal`
  - `delivery-pack`, `incident-pack`
- `full`
  - `delivery-pack`, `incident-pack`, `founder-pack`, `ops-pack`
- `company`
  - `founder-pack`, `ops-pack`

`company` and `full` now render actual company-operation assets into the
provider-native surfaces, not just metadata.

- `RALPH_PLAN.md`
- `FOUNDER_LOOP.md`
- `OPERATING_REVIEW.md`
- `OPERATING_RECORDS.md`
- `CONNECTORS.md`
- `AUTOMATIONS.md`
- Codex skills, Gemini commands, and Claude skills for the company lanes
- `record-work` Codex/Claude skill and Gemini command

Examples:

```bash
cargo run -- install --providers codex,gemini --preset normal
cargo run -- install --providers codex,gemini,claude --preset full
cargo run -- doctor --providers codex,gemini --preset company --json
cargo run -- record --type project --title "MVP scope" --next-action "create first issue"
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
    - Codex: `llm-dev-kit`, `delivery-skills`
    - Gemini: `llm-bootstrap-dev`, `delivery-commands`
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
- `company`
  - packs: `founder-pack`, `ops-pack`
  - connector apps: `linear`, `gmail`, `calendar`, `drive`, `figma`, `stitch`
  - MCP: `exa`
  - surfaces:
    - Codex: company skills
    - Gemini: company commands
    - Claude: company skills

`doctor --json` exposes the same pack mapping directly. It now also records the
installed preset state per provider, including connectors, automations,
surfaces, and pack-projected managed paths.

Runtime boundaries:

- app connector auth is owned by the provider runtime and reported as `runtime-managed`
- automation contracts are rendered into installed state, while recurring scheduler registration remains runtime-managed

`doctor --json` now also exposes runtime handoff hints for active connectors and
automations:

- connectors: `runtime_owner`, `verification_mode`, `connection_status`, `next_step`
- automations: `scheduler_owner`, `registration_status`, `next_step`
- runtime queue: `runtime_handoff.connector_queue`, `runtime_handoff.automation_queue`, `runtime_handoff.next_steps`
- records: `active_record_templates`, `record_templates`, `record_readiness`

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
- removes known legacy `omx`, `omc`, `omg`, and `oh-my-*` user-level artifacts
  after backing them up
- removes known legacy env keys from `launchctl` and the managed CLI env file

## Backup and restore

Every `install`, `replace`, and `uninstall` creates provider-level backups first.
Home-level legacy cleanup creates `~/.llm-bootstrap-legacy-backups/*` backups.

Restore the latest backup for selected providers:

```bash
cargo run -- restore --providers codex,gemini,claude
```

Restore a specific backup directory:

```bash
cargo run -- restore --providers codex --backup llm-bootstrap-1712550000
```

List available backups:

```bash
cargo run -- backups --providers codex,gemini,claude
cargo run -- restore --providers codex,gemini --list --json
```

Preview a restore without changing files:

```bash
cargo run -- restore --providers codex,gemini --backup llm-bootstrap-1712550000 --dry-run
```

The restore command first backs up the current state again, then restores the
selected backup for bootstrap-managed files.

## Repository layout

- [bootstrap.toml](bootstrap.toml): shared manifest
- [src/main.rs](src/main.rs): CLI orchestration
- [src/providers/](src/providers): provider installers
- [templates/codex/](templates/codex): Codex templates
- [templates/gemini/](templates/gemini): Gemini templates
- [templates/claude/](templates/claude): Claude templates
- [plugins/llm-dev-kit/](plugins/llm-dev-kit): Codex plugin bundle

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
