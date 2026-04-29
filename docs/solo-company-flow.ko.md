# Solo Company Flow

이 문서는 실제 1인회사를 운영할 때 `StackPilot`가 어떤 순서로 도움을 줘야
하는지 정의한다.

목표는 "회사 기능 목록"을 만드는 것이 아니다.
시장에서 출발해 아이디어, 검증, 제품화, 출시, 성장, CS, 운영으로 이어지는
실제 업무 흐름을 capability와 harness로 고정하는 것이다.

## 한 줄 목표

> 한 사람이 AI들과 함께 시장을 읽고, 아이디어를 고르고, 제품을 만들고,
> 고객을 얻고, 지원하고, 숫자로 판단하며, 회사를 계속 운영할 수 있어야 한다.

## Lean Operating Principle

이 문서의 stage는 모두 구현해야 하는 절차 목록이 아니다.
실제 제품은 가능한 한 가볍게 둔다.

기본 판단 순서:

1. LLM이 대화 안에서 바로 처리할 수 있으면 별도 harness로 만들지 않는다
2. 기존 SaaS나 automation tool이 잘하는 일은 직접 구현하지 않고 handoff한다
3. 반복되고, 실수 비용이 높고, artifact가 남아야 하는 일만 capability로 만든다
4. 돈, 고객 발송, 법무, 보안, 개인정보, 채용, 계약은 approval boundary를 둔다

`StackPilot`가 직접 해야 하는 것:

- baseline 설치와 복구
- provider-native command/skill 배포
- 업무별 artifact contract
- 필요한 connector와 runtime handoff 표시
- doctor readiness

`StackPilot`가 직접 하지 않을 것:

- CRM, analytics, helpdesk, accounting, ads platform runtime 구현
- 고객 이메일이나 광고를 자동 발송
- 결제, 환불, 계약, 법무 판단 자동 실행
- 모든 사업 기능을 giant process로 강제
- 매번 긴 PRD/spec를 요구

따라서 실제 UX는 "지금 필요한 다음 행동"을 고르는 방식이어야 한다.
문서의 12개 stage는 구현 후보 catalog이며, 기본 wizard는 더 짧은 흐름으로 보여준다.

복잡한 업무는 한 번에 끝내려 하지 않는다.
stage가 바뀌거나 외부 tool로 넘어갈 때는
[operating-record-model.ko.md](operating-record-model.ko.md)의 record contract를 남긴다.
record는 다음 실행을 재개하기 위한 결정, 다음 행동, 근거, 외부 링크만 담고,
project management, CRM, helpdesk, docs 같은 실제 source of truth는 외부 tool에 둔다.

## 전체 흐름

```text
market discovery
  -> idea funnel
  -> opportunity brief
  -> validation
  -> product strategy
  -> UX/design
  -> technical plan
  -> build
  -> QA
  -> launch
  -> growth
  -> sales/support
  -> operations
  -> company review
  -> next bet
```

이 흐름은 선형으로 한 번 끝나지 않는다.
`company review`가 다시 `market discovery`나 `idea funnel`로 돌아가는 루프다.

일반 사용자가 보는 기본 흐름은 더 짧다.

```text
discover -> decide -> validate -> build -> launch -> learn
```

상세 stage는 advanced mode나 capability 내부에서만 노출한다.

## Build / Delegate Matrix

| 영역 | `StackPilot`가 직접 제공 | LLM에 맡김 | 외부 툴에 맡김 |
|---|---|---|---|
| 시장조사 | `market-scan` artifact contract | 검색 요약, 경쟁사 비교, pain 정리 | 검색엔진, Reddit/HN/X, app review, analytics |
| 아이디어 | `idea-score`, `assumption-map` | 브레인스토밍, 중복 제거, 리스크 정리 | 없음 또는 메모/문서 툴 |
| 검증 | validation checklist | 인터뷰 질문, landing copy 초안 | 폼, landing builder, ads, email tool |
| 제품기획 | product brief contract | PRD 초안, scope cut | issue tracker, docs |
| 디자인 | UX/design QA contract | user flow, copy, critique | Figma, design system |
| 개발 | build/review/QA/ship loop | 코드 작성, 리뷰, 테스트 해석 | GitHub, CI, hosting, monitoring |
| 마케팅 | launch/growth artifact contract | positioning, content, campaign 초안 | SEO tool, analytics, ads, email platform |
| 세일즈 | approval boundary와 account brief contract | outreach/proposal 초안 | CRM, email, calendar |
| CS | triage/repro/reply contract | 답변 초안, feedback clustering | helpdesk, inbox, issue tracker |
| 재무 | finance-lite checklist | runway/가격 시나리오 초안 | Stripe, accounting, spreadsheet |
| 법무/보안 | risk checklist | 문서 검토 보조 | 변호사, security scanner, compliance tool |
| 운영리뷰 | weekly review contract | signal synthesis, next bet 초안 | analytics, CRM, support, accounting |

