# 참고 레포 surface 매트릭스

> 최상위 목표와 제품 정의는 [product-goal.ko.md](product-goal.ko.md)를 따른다.
> 참고 레포 backlog 전체는 [reference-repo-backlog.ko.md](reference-repo-backlog.ko.md)를 따른다.
> 이 문서는 레포별 "surface"를 기준으로 무엇을 가져올지 고정한다.

`llm-bootstrap`의 다음 단계는 단순 기능 수집이 아니다.

- provider-native baseline을 유지하면서
- 다른 레포의 강한 surface를 capability로 흡수하고
- bootstrap core를 runtime product처럼 두껍게 만들지 않는 것

이 문서는 그 판단을 `plugin`, `extension`, `skill`, `command`, `hook`, `MCP`, `apps`,
`CI check`, `gateway/channel` 같은 surface 단위로 정리한다.

## 요약 판단

가져올 가치가 큰 surface는 아래 여섯 가지다.

1. 강한 entrypoint 이름과 mode alias
2. hook gate와 deterministic workflow
3. 얇은 task-state와 resumable context
4. review/QA를 repo automation lane으로 미는 계약
5. MCP 확장 UX와 tool catalog 정리
6. company connector를 app 목록이 아니라 inbox/channel로 보는 관점

반대로 core에 바로 넣으면 안 되는 것도 분명하다.

1. editor runtime 전체
2. tmux worker orchestration 전체
3. gateway daemon 전체
4. provider-native auth/session을 대체하는 persistence runtime
5. repo 파일 생성을 기본값으로 강제하는 구조

## 레포별 surface 판단

### 1. oh-my-codex

