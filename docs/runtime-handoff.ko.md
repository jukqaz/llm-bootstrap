# Runtime Handoff

이 문서는 `llm-bootstrap`가 어디까지 책임지고, 그 다음 어떤 runtime이
실제 연결과 실행을 이어받는지 정리한다.

핵심 원칙은 단순하다.

- bootstrap은 설치 상태와 계약을 맞춘다
- provider runtime은 계정 연결과 실제 실행을 소유한다

## 경계

`llm-bootstrap`가 책임지는 것:

- `preset -> pack -> harness -> connectors -> MCP -> provider surface`
- provider 홈에 필요한 문서, skill, command, script 설치
- `doctor`를 통한 requested state 와 installed state 비교
- provider별 `llm-bootstrap-state.json` 기록

runtime이 책임지는 것:

- app connector 로그인과 계정 연결
- 실제 inbox/calendar/design/project 데이터 접근
- recurring automation 스케줄 등록과 실행

record handoff:

- 복잡한 업무는 record contract를 남긴 뒤 이어서 처리한다
- record schema는 [operating-record-model.ko.md](operating-record-model.ko.md)를 따른다
- GitHub, Linear, CRM, helpdesk, docs, analytics가 source of truth인 데이터는
  `llm-bootstrap`가 복제하지 않는다
- bootstrap은 어떤 record가 필요하고 어느 runtime이 소유하는지만 설치/표시한다

## Connector handoff

### App connectors

현재 기본 app connector:

- `github`
- `linear`
- `gmail`
- `calendar`
- `drive`
- `figma`
- `stitch`

활성 app connector는 `doctor`에서 다음처럼 보인다.

- `runtime_owner = provider-runtime`
- `verification_mode = manual-runtime-check`
- `connection_status = not-verified`
- `auth_state = external-runtime`

비활성 app connector는 다음처럼 보인다.

- `connection_status = not-requested`
- `next_step = null`

의미:

- bootstrap은 이 connector가 필요하다는 계약만 설치한다
- 실제 로그인/세션/권한은 provider runtime 또는 외부 app connector가 소유한다

운영 체크리스트:

1. target provider를 실행한다
2. 해당 app connector가 UI나 tool 목록에 보이는지 확인한다
3. 실제 계정이 연결돼 있는지 확인한다
4. 읽기 동작 하나를 직접 수행한다
5. 쓰기 approval 정책이 필요한 경우 승인 경계가 기대대로 노출되는지 확인한다

### MCP connectors

향후 `tool_source = mcp` connector를 추가하면 다음 계약을 따른다.

- `runtime_owner = bootstrap`
- `verification_mode = bootstrap-check`
- `connection_status = managed`

의미:

- bootstrap이 wiring을 생성하고
- `doctor`와 실제 MCP 호출로 상태를 확인한다

### Native connectors

`tool_source = native`는 provider 자체가 바로 제공하는 표면이다.

- `runtime_owner = provider-native`
- `verification_mode = native-check`
- `connection_status = ready`

## Automation handoff

현재 automation contract:

- `daily-founder-brief`
- `weekly-market-scan`
- `weekly-operating-review`
- `weekly-pipeline-review`
- `pr-review-gate`
- `release-readiness-gate`

활성 automation contract는 `doctor`에서 다음처럼 보인다.

- `status = rendered`
- `lane = runtime-scheduler` 또는 `lane = repo-automation`
- `scheduler_owner = runtime-managed` 또는 `scheduler_owner = repo-managed`
- `registration_status = not-registered` 또는 `registration_status = not-configured`

비활성 automation contract는 다음처럼 보인다.

- `status = inactive`
- `registration_status = not-requested`
- `next_step = null`

의미:

- 어떤 automation이 active인지
- 어떤 pack/connectors/artifact를 쓰는지는 bootstrap이 설치한다
- 하지만 `runtime-scheduler` lane의 실제 recurring scheduler 등록은 runtime이나 외부 automation 계층이 맡는다
- `repo-automation` lane의 실제 workflow, required check, branch protection 등록은 repository 쪽이 맡는다

운영 체크리스트:

1. active automation 목록을 `doctor --json`으로 확인한다
2. `doctor --json`에서 automation lane을 확인한다
3. `runtime-scheduler`면 해당 runtime에서 예약 기능이 있는지 확인하고 필요한 cadence로 실제 스케줄을 등록한다
4. `repo-automation`이면 target repository workflow, required check, branch protection에 gate를 등록한다
5. 첫 실행 결과가 artifact 계약과 맞는지 확인한다

## Preset별 handoff

### `light`

- focus: delivery baseline
- connector handoff: `github`, `linear`
- automation handoff: 없음

### `normal`

- focus: delivery + incident
- connector handoff: `github`, `linear`
- automation handoff: 없음

### `full`

- focus: delivery + company
- connector handoff:
  - delivery: `github`, `linear`
  - company: `gmail`, `calendar`, `drive`, `figma`, `stitch`
- automation handoff:
  - `daily-founder-brief`
  - `weekly-market-scan`
  - `weekly-operating-review`
  - `weekly-pipeline-review`

### `company`

- focus: founder + ops
- connector handoff:
  - `linear`, `gmail`, `calendar`, `drive`, `figma`, `stitch`
- automation handoff:
  - `daily-founder-brief`
  - `weekly-market-scan`
  - `weekly-operating-review`
  - `weekly-pipeline-review`

### `review-automation`

- focus: repository PR + release gate
- connector handoff: `github`, `linear`
- automation handoff:
  - `pr-review-gate`
  - `release-readiness-gate`

## 실무적으로 보면

지금 남은 일은 bootstrap 버그가 아니라 운영 handoff다.

- app connector는 실제로 로그인돼 있어야 한다
- runtime automation은 실제로 등록돼 있어야 돈다
- repo automation lane은 실제 repository workflow나 branch protection에 연결돼 있어야 돈다

즉 bootstrap 완료 뒤 다음 단계는:

1. provider runtime 열기
2. connector 연결 확인
3. scheduler 또는 repository gate 등록
4. 첫 실행 검증
