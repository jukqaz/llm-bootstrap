# Capability OS Strategy

> 최상위 목표와 제품 정의는 [product-goal.ko.md](product-goal.ko.md)를 따른다.
> 이 문서는 그 목표를 capability 구조와 명령 모델로 풀어낸다.

이 문서는 `StackPilot`의 다음 제품 기준선이다.

기존 목표는 LLM별 baseline을 같게 맞추는 것이었다.
현재 목표는 다른 프로젝트의 좋은 기능을 중복 없이 합쳐서 프로젝트와 회사를
혼자 운영할 수 있는 작업 효율을 만드는 것이다.

따라서 `StackPilot`는 이제 두 레이어로 나눈다.

1. `baseline`: Codex, Gemini, Claude 홈과 MCP, env, backup, doctor를 안정화한다
2. `capability`: 프로젝트, 마케팅, CS, 회사 운영 능력을 opt-in으로 추가한다

## 제품 정의

`StackPilot`는 여러 LLM CLI의 baseline 설치기를 넘어서,
검증된 LLM workflow 기능을 provider-native surface로 재배포하는 개인 AI 운영체계다.

단, 모든 회사 업무를 직접 구현하는 제품은 아니다.
LLM이 대화 안에서 처리할 수 있는 일은 LLM에 맡기고,
SaaS와 automation runtime이 더 잘하는 일은 connector handoff로 남긴다.
`StackPilot`는 반복되는 workflow contract, provider 배포, readiness 확인만 직접 소유한다.

단계는 아래 순서로 간다.

1. `Project OS`: 프로젝트 하나를 기획, 디자인, 개발, QA, 운영, CS까지 굴린다
2. `Growth/Support OS`: 마케팅, 고객지원, 피드백, 문서화를 붙인다
3. `Company OS`: KPI, 파이프라인, 재무 light review, 전략, 운영 리뷰를 붙인다
4. `Solo Company OS`: 한 사람이 AI들과 함께 회사 전 영역을 운영한다

실제 사업 흐름은 [solo-company-flow.ko.md](solo-company-flow.ko.md)를 기준으로 한다.
이 문서는 capability 구조를 정의하고, `solo-company-flow`는 시장조사에서 회사 운영까지
업무가 이어지는 순서를 정의한다.
복잡한 업무를 이어서 처리하기 위한 기록 계약은
[operating-record-model.ko.md](operating-record-model.ko.md)를 기준으로 한다.

## 명령 모델

기존 `install --preset`은 호환 경로로 남긴다.
새 제품 표면은 아래 명령을 중심으로 둔다.

```bash
stack-pilot baseline
stack-pilot add <capability>
stackpilot remove <capability>
stack-pilot sync
stack-pilot doctor
stack-pilot wizard
```

### `baseline`

LLM 작업 머신으로서의 바닥을 맞춘다.

포함:

- provider root 생성
- 기본 provider 문서 설치
- baseline MCP 설치
- env-gated MCP 활성화
- RTK init
- backup / restore / uninstall / doctor state
- 이전 `omx`, `omg`, `omc`, `oh-my-*` 파일과 env cleanup
- auth, session, conversation history 보존

포함하지 않음:

- project workflow command
- growth/marketing workflow
- support/CS workflow
- company ops automation
- heavy specialist agent catalog

### `add`

업무 능력을 붙인다.

예:

```bash
stack-pilot add project
stack-pilot add growth
stack-pilot add support
stack-pilot add company
stack-pilot add solo-company
```

`add`는 capability를 pack/harness/provider surface로 렌더링한다.
baseline이 없으면 먼저 baseline 적용을 권장한다.

### `remove`

선택한 capability가 만든 surface만 제거한다.
baseline은 유지한다.

### `sync`

현재 state 기준으로 baseline과 active capability를 다시 렌더링한다.
drift를 고치되, provider auth와 history는 건드리지 않는다.

### `wizard`

wizard는 위 명령들의 UI wrapper다.
일반 사용자는 내부 용어 대신 목적을 선택한다.

- baseline only
- project
- growth
- support
- company
- solo-company

advanced mode에서만 pack/harness 세부 선택을 보여준다.

## 내부 모델

source of truth는 하나만 둔다.

```text
capability -> pack -> harness -> artifact contract -> provider surface
```

provider surface는 파생물이다.

- Codex: plugin, skill, AGENTS, agent TOML
- Gemini: extension, command TOML, agent markdown, GEMINI
- Claude: skill, subagent, CLAUDE, official MCP registration

같은 기능을 provider마다 다른 이름으로 재정의하지 않는다.
예를 들어 `qa`는 내부 contract 하나이고, Codex skill, Gemini command,
Claude skill은 모두 그 contract의 renderer output이다.

## Capability Catalog

### `project`

