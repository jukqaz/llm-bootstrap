# External Tool Landscape

This document tracks external tools that are useful references for
`StackPilot` beyond the existing inspiration repos such as `gstack` and the
`oh-my-*` family.

The goal is to keep two things separate:

- tools we may adopt directly
- tools whose patterns are worth studying without adopting wholesale

## Categories

We group external tools into five buckets:

- output and token compression
- context packing and repo ingest
- rule and task context assembly
- MCP package and profile management
- agent runtime and harness references

## 1. Output / Token Compression

### RTK

- Link: [rtk-ai/rtk](https://github.com/rtk-ai/rtk)
- Positioning: shell command output compression proxy
- Why it matters:
  - compresses high-volume command output before it enters LLM context
  - already supports Codex, Gemini, Copilot, and more
  - single-binary install keeps the operational cost low
- How to use it as a reference:
  - command-output compression is often a higher-leverage win than prompt tweaking
  - good candidate for a core or near-core default

### Caveman

- Link: [JuliusBrussee/caveman](https://github.com/JuliusBrussee/caveman)
- Positioning: terse response and memory compression skill
- Why it matters:
  - compresses answer style rather than command output
  - includes `caveman-compress` for memory-style files
  - ships across skills, plugins, and Codex plugin surfaces
- How to use it as a reference:
  - useful as a terse-output reference only
  - better folded into a shared common capability than shipped as its own pack

### Compresr Context Gateway

- Link: [compresr Context Gateway](https://compresr.ai/docs/gateway)
- Positioning: transparent compression proxy in front of the LLM API
- Why it matters:
  - compresses history and tool outputs automatically
  - provides threshold-based background compaction and logs
- How to use it as a reference:
  - strong pattern for advanced or enterprise lanes
  - too heavy for the default bootstrap surface

## 2. Context Packing / Repo Ingest

### Repomix

- Link: [yamadashy/repomix](https://github.com/yamadashy/repomix)
- Positioning: AI-friendly repository packer
- Why it matters:
  - packs a repository into a single LLM-friendly artifact
  - supports structural compression, token-count analysis, split output, and MCP mode
- How to use it as a reference:
  - strong optional tool for large-repo analysis
  - useful when multi-agent work needs a compact repo snapshot

### Gitingest

- Link: [coderamp-labs/gitingest](https://github.com/coderamp-labs/gitingest)
- Positioning: prompt-friendly remote repo digest
- Why it matters:
  - fast remote-repo triage
  - simpler than a full repo packer
- How to use it as a reference:
  - good lightweight benchmark for remote ingest UX
  - narrower than Repomix, so likely reference-only unless we want a minimal remote mode

## 3. Rule / Task Context Assembly

### Coding Context CLI

- Link: [kitproj/coding-context-cli](https://github.com/kitproj/coding-context-cli)
- Docs: [How to Use with AI Agents](https://kitproj.github.io/coding-context-cli/how-to/use-with-ai-agents.html)
- Positioning: rules, tasks, and skills context assembler
- Why it matters:
  - cleanly separates reusable rules from task-specific prompts
  - `-w` write-rules mode writes rules to agent-native config files and keeps task output clean
  - includes token estimation and selector-driven context filtering
- How to use it as a reference:
  - high-value idea source for a shared harness spec plus provider renderers
  - more useful as a design pattern than as a direct dependency

## 4. MCP Package / Profile Management

### MCPM

- Link: [pathintegral-institute/mcpm.sh](https://github.com/pathintegral-institute/mcpm.sh)
- Positioning: MCP package manager and registry CLI
- Why it matters:
  - installs MCP servers once and organizes them with profiles
  - manages client integrations separately from server definitions
  - provides a registry search and edit workflow
- How to use it as a reference:
  - strong example for optional advanced MCP profile management
  - aligns well with pack and catalog thinking

## 5. Agent Runtime / Harness References

### GitHub Copilot CLI

- Link: [github/copilot-cli](https://github.com/github/copilot-cli)
- Positioning: terminal-native coding agent
- Why it matters:
  - combines MCP extensibility, autopilot mode, and LSP configuration
  - shows how a user-facing agent CLI can expose runtime power without overwhelming users
- How to use it as a reference:
  - runtime reference only
  - useful for studying surfaced capabilities, not for direct bootstrap adoption

### OpenHands

- Link: [OpenHands/OpenHands](https://github.com/OpenHands/OpenHands)
- Positioning: AI software development agent platform
- Why it matters:
  - spans CLI, GUI, cloud, and SDK
  - includes broader integration and deployment patterns
- How to use it as a reference:
  - architecture reference only
  - useful for understanding the heavy end of the runtime spectrum

## Adoption Priority

### High-value references now

- `RTK`
- `Repomix`
- `Coding Context CLI`
- `MCPM`

These solve different problems with limited overlap:

- `RTK`: command-output compression
- `Repomix`: repo packing and compression
- `Coding Context CLI`: rules and task separation
- `MCPM`: MCP profile management

### Optional references

- `Caveman`
- `Gitingest`

They are useful, but not strong enough to reshape the baseline by themselves.

### Heavy reference lane

- `Compresr`
- `OpenHands`
- `GitHub Copilot CLI`

These are valuable for studying upper-layer runtime patterns, but they are not
good baseline dependencies for a lean bootstrap.

## What This Means for stackpilot

- keep output compression centered on `RTK`
- avoid a standalone terse pack; fold terse-response patterns into the common
  capability catalog only if repeated use proves the need
- expose repo ingest through an optional pack based on `Repomix`-style ideas
- absorb the `Coding Context CLI` write-rules idea into shared harness specs and
  provider renderers
- treat MCP through profile and client-integration separation, similar to `MCPM`
- avoid adding a default proxy or gateway layer

## One-Line Conclusion

For the current `StackPilot` direction, the best external references to keep
actively in view are `RTK`, `Repomix`, `Coding Context CLI`, and `MCPM`, while
`Caveman` stays reference-only and `Compresr`, `OpenHands`, and `GitHub Copilot CLI`
remain higher-level runtime references.
