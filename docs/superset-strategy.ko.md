# llm-bootstrap 슈퍼셋 전략

이 문서는 참고 레포와 공식 문서에서 "좋은 것만" 가져와
`llm-bootstrap`용 슈퍼셋 구조를 정의한다.

핵심 목표는 두 가지다.

1. 좋은 요소는 합친다
2. 중복과 짬뽕은 막는다

즉 "기능을 많이 넣는 것"이 아니라
"같은 기능을 여러 이름과 구조로 중복 구현하지 않는 것"이 목적이다.

## 한 줄 원칙

> source of truth는 하나만 두고, 나머지는 모두 파생물로 만든다.

여기서 유일한 중심축은 다음이다.

- `preset`
- `pack`
- `harness`
- `provider-native surface`

다른 것은 모두 이 축에서 파생돼야 한다.

- mode
- skill
- command
- hook gate
- MCP profile
- company loop
- review automation

## 참고 레포에서 최종 채택할 것

### 하네스

- `gstack`
  - 채택: `office-hours -> plan -> review -> qa -> ship` artifact chain
  - 버릴 것: runtime 전체
  - 이유: 우리에게 필요한 것은 process contract이지 host runtime이 아니다

- `oh-my-claudecode`
  - 채택: `team-plan -> team-prd -> team-exec -> team-verify -> team-fix`
  - 버릴 것: tmux worker runtime 기본값
  - 이유: staged pipeline은 좋지만 bootstrap core가 worker orchestrator가 되면 안 된다

- `oh-my-gemini`
  - 채택: `phase-gate`, `ralph-retry`
  - 버릴 것: Conductor 전체 state system
  - 이유: gate는 얇게 가져오고, 상태 계층은 최소화해야 한다

### 멀티에이전트

- `oh-my-codex`
  - 채택: 짧은 진입점, parallel verify/fix 감각
  - 버릴 것: 큰 agent catalog, heavy orchestration runtime
  - 이유: 실행 진입점은 필요하지만 giant runtime은 범위를 넘는다

- `OpenClaw`
  - 채택: control plane과 worker runtime 분리 사고
  - 버릴 것: multi-channel gateway runtime
  - 이유: 회사운영 connector를 더 잘 설계하는 데만 참고하면 충분하다

### 스킬 / 진입점

- `Roo Code`
  - 채택: `architect`, `ask`, `build` 같은 짧은 mode naming 감각
  - 버릴 것: broad mode catalog
  - 이유: 진입점은 짧고 명확해야 하지만 새 mode 체계를 또 만들면 중복이다

- `Aider`
  - 채택: precision-first, git-centric, 작은 실행면
  - 버릴 것: auto-commit default
  - 이유: 실행면을 얇게 유지하는 데 가장 좋은 기준점이다

### MCP / 툴 / 자동화

- `Cline`
  - 채택: browser validation 진입점, `add a tool` UX
  - 버릴 것: editor snapshot runtime
  - 이유: validation UX는 필요하지만 editor runtime은 다른 층이다

- `Continue`
  - 채택: markdown-defined PR checks
  - 버릴 것: repo-level generated workflow files 기본값
  - 이유: review-gate advanced lane에는 좋지만 bootstrap core에는 과하다

## 최종 슈퍼셋 구조

### Layer 1. Core

core는 지금 저장소의 정체성이다.

- bootstrap installer
- provider-native renderer
- backup / restore / uninstall / doctor
- `preset -> pack -> harness -> apps/MCP/surface`
- 최소 MCP baseline

여기에 새 runtime을 넣지 않는다.

### Layer 2. Execution

execution은 core에서 파생되는 얇은 실행면이다.

- `autopilot`
- `team`
- `office-hours`
- `review`
- `qa`
- `ship`
- `operating-review`

중요:

- 별도 mode 시스템을 만들지 않는다
- entrypoint는 모두 `pack`과 `harness`에서 파생된다
- provider마다 native surface로만 렌더링한다

