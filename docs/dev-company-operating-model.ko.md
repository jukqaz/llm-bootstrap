# Dev + Company Operating Model

이 문서는 `llm-bootstrap`를 "개발용 bootstrap"에서
"개발 + 회사운영 operating system bootstrap"으로 확장할 때의 기준 모델을 정리한다.

핵심은 두 가지를 동시에 만족하는 것이다.

- 개발 실행이 강해야 한다
- 회사 운영도 같은 베이스 위에서 돌아가야 한다

단, 둘을 한 덩어리로 섞지 않는다.
공통 제어층은 공유하되, 업무 pack과 connector는 분리해서 올린다.

## 최종 목표

`llm-bootstrap`의 최종 목표는 아래처럼 고정한다.

> provider-native surface 위에 공통 harness를 설치하고,
> 그 위에 개발 pack과 회사운영 pack을 조합해
> 작은 팀의 개발, 제품, 운영 루프를 같이 실행할 수 있게 한다.

즉 제품은 단순 `coding bootstrap`이 아니다.
또 그렇다고 거대한 universal runtime을 직접 만드는 것도 아니다.

정확한 포지션은:

- 작은 핵심 bootstrap core
- 공통 harness layer
- 개발 pack
- 회사운영 pack
- connector pack
- automation layer

## 공통 원칙

### 1. core는 계속 작고 안전하게 유지

core가 담당하는 것은 여전히 아래다.

- user-home 설치
- backup / restore
- provider-native config 렌더링
- 최소 MCP baseline
- doctor

core에는 회사운영 도메인 로직을 직접 넣지 않는다.

### 2. 개발과 운영은 같은 control layer를 공유

개발과 회사운영은 완전히 다른 제품이 아니라,
같은 팀이 같은 제어 방식으로 돌리는 업무다.

공통으로 가져갈 것은 아래다.

- harness
- artifact contract
- approval boundary
- verification gate
- automation metadata

### 3. 업무 기능은 pack으로 분리

같은 runtime 위에서 돌아가더라도 pack은 분리해야 한다.

- `delivery-pack`
- `founder-pack`
- `growth-pack`
- `support-pack`
- `ops-pack`
- `finance-pack`

이렇게 해야 lean baseline을 유지하면서도 기능은 풍부하게 가져갈 수 있다.

### 4. 외부 시스템은 connector pack으로만 붙인다

회사운영 기능은 connector 없이는 성립하지 않는다.
하지만 기본 설치에 다 넣으면 무거워진다.

따라서:

- 기본선에는 connector를 거의 넣지 않는다
- 업무 pack이 필요로 할 때만 connector pack을 활성화한다
- read-only / write-capable / approval-needed를 metadata로 나눈다

## 목표 구조

### Layer 0. Bootstrap Core

역할:

- provider 홈 baseline 설치
- backup / restore
- 최소 MCP wiring
- RTK와 같은 near-core 도구 보장
- provider renderer 실행

대표 파일:

- [bootstrap.toml](../bootstrap.toml)
- [src/manifest.rs](../src/manifest.rs)
- [src/providers/codex.rs](../src/providers/codex.rs)
- [src/providers/gemini.rs](../src/providers/gemini.rs)
- [src/providers/claude.rs](../src/providers/claude.rs)

### Layer 1. Harness Layer

모든 업무의 공통 제어층이다.

최소 하네스:

- `ralph-loop`
- `delivery`
- `parallel-build`
- `incident`
- `review-gate`
- `founder-loop`
- `operating-review`

각 하네스는 최소한 아래를 가진다.

- team topology
- role ownership
- handoff schema
- stop rule
- verification rule
- final artifact contract

### Layer 2. Work Packs

여기서부터 업무가 갈린다.

#### 개발 pack

- `delivery-pack`
- `release-pack`
- `incident-pack`

포함 예시:

- office hours
- build plan
- implement
- review
- QA
- ship
- retro

#### 회사운영 pack

- `founder-pack`
- `growth-pack`
- `support-pack`
- `ops-pack`
- `finance-pack`

포함 예시:

- founder review
- market scan
- pipeline review
- support digest
- KPI review
- operating retro
- finance check

### Layer 3. Connector Packs

업무 pack를 실제 데이터와 연결하는 층이다.

최소 connector category:

- communication
  - Gmail
  - Calendar
  - Slack
- knowledge
  - Drive
  - Docs
  - Notion
- product delivery
  - GitHub
  - Linear
  - Figma
- customer
  - CRM
  - helpdesk
- commerce / finance
  - Stripe
  - billing
  - accounting export
