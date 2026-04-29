# Business Ops Blueprint

`StackPilot`를 `gstack`나 `oh-my-codex`와 같은 개발 워크플로 도구로만 보지 않고,
사업 전반을 보조하는 운영 레이어까지 확장하려면 제품 정의를 한 단계 바꿔야 한다.

이 문서는 현재 저장소의 실제 책임 범위를 기준으로, 어떤 기능을 추가해야
"개발 bootstrap"에서 "business-capable operating system"으로 올라갈 수 있는지
정리한다.

## 현재 상태

현재 `StackPilot`는 다음에 집중한다.

- provider 홈 baseline 설치와 복구
- 최소 MCP baseline 관리
- workflow 문서와 경량 skill/agent bundle 배포
- `install`, `doctor`, `restore`, `uninstall`, `wizard` 같은 안전한 운영 명령

현재 저장소는 의도적으로 다음을 하지 않는다.

- 프로젝트 taskboard나 session memory 운영
- 대규모 state machine
- 과도한 기본 MCP 등록
- business connector 기본 탑재
- 매출, 고객지원, 마케팅, 세일즈 같은 business surface 통합

즉 현재 제품은 `AI runtime`이 아니라 `AI workbench bootstrap`에 가깝다.

## 외부 비교

### gstack

공개 README 기준 `gstack`는 개발 스프린트 전체를 강한 프로세스로 묶는다.

- `office-hours -> plan -> review -> qa -> ship -> retro`
- 제품 정의, 설계 검토, DX 검토, 보안, QA, 릴리즈를 하나의 chain으로 운영
- 단계별 산출물이 다음 단계 입력으로 이어지는 artifact-first 방식
- founder/product framing까지 다루는 상위 레이어

강점:

- 개발과 제품 판단 사이의 간격이 짧다
- "무엇을 만들지"와 "어떻게 검증할지"를 함께 강제한다
- 출시 전후 점검 루프가 명확하다

한계:

- 기본적으로 software sprint 중심이다
- connector와 권한 체계를 직접 제품화한 것은 아니다
- business data plane보다 개발 프로세스 plane이 더 강하다

### oh-my-codex

공개 README 기준 `oh-my-codex`는 OpenAI Codex CLI용 multi-agent orchestration에
초점을 둔다.

- 다수의 specialist agent
- mode 기반 라우팅
- 병렬 실행
- session persistence
- model/reasoning auto-routing

강점:

- 큰 개발 작업을 병렬/오케스트레이션으로 밀어붙이기 좋다
- executor, tester, reviewer 같은 역할 분리가 명확하다
- Codex 단독 사용보다 런타임 운영층이 두껍다

한계:

- business-wide connector와 artifact contract는 상대적으로 약하다
- 제품/시장/운영 기능보다 coding runtime이 중심이다
- cross-provider home bootstrap보다는 Codex 중심 실행기에 가깝다

### StackPilot 현재 위치

`StackPilot`는 둘보다 낮은 층이다.

- 장점: 안전한 baseline, backup/restore, provider 간 일관성, 최소 MCP
- 약점: 실제 일을 수행하는 운영 루프가 얇다
- 결과: 체감상 "환경은 정리해주지만 사업 전체를 돌리지는 못한다"로 보인다

이 평가는 자연스럽다. 현재 제품 정의가 그렇게 되어 있기 때문이다.

## 문제 정의

사용자가 원하는 것은 단순한 개발 bootstrap이 아니다.

원하는 것은 아래를 한 시스템에서 다루는 것이다.

- 제품 정의
- 개발 실행
- 디자인 검토
- QA와 릴리즈
- 시장 조사
- 세일즈 follow-up
- 고객지원 triage
- 운영 리포트
- 주간 회고

즉 "코드를 잘 쓰는 에이전트"가 아니라 "작은 팀의 운영체계"가 필요하다.

## 목표 제품 정의

`StackPilot`의 다음 단계 목표는 아래처럼 정의한다.

> 개발자 홈 baseline을 안전하게 재현하는 core 위에,
> 역할별 workflow pack과 business connector pack을 조합해
> 작은 팀의 개발, 제품, 운영 루프를 실행할 수 있게 하는 cross-provider OS

핵심은 기존 bootstrap core를 버리지 않고 상위 레이어를 쌓는 것이다.

## 설계 원칙

