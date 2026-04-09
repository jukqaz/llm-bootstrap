# Provider Surface Strategy

이 문서는 `llm-bootstrap`가 멀티에이전트 하네스와 `gstack`식 workflow contract를
각 LLM의 native surface에 맞게 어떻게 배포할지 정리한다.

핵심 원칙은 하나다.

> 카탈로그는 공통으로 관리하되, 설치 표면은 provider마다 다르게 쓴다.

즉 `plugin`, `extension`, `skill`, `command`, `mcp`를 모두 억지로 하나로 통일하지
않는다. 각 LLM이 잘 먹는 표면에 맞춰서 "가장 유익한 방식"으로 배포한다.

## 왜 이렇게 가야 하나

현재 구현을 보면 provider마다 강점이 다르다.

- `Codex`: plugin + agent + config 기반 멀티에이전트 하네스가 가장 강하다
- `Gemini`: extension + settings merge + native command 표면이 가장 자연스럽다
- `Claude Code`: subagent + official MCP CLI + hook/settings 축을 우선하는 것이 가장 안전하다

실제로 현재 저장소도 그렇게 흘러가고 있다.

- Codex는 `.agents/plugins/marketplace.json`, local plugin, agent TOML을 설치한다
- Gemini는 `extensions/llm-bootstrap-dev`, `settings.json`, command TOML을 중심으로 설치한다
- Claude는 `agents/*`, official MCP 등록, workflow skill pack을 함께 설치한다

따라서 앞으로도 공통 추상화보다 provider-native distribution을 우선해야 한다.

## 공식 권장 방식 우선 원칙

커뮤니티 패턴보다 먼저 따라야 하는 것은 각 provider의 공식 문서다.

우선순위:

1. provider 공식 문서가 직접 권장하는 표면
2. provider가 공식적으로 지원하는 설정/배포 경로
3. 커뮤니티에서 검증된 패턴
4. 우리 고유 추상화

즉 `llm-bootstrap`는 provider native surface를 "대체"하지 않고
"조합하고 자동화"하는 쪽으로 가야 한다.

## 공식 문서 기준 요약

### Codex / OpenAI

OpenAI 공식 문서는 Codex를 다음 표면으로 설명한다.

- `config.toml`
- `AGENTS.md`
- hooks
- MCP
- plugins
- skills
- subagents

또한 OpenAI 공식 Docs MCP는 Codex CLI와 IDE extension에서 같은 설정을 공유한다고 설명한다.

의미:

- Codex는 plugin만의 도구가 아니다
- config, rules, MCP, plugin, subagent를 함께 쓰는 종합 표면이다
- 따라서 Codex renderer는 `config + AGENTS + MCP`를 기본축으로 두고
  plugin/skill/subagent를 그 위에 얹어야 한다

### Gemini CLI / Google

Gemini CLI 공식 문서는 다음을 전면에 둔다.

- extensions
- `settings.json`
- custom commands
- `GEMINI.md`
- hooks
- MCP servers
- subagents
- agent skills

특히 extension reference는 command, hook, skill, policy, theme까지 extension 안에 넣는 구조를 공식화한다.

의미:

- Gemini는 extension-first가 맞다
- command와 settings merge도 official path다
- skill과 hook도 extension 안에 자연스럽게 녹여야 한다

### Claude Code / Anthropic

Anthropic 공식 문서는 Claude Code에서 다음 축을 강하게 강조한다.

- subagents
- MCP
- hooks
- settings
- user/project scope hierarchy

또한 MCP는 `claude mcp add`와 `.mcp.json` 환경변수 확장, scope precedence를 공식적으로 설명한다.

의미:

- Claude는 community skill 패턴보다 subagent/MCP/hook 축을 먼저 따라야 한다
- skill은 보조 packaging layer로 둘 수 있지만, 공식 기본축은 아니다
- Claude renderer는 `subagents + MCP + hooks/settings`를 우선하고,
  skill pack은 workflow artifact 배포 수단으로 얹는 쪽이 맞다

## 더 빠르고 가벼운 구현 원칙

같은 기능을 하면서도 더 빠르고 가볍게 가려면, 구조를 늘리는 대신 중복을 줄여야 한다.

우선 원칙:

