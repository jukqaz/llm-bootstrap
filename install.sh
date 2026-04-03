#!/usr/bin/env bash
set -euo pipefail

CODEX_HOME="${CODEX_HOME:-$HOME/.codex}"
OH_MY_CODEX_PKG="${OH_MY_CODEX_PKG:-oh-my-codex}"
RTK_FORMULA="${RTK_FORMULA:-rtk-ai/tap/rtk}"
ICM_FORMULA="${ICM_FORMULA:-rtk-ai/tap/icm}"
TIMESTAMP="$(date +%Y%m%dT%H%M%S)"
BACKUP_DIR="${CODEX_HOME}/backups/codex-bootstrap-${TIMESTAMP}"

log() {
  printf '[codex-bootstrap] %s\n' "$*"
}

fail() {
  printf '[codex-bootstrap] %s\n' "$*" >&2
  exit 1
}

require_cmd() {
  local name="$1"
  command -v "$name" >/dev/null 2>&1 || fail "required command not found: ${name}"
}

backup_path() {
  local src="$1"
  local rel dest

  [[ -e "$src" ]] || return 0

  mkdir -p "$BACKUP_DIR"

  if [[ "$src" == "$HOME/"* ]]; then
    rel="${src#"$HOME/"}"
  else
    rel="$(basename "$src")"
  fi

  dest="${BACKUP_DIR}/${rel}"
  mkdir -p "$(dirname "$dest")"
  cp -R "$src" "$dest"
}

backup_existing_state() {
  local backup_targets=(
    "$HOME/AGENTS.md"
    "$CODEX_HOME/config.toml"
    "$CODEX_HOME/AGENTS.md"
    "$CODEX_HOME/hooks.json"
    "$CODEX_HOME/RTK.md"
    "$CODEX_HOME/OPERATIONS.md"
    "$CODEX_HOME/ROLE-MAP.md"
    "$CODEX_HOME/code_review.md"
    "$CODEX_HOME/MCP_SERVERS.md"
    "$CODEX_HOME/SKILLS.md"
    "$CODEX_HOME/SKILLS.generated.md"
    "$CODEX_HOME/agents"
    "$CODEX_HOME/prompts"
    "$CODEX_HOME/skills"
    "$CODEX_HOME/hooks"
    "$CODEX_HOME/scripts"
    "$HOME/.omx/hud-config.json"
    "$HOME/.omx/setup-scope.json"
  )

  for target in "${backup_targets[@]}"; do
    backup_path "$target"
  done

  if [[ -d "$BACKUP_DIR" ]]; then
    log "backup created at $BACKUP_DIR"
  else
    log "no existing Codex overlay files detected"
  fi
}

ensure_brew_formula() {
  local formula="$1"
  local binary="$2"

  if command -v "$binary" >/dev/null 2>&1; then
    log "$binary already installed"
    return 0
  fi

  log "installing $formula"
  brew install "$formula"
}

ensure_node() {
  if command -v npm >/dev/null 2>&1; then
    return 0
  fi

  log "npm not found; installing Homebrew node"
  brew install node
}

install_oh_my_codex() {
  log "installing/updating $OH_MY_CODEX_PKG"
  npm install -g "$OH_MY_CODEX_PKG"
}

run_omx_setup() {
  log "running omx setup"
  (
    cd "$HOME"
    omx setup --scope user --force --verbose
  )
}

run_rtk_init() {
  log "running rtk init for Codex"
  rtk init -g --codex
}

ensure_icm_mcp() {
  local config_path="$CODEX_HOME/config.toml"
  local icm_bin

  icm_bin="$(command -v icm)"
  [[ -n "$icm_bin" ]] || fail "icm binary not found after installation"
  [[ -f "$config_path" ]] || fail "missing Codex config at $config_path"

  if grep -q '^\[mcp_servers\.icm\]' "$config_path"; then
    log "icm MCP already configured"
    return 0
  fi

  log "adding icm MCP block to $config_path"
  cat >> "$config_path" <<EOF

# ICM Memory MCP Server
[mcp_servers.icm]
command = "$icm_bin"
args = ["serve"]
enabled = true
EOF
}

verify_setup() {
  log "running omx doctor"
  (
    cd "$HOME"
    omx doctor
  )

  log "listing Codex MCP servers"
  codex mcp list
}

main() {
  require_cmd brew
  require_cmd codex
  backup_existing_state
  ensure_node
  ensure_brew_formula "$RTK_FORMULA" rtk
  ensure_brew_formula "$ICM_FORMULA" icm
  install_oh_my_codex
  require_cmd omx
  run_omx_setup
  run_rtk_init
  ensure_icm_mcp
  verify_setup
  log "done"
}

main "$@"
