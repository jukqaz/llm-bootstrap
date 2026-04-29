# Official Best Practices

As of: `2026-04-09`

This document curates the official best-practice articles from
`OpenAI Codex`, `Gemini CLI`, and `Claude Code` that most directly affect
`StackPilot` design.

The point is not feature inventory.
The point is what the official docs actively recommend.

## OpenAI / Codex

### Priority reading

- [Codex Prompting Guide](https://developers.openai.com/cookbook/examples/gpt-5/codex_prompting_guide)
- [Safety in building agents](https://developers.openai.com/api/docs/guides/agent-builder-safety)
- [Reasoning best practices](https://developers.openai.com/api/docs/guides/reasoning-best-practices)
- [Production best practices](https://developers.openai.com/api/docs/guides/production-best-practices)
- [Docs MCP](https://developers.openai.com/learn/docs-mcp)

### What the official docs emphasize

- Codex is treated as OpenAI’s recommended coding-model lane.
- reasoning effort should be tuned by workload, not pinned high everywhere
- long coding sessions are designed around compaction
- reasoning models prefer brief, direct developer instructions
- delimiters and input structure matter more than chain-of-thought forcing
- agent safety requires prompt-injection defenses, structured outputs, tool
  approvals, and evals together
- documentation retrieval should prefer an official read-only MCP path
- production readiness starts with org setup, secrets, billing, and rollout
  planning

### What this means for stackpilot

- the Codex renderer should keep `config + AGENTS + MCP` as the base surface
- role instructions should lean toward direct instructions plus output contracts
- token efficiency should include compaction, retrieval, and prompt caching, not
  just terse prompts
- safety and eval should be part of harness design, not a later add-on

## Gemini CLI / Google

### Priority reading

- [Hooks Best Practices](https://geminicli.com/docs/hooks/best-practices/)
- [Writing hooks for Gemini CLI](https://geminicli.com/docs/hooks/writing-hooks/)
- [Gemini CLI extension best practices](https://geminicli.com/docs/extensions/best-practices/)
- [Plan Mode](https://geminicli.com/docs/cli/plan-mode/)
- [Sandboxing in the Gemini CLI](https://geminicli.com/docs/cli/sandbox/)

### What the official docs emphasize

- hooks run synchronously in the agent loop, so slow hooks directly degrade the
  user experience
- stdout should contain only final JSON; logs belong on stderr
- extensions are expected to use TypeScript, clear `src/` vs `dist/`
  separation, and link-based local iteration
- `GEMINI.md` should stay concise and goal-oriented
- permissions should follow least privilege with strong input validation and
  secret protection
- Plan Mode explicitly supports preserving approved plan artifacts through hooks
- sandbox guidance recommends the most restrictive viable profile

### What this means for stackpilot

- extension-first remains the correct Gemini direction
- hook and command count must be balanced against synchronous latency cost
- `GEMINI.md` should remain a concise renderer artifact, not a giant dump
- plan artifacts plus hooks map well onto future `ralph-loop` and
  `delivery-harness` design
- sandboxing belongs in an advanced lane, but the native path is clear enough
  to support

## Claude Code / Anthropic

### Priority reading

- [Claude Code settings](https://code.claude.com/docs/en/settings)
- [Create custom subagents](https://code.claude.com/docs/en/sub-agents)
- [Hooks reference](https://code.claude.com/docs/en/hooks)
- [Get started with Claude Code hooks](https://docs.anthropic.com/en/docs/claude-code/hooks-guide)
- [Connect Claude Code to tools via MCP](https://docs.anthropic.com/en/docs/claude-code/mcp)

### What the official docs emphasize

- settings follow a clear `user / project / local / enterprise managed`
  hierarchy
- `permissions.deny` should completely exclude sensitive files from read and
  search access
- subagents should stay focused, with narrow descriptions and tool grants
- subagent frontmatter can control memory, hooks, MCP servers, permissions, and
  effort explicitly
- hooks are treated as deterministic control, with a dedicated security
  best-practices section
- hook safety requires input sanitization, absolute paths, path-traversal
  blocking, and skipping sensitive files
- MCP is formally scoped through `--scope`, `.mcp.json`, user scope, project
  scope, and approval flows
- large MCP output is an official concern with output limits such as
  `MAX_MCP_OUTPUT_TOKENS`

### What this means for stackpilot

- Claude should stay `subagent + MCP + hooks/settings` first, not skill-first
- workflow skills belong as a secondary packaging layer
- `.mcp.json` and user scope should remain explicitly separated
- output-heavy MCP should be evaluated together with token limits and approval
  flows
- permissions and sensitive-file deny rules are candidates for baseline doctor
  checks or policy guidance

## Shared Conclusion

All three official ecosystems are converging on similar themes:

- explicit contracts over giant prompts
- least privilege over broad tool access
- structured outputs and scoped retrieval over uncontrolled context growth
- deterministic hooks or approval flows over runtime magic
- output-shape and safety management over simple tool-count growth

So the official baseline for `StackPilot` is:

1. keep shared intent short and structured
2. prefer provider-native surfaces
3. treat permissions, hooks, approvals, and sandboxing as separate control
   layers
4. prefer on-demand retrieval over preloading large docs
5. keep plan, review, qa, and ship flows as explicit artifacts

## Linked Data

These articles are also reflected in the official source catalog:

- [catalog/sources/official/openai_codex.toml](../catalog/sources/official/openai_codex.toml)
- [catalog/sources/official/gemini_cli.toml](../catalog/sources/official/gemini_cli.toml)
- [catalog/sources/official/claude_code.toml](../catalog/sources/official/claude_code.toml)
