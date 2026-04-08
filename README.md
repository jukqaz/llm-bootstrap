# llm-bootstrap

Developer home bootstrap for `Codex`, `Gemini`, and optional `Claude Code`.

This repository configures user-scope LLM tooling on macOS without touching
provider auth tokens or project-level files. It applies a small, reproducible
baseline for MCP wiring, workflow docs, native skills or commands, and RTK
integration.

## Install now

Current release: `v0.1.1`

```bash
curl -fsSL https://raw.githubusercontent.com/jukqaz/llm-bootstrap/main/install-release.sh | bash -s -- --providers codex,gemini
```

Release assets:
- [GitHub Releases](https://github.com/jukqaz/llm-bootstrap/releases)

## Documentation

English:
- [README.md](README.md)
- [docs/codex-first-blueprint.md](docs/codex-first-blueprint.md)
- [docs/legacy-migration.md](docs/legacy-migration.md)

Korean:
- [README.ko.md](README.ko.md)
- [docs/codex-first-blueprint.ko.md](docs/codex-first-blueprint.ko.md)
- [docs/legacy-migration.ko.md](docs/legacy-migration.ko.md)

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
  - user-scope docs, skills, and lightweight agent pack

This repository does not ship project-specific MCP such as payment, internal,
or app-specific tools. In `merge` mode, unmanaged MCP already present in a
user's local home stays intact.

## Safety model

- backups are created before `install`, `replace`, `restore`, and `uninstall`
- `merge` preserves unmanaged assets
- `replace` resets managed assets and removes known legacy traces
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

## Install

Fastest path on macOS or Linux:

```bash
curl -fsSL https://raw.githubusercontent.com/jukqaz/llm-bootstrap/main/install-release.sh | bash -s -- --providers codex,gemini
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
curl -fsSL https://raw.githubusercontent.com/jukqaz/llm-bootstrap/main/install-release.sh | \
  LLM_BOOTSTRAP_VERSION=v0.1.1 bash -s -- --providers codex,gemini
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
```

Mode examples:

```bash
cargo run -- install --providers codex --mode replace
cargo run -- install --providers gemini --mode merge --without-rtk
cargo run -- install --providers codex,gemini --mode replace --dry-run
```

Status and cleanup:

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
- preserves previous unmanaged assets, including old oh-my or OMC traces

If an older install leaves conflicting commands, skills, or extensions behind,
delete those paths manually or use `replace`.

`replace`
- removes managed bootstrap files first
- keeps only the current baseline MCP set
- preserves known auth or session state where supported
- also removes known legacy oh-my or OMC artifacts for the selected providers

Optional legacy cleanup for `merge`:

```bash
cargo run -- install --providers codex,gemini,claude --cleanup legacy
```

This is off by default for `merge`. Use it when you explicitly want to remove
known legacy artifacts from older oh-my or OMC style installs while keeping
normal unmanaged assets intact.

Migration guide:
- [docs/legacy-migration.md](docs/legacy-migration.md)

## Backup and restore

Every `install`, `replace`, and `uninstall` creates provider-level backups first.

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
selected backup for bootstrap-managed files and known legacy cleanup targets.

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