가장 먼저 완성할 capability다.
프로젝트 하나를 end-to-end로 운영한다.

Harness:

- `product-brief`
- `office-hours`
- `ux-design`
- `tech-plan`
- `build-plan`
- `implement`
- `investigate`
- `review`
- `qa`
- `ship`
- `operate`
- `cs-triage`
- `retro`

Artifacts:

- product brief
- UX flow
- technical plan
- implementation plan
- review report
- QA report with evidence
- release note
- operation checklist
- CS triage memo
- retro memo

### `growth`

마케팅과 성장을 담당한다.

Harness:

- `market-scan`
- `idea-brainstorm`
- `idea-score`
- `validation-plan`
- `market-research`
- `positioning`
- `landing-copy`
- `content-plan`
- `seo-review`
- `campaign-plan`
- `analytics-review`
- `conversion-review`
- `launch-plan`

Artifacts:

- market scan memo
- idea backlog
- validation report
- positioning memo
- landing page copy
- content calendar
- SEO checklist
- campaign brief
- analytics insight report

### `support`

고객지원과 피드백 순환을 담당한다.

Harness:

- `support-triage`
- `bug-repro`
- `customer-reply`
- `feedback-clustering`
- `help-doc-update`
- `changelog-draft`
- `support-retro`

Artifacts:

- customer reply draft
- repro steps
- issue ticket
- feedback cluster report
- help doc patch
- changelog draft

### `company`

회사 운영 루프를 담당한다.
초기에는 contract와 handoff 중심으로 둔다.

Harness:

- `daily-founder-brief`
- `weekly-operating-review`
- `pipeline-review`
- `finance-lite`
- `risk-review`
- `hiring-review`
- `investor-update`
- `strategy-memo`
- `decision-log`

Artifacts:

- daily brief
- weekly operating memo
- KPI review
- pipeline report
- investor update
- decision memo
- decision log
- risk register

### `solo-company`

최종 alias다.

```text
solo-company = project + growth + support + company
```

처음부터 무겁게 구현하지 않는다.
각 capability가 안정화되면 묶음 alias로 제공한다.

## Reference Import Rules

가져올 때는 기능을 그대로 복제하지 않는다.
항상 내부 contract로 재해석한다.

### `gstack`

가져올 것:

- `office-hours`
- plan review
- review
- QA
- ship
- retro
- TDD/review gate discipline

버릴 것:

- 전체 runtime 구조
- 독립 taskboard/state layer
- provider-native surface를 무시한 통합 조작 모델

적용 위치:

- `project` capability의 execution loop

### `BMAD Method`

가져올 것:

- PM / Architect / Dev / QA 역할 구분
- PRD -> architecture -> story -> implementation -> QA 흐름
- artifact-first agile contract

버릴 것:

- giant persona set 기본 설치
- 특정 host에 묶인 조작 모델

적용 위치:

- `project` capability의 role contract와 artifact flow

### `SuperClaude`

가져올 것:

- command UX
- persona activation 감각
- MCP integration pattern

버릴 것:

- 모든 persona와 command를 baseline에 넣는 방식
- provider-specific framework를 공통 runtime으로 삼는 방식

적용 위치:

- provider별 command/skill naming과 opt-in capability UX

### `spec-kit`

가져올 것:

- spec / plan / task artifact 구조
- acceptance criteria
- traceability

버릴 것:

- 과한 waterfall
- 구현보다 문서가 커지는 흐름

적용 위치:

- `product-brief`, `tech-plan`, `build-plan`, `qa`

### GitHub Agentic Workflows

가져올 것:

- issue/PR comment 기반 ChatOps trigger
- read-only agent와 validated write 분리
- repository automation lane

버릴 것:

- repo-level workflow 파일을 baseline 기본값으로 생성하는 것

적용 위치:

- advanced project automation
- GitHub connector handoff

### n8n / AgentKit

가져올 것:

- business automation flow
- connector registry 사고
- support/growth/company agent pattern
- guardrail/eval 개념

버릴 것:

- n8n이나 AgentKit runtime 자체를 baseline에 포함
- 외부 계정 로그인과 schedule 실행을 bootstrap이 직접 소유

적용 위치:

- `growth`, `support`, `company` capability의 runtime handoff

## 중복 제거 규칙

### 규칙 1. baseline과 capability를 섞지 않는다

baseline은 설치 안전성이다.
capability는 업무 능력이다.
`replace`는 baseline cleanup을 해도, capability runtime이 되어서는 안 된다.

### 규칙 2. mode를 새 source of truth로 만들지 않는다

`project`, `growth`, `support`, `company`는 capability다.
각 capability의 내부 기능은 pack/harness에서 파생한다.

### 규칙 3. skill, command, hook은 renderer output이다