이 표에서 외부 툴에 맡긴 영역은 `runtime handoff`로 남긴다.
bootstrap이 그 tool의 runtime이 되지 않는다.

## Stage 1. Market Discovery

목적:

- 어떤 시장이 변하고 있는지 본다
- 고객이 돈을 내거나 시간을 쓰는 문제를 찾는다
- 경쟁 제품과 대체재를 파악한다
- 단순 아이디어가 아니라 "왜 지금 가능한가"를 만든다

Harness:

- `market-scan`
- `trend-scan`
- `competitor-map`
- `customer-pain-map`
- `wedge-finder`

Inputs:

- web/search
- Reddit/HN/X/community signal
- GitHub trends
- app store/review data
- existing customer notes
- founder intuition

Artifacts:

- market scan memo
- competitor map
- customer pain list
- wedge candidates
- "why now" note

LLM 도움:

- 검색 결과 요약이 아니라 기회 구조를 만든다
- 경쟁사 기능표보다 고객 pain과 유료 의지를 분리한다
- 너무 넓은 시장을 좁은 wedge 후보로 줄인다

주의:

- X, Reddit 같은 곳은 source of truth가 아니라 signal source다
- 최종 판단은 직접 고객 접촉이나 실제 데이터로 검증해야 한다

## Stage 2. Idea Funnel

목적:

- 아이디어를 많이 만들고
- 같은 아이디어를 합치고
- 검증 가능한 후보만 남긴다

Harness:

- `idea-brainstorm`
- `idea-deduper`
- `idea-score`
- `risk-map`
- `assumption-map`

Inputs:

- market scan memo
- customer pain list
- founder constraints
- available skills/resources
- time budget

Artifacts:

- idea backlog
- scored opportunity list
- assumption map
- kill criteria
- top 3 opportunity brief

LLM 도움:

- 아이디어를 단순 나열하지 않고 ICP, pain, willingness-to-pay, channel, build cost로 나눈다
- 비슷한 아이디어를 묶고 중복을 제거한다
- "좋아 보임"과 "검증 가능함"을 분리한다

Decision gate:

- top 1 idea를 고르기 전에 top 3를 비교한다
- 각 아이디어마다 가장 위험한 가정 1개를 명시한다

## Stage 3. Validation

목적:

- 만들기 전에 고객, 시장, 채널 가정을 검증한다
- 최소 비용으로 kill/continue 결정을 내린다

Harness:

- `validation-plan`
- `interview-script`
- `landing-test`
- `smoke-test`
- `pricing-test`
- `validation-review`

Inputs:

- top opportunity brief
- assumption map
- target customer hypothesis
- channel hypothesis

Artifacts:

- validation plan
- interview script
- landing page draft
- smoke test checklist
- pricing hypothesis
- validation report
- continue/pivot/kill decision

LLM 도움:

- 고객 인터뷰 질문을 유도질문이 아니게 다듬는다
- landing copy를 포지셔닝별로 여러 버전 만든다
- 검증 결과를 감정이 아니라 evidence로 정리한다

Approval boundary:

- 고객에게 실제 발송할 메시지
- 유료 광고 집행
- 가격/약속이 포함된 문구

## Stage 4. Product Strategy

목적:

- 검증된 기회를 제품으로 바꾼다
- MVP scope와 성공 기준을 정한다
- 하지 않을 것을 명확히 한다

Harness:

- `product-brief`
- `office-hours`
- `prd`
- `scope-cut`
- `roadmap-slice`
- `success-metric`

Inputs:

- validation report
- customer notes
- founder constraints
- available build time

Artifacts:

- product brief
- PRD
- MVP scope
- non-goals
- success metrics
- release slice plan

LLM 도움:

