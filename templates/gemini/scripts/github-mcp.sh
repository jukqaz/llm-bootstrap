#!/usr/bin/env bash
set -euo pipefail

token="${GITHUB_PERSONAL_ACCESS_TOKEN:-${GH_TOKEN:-}}"

if [[ -z "$token" ]]; then
  token="$(gh auth token)"
fi

exec env GITHUB_PERSONAL_ACCESS_TOKEN="$token" npx -y @modelcontextprotocol/server-github
