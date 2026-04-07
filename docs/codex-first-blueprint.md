# Codex-First Blueprint

`llm-bootstrap`는 여러 agent harness의 아이디어를 섞되, 그대로 복제하지 않는다.
목표는 `Codex -> Gemini -> Claude Code` 순으로 지원하는 개발용 bootstrap을 만드는 것이다.
메인라인은 Codex다. 나머지 provider는 Codex-first 설계를 얇게 이식한다.

## Provider 우선순위

1. Codex
2. Gemini
3. Claude Code

원칙:

- richest support는 Codex에 먼저 구현한다
- Gemini는 extension/hook/settings merge 방식으로 최대한 등가 기능만 맞춘다
- Claude Code는 compatibility lane으로 도입하되, Codex 기준 설계를 무너뜨리지 않는다

## 참고 repo에서 가져올 것

### `oh-my-codex`

- Codex home 중심의 실전 baseline
- custom agent role map
- `AGENTS.md` 기반 운영 규율
- 과한 profile 분기보다 하나의 강한 기준선

### `oh-my-gemini-cli` / `oh-my-gemini`

- Gemini extension 중심 설치 구조
- hook과 문서 주입을 분리하는 구조
- `settings.json`을 덮어쓰지 않고 merge 하는 운영 방식

### `oh-my-claudecode`

- Claude Code compatibility lane
- Claude 생태계의 hook/skill/command 자산을 최대한 얇게 재사용하는 접근

### `oh-my-openagent`

- category-to-model routing
- background specialist fan-out
- 계층형 `AGENTS.md` 문맥 주입
- built-in MCP를 적게, 하지만 역할이 분명하게 고르는 방식
- LSP/AST/tooling을 프롬프트가 아니라 실제 실행 수단으로 대우하는 관점

### `oh-my-agent`

- 팀 단위 역할 분리
- provider/model별 specialist lane
- 설치 후 바로 쓰는 opinionated workflow pack

### `OpenHarness`

- open, extensible harness 구조
- role/tool/plugin을 묶는 플랫폼적 시각
- 도구를 기능군 단위로 모듈화하는 관점

### `gstack`

- plan -> review -> qa -> ship 순서의 강한 delivery loop
- 설치형 skill pack 철학
- prompt보다 workflow contract를 중시하는 방식

### `harness/harness`

- 로컬 bootstrap 자체보다, artifact/release/verification discipline 참고용
- 개발 플랫폼 전체를 들여오지 않고, 검증과 릴리즈 계약만 차용

## 가져오지 않을 것

- 거대한 state machine
- session/taskboard 영속 계층
- provider마다 완전히 다른 조작 모델
- default로 과도한 MCP 등록
- giant agent catalog
- 자동 학습, quota-watch, telemetry 성격의 기본 훅

## llm-bootstrap에 실제로 반영할 방향

### Core

- Rust installer
- `bootstrap.toml` manifest
- idempotent apply / doctor
- install / uninstall both available from the CLI
- provider별 backup
- apply mode 분리: `merge` 기본, `replace` opt-in, provider auth/session 상태는 최대한 보존
- 외부 도구는 공식 init 경로가 있으면 그 결과를 우선 사용
- 단일 default baseline
- shared MCP baseline 최소화
- bootstrap 범위는 user/home 설정까지만 유지하고 project-level 설정은 제외
- env가 없는 선택 MCP는 disabled 상태로 두고 생성하지 않음
- secret manager SDK/CLI에 직접 결합하지 않고 env consumer로 유지
- doctor는 blocking missing과 disabled warning을 분리

### Codex mainline

- strongest agent roster
- richest plugin/skill pack
- review/qa/ship artifacts
- app/plugin lane for Figma/Linear
- long-context lane와 fanout 운영 기준

### Gemini lane

- extension 기반 문서/agent pack
- RTK hook
- settings merge
- Codex baseline의 얇은 대응물만 유지

### Claude Code lane

- 이후 단계에서 추가
- 목표는 feature parity가 아니라 compatibility 우선
- Codex 기준 workflow를 Claude Code 문맥으로 번역하는 수준에서 시작

## 모듈 구조 제안

- `core`
  - provider apply/doctor
  - backup
  - runtime dependency bootstrap
- `packs/codex-main`
  - Codex agents
  - Codex plugin
  - review/qa/ship docs
- `packs/gemini-bridge`
  - Gemini extension
  - Gemini hooks
  - Gemini agent docs
- `packs/claudecode-bridge`
  - Claude Code AGENTS/hooks/commands compatibility
- `integrations/context7`
- `integrations/exa`
- `integrations/browser-devtools`
- `integrations/figma`
- `integrations/linear`

## 현재 기준으로 채택할 기본값

- `context7`
- `exa`
- `chrome-devtools`
- Codex plugin
- Gemini extension
- Codex review/qa/ship pack
- Gemini QA pack

## 다음 구현 우선순위

1. Codex-centric pack 분리
2. Gemini parity pack 정리
3. Figma/Linear integration lane 추가
4. Claude Code compatibility lane 추가
5. role routing / workflow pack 고도화
