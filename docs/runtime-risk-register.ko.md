# Runtime Risk Register

이 문서는 bootstrap 이후에도 남는 runtime-managed 작업을 명시적인 운영 큐로
바꿔서 정리한다.

bootstrap 범위를 다시 넓히는 문서가 아니다.
`install`과 `doctor`가 이미 성공한 뒤에 운영자가 무엇을 더 확인해야 하는지
정리하는 문서다.

## 리스크 분류

### R1. App connector 연결 확인

예:

- `github`
- `linear`
- `gmail`
- `calendar`
- `drive`
- `figma`
- `stitch`

bootstrap이 이미 증명하는 것:

- active preset이 해당 connector를 선택했다
- connector contract가 provider 홈에 렌더링됐다
- `doctor --json`이 handoff 필드를 노출한다

bootstrap이 증명하지 않는 것:

- 실제 외부 계정이 로그인돼 있는가
- 원하는 workspace/account가 선택돼 있는가
- provider runtime 안에서 read/write가 실제로 동작하는가

운영 액션:

1. 대상 runtime을 연다
2. connector가 보이는지 확인한다
3. 원하는 계정이나 workspace가 선택됐는지 확인한다
4. 실제 read 동작을 한 번 수행한다
5. write가 필요하면 approval 경계도 확인한다

### R2. Runtime scheduler 등록

예:

- `daily-founder-brief`
- `weekly-market-scan`
- `weekly-operating-review`
- `weekly-pipeline-review`

bootstrap이 이미 증명하는 것:

- 선택한 preset 기준으로 automation contract가 active다
- artifact 이름과 connector 의존성이 정리돼 있다
- `doctor --json`이 scheduler handoff 필드를 노출한다

bootstrap이 증명하지 않는 것:

- 실제 recurring schedule이 등록됐는가
- 첫 실행이 실제로 도는가
- 결과 artifact가 기대한 위치에 전달되는가

운영 액션:

1. scheduler를 소유할 runtime을 연다
2. cadence를 명시적으로 등록한다
3. 첫 실행 또는 수동 실행을 한 번 돌린다
4. artifact 결과를 검증한다

### R3. Provider runtime drift

예:

- Codex state DB migration 경고
- `StackPilot`가 관리하지 않는 plugin 경고
- 세션 모드별 MCP 노출 차이

bootstrap이 이미 증명하는 것:

- managed file과 state가 요청한 preset과 일치한다
- managed MCP script와 설정이 존재한다

bootstrap이 증명하지 않는 것:

- provider runtime 내부 상태가 깨끗한가
- unrelated third-party plugin이 정상인가
- 모든 세션 모드에서 MCP가 똑같이 노출되는가

운영 액션:

1. provider runtime 경고는 먼저 runtime 이슈로 본다
2. 최소 명령으로 재현한다
3. managed asset이 원인일 때만 bootstrap 수정으로 내린다

## 우선순위

1. active app connector 확인
2. active automation 등록
3. bootstrap state가 깨끗한 뒤 runtime 경고 조사

## `doctor --json` 매핑

먼저 봐야 하는 필드:

- `catalog.runtime_handoff.connector_queue`
- `catalog.runtime_handoff.automation_queue`
- `catalog.runtime_handoff.next_steps`
- `catalog.connectors[].connection_status`
- `catalog.automations[].registration_status`

## 실무 규칙

`doctor`가 녹색이고 남은 큐가 runtime handoff뿐이면, 다음 수정은
`StackPilot`가 아니라 provider runtime이나 운영 절차 쪽에 있어야 한다.
