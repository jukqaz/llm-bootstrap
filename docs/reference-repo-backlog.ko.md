# 참고 레포 확장 백로그

> 최신 제품 기준선과 capability 구조는
> [capability-os-strategy.ko.md](capability-os-strategy.ko.md)를 따른다.
> 이 문서는 개별 참고 레포의 backlog 판단을 보관하는 보조 문서다.

이 문서는 기존 `oh-my-*`, `gstack`, `spec-kit`, `GitHub Agentic Workflows` 외에
추가로 살펴본 참고 레포를 `llm-bootstrap` 계획에 어떻게 반영할지 정리한다.

목적은 단순 수집이 아니다.

- 계획에 넣을 것
- 참고만 할 것
- core에 넣지 않을 것

을 분리해서 backlog로 고정한다.

## 이번에 추가로 본 레포

- `OpenClaw`
- `Cline`
- `Roo Code`
- `Continue`
- `Aider`

## 핵심 판단

현재 저장소는 `preset -> pack -> harness -> apps/MCP/surface` 구조가 강하다.
추가 참고 레포가 주는 보완점은 대부분 아래 다섯 가지다.

1. 강한 실행 진입점
2. browser or runtime validation
3. task-state or snapshot
4. workflow-as-code review gate
5. multi-channel or control-plane 사고

## 레포별 평가

### 1. OpenClaw

- 링크: [openclaw/openclaw](https://github.com/openclaw/openclaw)
- 가져올 것:
  - multi-channel inbox 사고
  - channel/account 기준 agent routing
  - control plane과 worker runtime 분리
- 보류:
  - founder/ops pack이 실제 외부 채널로 확장될 때
- 안 가져올 것:
  - 메시징 gateway 자체를 bootstrap core에 넣는 것

판단:

`llm-bootstrap`가 나중에 회사운영 pack을 더 키울 때, connector를 단순 앱 목록이 아니라
"account / channel / inbox entrypoint"로 볼 수 있게 해주는 참고 레포다.

### 2. Cline

- 링크: [cline/cline](https://github.com/cline/cline)
- 가져올 것:
  - browser validation 진입점 설계
  - task snapshot / restore 아이디어
  - "add a tool" 식 MCP 확장 UX
- 보류:
  - editor runtime과 강하게 묶인 부분
- 안 가져올 것:
  - workspace snapshot machinery를 기본 home bootstrap state로 넣는 것

판단:

지금 저장소의 `doctor`, `backup`, `restore`는 설치 상태 중심이다.
여기에 나중에 작업 상태를 아주 얇게 추가할 때, Cline의 task snapshot 감각이 도움이 된다.

### 3. Roo Code

- 링크: [RooCodeInc/Roo-Code](https://github.com/RooCodeInc/Roo-Code)
- 가져올 것:
  - mode naming 아이디어
  - `architect`, `ask`, `build`처럼 역할보다 성격을 먼저 드러내는 진입점
  - custom mode packaging 감각
- 보류:
  - editor extension 중심 UX
- 안 가져올 것:
  - broad mode catalog를 pack/harness와 별개 축으로 늘리는 것

판단:

현재 저장소는 `preset`은 강하지만, `실행 진입점`은 약하다.
Roo Code는 `mode`를 더 짧고 직관적으로 노출하는 쪽에서 참고 가치가 있다.

### 4. Continue

- 링크: [continuedev/continue](https://github.com/continuedev/continue)
- 가져올 것:
  - markdown-defined PR checks
  - review-gate를 repository automation lane으로 옮기는 아이디어
  - security or QA gate를 contract file로 두는 방식
- 보류:
  - repo-level workflow-as-code를 기본 bootstrap에 넣는 것
- 안 가져올 것:
  - project repo 파일 생성을 기본값으로 강제하는 것

판단:

현재 `review-gate`는 user-home 하네스로 존재한다.
나중에 advanced lane이 생기면 Continue처럼 PR check contract를 별도 markdown으로 관리하는 방향이 좋다.

### 5. Aider

- 링크: [Aider-AI/aider](https://github.com/Aider-AI/aider)
- 가져올 것:
  - precision-first loop
  - git-centric commit ergonomics
  - 최소 표면 실행 모델
- 보류:
  - auto-commit behavior
- 안 가져올 것:
  - provider-native lane을 대체하는 단일 runtime

판단:

Aider는 `기능이 적어도 실제 개발에서 강한 도구`라는 점이 중요하다.
현재 저장소가 실행 진입점을 추가하더라도, Aider처럼 표면을 얇게 유지해야 한다는 교훈이 있다.

## backlog 편입

### P1

1. `entrypoint layer` 추가
   - 후보: `autopilot`, `team`, `office-hours`, `operating-review`
   - 참고: `oh-my-*`, `Roo Code`

2. `hook gate` 추가
   - 후보: `phase-gate`, `review-gate`, `ralph-retry`
   - 참고: `oh-my-gemini`

3. `company live loop` 강화
   - 후보: `Linear`, `Gmail`, `Calendar`, `Drive`, `Figma` health/auth surface
   - 참고: `OpenClaw`

### P2

4. `task-state` 계층
   - 후보: `track/spec/plan/status`
   - 참고: `Cline`, `oh-my-gemini`, `spec-kit`

5. `review-gate`의 repo automation lane
   - 후보: PR check contract
   - 참고: `Continue`, `GitHub Agentic Workflows`

6. `precision loop` 보완
   - 후보: 얇은 edit/verify/commit 흐름
   - 참고: `Aider`

### P3

7. `company connector`를 inbox/channel 단위로 재해석
   - 참고: `OpenClaw`

8. `team runtime` optional lane
   - 참고: `oh-my-claudecode`, `oh-my-codex`

## 지금 넣지 않을 것

- multi-channel gateway runtime
- editor extension runtime 자체
- giant mode catalog
- repo-level generated workflow files 기본값
- auto-commit default

## 한 줄 결론

추가 참고 레포를 보고 나니 방향은 더 선명하다.

- core는 계속 `bootstrap + provider-native renderer`
- 보완은 `entrypoint + hook gate + task-state + review automation`
- 회사운영 확장은 `connector list`보다 `channel/control-plane` 사고를 더 가져간다

즉 다음 backlog는 "기능을 많이 늘리자"가 아니라,
"지금 구조 위에 실행면을 얇게 더 얹자"가 맞다.