Codex skill, Gemini command, Claude skill이 같은 기능을 반복 설명하면 drift가 생긴다.
공통 harness contract를 먼저 쓰고 provider별 파일은 얇게 만든다.

### 규칙 4. connector는 contract, 로그인은 runtime

bootstrap은 필요한 connector와 next step을 알려준다.
실제 계정 연결과 recurring schedule 등록은 provider runtime이나 외부 runtime이 맡는다.

### 규칙 5. task-state와 install-state를 분리한다

`stackpilot-state.json`은 설치 상태다.
나중에 task-state를 넣더라도 별도 advanced lane이어야 한다.

### 규칙 6. 사업 lifecycle을 capability 목록으로 쪼개지 않는다

사용자는 "마케팅 capability"를 고르고 싶은 것이 아니라
"지금 시장조사 중인지, MVP 개발 중인지, 출시 중인지"를 고르고 싶다.

따라서 wizard와 문서는 business stage를 먼저 보여주고,
내부적으로 stage를 capability/harness에 매핑한다.

### 규칙 7. LLM과 외부 툴이 잘하는 것은 직접 구현하지 않는다

요즘 추세는 agent가 모든 runtime을 직접 소유하는 쪽보다,
LLM이 reasoning/drafting/synthesis를 맡고 외부 SaaS와 automation tool이 실행을 맡는 쪽이 더 실용적이다.

따라서 아래는 직접 구현하지 않는다.

- CRM runtime
- helpdesk runtime
- accounting runtime
- ads platform runtime
- analytics ingestion runtime
- email/calendar send runtime
- legal/compliance decision runtime

대신 `StackPilot`는 다음만 제공한다.

- 필요한 connector contract
- approval boundary
- artifact template
- provider별 command/skill entrypoint
- doctor handoff status

### 규칙 8. 복잡한 업무는 record-first로 처리한다

회사 운영 workflow는 한 번의 대화에서 끝내는 것이 아니라,
record를 남기고 다음 실행으로 이어간다.

단, record는 새 runtime DB가 아니다.
외부 tool이 source of truth인 데이터는 링크와 요약만 남기고,
`StackPilot`는 record contract와 provider별 entrypoint를 설치한다.

최소 record는 아래를 포함한다.

- 결정 또는 현재 상태
- 다음 행동
- 외부 tool 링크
- 근거와 검증 결과
- approval boundary
- runtime handoff

자세한 record schema와 tool ownership은
[operating-record-model.ko.md](operating-record-model.ko.md)를 따른다.

### 규칙 9. 기본값은 작게, opt-in은 강하게

기본 설치는 lean해야 한다.
강한 capability는 `add`와 wizard 선택으로 켠다.

## Wizard Model

wizard는 일반 사용자의 주 진입점이다.

기본 흐름:

1. 목적 선택: baseline only, project, growth, support, company, solo-company
2. provider 선택: Codex, Gemini, Claude
3. baseline mode 선택: merge, replace
4. capability 선택
5. env/API key 입력
6. plan preview
7. apply
8. runtime handoff 안내

advanced mode:

- pack 선택
- harness 선택
- connector contract 선택
- automation contract 선택

wizard는 내부적으로 `baseline`, `add`, `doctor`를 호출하는 UI wrapper여야 한다.

## Doctor Model

doctor는 두 섹션으로 나눈다.

```text
baseline:
  providers
  MCP
  env
  backup state
  old-tool cleanup

capabilities:
  active capabilities
  packs
  harnesses
  provider surfaces
  runtime handoff queue
```

JSON도 같은 구조로 간다.

```json
{
  "baseline": {
    "ok": true,
    "providers": ["codex", "gemini", "claude"],
    "old_tool_clean": true
  },
  "capabilities": {
    "active": ["project"],
    "packs": ["project-pack"],
    "missing": []
  },
  "runtime_handoff": {
    "connectors": ["github", "linear"],
    "automations": []
  }
}
```

## 구현 순서

코드는 아래 순서로만 들어간다.

1. 문서 기준 확정
2. CLI에 `baseline`, `add`, `remove`, `sync` 추가
3. 기존 `install`을 호환 alias로 유지
4. manifest에 `capabilities` 추가
5. state schema를 baseline/capabilities로 분리
6. `project` capability부터 provider surface 렌더링
7. doctor를 baseline/capability 구조로 재정렬
8. wizard를 목적 기반 선택 UI로 변경
9. `growth`, `support`, `company` 순서로 확장
10. `solo-company` alias 추가

## 현재 기준 최종 문장

`StackPilot`는 LLM별 baseline을 맞추는 설치기를 넘어서,
검증된 workflow 프로젝트의 좋은 기능을 중복 없이 흡수해
한 사람이 프로젝트와 회사를 운영할 수 있게 하는 AI 작업환경 레이어다.