- 공통 정의는 한 번만 쓴다
- provider별로 런타임을 새로 만들지 않는다
- 추가 영속 상태를 만들지 않는다
- 기본 설치는 core MCP와 core harness만 넣는다
- heavy lane는 opt-in으로만 연다

실행 규칙:

- role taxonomy는 공통 source 하나만 유지한다
- harness definition도 공통 source 하나만 유지한다
- provider별 plugin/extension/skill은 thin renderer output으로만 둔다
- planner, triage, executor, reviewer, verifier 다섯 역할을 기본으로 삼는다
- specialist role은 기본값이 아니라 필요 시 확장한다

피해야 할 것:

- provider마다 따로 하네스 문서를 복제해서 drift 만들기
- plugin, extension, skill마다 같은 설명과 contract를 반복 작성하기
- taskboard, session DB, runtime cache 같은 새 상태 계층 추가
- 무거운 MCP를 default install에 넣기

한 줄로 말하면 "공통 스펙 1개, native renderer 3개"가 가장 가볍다.

## 최상위 구조

앞으로는 아래 다섯 가지를 구분해서 관리하는 것이 맞다.

- `mcp catalog`
- `plugin catalog`
- `harness catalog`
- `pack catalog`
- `provider renderer`

의미는 다음과 같다.

- `mcp catalog`: 어떤 MCP가 있는지, 어떤 env와 권한이 필요한지
- `plugin catalog`: plugin 또는 extension처럼 배포 단위가 있는지
- `harness catalog`: 멀티에이전트 팀 구성, workflow, artifact contract
- `pack catalog`: 설치 조합
- `provider renderer`: 위 카탈로그를 각 LLM의 native surface에 맞게 풀어내는 렌더러

핵심은 카탈로그는 공통이어도 렌더링은 다르게 해야 한다는 점이다.

여기서도 가볍게 가려면 카탈로그 수를 최소화하는 편이 좋다.

권장 최소 구조:

- `mcp catalog`
- `harness catalog`
- `pack catalog`
- `provider renderer`

`plugin catalog`은 실제로 독립 plugin 공급원이 여러 개 생길 때만 별도 카탈로그로 승격한다.
초기에는 `pack catalog` 안의 provider surface metadata로 흡수하는 편이 더 단순하다.

## 공통으로 가져갈 것

세 provider 모두 아래는 공통으로 가져간다.

- role taxonomy
- harness definitions
- workflow contract
- artifact schemas
- MCP metadata
- permission metadata
- doctor categories

예시:

- role taxonomy: `triage`, `planner`, `executor`, `reviewer`, `verifier`
- harness: `delivery`, `parallel-build`, `incident`, `review-gate`
- workflow: `office-hours`, `autopilot`, `review`, `qa`, `ship`, `retro`

## Codex 전략

### 권장 표면

- `config.toml`
- `AGENTS.md`
- MCP
- plugin
- agent TOML
- hooks
- agent TOML / subagents
- skills
- workflow docs

### 왜 이렇게

Codex는 멀티에이전트와 plugin 표면이 가장 강하다.

현재도 다음을 이미 활용한다.

- `.agents/plugins/marketplace.json`
- `plugins/llm-dev-kit`
- `agents/*.toml`
- `config.toml`의 `[agents]`, `enable_fanout`, `multi_agent_v2`

공식 문서도 `Config`, `AGENTS.md`, `Hooks`, `MCP`, `Plugins`, `Skills`, `Subagents`
를 Codex의 기본 표면으로 다룬다.

즉 Codex는 기본 런타임으로 삼는 것이 맞다.

### 앞으로의 방향

- 멀티에이전트 하네스의 기준 provider는 Codex로 둔다
- squad 구성과 handoff contract도 Codex 표면에서 먼저 고정한다
- `gstack`식 contract는 rules, skill, workflow doc, subagent contract로 공급한다
- MCP는 `scripts/*.sh` + `config.toml` 블록 렌더링으로 계속 관리한다
- plugin은 rich distribution unit이지만, config/MCP/AGENTS를 대체하지 않는다

### Codex에서 특히 좋은 것

- planner가 squad staffing을 하도록 하기
- reviewer / verifier / test-engineer를 분리 유지하기
- long-context lane는 `*-1m`만 opt-in 유지하기
- plugin을 "workflow + harness" 배포 단위로 사용하기

