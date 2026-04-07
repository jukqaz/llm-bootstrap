#!/usr/bin/env bash
set -euo pipefail

exec npx -y chrome-devtools-mcp@latest --headless=true --no-usage-statistics
