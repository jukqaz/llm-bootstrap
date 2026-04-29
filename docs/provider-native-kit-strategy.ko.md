# Provider Native Kit Strategy

이 문서는 `Codex`, `Gemini`, `Claude Code`를 하나의 동일한 UX로 묶지 않고,
각 provider별 native kit으로 분리해서 운영한다는 결정을 고정한다.

## 결정

`StackPilot`는 세 LLM을 같은 방식으로 쓰게 만드는 제품이 아니다.

앞으로 제품 표면은 아래처럼 나눈다.

- `codex-kit`
  - Codex용 config, AGENTS, MCP, plugin, skills, subagents, agent TOML
- `gemini-kit`
  - Gemini용 settings, extension, commands, GEMINI.md, hooks, MCP
- `claude-kit`
  - Claude Code용 CLAUDE.md, subagents, official MCP CLI, hooks, skills

`StackPilot`는 이 kit들을 설치, 업데이트, 검증, 복구, 마이그레이션하는 umbrella
CLI와 공통 카탈로그로 남긴다.

## 왜 분리하는가

세 provider는 업데이트 속도와 설정 모델이 다르다.

- Codex는 plugin, skill, subagent, MCP, config 조합이 빠르게 바뀐다.
- Gemini는 extension, command, settings, auth shape 변화가 중요하다.
- Claude는 subagent, MCP CLI, hook, user/project scope가 핵심이다.

이 차이를 숨기려고 공통 설정 하나로 퉁치면 두 문제가 생긴다.

1. provider native 장점을 못 쓴다.
2. 업데이트 때마다 공통 추상화가 깨진다.

따라서 목표는 `동일한 설정`이 아니라 `동일한 의도`다.

예:

- "review gate를 켠다"는 의도는 공통이다.
- Codex에서는 skill/plugin/subagent로 간다.
- Gemini에서는 extension command와 GEMINI.md contract로 간다.
- Claude에서는 subagent와 skill/hook contract로 간다.

## gstack 같은 기능의 위치

`gstack`류의 좋은 기능은 별도 LLM runtime이 아니라 workflow/skill/harness
contract로 본다.

즉 `gstack`을 네 번째 provider처럼 복제하지 않는다. 대신 아래 contract를
provider별 kit surface에 렌더링한다.

- office-hours
- plan
- implementation
- review
- QA
- ship
- retro
- record-work
- operating review

이렇게 해야 Codex, Gemini, Claude가 각자 자기 방식으로 같은 업무 흐름을 지원한다.

## 공통으로 둘 것

공통화는 provider 기능을 평평하게 만드는 곳이 아니라, 반복과 검증을 줄이는 곳에만
둔다.

공통 source of truth:

- MCP catalog
- harness contract
- pack/preset metadata
- env-gated key metadata
- backup, restore, uninstall 규칙
- doctor/probe category
- old-tool cleanup registry

공통화하지 않을 것:

- provider config 전체 구조
- plugin/extension/skill packaging 형식
- provider별 agent/subagent 세부 설정
- UI, auth, auto-update, model preference
- provider가 새 버전에서 추가한 runtime state

## 제품 경계

사용자-facing 제품은 이렇게 설명한다.

```text
stackpilot
  ├─ codex-kit
  ├─ gemini-kit
  └─ claude-kit
```

`install --providers codex,gemini`는 두 kit을 함께 설치하는 orchestration일 뿐이다.
하나의 universal config를 만드는 동작이 아니다.

CLI도 이 모델을 따른다. provider별 명령은 `--providers` 선택자로 표현하고,
아직 별도 `stack-pilot codex install` 같은 provider subcommand는 만들지 않는다.

```bash
stack-pilot install --providers codex,gemini
stack-pilot install --providers codex
stack-pilot install --providers gemini
stack-pilot install --providers claude
```

## 디렉터리 방향

당장 repo split은 하지 않는다. 먼저 모노레포 안에서 provider kit 경계를 드러낸다.

권장 방향:

```text
src/                       -> umbrella CLI, state, backup, doctor, probe
src/providers/             -> provider kit renderer implementation
kits/codex/                -> Codex kit source assets
kits/gemini/               -> Gemini kit source assets
kits/claude/               -> Claude kit source assets
addons/stackpilot-dev-kit/        -> provider-neutral workflow source
catalog/                   -> MCP, tool, reference catalog
docs/                      -> product and architecture decisions
```

현재 `templates/{provider}`와 provider별 addon output은 `kits/{provider}`로 이동하는
것이 맞다. 단, 큰 파일 이동은 별도 작업으로 처리한다.

## 릴리스 방향

초기에는 하나의 release artifact를 유지한다.

나중에 아래 조건이 충족되면 kit별 release를 고려한다.

1. provider별 변경 주기가 명확하게 갈라진다.
2. 하나의 release가 불필요한 회귀 검증을 너무 많이 요구한다.
3. 사용자가 특정 provider kit만 설치하는 비율이 높아진다.
4. install script가 kit별 artifact를 안정적으로 고를 수 있다.

그 전까지는 하나의 binary가 세 kit을 관리하는 편이 더 단순하다.

## 구현 순서

1. 문서와 README에서 provider-native kit 모델을 먼저 고정한다.
2. `doctor` 출력에서 provider kit status를 더 명확히 나눈다.
3. `templates/{provider}`를 `kits/{provider}`로 옮기는 migration을 설계한다.
4. manifest의 `surfaces`를 kit별 native surface 기준으로 정리한다.
5. `install`은 기존 provider flag를 유지하되, kit alias를 추가한다.
6. `gstack`류 기능은 공통 harness contract로 두고 kit별 renderer만 다르게 만든다.
7. provider별 release note와 compatibility check를 분리한다.

## 피할 것

- 세 provider를 같은 파일 구조로 맞추기
- Codex plugin 개념을 Gemini와 Claude에 그대로 복제하기
- Gemini extension을 Codex plugin처럼 설명하기
- Claude subagent/MCP/hook 구조를 skill pack으로 덮어버리기
- provider native 기능을 잃어가며 parity 표만 맞추기

## 결론

이 프로젝트의 의미는 `하나의 설정으로 모든 LLM을 통일`하는 데 있지 않다.

의미는 아래에 있다.

- provider별로 가장 좋은 native baseline을 만든다.
- 좋은 workflow 기능은 공통 contract로 관리한다.
- 각 provider에는 자기 방식으로 렌더링한다.
- 설치, 복구, 검증, 마이그레이션은 하나의 umbrella CLI로 안전하게 처리한다.

즉 앞으로의 제품 정의는 `one config for all LLMs`가 아니라
`provider-native kits with shared bootstrap operations`다.
