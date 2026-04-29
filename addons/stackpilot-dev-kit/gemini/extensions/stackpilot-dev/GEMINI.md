# StackPilot Gemini Extension

This extension adds a development-first baseline for Gemini CLI on macOS.

## Included

- Direct execution bias with compact reporting
- RTK shell hook for noisy commands
- Context7, Exa, and Chrome DevTools MCP baseline when enabled
- Lightweight planner / reviewer / executor extension notes
- Native Gemini custom commands for `office-hours`, `intent`, `doctor`, `autopilot`, `team`, `review`, `qa`, `ship`, and `retro`

## Model note

Gemini CLI officially exposes model selection at the global `model.name` settings layer.
This extension keeps role files prompt-only instead of inventing per-agent model pins that Gemini CLI does not officially support.
The bootstrap baseline sets `model.name = "auto"` so Gemini 3 routing can follow the account's current availability, and enables native Memory v2 plus Auto Memory instead of external ICM-style memory.
