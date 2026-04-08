# Contributing

Thanks for contributing.

## Language

English is the default language for repository-facing documentation.
Korean is also accepted for issues, discussions, and documentation follow-up.

## Local checks

Run the full local gate before opening a pull request:

```bash
bash -n install.sh
bash -n uninstall.sh
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
```

## Scope rules

- keep changes small and reversible
- prefer deletion over addition
- do not commit secrets or local generated state
- do not widen the baseline MCP set without clear justification
- preserve the Codex -> Gemini -> Claude Code support order

## Pull requests

Please include:

- what changed
- how it was verified
- any remaining gaps or tradeoffs