예:

- Codex: skill / subagent / AGENTS guidance
- Gemini: command / hook / extension docs
- Claude: subagent / hook / workflow skill

### Layer 3. Advanced

advanced는 optional lane이다.

- task-state
- repo automation lane
- company live loop
- optional team runtime
- channel/control-plane modeling

이 계층은 core를 건드리지 않고 opt-in으로만 붙는다.

### Layer 4. Never

다음은 기본 제품에 넣지 않는다.

- giant mode catalog
- tmux worker runtime 기본값
- multi-channel gateway runtime
- editor extension runtime 자체
- repo-level generated workflow files 기본값
- auto-commit default
- 큰 session DB / telemetry / memory system

## 중복 제거 규칙

### 규칙 1. mode는 독립 개념이 아니다

mode를 새 source of truth로 만들지 않는다.

- 허용: `autopilot`이라는 entrypoint 이름
- 금지: `pack`과 별개인 독립 mode taxonomy

정리:

- mode는 `entrypoint alias`
- source of truth는 여전히 `pack + harness`

### 규칙 2. skill, command, hook은 같은 contract의 다른 표면이다

같은 기능을 세 번 쓰지 않는다.

예:

- `review-gate`
  - Codex에서는 skill
  - Gemini에서는 command + hook
  - Claude에서는 workflow skill + hook

하지만 내부 contract 이름은 하나만 둔다.

### 규칙 3. connector를 업무 계약으로 고정한다

둘 중 하나가 중심이 돼야 한다.

현재 기준:

- `connector`가 업무 의미 단위
- `app`은 `tool_source = app` connector에서 파생되는 구현 source

즉 pack은 connector를 들고, 사용자-facing app 목록은 파생해서 보여준다.

### 규칙 4. task-state는 install-state와 분리한다

현재 `state`는 설치 상태용이다.

- preset drift
- pack drift
- managed paths

나중에 추가할 task-state는 별도 계층이어야 한다.

- `track`
- `plan`
- `status`

이 둘을 합치면 bootstrap이 runtime DB가 된다.

### 규칙 5. repo automation은 advanced다

`Continue`, `GitHub Agentic Workflows`에서 가져올 것은 좋지만,
repo-level generated workflow를 기본값으로 넣지 않는다.

즉:

- user-home bootstrap core
- repo automation advanced lane

이 둘을 섞지 않는다.

## 지금 backlog에 넣을 것

### P1

1. `entrypoint layer`
   - `autopilot`, `team`, `office-hours`, `operating-review`
   - 단, 모두 `pack`에서 파생

2. `hook gate`
   - `phase-gate`
   - `review-gate`
   - `ralph-retry`

3. `company live loop`
   - `Linear`, `Gmail`, `Calendar`, `Drive`, `Figma` health/auth surface

### P2

4. `task-state`
   - 얇은 `track/spec/plan/status`

5. `review automation`
   - markdown-defined PR checks

6. `precision loop`
   - edit / verify / commit ergonomics

### P3

7. `team runtime` optional lane
8. `channel/control-plane` company modeling

## 지금 backlog에서 내려야 할 것

다음은 문서에 있더라도 "바로 구현" 대상에서 내리는 편이 맞다.

- giant mode system
- Conductor급 memory layer
- tmux worker runtime 기본 도입
- OpenClaw식 gateway
- Continue식 repo artifact 기본 생성

## 최종 판단

좋은 것만 합친 슈퍼셋은 가능하다.

하지만 전제는 하나다.

> 모든 좋은 아이디어를 같은 층에 넣지 않는다.

정리하면:

- `core`는 계속 bootstrap
- `execution`은 얇은 진입점
- `advanced`는 opt-in
- `never`는 명시적으로 금지

이 구조를 지키면 짬뽕이 아니라 잘 정리된 슈퍼셋이 된다.
