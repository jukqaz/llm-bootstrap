# 참고 레포 채택 분석

기준일: `2026-04-13`

> 제품 경계는 [product-goal.ko.md](product-goal.ko.md),
> core/addon 경계는 [monorepo-boundary.ko.md](monorepo-boundary.ko.md),
> 현재 source 분류는 [reference-coverage.ko.md](reference-coverage.ko.md)를 따른다.

이 문서는 "다른 레포에서 무엇을 가져와야 `StackPilot`가 실제로 더 강해지는가?"를
현재 README와 공식 문서 기준으로 다시 정리한 채택 분석이다.

목적은 단순 수집이 아니다.

- 무엇을 가져올지
- 어디에 붙일지
- 왜 지금 필요한지
- 무엇은 복사하지 않을지

를 우선순위까지 포함해 고정한다.

## 한 줄 결론

`oh-my` 급 체감을 만들려면, 이제 필요한 것은 "더 많은 레포"가 아니다.

필요한 것은 아래 여덟 가지 능력을 정확히 채택하는 것이다.

1. 강한 entrypoint alias
2. hook/gate enforcement
3. resumable context and checkpoint
4. precision-first 실행 루프
5. repo automation contract
6. browser QA와 tool onboarding
7. company inbox/control-plane 관점
8. context/rule/MCP profile 조립 능력

## 분석 방법

이번 분석은 아래 현재 소스를 직접 다시 본 뒤 정리했다.

