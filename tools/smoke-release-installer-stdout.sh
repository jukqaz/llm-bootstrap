#!/usr/bin/env bash
set -euo pipefail

fail() {
  printf '[smoke-release-installer] %s\n' "$*" >&2
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
    *) fail "unsupported platform: $os/$arch" ;;
  esac
}

need_cmd curl
need_cmd python3
need_cmd shasum
need_cmd tar
need_cmd mktemp

target="$(detect_target)"
tmp_dir="$(mktemp -d)"
trap 'rm -rf "$tmp_dir"' EXIT

archive_dir="$tmp_dir/assets"
package_root="stackpilot-smoke-${target}"
mkdir -p "$archive_dir" "$tmp_dir/package/${package_root}"

cat > "$tmp_dir/package/${package_root}/stack-pilot" <<'FAKE_STACK_PILOT'
#!/usr/bin/env bash
set -euo pipefail

case "${1:-}" in
  --version|-V)
    printf 'stack-pilot smoke\n'
    exit 0
    ;;
  doctor)
    case "${2:-}" in
      --help|-h)
        printf 'doctor smoke help\n'
        exit 0
        ;;
      --json)
        printf '{"ok":true,"warning_count":0,"providers":[]}\n'
        exit 0
        ;;
    esac
    ;;
esac

printf 'stack-pilot smoke command\n'
FAKE_STACK_PILOT
chmod 755 "$tmp_dir/package/${package_root}/stack-pilot"

archive_path="$archive_dir/stackpilot-${target}.tar.gz"
tar -C "$tmp_dir/package" -czf "$archive_path" "$package_root"
shasum -a 256 "$archive_path" > "${archive_path}.sha256"

stdout_path="$tmp_dir/stdout.json"
stderr_path="$tmp_dir/stderr.log"

STACKPILOT_DOWNLOAD_BASE_URL="file://${archive_dir}" \
  STACKPILOT_VERSION=latest \
  bash install-release.sh doctor --json >"$stdout_path" 2>"$stderr_path"

python3 - "$stdout_path" <<'PY'
import json
import sys

with open(sys.argv[1], "r", encoding="utf-8") as handle:
    payload = json.load(handle)

if payload.get("ok") is not True:
    raise SystemExit("doctor payload did not report ok=true")
PY

if grep -Eq '^\[stackpilot\]|: OK$' "$stdout_path"; then
  fail "installer wrote progress output to stdout"
fi

grep -q '^\[stackpilot\] downloading ' "$stderr_path" || fail "missing download log on stderr"
grep -q ': OK$' "$stderr_path" || fail "missing checksum result on stderr"