- core와 ops를 분리한다
- 기본 설치는 계속 lean 하게 유지한다
- business 기능은 preset이나 pack으로 opt-in 한다
- provider native surface를 최대한 활용한다
- business connector는 명시적 권한과 scope를 가져야 한다
- 자동화는 결과를 남기되 hidden state 의존은 최소화한다
- 산출물은 다음 단계 입력으로 재사용 가능해야 한다
- 개발 외 업무도 "artifact contract" 중심으로 표준화한다

## 목표 아키텍처

### Layer 0: Bootstrap Core

현재 저장소가 담당하는 층이다.

- provider 설치/복구
- backup/restore
- 최소 MCP wiring
- env-gated MCP
- native docs/skills/commands 배포
- doctor와 runtime dependency 보장

이 층은 계속 보수적으로 유지한다.

### Layer 1: Workflow Packs

개발과 제품 운영의 playbook 모음이다.

후보 pack:

- `founder-pack`
- `delivery-pack`
- `launch-pack`
- `support-pack`
- `growth-pack`
- `ops-pack`

각 pack은 문서와 skill, command, agent roster, output contract를 함께 가진다.

예시:

- `founder-pack`: problem framing, market scan, positioning, weekly review
- `delivery-pack`: office hours, plan, implement, review, qa, ship
- `launch-pack`: release brief, launch checklist, post-launch watch
- `support-pack`: ticket triage, incident summary, churn-risk digest
- `growth-pack`: campaign review, funnel diagnostics, creator outreach
- `ops-pack`: KPI review, weekly operating review, cross-function retro

### Layer 2: Connector Packs

사업 전체를 하려면 외부 시스템이 반드시 필요하다.

최소 connector category:

- communication: Gmail, Calendar, Slack
- knowledge: Drive, Docs, Notion
- product delivery: GitHub, Linear, Figma
- customer: Intercom, Zendesk, HubSpot
- commerce and finance: Stripe, Shopify, accounting export
- analytics and growth: GA4, Search Console, Ads platforms

각 connector는 다음 계약을 가져야 한다.

- 어떤 MCP/app/tool을 쓰는지
- 필요한 권한 scope가 무엇인지
- 읽기 전용인지 쓰기 가능한지
- 자동 실행 허용 여부
- 실패 시 fallback이 무엇인지

### Layer 3: Business Automations

반복 운영은 automation으로 닫아야 한다.

후보 automation:

- daily founder brief
- weekly market scan
- weekly pipeline review
- daily support digest
- release readiness check
- post-launch canary review
- monthly KPI narrative

이 층의 원칙:

- 판단은 current-run evidence를 우선한다
- 외부 시스템에 쓰는 작업은 명시적 승인 경계를 둔다
- 자동화는 thread inbox에 결과를 남긴다
- "아무것도 없음"보다 다음 액션이 보이는 output contract를 유지한다

## 기능 축

### 1. Product Strategy

필요 기능:

- founder office hours
- problem statement refinement
- opportunity sizing
- competitor compare
- pricing hypothesis review
- roadmap slicing

대표 산출물:

- `Opportunity Brief`
- `Wedge Decision`
- `Next Slice Plan`

### 2. Delivery

필요 기능:

- office hours
- implementation plan
- eng review
- design review
- QA
- ship checklist

대표 산출물:

- `Build Plan`
- `Risk Register`
- `QA Report`
- `Ship Decision`

### 3. Go-To-Market

필요 기능:

- launch plan
- campaign asset checklist
- channel experiment review
- creator or partner outreach drafting
- landing page critique

대표 산출물:

- `Launch Brief`
- `Campaign Review`
- `Landing Page Audit`

### 4. Sales and Customer Success

필요 기능:

- lead digest
- meeting prep
- follow-up draft
- pipeline health review
- churn-risk review

대표 산출물:

- `Pipeline Review`
- `Meeting Brief`
- `Follow-up Draft`

### 5. Support and Operations

필요 기능:

- issue triage
- incident summary
- refund or dispute prep
- VOC summarization
- weekly operating review

대표 산출물:

- `Support Digest`
- `Incident Review`
- `VOC Summary`
- `Weekly Ops Review`

## Pack 중심 제품 구조 제안

현재 저장소의 preset 철학을 유지하려면 "기능 팩" 단위가 맞다.

### 권장 preset

- `light`
  - bootstrap core only
- `normal`
  - core + delivery baseline
- `full`
  - core + delivery + founder baseline
- `business`
  - full + connector packs + automation templates

`business`는 기본값이 되면 안 된다.
권한, 비용, 환경 의존성이 크기 때문이다.

### 권장 pack 구조

- `packs/delivery`
- `packs/founder`
- `packs/growth`
- `packs/support`
- `packs/ops`
- `packs/connectors`