- [`oh-my-codex`](https://github.com/junghwaYang/oh-my-codex)
- [`oh-my-gemini`](https://github.com/richardcb/oh-my-gemini)
- [`oh-my-claudecode`](https://github.com/Yeachan-Heo/oh-my-claudecode)
- [`gstack`](https://github.com/garrytan/gstack)
- [`GitHub Spec Kit`](https://github.com/github/spec-kit)
- [`GitHub Agentic Workflows`](https://github.github.com/gh-aw/)
- [`githubnext/agentics`](https://github.com/githubnext/agentics)
- [`Cline`](https://github.com/cline/cline)
- [`Roo Code`](https://github.com/RooCodeInc/Roo-Code)
- [`Continue`](https://github.com/continuedev/continue)
- [`Aider`](https://github.com/Aider-AI/aider)
- [`OpenClaw`](https://github.com/openclaw/openclaw)
- [`Repomix`](https://github.com/yamadashy/repomix)
- [`Coding Context CLI`](https://github.com/kitproj/coding-context-cli)
- [`MCPM`](https://github.com/pathintegral-institute/mcpm.sh)
- [`GitHub Copilot CLI`](https://github.com/github/copilot-cli)
- [`OpenHands`](https://github.com/OpenHands/OpenHands)
- [`Crush`](https://github.com/charmbracelet/crush)
- [`AgentAPI`](https://github.com/coder/agentapi)
- [`Caveman`](https://github.com/JuliusBrussee/caveman)

## P0. 바로 가져와야 할 것

### 1. 강한 entrypoint alias layer

출처:

- `oh-my-codex`
- `oh-my-claudecode`
- `Roo Code`
- `gstack`

왜 중요한가:

- `oh-my-*`가 강하게 느껴지는 첫 이유는 기능 수보다
  `autopilot`, `team`, `ralph`, `review`처럼 바로 기억되는 진입점 때문이다.
- 지금 저장소는 `pack`은 강하지만, 사용자가 바로 손에 쥐는 alias 표면이 약하다.

가져올 것:

- 짧고 강한 실행 alias
- mode 성격이 바로 보이는 naming
- provider마다 같은 intent를 가진 entrypoint 이름

이 저장소에 붙일 위치:

- [`bootstrap.toml`](../bootstrap.toml): preset/pack alias metadata
- [`addons/stackpilot-dev-kit/`](../addons/stackpilot-dev-kit): Codex/Gemini/Claude surface별 alias 자산
- [`README.ko.md`](../README.ko.md): 사용자-facing 진입점 설명

복사하지 않을 것:

- `omx`, `omc`, `omg` 같은 별도 runtime CLI 자체
- giant mode catalog 전체

### 2. hook/gate enforcement

출처:

- `oh-my-gemini`
- `gstack`
- `GitHub Agentic Workflows`

왜 중요한가:

- "기억해" 식 지시는 결국 drift가 난다.
- 현재 저장소도 `phase-gate`, `ralph-retry`, `review-gate`는 있지만,
  `all-in-one`을 강하게 만들려면 gate가 더 전면에 나와야 한다.

가져올 것:

- deterministic phase gate
- retry semantics
- human approval 경계
- ship 직전 review/qa/verify 강제

이 저장소에 붙일 위치:

- [`src/main.rs`](../src/main.rs)
- [`src/state.rs`](../src/state.rs)
- [`addons/stackpilot-dev-kit/`](../addons/stackpilot-dev-kit): provider surface별 gate entrypoint

복사하지 않을 것:

- GitHub Actions runtime 전체
- Gemini hook stack 전체를 다른 provider에 억지 이식하는 것

### 3. resumable context / checkpoint / conductor-lite

출처:

- `oh-my-gemini`
- `Spec Kit`
- `Cline`
- `Roo Code`

왜 중요한가:

- 지금 저장소의 약점은 "실행 중 상태"가 약하다는 점이다.
- `task-state`는 시작점은 있지만, 아직 체감 persistence가 강하지 않다.

가져올 것:

- spec -> plan -> tasks 흐름
- task checkpoint
- resumable summary
- session 재진입용 compact state

이 저장소에 붙일 위치:

- [`src/state.rs`](../src/state.rs)
- [`src/main.rs`](../src/main.rs)
- [`docs/operating-record-model.ko.md`](operating-record-model.ko.md)
- provider별 `record-work`, `ralph-plan`, `task-state` 표면

복사하지 않을 것:

- Conductor 전체
- editor snapshot machinery 전체

### 4. precision-first execution loop

출처:

- `Aider`
- `gstack`

왜 중요한가:

- `oh-my`류의 체감은 강하지만, 실제 반복 사용에서는
  작은 edit/verify/commit 루프가 더 중요하다.
- `Aider`는 표면이 얇아도 실제 개발 강도가 높다는 점이 중요하다.

가져올 것:

- edit -> test -> review -> ship의 짧은 루프
- git-centric verification discipline
- commit 전에 evidence를 남기는 흐름

이 저장소에 붙일 위치:

- `delivery-pack`
- `incident-pack`
- `review-gate`
- `ship-check`

복사하지 않을 것:

- auto-commit 기본값
- provider-native CLI를 대체하는 단일 실행기

### 5. repo automation contract

출처:

- `Continue`
- `GitHub Agentic Workflows`
- `githubnext/agentics`

왜 중요한가:

- review, CI diagnosis, release-readiness는 로컬 세션 안에서만 닫히지 않는다.
- 현재 저장소는 `repo-automation scaffold`가 있지만, 채택 기준이 더 선명해져야 한다.

가져올 것:

- markdown/frontmatter 기반 workflow contract
- PR/release gate를 data로 유지하는 방식
- repo-specific check names와 lane 분리

이 저장소에 붙일 위치:

- `review-automation-pack`
- `internal repo-automation scaffold`
- `.github/stackpilot/*` generated contract

복사하지 않을 것:

- repo 파일 생성 강제
- GitHub Actions 중심 runtime을 core로 승격하는 것

### 6. browser QA와 tool onboarding

출처:

- `Cline`

왜 중요한가:

- 개발 완료 체감은 코드 생성보다 검증에서 갈린다.
- `Cline`의 강점은 editor 자체보다 browser loop와 MCP add-a-tool 감각이다.

가져올 것:

- browser QA entrypoint
- tool onboarding 설명 표면
- verification timeline 사고

이 저장소에 붙일 위치:

- `delivery-pack`
- `qa-browser`
- docs와 wizard

복사하지 않을 것:

- editor extension runtime
- GUI workflow 전체

## P1. 다음으로 가져올 것

### 7. company inbox/control-plane 관점

출처:

- `OpenClaw`

가져올 것:

- connector를 app 목록이 아니라 inbox/channel 단위로 보는 관점
- channel health와 account routing

이 저장소에 붙일 위치:

- `founder-pack`
- `ops-pack`
- `CONNECTORS.md`
- runtime handoff 문서

복사하지 않을 것:

- gateway daemon
- 실제 메시징 runtime

### 8. context/rule assembly

출처:

- `Coding Context CLI`
- `Repomix`
- `Gitingest`

가져올 것:

- rule vs task 분리
- repo ingest lane
- remote triage용 light digest

이 저장소에 붙일 위치:

- optional docs/context pack
- future `repo-intake` lane

복사하지 않을 것:

- heavy context pack을 baseline 기본값으로 두는 것

### 9. MCP profile management

출처:

- `MCPM`

가져올 것:

- profile 기반 MCP 사고
- client integration과 global catalog 분리

이 저장소에 붙일 위치:

- `doctor`
- source catalog
- optional MCP lane

복사하지 않을 것:

- 별도 MCP package manager를 core 의존성으로 넣는 것

### 10. terse mode

출처:

- `Caveman`

가져올 것:

- terse 출력 제약 아이디어
- commit/review terse helper 감각

이 저장소에 붙일 위치:

- 지금은 source catalog의 reference-only 항목
- 반복 수요가 생기면 공통 capability catalog의 옵션

복사하지 않을 것:

- terse를 기본 응답 정책으로 강제하는 것
- Caveman을 별도 pack으로 바로 설치하는 것

## P2. watch only

아래는 추적은 하되, 지금 직접 채택 우선순위는 낮다.

- `GitHub Copilot CLI`
  - GitHub-native terminal runtime 참고용
- `OpenHands`
  - heavy remote execution platform 참고용
- `Goose`
  - local runtime 참고용
- `Crush`
  - terminal runtime 참고용
- `AgentAPI`
  - universal adapter 참고용
- `mini-swe-agent`
  - minimal loop 교훈 참고용
- `Kelos`
  - sandbox orchestration 참고용
- `MCP-Universe`
  - evaluation harness 참고용

이들은 현재 `StackPilot`에 기능을 바로 붙이기보다,
제품 경계를 흔들 때만 다시 검토하면 된다.

## 절대 복사하지 않을 것

다른 레포를 보더라도 아래는 계속 금지한다.

1. editor extension runtime 전체
2. tmux worker orchestration 전체
3. gateway daemon 전체
4. provider auth/session을 대체하는 persistence runtime
5. project repo scaffold를 기본 install 경로로 강제하는 것
6. giant specialist catalog를 baseline core에 넣는 것

## 실제 구현 순서

지금 기준 권장 순서는 아래다.

1. `all-in-one` preset에 맞는 alias/entrypoint 정리
2. `task-state`를 conductor-lite 수준으로 보강
3. `review-automation-pack`를 markdown contract 중심으로 강화
4. `qa-browser`와 tool onboarding을 더 전면에 노출
5. `founder-pack` / `ops-pack`에 channel/inbox 관점을 추가

즉 다음 단계는 "레포를 더 찾는 것"이 아니라,
"이미 확인한 참고 레포에서 뽑을 것들을 P0부터 실제 코드로 옮기는 것"이다.

## 최종 판단

제대로 분석하면 결론은 단순하다.

- `oh-my-*`에서 가져와야 할 것은 강한 진입점, gate, persistence 체감이다
- `gstack`, `Spec Kit`, `Aider`, `Continue`에서 가져와야 할 것은 실행 규율이다
- `OpenClaw`, `MCPM`, `Coding Context CLI`에서 가져와야 할 것은 상위 운영 모델이다

즉 `StackPilot`가 강해지려면
"모든 레포를 조금씩 복사"하는 게 아니라,
"P0 능력을 강하게 채택해서 all-in-one 표면을 실제로 살리는 것"이 맞다.
