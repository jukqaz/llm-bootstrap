# Migrating from older oh-my or OMC installs

`llm-bootstrap` preserves unmanaged assets in `merge` mode. That is usually the
right default for existing machines, but it also means older harness traces can
stay behind.

## When to use `merge`

Use `merge` when you want to keep your current unmanaged MCP, commands, skills,
or extensions and only layer the bootstrap baseline on top.

```bash
cargo run -- install --providers codex,gemini,claude --mode merge
```

If an older install left conflicting commands, skills, or extensions behind,
either remove those paths manually or switch to `replace`.

## When to use `replace`

Use `replace` when you want a clean baseline reset for the selected providers.

```bash
cargo run -- install --providers codex,gemini,claude --mode replace
```

`replace` does three things:

1. removes bootstrap-managed files for the selected providers
2. keeps only the current baseline MCP set
3. removes known legacy oh-my or OMC artifacts for the selected providers

`replace` still preserves supported auth or session state where the provider
allows it.

## Optional legacy cleanup in `merge`

If you want to stay on `merge` but clean only known legacy traces, run:

```bash
cargo run -- install --providers codex,gemini,claude --mode merge --cleanup legacy
```

This only removes known legacy locations. It does not wipe arbitrary user-owned
assets.

## Safety notes

- Every install creates provider-level backups first.
- You can restore the latest backup with `cargo run -- restore --providers ...`.
- Automatic legacy cleanup is intentionally narrow.
- If you have custom user-owned paths outside the known cleanup list, remove
  them manually after checking the backup.
