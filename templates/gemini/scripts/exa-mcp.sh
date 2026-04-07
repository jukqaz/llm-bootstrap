#!/usr/bin/env bash
set -euo pipefail

if [[ -n "${EXA_API_KEY:-}" ]]; then
  exec env EXA_API_KEY="$EXA_API_KEY" npx -y exa-mcp-server
fi

exec npx -y exa-mcp-server
