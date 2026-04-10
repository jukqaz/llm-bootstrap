# Operating Record Model

이 문서는 `llm-bootstrap`가 회사 운영을 도울 때 작업 기록을 어떻게 남기고
다음 실행으로 이어갈지 정의한다.

목표는 CRM, taskboard, helpdesk를 새로 만드는 것이 아니다.
복잡한 일을 한 번에 끝내려 하지 않고, 다음에 이어서 처리할 수 있는 작은 기록을
남기는 것이다.

## 핵심 원칙

1. 복잡한 workflow는 반드시 record를 남긴다
2. record는 긴 회의록이 아니라 결정, 다음 행동, 근거, 외부 링크를 담는다
3. 외부 tool이 source of truth인 데이터는 복제하지 않는다
4. `llm-bootstrap`는 record contract와 provider entrypoint만 설치한다
5. 실제 project/task/customer/support 데이터는 GitHub, Linear, CRM, helpdesk,
   docs, calendar, analytics 같은 runtime이 소유한다
6. record는 재개 가능해야 하고, 승인 경계와 근거를 따라갈 수 있어야 한다
7. 기본 baseline에 거대한 task DB나 회사 memory DB를 넣지 않는다

## Record Contract

모든 record는 최소한 아래 필드를 가진다.

```yaml
id: "rec_..."
type: "opportunity | decision | project | task | customer | support | growth | ops | risk | handoff"
title: ""
status: "draft | active | blocked | decided | shipped | closed"
source: ""
owner: ""
updated_at: ""
next_action: ""
linked_tools:
  github: ""
  linear: ""
  figma: ""
  docs: ""
  calendar: ""
  crm: ""
  helpdesk: ""
  analytics: ""
context:
  summary: ""
  assumptions: []
  task_state:
    source: ""
    id: ""
    phase: ""
    status: ""
    providers: []
    packs: []
    harnesses: []
    completed_signals: []
    attempt_count: 0
    last_failure: ""
decision:
  chosen: ""
  alternatives: []
  rationale: ""
evidence:
  links: []
  notes: []
approvals:
  required: false
  reason: ""
  approver: ""
handoff:
  runtime_owner: ""
  external_object_id: ""
  next_step: ""
```

provider별 command나 skill은 이 contract를 얇게 렌더링한다.
필드가 비어 있으면 LLM이 질문을 던지거나, 외부 tool handoff로 남긴다.

active local task-state가 있으면 `llm-bootstrap record --from-task-state`로
owner, next action, lane context를 record에 붙일 수 있다.

## Record Type

| record | 목적 | source of truth |
|---|---|---|
| `OpportunityRecord` | 시장, 고객 pain, wedge 후보를 정리한다 | docs, web/search, analytics |
| `DecisionRecord` | 선택지, 결정, 근거, 재검토 시점을 남긴다 | docs, issue tracker |
| `ProjectRecord` | 제품 brief, scope, milestone, artifact 링크를 묶는다 | GitHub, Linear, docs |
| `TaskRecord` | 실행 단위, owner, next action, 검증 근거를 남긴다 | GitHub issue, Linear issue |
| `CustomerRecord` | 고객/계정, pain, 접촉 이력, follow-up을 연결한다 | CRM, inbox |
| `SupportRecord` | 문의, severity, 답변 초안, 재현 링크, 해결 상태를 남긴다 | helpdesk, Gmail, GitHub |
| `GrowthRecord` | campaign, channel, 가설, 결과, 다음 실험을 남긴다 | analytics, ads, email tool |
| `OpsRecord` | weekly review, KPI, risk, next bet을 묶는다 | docs, analytics, CRM |
| `RiskRecord` | 법무, 보안, 개인정보, 재무 리스크와 승인 경계를 남긴다 | docs, specialist tool |
| `HandoffRecord` | 외부 runtime이 소유하는 작업의 링크와 다음 행동을 남긴다 | provider runtime, external SaaS |

## Workflow Mapping

기본 wizard 흐름은 record를 이렇게 만든다.

| stage | 생성/갱신 record |
|---|---|
| `discover` | `OpportunityRecord`, `HandoffRecord` |
| `decide` | `DecisionRecord`, `ProjectRecord` |
| `validate` | `GrowthRecord`, `DecisionRecord` |
| `build` | `ProjectRecord`, `TaskRecord`, `RiskRecord` |
| `launch` | `GrowthRecord`, `SupportRecord`, `HandoffRecord` |
| `learn` | `OpsRecord`, `DecisionRecord`, `OpportunityRecord` |

