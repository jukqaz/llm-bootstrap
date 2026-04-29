#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)"
BUNDLED_BIN="$SCRIPT_DIR/stack-pilot"

log() {
  printf '[stackpilot] %s\n' "$*"
}

fail() {
  printf '[stackpilot] %s\n' "$*" >&2
  exit 1
}

resolve_toolchain_home() {
  local current_user toolchain_home
  current_user="$(id -un)"

  if [[ -n "${STACKPILOT_TOOLCHAIN_HOME:-}" ]]; then
    printf '%s' "$STACKPILOT_TOOLCHAIN_HOME"
    return
  fi

  if command -v dscl >/dev/null 2>&1; then
    toolchain_home="$(dscl . -read "/Users/${current_user}" NFSHomeDirectory 2>/dev/null | awk '{print $2}')"
  elif command -v getent >/dev/null 2>&1; then
    toolchain_home="$(getent passwd "$current_user" | cut -d: -f6)"
  else
    toolchain_home="$(eval "printf '%s' ~${current_user}")"
  fi

  printf '%s' "$toolchain_home"
}

configure_rust_toolchain_env() {
  local toolchain_home
  toolchain_home="$(resolve_toolchain_home)"

  [[ -n "$toolchain_home" ]] || return

  if [[ -z "${CARGO_HOME:-}" && -d "$toolchain_home/.cargo" ]]; then
    export CARGO_HOME="$toolchain_home/.cargo"
  fi

  if [[ -z "${RUSTUP_HOME:-}" && -d "$toolchain_home/.rustup" ]]; then
    export RUSTUP_HOME="$toolchain_home/.rustup"
  fi
}

resolve_command() {
  if [[ $# -eq 0 ]]; then
    printf 'wizard'
    return
  fi

  case "$1" in
    install|wizard|doctor|restore|backups|uninstall|help|--help|-h)
      printf '%s' "$1"
      ;;
    *)
      printf 'install'
      ;;
  esac
}

COMMAND="$(resolve_command "$@")"
FIRST_ARG="${1-}"

if [[ -x "$BUNDLED_BIN" ]]; then
  if [[ -n "$FIRST_ARG" && ( "$COMMAND" == "$FIRST_ARG" || "$COMMAND" == "--help" || "$COMMAND" == "-h" ) ]]; then
    exec "$BUNDLED_BIN" "$@"
  fi
  exec "$BUNDLED_BIN" "$COMMAND" "$@"
fi

if ! command -v cargo >/dev/null 2>&1; then
  command -v brew >/dev/null 2>&1 || fail "cargo not found and Homebrew is unavailable"
  log "cargo not found; installing rust toolchain with Homebrew"
  brew install rust
fi

if [[ ! -f "$SCRIPT_DIR/Cargo.toml" ]]; then
  fail "Cargo.toml not found and bundled stack-pilot binary is unavailable"
fi

configure_rust_toolchain_env

if [[ $# -gt 0 && ( "$COMMAND" == "$FIRST_ARG" || "$COMMAND" == "--help" || "$COMMAND" == "-h" ) ]]; then
  exec cargo run --quiet --manifest-path "$SCRIPT_DIR/Cargo.toml" --bin stack-pilot -- "$@"
fi

exec cargo run --quiet --manifest-path "$SCRIPT_DIR/Cargo.toml" --bin stack-pilot -- "$COMMAND" "$@"