- analytics
  - GA4
  - Search Console
  - ads / attribution

각 connector metadata는 최소 아래를 가져야 한다.

- tool source
- required scope
- read/write level
- approval requirement
- automation allowed 여부
- fallback

### Layer 4. Automation Layer

반복 운영을 닫는 층이다.

최소 automation:

- daily founder brief
- weekly operating review
- weekly market scan
- weekly pipeline review
- daily support digest
- release readiness
- KPI summary

원칙:

- current-run evidence 우선
- 쓰기 작업은 approval boundary 필요
- thread inbox에 결과 남김
- "다음 액션"이 보이는 output contract 유지

## Pack 구분

### 개발 pack

#### `delivery-pack`

목표:

- 아이디어에서 배포까지 개발 루프를 닫는다

대표 artifact:

- `Build Plan`
- `Risk Register`
- `QA Report`
- `Ship Decision`

필요 하네스:

- `ralph-loop`
- `delivery`
- `review-gate`

#### `incident-pack`

목표:

- 장애와 회귀를 빠르게 triage하고 복구한다

대표 artifact:

- `Incident Summary`
- `Root Cause Note`
- `Fix Verification`

필요 하네스:

- `incident`
- `review-gate`

### 회사운영 pack

#### `founder-pack`

목표:

- 제품 방향, 시장, 우선순위를 좁힌다

대표 artifact:

- `Opportunity Brief`
- `Wedge Decision`
- `Next Slice Plan`

필요 하네스:

- `founder-loop`
- `operating-review`

#### `growth-pack`

목표:

- acquisition과 funnel 문제를 본다

대표 artifact:

- `Growth Review`
- `Funnel Diagnosis`
- `Outreach Plan`

#### `support-pack`

목표:

- 고객 이슈와 churn risk를 빠르게 요약한다

대표 artifact:

- `Support Digest`
- `Escalation Note`
- `Churn Risk Summary`

#### `ops-pack`

목표:

- 팀 운영, KPI, cross-function review를 묶는다

대표 artifact:

- `Weekly Operating Review`
- `KPI Narrative`
- `Ops Retro`

#### `finance-pack`

목표:

- 매출, 비용, 정산 관련 체크를 운영 루프에 붙인다

대표 artifact:

- `Finance Check`
- `Revenue Snapshot`
- `Collection Risk Note`

## 구현 순서

순서는 반드시 아래처럼 간다.

### Phase 1. 공통 제어층 고정

- `source catalog` 유지
- `harness catalog` 설계
- `pack catalog` 설계
- provider renderer가 공통 metadata를 읽을 구조 준비

이 단계에서 아직 회사운영 connector를 붙이지 않는다.

### Phase 2. 개발 pack 우선 구현

먼저 아래를 구현한다.

- `ralph-loop`
- `delivery-pack`
- `review-gate`
- `incident-pack`

이 단계에서 품질 기준과 doctor 구조를 먼저 안정화한다.

### Phase 3. founder / ops pack 추가

그다음 회사운영 최소 pack을 올린다.

- `founder-pack`
- `ops-pack`

이 둘이 들어오면 "개발 + 회사운영"이 처음으로 한 제품 위에서 같이 돈다.

### Phase 4. connector metadata 추가

우선순위:

1. GitHub
2. Gmail
3. Calendar
4. Drive
5. CRM / helpdesk
6. finance / analytics

### Phase 5. automation 연결

우선순위:

1. weekly operating review
2. daily founder brief
3. support digest
4. pipeline review
5. release readiness

## 지금 바로 할 일

현재 저장소 기준 다음 작업이 맞다.

1. `harness catalog` 문서와 스키마 정의
2. `pack catalog` 문서와 스키마 정의
3. `delivery-pack`와 `founder-pack`를 첫 구현 대상으로 고정
4. `doctor`에 `harness`와 `pack` 상태 개념 추가

## 하지 말아야 할 것

- business connector를 기본 설치에 넣기
- provider-native surface를 덮는 공통 runtime 만들기
- 개발과 운영 pack를 하나로 섞기
- state-heavy taskboard를 먼저 넣기
- automation부터 먼저 만드는 것

## 최종 판단

`llm-bootstrap`는 앞으로 아래 구조로 가야 한다.

- 작은 bootstrap core
- 공통 harness layer
- 개발 pack
- 회사운영 pack
- connector pack
- automation layer

즉 "개발용 도구"가 아니라,
"개발과 회사운영을 같이 돌릴 수 있는 operating base"가 되어야 한다.