### Codex에서 피할 것

- giant plugin 하나에 모든 걸 우겨넣기
- 하네스 정의 없이 role 숫자만 늘리기
- MCP, plugin, harness를 같은 설정 블록으로 섞기

## Gemini 전략

### 권장 표면

- extension
- `settings.json`
- native custom commands
- `GEMINI.md`
- hooks
- MCP entries in `settings.json`
- subagents
- skills
- lightweight agent markdown

### 왜 이렇게

Gemini는 plugin보다 extension과 command가 더 자연스럽다.

현재 구현도 다음을 중심으로 한다.

- `extensions/llm-bootstrap-dev`
- `commands/*.toml`
- `settings.json` merge
- `extension-enablement.json`

공식 문서도 `Extensions`, `Settings`, `Custom commands`, `GEMINI.md`, `Hooks`,
`MCP servers`, `Subagents`, `Agent Skills`를 함께 설명한다.

Gemini는 Codex처럼 provider 내부에서 rich agent runtime을 세밀하게 제어하는 방식보다,
extension에 workflow와 command를 잘 배치하는 쪽이 낫다.

### 앞으로의 방향

- Gemini는 `workflow extension` 중심으로 간다
- 멀티에이전트 역할 파일은 prompt contract 위주로 유지한다
- Codex용 role taxonomy를 Gemini용 command와 agent 문서로 번역한다
- MCP는 `settings.json` patch로 계속 관리한다
- plugin이라는 말을 억지로 붙이지 않고 extension 단위로 부른다
- skill, hook, policy도 extension 내부에 포함하는 방향을 우선한다

### Gemini에서 특히 좋은 것

- `intent`, `doctor`, `autopilot`, `review`, `ship` 같은 명시적 command lane
- extension docs에서 role handoff contract 노출
- settings merge로 사용자 runtime state 보존

### Gemini에서 피할 것

- 공식적으로 없는 per-agent model pin 흉내내기
- Codex plugin 개념을 그대로 복제하기
- 무거운 subagent 체계를 Gemini 표면에 과장해서 투영하기

## Claude Code 전략

### 권장 표면

- subagents
- official `claude mcp add/remove --scope user`
- `.mcp.json`
- hooks
- settings
- official `claude mcp add/remove --scope user`
- lightweight subagent docs
- workflow skill pack
- workflow markdown
- 최소 settings patch

### 왜 이렇게

Claude Code는 공식 문서 기준으로 subagent와 MCP, hook, settings 경로가 가장 안정적이다.

현재 구현도 다음 원칙을 따른다.

- `agents/*.md`
- official MCP registration
- user-scope `CLAUDE.md`
- `skills/*/SKILL.md`

즉 Claude는 compatibility lane이지만, 공식 기본축은 `subagents + MCP + hooks/settings`다.

### 앞으로의 방향

- Claude는 `subagents + MCP + hooks/settings`를 먼저 따른다
- `autopilot`, `investigate`, `review`, `qa`, `ship`, `retro`는
  workflow skill pack과 문서 artifact로 유지하되 부차 레이어로 둔다
- squad 조합은 subagent contract에 먼저 녹이고, skill은 실행 가이드로 보완한다
- MCP는 반드시 official CLI를 통해 관리한다
- settings는 최소 수정만 허용한다

### Claude에서 특히 좋은 것

- 명확한 subagent 분리
- 공식 경로를 통한 MCP 등록/해제
- hooks와 scope hierarchy
- 명확한 workflow skill pack
- 읽기 쉬운 lightweight agent docs

### Claude에서 피할 것

- Codex plugin 구조 흉내내기
- provider가 직접 지원하지 않는 무거운 runtime abstraction
- user home 바깥으로 새 상태 계층 만들기

## MCP 전략

MCP는 provider 공통 카탈로그를 유지하되, 적용 방식은 다르게 간다.

### 공통 원칙

- MCP는 `core`, `optional`, `specialized`로 나눈다
- env-gated 여부와 permission scope를 metadata로 둔다
- read-only와 write-capable을 분리한다
- provider마다 native install path를 다르게 탄다

### 권장 분류

- `core`
  - `chrome-devtools`
  - `context7`
  - `exa`