- 링크: [junghwaYang/oh-my-codex](https://github.com/junghwaYang/oh-my-codex)
- 보이는 surface:
  - terminal CLI `omx`
  - mode keyword surface: `autopilot`, `ultrawork`, `team`, `ralph`, `plan`, `review`, `debug`
  - agent catalog
  - skill pack
  - tool catalog
  - session persistence
- 바로 가져올 것:
  - mode alias naming
  - `autopilot`, `team`, `ralph`, `review` 같은 entrypoint 노출 방식
  - skill catalog를 surface별로 잘게 나누는 방식
- capability로만 둘 것:
  - optional `team runtime`
  - optional persistent execution lane
- 안 가져올 것:
  - 자체 multi-agent runtime 전체
  - provider-native surface를 덮는 독자 실행기

판단:

`oh-my-codex`는 `Codex CLI` 위에 orchestration runtime을 얹는 제품이다.
이 저장소는 같은 방향으로 가지 않고,
`entrypoint layer`와 `team-pack`만 얇게 흡수하는 것이 맞다.

### 2. oh-my-gemini

- 링크: [richardcb/oh-my-gemini](https://github.com/richardcb/oh-my-gemini)
- 보이는 surface:
  - Gemini extension
  - slash command
  - hook stack: `session-start`, `before-agent`, `tool-filter`, `before-tool`, `after-tool`, `phase-gate`, `ralph-retry`
  - Conductor context layer
  - memory MCP/tool surface
  - policy file
- 바로 가져올 것:
  - `phase-gate`, `ralph-retry` 개념
  - hook를 workflow contract로 쓰는 방식
  - keyword 기반 deterministic mode selection
  - plan/spec/task를 얇게 유지하는 context 구조
- capability로만 둘 것:
  - Gemini 전용 `hook-pack`
  - optional `task-state` / `track-state`
  - optional memory lane
- 안 가져올 것:
  - Node/TypeScript runtime 전체
  - Conductor 전체를 baseline 필수요소로 두는 구조

판단:

`oh-my-gemini`는 가장 강한 `hook-enforced workflow` 참고 레포다.
이 저장소가 가져와야 하는 것은 hook script 자체보다
`gate contract`와 `retry semantics`다.

### 3. oh-my-claudecode

- 링크: [Yeachan-Heo/oh-my-claudecode](https://github.com/Yeachan-Heo/oh-my-claudecode)
- 보이는 surface:
  - Claude marketplace plugin install
  - npm CLI/runtime path
  - in-session skills
  - terminal CLI `omc`
  - native team workflow vs tmux worker runtime 이원화
- 바로 가져올 것:
  - plugin path와 runtime path를 분리해서 설명하는 방식
  - in-session skill과 terminal command를 분리하는 표면 설계
  - `deep-interview`처럼 요구사항 정리를 먼저 시키는 진입점
- capability로만 둘 것:
  - optional `team runtime`
  - optional `deep-interview` lane
- 안 가져올 것:
  - tmux worker runtime 전체
  - plugin/runtime 이중 설치를 baseline 기본값으로 두는 구조

판단:

`oh-my-claudecode`는 surface 분리가 가장 명확하다.
이 저장소도 `baseline 설치 surface`와 `capability 실행 surface`를 더 분명히 나눠야 한다.

### 4. Cline

- 링크: [cline/cline](https://github.com/cline/cline)
- 보이는 surface:
  - VS Code extension
  - MCP tool 확장
  - terminal + browser + screenshot + console GUI loop
  - file diff/timeline
- 바로 가져올 것:
  - browser verification 진입점
  - "tool을 추가한다"는 MCP 확장 UX
  - 작업 중 diff/timeline을 보여주는 설명 방식
- capability로만 둘 것:
  - optional browser QA pack
  - optional task snapshot view
- 안 가져올 것:
  - editor extension 전체
  - human-in-the-loop GUI를 baseline core 요구사항으로 두는 것

판단:

`Cline`은 editor UX가 아니라
`browser QA`와 `tool onboarding` 감각만 가져오면 된다.

### 5. Roo Code

- 링크: [RooCodeInc/Roo-Code](https://github.com/RooCodeInc/Roo-Code)
- 보이는 surface:
  - VS Code extension
  - mode system: `Code`, `Architect`, `Ask`, `Debug`, `Custom Modes`
  - checkpoints
  - context management
  - MCP usage
- 바로 가져올 것:
  - mode naming
  - custom mode packaging 관점
  - checkpoint를 얇은 상태 레이어로 보는 감각
- capability로만 둘 것:
  - optional custom mode pack
  - optional checkpoint/task-state
- 안 가져올 것:
  - extension runtime 전체
  - broad mode catalog 자체를 baseline으로 노출하는 것

판단:

`Roo Code`는 기능보다 이름을 잘 짓는 레포다.
이 저장소는 `pack` 내부의 복잡도를 그대로 노출하지 말고
짧은 capability 이름으로 번역해야 한다.

### 6. Continue

- 링크: [continuedev/continue](https://github.com/continuedev/continue)
- 보이는 surface:
  - CLI
  - repo-local `.continue/checks/`
  - PR status check
  - CI-enforced markdown agent contract
- 바로 가져올 것:
  - review/QA/security 계약을 markdown file로 두는 방식
  - `review-gate`를 repo automation lane으로 미는 방식
- capability로만 둘 것:
  - optional `review-automation-pack`
  - optional `ci-check-pack`
- 안 가져올 것:
  - repo-level check file 생성을 bootstrap 기본값으로 강제하는 것

판단:

`Continue`는 baseline reference가 아니라
`advanced repo lane` reference다.
사용자가 원할 때만 project repo에 내려보내는 방식이 맞다.

### 7. Aider

- 링크: [Aider-AI/aider](https://github.com/Aider-AI/aider)
- 보이는 surface:
  - terminal CLI
  - codebase map
  - git integration
  - lint/test loop
  - IDE bridge
- 바로 가져올 것:
  - precision-first 실행 루프
  - git-centric verification/commit ergonomics
  - 최소 표면 실행 모델
- capability로만 둘 것:
  - optional `precision-pack`
  - optional commit helper lane
- 안 가져올 것:
  - auto-commit 기본값
  - provider-native surface를 대체하는 단일 CLI

판단:

`Aider`는 "작은 surface가 실제로 강하다"는 기준점이다.
이 저장소의 새 capability도 Aider처럼 얇고 명확해야 한다.

### 8. OpenClaw

- 링크: [openclaw/openclaw](https://github.com/openclaw/openclaw)
- 보이는 surface:
  - onboarding CLI
  - gateway daemon
  - channel integrations
  - assistant control plane
  - long-lived inbox/channel runtime
- 바로 가져올 것:
  - company connector를 inbox/channel 단위로 보는 관점
  - onboarding/wizard가 계정, 채널, skill을 묶어 설명하는 방식
  - control plane과 worker/runtime를 분리하는 사고
- capability로만 둘 것:
  - optional `company-inbox-pack`
  - optional `channel health` surface
- 안 가져올 것:
  - gateway daemon 전체
  - 다채널 메시징 runtime 자체

판단:

`OpenClaw`는 개발 툴 참고 레포가 아니라
나중에 회사 운영 capability가 커질 때의 구조 참고 레포다.

## surface별 채택 규칙

### baseline에 남길 것

- provider-native plugin / extension / skill / command packaging
- 공식 MCP 등록 경로
- env-gated MCP
- 최소 workflow docs
- backup / restore / doctor / uninstall

### capability로만 둘 것

- mode alias / entrypoint layer
- hook gate
- task-state / checkpoint / track-state
- review automation / CI checks
- company inbox / channel health
- optional team runtime

### 기본값으로 넣지 않을 것

- editor extension runtime 전체
- daemon / gateway runtime 전체
- repo-local automation file 생성 강제
- provider auth/session 대체 persistence
- giant mode catalog

## 지금 backlog에 추가할 일

### P1

1. `hook-pack`
   - Gemini 중심
   - `phase-gate`, `ralph-retry`, `after-tool verification` 계약만 먼저 도입

2. `team-pack`
   - Codex/Claude optional lane
   - `team`, `deep-interview`, `ralph` 같은 강한 entrypoint를 capability로 분리

3. `surface metadata`
   - manifest에 각 surface의 `kind`, `runtime_owner`, `default_lane` 추가

### P2

4. `review-automation-pack`
   - Continue식 markdown contract를 optional repo lane으로 제공

5. `task-state`
   - Roo/Cline/OMG 참고
   - install-state와 분리된 얇은 resumable state

6. `company-inbox-pack`
   - OpenClaw 참고
   - Gmail/Calendar/Drive/Slack류를 "channel"로 다시 모델링

## 한 줄 결론

레퍼런스 레포에서 그대로 복사해야 하는 것은 거의 없다.
대신 반드시 흡수해야 하는 것은 있다.

- `oh-my-codex`: entrypoint와 mode 체감
- `oh-my-gemini`: hook gate와 task-state
- `oh-my-claudecode`: plugin/runtime 이원화
- `Cline`/`Roo Code`: tool UX와 mode naming
- `Continue`: repo automation lane
- `Aider`: 얇은 실행 루프
- `OpenClaw`: channel/control-plane 관점

즉 다음 단계는 "더 많은 기능 추가"가 아니라
"어떤 surface를 baseline에 두고 어떤 surface를 capability로 분리할지"
제품 구조를 더 명확히 만드는 것이다.
