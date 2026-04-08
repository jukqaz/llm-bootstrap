#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)"
BUNDLED_BIN="$SCRIPT_DIR/llm-bootstrap"

log() {
  printf '[llm-bootstrap] %s\n' "$*"
}

fail() {
  printf '[llm-bootstrap] %s\n' "$*" >&2
  exit 1
}

if [[ -x "$BUNDLED_BIN" ]]; then
  exec "$BUNDLED_BIN" install "$@"
fi

if ! command -v cargo >/dev/null 2>&1; then
  command -v brew >/dev/null 2>&1 || fail "cargo not found and Homebrew is unavailable"
  log "cargo not found; installing rust toolchain with Homebrew"
  brew install rust
fi

if [[ ! -f "$SCRIPT_DIR/Cargo.toml" ]]; then
  fail "Cargo.toml not found and bundled llm-bootstrap binary is unavailable"
fi

exec cargo run --quiet --manifest-path "$SCRIPT_DIR/Cargo.toml" -- install "$@"
