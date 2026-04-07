#!/usr/bin/env bash
set -euo pipefail

if [[ -n "${EXA_API_KEY:-}" ]]; then
  exec env EXA_API_KEY="$EXA_API_KEY" npx -y exa-mcp-server@3.2.0
fi

exec npx -y exa-mcp-server@3.2.0