- 모호한 제품 설명을 사용자 workflow와 acceptance criteria로 바꾼다
- MVP를 기능 목록이 아니라 "검증할 행동" 중심으로 자른다
- scope creep를 막는다

Decision gate:

- MVP는 한 고객 행동을 바꿔야 한다
- success metric은 출시 전에 정한다

## Stage 5. UX and Design

목적:

- 사용자가 실제로 쓸 수 있는 흐름을 만든다
- 기능보다 onboarding, empty state, error state, copy를 먼저 점검한다

Harness:

- `ux-flow`
- `information-architecture`
- `screen-map`
- `copy-review`
- `design-qa`
- `accessibility-pass`

Inputs:

- PRD
- target user
- core task
- brand constraints

Artifacts:

- user flow
- screen map
- IA
- UI copy
- design QA checklist
- accessibility checklist

LLM 도움:

- 화면 목록을 사용자의 실제 task flow로 재정렬한다
- 실패/빈상태/모바일을 빠뜨리지 않게 한다
- copy를 설명문이 아니라 행동 중심으로 줄인다

Approval boundary:

- 브랜드 포지셔닝 문구
- 유료 전환 문구
- 법적/개인정보 관련 UI 문구

## Stage 6. Technical Plan

목적:

- 제품 요구를 구현 가능한 architecture와 작업 slice로 바꾼다
- 위험한 변경과 검증 방법을 먼저 정한다

Harness:

- `tech-plan`
- `data-model`
- `api-contract`
- `security-review-plan`
- `test-strategy`
- `task-slice`

Inputs:

- PRD
- UX flow
- existing codebase
- infra constraints

Artifacts:

- technical plan
- data model
- API contract
- task breakdown
- test strategy
- risk register

LLM 도움:

- 구현 전에 코드베이스와 문서 drift를 비교한다
- 큰 기능을 merge 가능한 작은 slice로 나눈다
- test와 rollback을 계획에 포함한다

Decision gate:

- 위험도가 높은 변경은 승인 후 구현한다
- 테스트 없이 ship할 수 없는 부분을 먼저 표시한다

## Stage 7. Build

목적:

- 작은 slice를 실제로 구현한다
- 중간에 방향이 바뀌면 plan으로 되돌아간다

Harness:

- `implement`
- `investigate`
- `parallel-build`
- `code-review-prep`
- `test-fix-loop`

Inputs:

- technical plan
- task slice
- acceptance criteria
- codebase

Artifacts:

- code changes
- test changes
- implementation note
- known gaps

LLM 도움:

- 코드베이스 패턴을 먼저 읽고 구현한다
- 실패 원인을 환경/설정/코드로 분리한다
- unrelated refactor를 막는다

Approval boundary:

- destructive migration
- production credential changes
- external billing or email writes

## Stage 8. QA and Release

목적:

- 실제 사용자 흐름으로 검증한다
- 출시 전 release readiness를 확인한다

Harness:

- `qa`
- `browser-qa`
- `regression-check`
- `release-readiness`
- `ship`
- `rollback-plan`

Inputs:

- build URL or local app
- acceptance criteria
- changed files
- test results

Artifacts:

- QA report
- screenshot/evidence
- bug list
- release note
- rollback checklist
- ship decision

LLM 도움:

- 브라우저에서 실제 클릭과 콘솔/네트워크를 확인한다
- 실패를 재현 가능한 bug로 바꾼다
- release note와 rollback plan을 만든다

Decision gate:

- critical bug가 있으면 ship하지 않는다
- docs/user-facing copy drift를 확인한다

## Stage 9. Launch and Growth

목적:

- 제품을 시장에 내고 사용자를 데려온다
- 채널별 메시지를 테스트한다

Harness:

- `launch-plan`
- `positioning`
- `landing-copy`
- `content-plan`
- `seo-review`
- `campaign-plan`
- `analytics-review`
- `conversion-review`

Inputs:

- product brief
- release note
- target ICP
- channel hypothesis
- analytics

Artifacts:

- launch checklist
- landing copy
- content calendar
- SEO checklist
- campaign brief
- analytics insight
- conversion backlog

LLM 도움:

- 같은 제품을 다른 ICP/채널별 메시지로 바꾼다
- content와 landing을 실제 고객 pain에 맞춘다
- analytics를 다음 실험으로 연결한다