advanced capability는 더 세분화될 수 있지만, 기본 UX는 위 6단계를 넘기지 않는다.

## External Tool Ownership

`llm-bootstrap`는 외부 tool의 runtime을 대신하지 않는다.

| 영역 | 권장 source of truth | `llm-bootstrap` 역할 |
|---|---|---|
| 개발 작업 | GitHub, Linear | issue/PR/release record contract와 command 설치 |
| 디자인 | Figma | design brief, design QA, handoff link 정리 |
| 긴 문서 | Drive, Docs, Notion | memo template과 요약 contract 설치 |
| 고객 문의 | Gmail, helpdesk | triage/reply/repro contract와 approval boundary |
| 세일즈 | CRM, Gmail, Calendar | account brief, outreach draft, follow-up record |
| 마케팅 | analytics, ads, email platform | campaign brief와 result review |
| 재무 | Stripe, accounting, spreadsheet | finance-lite checklist와 risk note |
| 운영 리뷰 | docs, analytics, CRM, support | weekly operating review contract |

외부 tool에 쓰기 작업을 할 때는 기본적으로 approval boundary를 둔다.
자동 쓰기는 capability별 opt-in으로만 허용한다.

## Doctor Readiness

나중에 `doctor --json`은 설치 상태와 record readiness를 분리해야 한다.

```json
{
  "record_readiness": {
    "enabled": true,
    "record_system": "github+docs",
    "runtime_owner": "external-tools",
    "active_records": ["project", "ops"],
    "missing_handoffs": ["crm", "helpdesk"],
    "last_recorded_at": null,
    "next_action": "choose record surface in wizard"
  }
}
```

`missing_handoffs`가 있다고 해서 bootstrap 실패는 아니다.
실제 회사 운영에서는 외부 tool 연결이 남은 정상 상태일 수 있다.

## Wizard Impact

wizard는 capability 선택 뒤 record surface를 묻는다.

기본 선택지:

- local docs only
- GitHub issues + repo docs
- Linear + docs
- Drive/Docs + provider runtime
- CRM/helpdesk handoff

처음에는 `local docs only`와 `GitHub issues + repo docs`만 구현해도 충분하다.
나머지는 connector contract와 handoff 안내로 남긴다.

## 구현 순서

1. record contract와 문서부터 고정한다
2. `project`, `growth`, `support`, `company`별 record template을 만든다
3. provider별 command/skill이 record contract를 출력하게 한다
4. `doctor`에 record readiness를 추가한다
5. `wizard`에서 record surface를 선택하게 한다
6. 반복 사용이 충분할 때만 lightweight local index를 추가한다
7. 외부 connector write는 approval boundary가 준비된 뒤 opt-in으로 연다

현재 구현된 것:

- `bootstrap.toml`의 `record_templates` catalog
- `OPERATING_RECORDS.md` provider doc
- Codex/Claude `record-work` skill
- Gemini `record-work` command
- `doctor`의 `active_record_templates`, `record_templates`, `record_readiness`
- wizard의 record surface 선택 UI
- `llm-bootstrap record` local docs 생성 명령
- `llm-bootstrap record --surface github-issue|both` GitHub issue 생성 명령
- `llm-bootstrap record --from-task-state` active task-state attach 및 owner/next_action fallback

아직 구현하지 않은 것:

- 외부 connector write

예시:

```bash
llm-bootstrap record --type project --title "MVP scope" --next-action "create first issue"
llm-bootstrap internal task-state begin --title "Build auth flow" --phase execute --owner codex --next-action "capture resumable record"
llm-bootstrap record --type task --title "Build auth flow" --from-task-state
llm-bootstrap record --type task --title "Build auth flow" --surface both --github-repo owner/repo
llm-bootstrap record --type handoff --title "CRM setup" --surface github-issue --dry-run
```

## Non-goals

- 자체 CRM 구현
- 자체 helpdesk 구현
- 자체 project management SaaS 구현
- 모든 대화를 자동으로 task DB에 저장
- 고객 발송, 결제, 계약, 법무 판단 자동 실행
- 외부 source of truth와 같은 데이터를 중복 저장

최종 기준은 단순하다.
`llm-bootstrap`는 일을 대신 저장하는 회사 DB가 아니라, 일을 이어서 처리할 수 있게
record contract와 실행 표면을 맞추는 bootstrap이어야 한다.
