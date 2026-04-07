#!/usr/bin/env bash
set -euo pipefail

exec npx -y chrome-devtools-mcp@0.21.0 --headless=true --no-usage-statistics
