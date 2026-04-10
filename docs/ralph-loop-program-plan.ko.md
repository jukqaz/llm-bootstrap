# Ralph Loop 프로그램 계획

이 문서는 현재 `llm-bootstrap` backlog를 몇 개의 하네스 lane으로 고정하고,
각 lane을 같은 Ralph loop로 반복 처리하기 위한 실행 계획서다.

핵심 의도는 단순하다.

- 매 턴 backlog를 다시 섞지 않는다
- 항상 next slice를 명시한다
- 한 번에 하나의 bounded lane을 끝까지 민다

## 프로그램 목표

baseline 안정성을 약하게 만들지 않으면서 enablement 레이어를 키운다.

## 완료 기준

현재 프로그램은 각 하네스 lane이 아래를 모두 만족할 때만 "done"으로 본다.

1. contract가 안정적이다
2. provider-facing surface와 코드가 일치한다
3. 최소 검증이 통과한다
4. 같은 lane 안의 blocker가 남아 있지 않다

## Ralph Loop

각 lane은 같은 순서로 돈다.

1. bounded goal 고정
2. 가장 작은 유효 slice 구현
3. drift와 중복 검토
4. 최소 credible QA 수행
5. lane을 떠나기 전에 next slice 재기록

## 하네스 Lane

### 1. `workflow-control-plane`

상태: `in-progress`

Goal:
- 얇은 local workflow contract가 phase 이동을 실제로 통제하도록 만든다

Scope:
- `phase-gate`
- `task-state`
- `review-gate`
- `ralph-retry`
- `record` handoff 정렬

이미 끝난 것:
- `phase-gate`가 이제 `plan -> execute -> review -> qa -> ship`을 다룬다
- ship 쪽 review gate는 이미 동작한다

이 lane 안에 아직 남은 것:
- `ralph-retry` 추가
- `task-state`와 `record`를 더 직접 연결
- gate 결과가 항상 resumable next action을 내도록 정리

Verification:
- `cargo test`
- `task-state`, `gate` 대상 CLI smoke
- install 및 provider surface refresh

Next slice:
- `ralph-retry`를 넣고 반복 실패를 task-state evidence에 연결한다

### 2. `repo-automation-lane`

상태: `in-progress`

Goal:
- review와 release gate를 optional repository automation lane으로 옮긴다

Scope:
- `pr-review-gate`
- `release-readiness-gate`
- branch protection contract
- optional scaffold command

이미 끝난 것:
- scaffold 명령이 있다
- workflow template가 있다
- 이 repo가 생성된 gate 파일을 직접 dogfood한다

이 lane 안에 아직 남은 것:
- PR checklist와 repo contract ergonomics 보강
- 수동 GitHub 설정 문서를 더 명확히 정리
- 실제 PR flow와 release-dispatch flow를 한 번씩 검증

Verification:
- `cargo test`
- YAML parse check
- scaffold dry-run / real-run 확인
- 가능할 때 실제 GitHub Actions 검증 1회

Next slice:
- repo-facing checklist 또는 template 지원을 추가하고 첫 live run을 검증한다

### 3. `entrypoint-layer`

상태: `next`

Goal:
- 짧고 즉시 이해되는 entrypoint를 통해 제품이 바로 실행 가능하게 느껴지도록 만든다

Scope:
- `autopilot`
- `team`
- `office-hours`
- `operating-review`
- mode / lane naming 정리

왜 중요한가:
- 현재 저장소는 pack과 harness는 강하다
- 반면 "지금 무엇으로 시작할까?"는 아직 약하다

Verification:
- provider surface diff 검토
- install / doctor
- 최소 help / usage smoke

Next slice:
- entrypoint contract를 먼저 고정하고 entrypoint 간 중복을 줄인다

### 4. `precision-loop`

상태: `next`

Goal:
- 무거운 runtime 없이도 edit -> verify -> commit 흐름을 더 날카롭게 만든다

Scope:
- tighter verification guidance
- retry behavior
- 더 작은 결과 요약
- commit ergonomics

Verification:
- 직접 CLI 또는 shell smoke
- 문서와 명령 예시 일치 여부

Next slice:
- 새 명령을 늘리기 전에 precision contract의 최소 단위를 먼저 정의한다

### 5. `company-live-loop`

상태: `later`

Goal:
- company-operation connector를 단순 앱 목록이 아니라 live operating surface로 바꾼다

Scope:
- `Linear`, `Gmail`, `Calendar`, `Drive`, `Figma` health/auth surface
- founder / ops handoff 품질
- channel / inbox model 준비

Verification:
- `doctor --json`
- connector별 runtime handoff 확인
- 외부 source-of-truth를 가리키는 record-first evidence

Next slice:
- operating loop를 늘리기 전에 connector health surface 기대값을 먼저 정의한다

## 순서

프로그램은 아래 순서로 진행한다.

1. `workflow-control-plane`
2. `repo-automation-lane`
3. `entrypoint-layer`
4. `precision-loop`
5. `company-live-loop`

실무적으로 이렇게 가야 하는 이유는 의존성 때문이다.

- `workflow-control-plane`이 제어 계약이다
- `repo-automation-lane`은 그 계약 위에서 돈다
- `entrypoint-layer`는 안정된 기반 동작을 노출해야 한다
- `precision-loop`는 안정된 실행면을 더 다듬는 일이다
- `company-live-loop`는 그 위에 올라가는 운영 레이어다

## 현재 범위에서 뺄 것

지금 프로그램 범위에는 아래를 넣지 않는다.

- heavy runtime orchestration
- provider-specific wow defaults
- giant mode catalog
- repo workflow generation 기본 경로
- auto-commit 기본 동작
