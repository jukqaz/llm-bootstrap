#!/usr/bin/env bash
set -euo pipefail

log() {
  printf '[llm-bootstrap] %s\n' "$*"
}

fail() {
  printf '[llm-bootstrap] %s\n' "$*" >&2
  exit 1
}

if ! command -v cargo >/dev/null 2>&1; then
  command -v brew >/dev/null 2>&1 || fail "cargo not found and Homebrew is unavailable"
  log "cargo not found; installing rust toolchain with Homebrew"
  brew install rust
fi

exec cargo run --quiet -- apply "$@"
