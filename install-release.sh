#!/usr/bin/env bash
set -euo pipefail

REPO_SLUG="${LLM_BOOTSTRAP_REPO:-jukqaz/llm-bootstrap}"
VERSION="${LLM_BOOTSTRAP_VERSION:-latest}"
DOWNLOAD_BASE_URL="${LLM_BOOTSTRAP_DOWNLOAD_BASE_URL:-}"

log() {
  printf '[llm-bootstrap] %s\n' "$*"
}

fail() {
  printf '[llm-bootstrap] %s\n' "$*" >&2
  exit 1
}

need_cmd() {
  command -v "$1" >/dev/null 2>&1 || fail "required command not found: $1"
}

detect_target() {
  local os arch
  os="$(uname -s)"
  arch="$(uname -m)"

  case "$os/$arch" in
    Darwin/arm64) printf 'aarch64-apple-darwin' ;;
    Darwin/x86_64) printf 'x86_64-apple-darwin' ;;
    Linux/x86_64) printf 'x86_64-unknown-linux-gnu' ;;
    *)
      fail "unsupported platform: $os/$arch"
      ;;
  esac
}

download_url() {
  local target="$1"
  local filename base_url
  if [[ "$VERSION" == "latest" ]]; then
    filename="llm-bootstrap-${target}.tar.gz"
    if [[ -n "$DOWNLOAD_BASE_URL" ]]; then
      printf '%s/%s' "$DOWNLOAD_BASE_URL" "$filename"
    else
      printf 'https://github.com/%s/releases/latest/download/%s' "$REPO_SLUG" "$filename"
    fi
  else
    filename="llm-bootstrap-${VERSION}-${target}.tar.gz"
    if [[ -n "$DOWNLOAD_BASE_URL" ]]; then
      printf '%s/%s' "$DOWNLOAD_BASE_URL" "$filename"
    else
      printf 'https://github.com/%s/releases/download/%s/%s' "$REPO_SLUG" "$VERSION" "$filename"
    fi
  fi
}

checksum_url() {
  local target="$1"
  printf '%s.sha256' "$(download_url "$target")"
}

archive_name() {
  local target="$1"
  if [[ "$VERSION" == "latest" ]]; then
    printf 'llm-bootstrap-%s.tar.gz' "$target"
  else
    printf 'llm-bootstrap-%s-%s.tar.gz' "$VERSION" "$target"
  fi
}

verify_checksum() {
  local archive="$1"
  local checksum_file="$2"
  local archive_name checksum_dir normalized_checksum
  archive_name="$(basename "$archive")"
  checksum_dir="$(dirname "$checksum_file")"
  normalized_checksum="${checksum_dir}/$(basename "$checksum_file").normalized"

  awk -v archive_name="$archive_name" '
    {
      $2 = archive_name
      print $1 "  " $2
    }
  ' "$checksum_file" > "$normalized_checksum"

  if command -v shasum >/dev/null 2>&1; then
    (cd "$(dirname "$archive")" && shasum -a 256 -c "$(basename "$normalized_checksum")")
    return
  fi

  if command -v sha256sum >/dev/null 2>&1; then
    (cd "$(dirname "$archive")" && sha256sum -c "$(basename "$normalized_checksum")")
    return
  fi

  fail "no sha256 verifier found (need shasum or sha256sum)"
}

main() {
  need_cmd curl
  need_cmd tar
  need_cmd mktemp

  local target archive_url checksum_download_url temp_dir archive_path checksum_path
  target="$(detect_target)"
  archive_url="$(download_url "$target")"
  checksum_download_url="$(checksum_url "$target")"

  temp_dir="$(mktemp -d)"
  trap 'rm -rf "$temp_dir"' EXIT

  archive_path="$temp_dir/$(archive_name "$target")"
  checksum_path="${archive_path}.sha256"

  log "downloading ${archive_url}"
  curl -fsSL "$archive_url" -o "$archive_path"
  curl -fsSL "$checksum_download_url" -o "$checksum_path"
  verify_checksum "$archive_path" "$checksum_path"

  tar -xzf "$archive_path" -C "$temp_dir"

  local extracted_dir
  extracted_dir="$(find "$temp_dir" -mindepth 1 -maxdepth 1 -type d -name 'llm-bootstrap-*' | head -n 1)"
  [[ -n "$extracted_dir" ]] || fail "failed to find extracted llm-bootstrap directory"
  [[ -x "$extracted_dir/llm-bootstrap" ]] || fail "bundled llm-bootstrap binary not found"

  if [[ $# -eq 0 ]]; then
    exec "$extracted_dir/llm-bootstrap" wizard
  fi

  case "$1" in
    --help|-h|--version|-V)
      exec "$extracted_dir/llm-bootstrap" "$@"
      ;;
  esac

  if [[ "$1" != -* ]] && "$extracted_dir/llm-bootstrap" "$1" --help >/dev/null 2>&1; then
    exec "$extracted_dir/llm-bootstrap" "$@"
  fi

  exec "$extracted_dir/llm-bootstrap" install "$@"
}

main "$@"