Approval boundary:

- 광고비 지출
- 대량 발송
- 공개 발표
- 가격/환불/보증 표현

## Stage 10. Sales and Customer Success

목적:

- 리드를 찾고
- 제안하고
- onboarding과 retention을 관리한다

Harness:

- `lead-research`
- `account-brief`
- `outreach-draft`
- `objection-handling`
- `proposal-draft`
- `onboarding-plan`
- `activation-review`
- `churn-risk-review`

Inputs:

- ICP
- lead list
- CRM notes
- customer usage data
- support history

Artifacts:

- lead shortlist
- account brief
- outreach draft
- objection response
- proposal draft
- onboarding checklist
- churn risk memo

LLM 도움:

- 무작위 영업이 아니라 계정별 pain과 trigger를 만든다
- follow-up 타이밍과 메시지를 정리한다
- onboarding에서 막힌 지점을 CS와 product backlog로 연결한다

Approval boundary:

- 고객에게 실제 발송
- 계약/가격 제안
- CRM write

## Stage 11. Support and Feedback

목적:

- 고객 문의를 빠르게 분류한다
- 버그와 기능 요청을 제품 개선으로 연결한다

Harness:

- `support-triage`
- `customer-reply`
- `bug-repro`
- `feedback-clustering`
- `help-doc-update`
- `changelog-draft`
- `support-retro`

Inputs:

- email/support inbox
- bug reports
- user sessions
- docs
- changelog

Artifacts:

- support queue summary
- customer reply draft
- repro steps
- issue ticket
- feedback cluster report
- help doc patch
- changelog draft

LLM 도움:

- 감정적인 문의와 실제 버그를 분리한다
- 고객 답변을 빠르게 초안화한다
- 반복 문의를 help doc과 product backlog로 연결한다

Approval boundary:

- 고객 답변 발송
- 환불/보상 약속
- 개인정보가 포함된 데이터 접근

## Stage 12. Company Operations

목적:

- 회사가 어디로 가고 있는지 매주 판단한다
- 제품/성장/지원/재무/리스크를 한 루프로 묶는다

Harness:

- `daily-founder-brief`
- `weekly-operating-review`
- `pipeline-review`
- `finance-lite`
- `risk-review`
- `strategy-memo`
- `decision-log`
- `investor-update`

Inputs:

- product metrics
- revenue/cost data
- pipeline
- support queue
- roadmap
- founder notes

Artifacts:

- daily brief
- weekly operating memo
- KPI review
- runway note
- risk register
- decision memo
- investor update
- next bets

LLM 도움:

- 흩어진 신호를 운영 판단으로 묶는다
- vanity metric과 actionable metric을 분리한다
- 다음 주의 가장 중요한 1~3개 결정을 고른다

Approval boundary:

- 재무 판단
- 투자자/파트너 발송
- 법무/보안 결정
- 채용/계약 결정

## Capability Mapping

```text
market discovery       -> growth, company
idea funnel            -> company, project
validation             -> project, growth
product strategy       -> project
UX/design              -> project
technical plan/build   -> project
QA/release             -> project
launch/growth          -> growth
sales/success          -> sales, success
support/feedback       -> support, project
company operations     -> company
```

초기 capability는 너무 많이 쪼개지 않는다.
구현 단계는 아래처럼 둔다.

### Stage A. MVP Operating Loop

먼저 구현할 capability:

- `project`
- `growth`
- `support`
- `company`

이 단계에서 `sales`, `success`, `analytics`, `finance`, `risk`는 별도 capability가 아니라
`growth`, `support`, `company` 안의 harness로 둔다.

### Stage B. Revenue and Data Loop

사용자가 실제 사업을 운영하기 시작하면 분리한다.

- `sales`
- `success`
- `analytics`
- `finance`
- `risk`

분리 기준:

- 별도 connector가 필요하다
- 별도 approval boundary가 있다
- 별도 weekly artifact가 있다
- 사용자 workflow가 충분히 자주 반복된다

### Stage C. Solo Company OS

최종 alias:

```text
solo-company =
  project
  + growth
  + support
  + sales
  + success
  + analytics
  + finance
  + risk
  + company
```

## Wizard Flow

wizard는 처음부터 기능 목록을 묻지 않는다.
사업 단계와 목적을 먼저 묻는다.
그다음 해당 stage의 기록을 어디에 남길지 묻는다.

