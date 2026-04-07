#!/usr/bin/env bash
set -euo pipefail

if [[ -n "${CONTEXT7_API_KEY:-}" ]]; then
  exec npx -y @upstash/context7-mcp@2.1.7 --api-key "$CONTEXT7_API_KEY"
fi

exec npx -y @upstash/context7-mcp@2.1.7
