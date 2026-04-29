# Official Best Practices

기준일: `2026-04-09`

이 문서는 `OpenAI Codex`, `Gemini CLI`, `Claude Code` 공식 사이트에서
`StackPilot` 설계에 바로 영향을 주는 best-practice 아티클만 추려서 정리한다.

핵심은 "기능 목록"이 아니라 "공식 문서가 무엇을 권장하는가"다.

## OpenAI / Codex

### 우선 볼 글

- [Codex Prompting Guide](https://developers.openai.com/cookbook/examples/gpt-5/codex_prompting_guide)
- [Safety in building agents](https://developers.openai.com/api/docs/guides/agent-builder-safety)
- [Reasoning best practices](https://developers.openai.com/api/docs/guides/reasoning-best-practices)
- [Production best practices](https://developers.openai.com/api/docs/guides/production-best-practices)
- [Docs MCP](https://developers.openai.com/learn/docs-mcp)

### 공식 문서가 강조하는 점

- Codex는 OpenAI가 권장하는 coding model 축으로 다룬다.
- reasoning effort는 무조건 높게 고정하는 것보다 목적별로 조절하라고 본다.
- 긴 개발 세션은 compaction을 전제로 설계한다.
- reasoning model은 짧고 직접적인 developer instruction을 더 잘 따른다.
- chain-of-thought 강요보다 delimiters와 명확한 구조가 중요하다.
- agent safety는 prompt injection, structured outputs, tool approvals, evals를 함께 보라고 한다.
- 문서 검색은 Docs MCP처럼 공식 read-only MCP를 우선 사용한다.
- production 전환은 권한, billing, org setup, secret handling부터 정리하라고 한다.

### StackPilot에 주는 의미

- Codex renderer는 `config + AGENTS + MCP`를 기본축으로 유지하는 게 맞다.
- `planner`, `reviewer`, `verifier` 같은 역할 지시도 장황하게 늘리기보다 direct instruction + output contract 쪽이 맞다.
- token 절약은 terse mode보다 compaction, MCP retrieval, prompt caching 같은 공식 경로와 같이 봐야 한다.
- safety와 eval을 runtime 밖 부가 기능이 아니라 기본 harness 설계 일부로 넣어야 한다.

## Gemini CLI / Google

### 우선 볼 글

- [Hooks Best Practices](https://geminicli.com/docs/hooks/best-practices/)
- [Writing hooks for Gemini CLI](https://geminicli.com/docs/hooks/writing-hooks/)
- [Gemini CLI extension best practices](https://geminicli.com/docs/extensions/best-practices/)
- [Plan Mode](https://geminicli.com/docs/cli/plan-mode/)
- [Sandboxing in the Gemini CLI](https://geminicli.com/docs/cli/sandbox/)

### 공식 문서가 강조하는 점

- hooks는 agent loop 안에서 synchronous 하므로 느리면 전체 체감이 나빠진다.
- stdout에는 최종 JSON만 쓰고, 로그는 stderr로 분리하라고 강하게 요구한다.
- extension은 TypeScript, `src/`와 `dist/` 분리, `gemini extensions link` 기반 반복 개발을 권장한다.
- `GEMINI.md`는 길게 덤프하지 말고 goal, usage, examples 중심으로 간결하게 쓰라고 한다.
- permissions는 최소 권한, input validation, sensitive setting 보호를 권장한다.
- Plan Mode는 승인된 plan artifact를 hook으로 외부 저장소에 보관하는 패턴까지 문서화한다.
- sandbox는 기본적으로 가장 restrictive한 프로필을 쓰라고 권장한다.

### StackPilot에 주는 의미

- Gemini renderer는 extension-first 방향이 확실히 맞다.
- hook이나 command를 많이 넣더라도 synchronous cost를 먼저 계산해야 한다.
- `GEMINI.md`는 거대한 instructions dump가 아니라 concise renderer output이 맞다.
- plan artifact와 hook을 연결하는 구조는 나중에 `ralph-loop`나 `delivery-harness`에 바로 녹일 수 있다.
- sandbox lane은 optional이지만 공식 경로가 분명하므로 advanced pack으로 다루기 좋다.

## Claude Code / Anthropic

### 우선 볼 글

- [Claude Code settings](https://code.claude.com/docs/en/settings)
- [Create custom subagents](https://code.claude.com/docs/en/sub-agents)
- [Hooks reference](https://code.claude.com/docs/en/hooks)
- [Get started with Claude Code hooks](https://docs.anthropic.com/en/docs/claude-code/hooks-guide)
- [Connect Claude Code to tools via MCP](https://docs.anthropic.com/en/docs/claude-code/mcp)

### 공식 문서가 강조하는 점

- settings는 `user / project / local / enterprise managed` 계층으로 동작한다.
- `permissions.deny`로 민감 파일을 읽기/검색에서 완전히 배제하는 방식을 권장한다.
- subagent는 한 가지 일에 집중시키고, description과 tools를 좁게 쓰라고 권장한다.
- subagent는 memory, hooks, mcpServers, permissionMode까지 frontmatter로 세밀하게 조절할 수 있다.
- hooks는 deterministic control layer로 취급하고, security best practices를 별도 섹션으로 둔다.
- hook은 input sanitization, absolute path, path traversal 차단, sensitive file skip을 권장한다.
- MCP는 `--scope`, `.mcp.json`, user scope, project scope, approval reset 흐름까지 공식화한다.
- MCP 출력이 커질 때는 output threshold와 `MAX_MCP_OUTPUT_TOKENS`를 공식적으로 관리한다.

### StackPilot에 주는 의미

- Claude는 skill-first보다 `subagent + MCP + hooks/settings` 우선이 맞다.
- workflow skills는 secondary packaging layer로 두는 현재 방향이 맞다.
- `.mcp.json`과 user scope를 분리해 관리해야 한다.
- output-heavy MCP는 무조건 늘리기보다 output token limit와 approval flow를 같이 고려해야 한다.
- permissions와 sensitive-file deny는 doctor나 baseline policy에도 반영할 가치가 있다.

## 공통 결론

세 공식 문서가 공통으로 밀고 있는 방향은 거의 같다.

- 긴 프롬프트보다 명확한 contract
- broad access보다 최소 권한
- freeform context보다 structured output과 scoped retrieval
- runtime magic보다 deterministic hook 또는 approval flow
- tool 추가보다 output shape와 safety 관리

즉 `StackPilot`가 따라야 할 공식 기준은 아래로 요약된다.

1. 공통 intent는 짧고 구조화된 spec로 유지
2. provider-native surface를 우선 사용
3. 안전 장치는 permission, hook, approval, sandbox로 분리
4. 문서/코드 retrieval는 on-demand 경로를 우선
5. plan, review, qa, ship 같은 하네스 contract는 명시적 artifact로 남김

## 연결된 데이터

이 문서의 항목은 아래 source catalog에도 반영했다.

- [catalog/sources/official/openai_codex.toml](../catalog/sources/official/openai_codex.toml)
- [catalog/sources/official/gemini_cli.toml](../catalog/sources/official/gemini_cli.toml)
- [catalog/sources/official/claude_code.toml](../catalog/sources/official/claude_code.toml)