- `orchestration-support`
  - docs, browser QA, repo search 성격
- `domain-specific`
  - payment, CRM, analytics, design, project tools

### Provider별 적용

- Codex: `config.toml`의 `[mcp_servers.*]`
- Gemini: `settings.json`의 `mcpServers`
- Claude: official CLI registration 결과

## Plugin/Extension 전략

plugin이나 extension은 "동작 단위"가 아니라 "배포 단위"로 봐야 한다.

권장 규칙:

- Codex: config + MCP + plugin
- Gemini: extension + commands + settings
- Claude: subagents + MCP + hooks, with skill pack as secondary packaging

즉 카탈로그 이름은 공통으로 `pack`이나 `harness`를 쓰고,
provider renderer가 이를 provider native packaging으로 풀어야 한다.

예시:

- `delivery-harness`
  - Codex에서는 rules + MCP + plugin skill + subagent contract
  - Gemini에서는 extension command + settings + docs + skills
  - Claude에서는 subagents + MCP + hooks + workflow skills

더 가볍게 가려면 plugin/extension/skill 자체를 source of truth로 두지 않는다.

권장 방식:

- source of truth는 `harness`와 `pack` metadata
- plugin/extension/skill은 install 시 렌더링되는 배포물
- 같은 contract를 여러 위치에서 수동 편집하지 않는다

## Harness 전략

하네스는 provider에 무관한 공통 정의를 가져야 한다.

추천 하네스:

- `delivery`
- `parallel-build`
- `incident`
- `review-gate`
- `ralph-loop`

각 하네스는 다음을 가진다.

- team topology
- role ownership
- handoff schema
- stop rule
- QA gate
- final artifact contract

`gstack`에서 가져올 것은 여기다.

- `office-hours`
- `review`
- `qa`
- `ship`
- `retro`

즉 `gstack`은 별도 제품이 아니라 하네스 contract source가 된다.

## 권장 설치 조합

### Codex-first

- core MCP
- `llm-dev-kit` plugin
- `delivery-harness`
- `parallel-build-harness`

### Gemini-first

- core MCP
- `llm-bootstrap-dev` extension
- `delivery-harness`
- `review-gate-harness`

### Claude compatibility

- core MCP
- skill pack
- `delivery-harness`
- `incident-harness`

### Full orchestrator

- core MCP
- provider-native pack
- `delivery`
- `parallel-build`
- `incident`
- `review-gate`
- `ralph-loop`

## Doctor 방향

앞으로 `doctor`는 파일 존재 여부만 보면 안 된다.

최소한 다음 카테고리를 분리해 보여줘야 한다.

- `runtime`
- `mcp`
- `plugins/extensions/skills`
- `harnesses`
- `agent parity`
- `workflow gates`

예시:

- `ok mcp chrome-devtools`
- `warn harness parallel-build missing on Gemini`
- `warn role security-reviewer has no Claude equivalent`

처음 구현에서는 이 전체를 한 번에 하지 않는 편이 낫다.

최소 doctor 우선순위:

1. `runtime`
2. `mcp`
3. `harness`
4. `agent parity`

`plugins/extensions/skills`는 결국 harness 렌더링 결과이므로, 초기에는 별도 대분류보다
`harness installed/not-installed`로 묶어도 충분하다.

## 구현 우선순위

1. manifest에 `harnesses`, `packs` 추가
2. MCP는 기존 구조를 유지하고 metadata만 보강
3. Codex/Gemini/Claude renderer가 같은 하네스를 다른 표면으로 렌더링하게 변경
4. `delivery`, `parallel-build`, `incident`, `review-gate`, `ralph-loop` 하네스 정의
5. doctor를 최소 category-aware 진단기로 개선
6. 그다음 필요할 때만 `plugins` 독립 카탈로그와 domain-specific MCP 확장

## 한 줄 정리

- Codex는 config/MCP/plugin-first
- Gemini는 extension/settings-first
- Claude는 subagent/MCP/hook-first
- MCP는 공통 카탈로그 + provider-native install
- 하네스는 공통 정의 + provider별 렌더링
- `gstack`은 기본 runtime이 아니라 하네스 contract source
- 가장 가벼운 구현은 `공통 하네스 스펙 1개 + provider renderer`다