기본 wizard는 6단계만 보여준다.

1. 지금 어디에 있나?
   - 발견: 시장/아이디어 탐색
   - 결정: 후보 선택과 scope cut
   - 검증: 고객/landing/smoke test
   - 제작: 제품 설계, 개발, QA
   - 출시: launch, marketing, sales, support
   - 학습: 운영 리뷰와 next bet

advanced wizard에서만 세부 stage를 보여준다.

1. 세부 업무를 고른다면 지금 어디에 있나?
   - 시장/아이디어 탐색
   - 검증
   - MVP 개발
   - 출시 준비
   - 성장/마케팅
   - 고객지원
   - 회사 운영
   - 전체 solo-company

2. 어떤 provider에 적용할까?
   - Codex
   - Gemini
   - Claude

3. baseline부터 맞출까?
   - baseline only
   - baseline + selected capability
   - capability only

4. 필요한 connector는 무엇인가?
   - GitHub
   - Linear
   - Gmail
   - Calendar
   - Drive
   - Figma
   - analytics
   - CRM/support inbox

5. record surface는 어디인가?
   - local docs only
   - GitHub issues + repo docs
   - Linear + docs
   - Drive/Docs + provider runtime
   - CRM/helpdesk handoff

6. 쓰기 전 plan preview를 보여준다.

wizard의 결과는 내부적으로 아래 명령으로 해석된다.

```bash
stack-pilot baseline --providers ...
stack-pilot add project,growth,support,company
stack-pilot doctor
```

## Doctor Readiness

나중에 `doctor`는 회사 운영 준비도를 이렇게 보여줘야 한다.

```text
solo-company readiness:

baseline: ok
project: ok
growth: needs analytics connector
support: needs Gmail or support inbox connector
sales: not installed
finance: manual CSV fallback
company: weekly operating review not scheduled
records: local docs only
retired_tools: clean
```

중요한 것은 `missing`을 실패로만 보지 않는 것이다.
사업 영역에서는 runtime handoff가 정상 상태일 수 있다.

## 구현 우선순위

1. 기본 wizard를 `discover -> decide -> validate -> build -> launch -> learn`로 단순화한다
2. `project` capability는 `build`, `review`, `qa`, `ship`처럼 반복 빈도가 높은 loop부터 구현한다
3. `growth`는 `market-scan`, `positioning`, `launch-plan`, `analytics-review`만 먼저 둔다
4. `support`는 `support-triage`, `bug-repro`, `customer-reply`만 먼저 둔다
5. `company`는 `daily-founder-brief`, `weekly-operating-review`, `decision-log`만 먼저 둔다
6. `doctor`에 readiness와 runtime handoff를 분리해서 보여준다
7. 실제 사용이 반복되는 영역만 `sales`, `success`, `analytics`, `finance`, `risk`로 분리한다
8. 외부 SaaS가 더 잘하는 영역은 connector handoff로만 남긴다
9. record surface는 먼저 `local docs only`와 `GitHub issues + repo docs`만 구현한다

## Minimal First Implementation

첫 구현은 아래 10개 이하로 제한한다.

- `market-scan`
- `idea-score`
- `validation-plan`
- `product-brief`
- `build-plan`
- `review`
- `qa`
- `ship`
- `launch-plan`
- `weekly-operating-review`

여기서도 실제 실행은 가능한 한 LLM 대화와 기존 tool을 사용한다.
`StackPilot`는 이 10개가 provider별 command/skill로 같은 contract를 갖게 만드는 데 집중한다.

## 중복 방지

- `market-research`와 `market-scan`은 하나로 합친다
- `idea-brainstorm`은 growth가 아니라 company/project 사이의 shared harness로 둔다
- `analytics-review`는 초기에 growth 안에 두고, 반복 사용이 많아지면 `analytics` capability로 승격한다
- `sales`와 `success`는 초기에는 growth/support 안의 harness로 둔다
- `finance-lite`와 `risk-review`는 초기에는 company 안에 둔다
- 모든 harness는 하나의 artifact contract만 가진다
- provider별 command/skill은 같은 harness의 renderer output이다
- 1회성 업무는 harness로 만들지 않는다
- 외부 tool이 source of truth인 데이터는 복제하지 않고 링크와 요약만 만든다