pack 내부는 공통적으로 아래를 가진다.

- docs
- skills
- commands
- agents
- automation templates
- permission manifest

## Artifact Contract 제안

개발 외 업무도 결과 형식을 강하게 고정해야 재사용이 된다.

예시 contract:

### Founder Office Hours

- `Problem:`
- `User Pain:`
- `Current Workaround:`
- `Why Now:`
- `Narrowest Wedge:`
- `Next Experiment:`

### Market Scan

- `Signal:`
- `Why It Matters:`
- `Implication:`
- `Recommended Move:`
- `Evidence:`

### Pipeline Review

- `Top Deals:`
- `Stalled Deals:`
- `Risks:`
- `Actions This Week:`

### Support Digest

- `Top Issues:`
- `Affected Users:`
- `Severity:`
- `Immediate Action:`
- `Longer-term Fix:`

## 권한과 보안 설계

사업 전체로 확장할수록 보안 모델이 중요해진다.

필수 원칙:

- connector별 read/write 권한 분리
- tool별 approval boundary 명시
- 고객 데이터와 재무 데이터는 high-sensitivity로 분류
- 자동 외부 쓰기는 opt-in
- home bootstrap repo에는 실제 시크릿을 저장하지 않음
- `doctor`에서 권한 누락과 미연결 connector를 별도 경고로 분리

## 경쟁 포지셔닝

### gstack 대비

`gstack`이 강한 이유는 process 강제력이다.
따라서 `StackPilot`는 단순히 skill 개수를 따라가면 안 된다.

대신 아래로 차별화해야 한다.

- cross-provider baseline과 home-state safety
- lean default + pack opt-in
- connector와 permission manifest를 제품 표면으로 승격
- 개발뿐 아니라 founder, growth, support, ops까지 같은 구조로 확장
- automation과 connector를 포함한 "work OS" 제공

### oh-my-codex 대비

`oh-my-codex`가 강한 이유는 orchestration runtime이다.

차별화 포인트:

- Codex 한정 runtime이 아니라 provider-neutral bootstrap
- multi-agent runtime보다 operating model과 connector pack에 집중
- business artifact와 recurring automation 기본 제공
- home-level install, migration, restore 품질을 유지

## 단계별 로드맵

### Phase 0

문서와 패키징 기준선 정리

- 새 blueprint 문서 추가
- pack/preset naming 고정
- 현재 scope와 future scope 분리

### Phase 1

개발+founder 레이어 도입

- `founder-pack` 초안
- `delivery-pack` 재구성
- `business` preset 초안
- founder artifact contract 추가

### Phase 2

connector pack 도입

- Gmail, Calendar, Drive, GitHub 우선
- permission manifest 포맷 도입
- connector doctor/report 추가

### Phase 3

support/growth/ops pack 도입

- support digest
- pipeline review
- weekly ops review
- launch brief

### Phase 4

automation productization

- recurring templates
- approval boundary
- run history summary
- pack별 recommended automation bundle

## 저장소 영향

이 방향으로 가면 저장소 구조도 약간 바뀐다.

예상 변경:

- `bootstrap.toml`에 preset/pack metadata 확장
- `src/manifest.rs`와 provider renderer가 pack-aware 해져야 함
- `templates/`와 `plugins/` 아래에 business pack 아티팩트 추가
- README와 wizard가 preset 설명을 더 명확히 가져야 함
- `doctor`가 connector/permission 상태를 보고해야 함

하지만 core installer는 유지한다.
핵심은 런타임을 새로 만드는 것이 아니라 배포 단위를 확장하는 것이다.

## 성공 기준

- 사용자가 개발 외 업무도 같은 진입점으로 실행할 수 있다
- 기본 설치는 여전히 가볍고 안전하다
- connector 권한과 자동화 경계가 명확하다
- 산출물이 다음 단계 입력으로 재사용된다
- `gstack` 대비 process, `oh-my-codex` 대비 runtime이 아니라
  `business-capable operating layer`로 인식된다

## 추천 다음 작업

1. `business` preset과 pack manifest 스키마를 먼저 설계한다.
2. `founder-pack`를 첫 vertical slice로 만든다.
3. connector는 `Gmail`, `Calendar`, `Drive`, `GitHub` 네 개만 먼저 붙인다.
4. automation은 `weekly market scan`, `pipeline review`, `support digest` 세 개만 먼저 연다.

## 참고

- [gstack](https://github.com/garrytan/gstack)
- [oh-my-codex](https://github.com/junghwaYang/oh-my-codex)
