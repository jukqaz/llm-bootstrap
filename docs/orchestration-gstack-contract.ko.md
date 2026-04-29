# Orchestration and gstack Contract

이 문서는 `StackPilot`이 `gstack`류 워크플로에서 가져올 것과 가져오지 않을 것을
정의한다.

결론은 단순하다. `StackPilot`은 별도 worker runtime이나 tmux orchestrator가
아니다. 대신 provider별 native 기능 위에 같은 실행 규율을 심는
`contract layer`다.

## 채택하는 것

`gstack`에서 가져올 핵심은 실행 순서와 기록 방식이다.

- 하나의 objective와 acceptance target을 먼저 고정한다
- 실행 전에 owner map을 만든다
- 병렬 작업은 파일, 모듈, 책임 범위가 겹치지 않을 때만 허용한다
- stage 전환마다 10-20줄 handoff를 남긴다
- reviewer와 verifier는 write owner 경로 밖에 둔다
- 실패한 gate는 한 번 targeted fix를 허용하고, 반복 실패 후에는 investigation note를 요구한다
- 최종 보고는 변경 파일, 검증, 남은 위험으로 닫는다

이 규칙은 `parallel-build`, `workflow-gate`, `team`, `ultrawork`,
`record-work` 표면으로 공급한다.

## 채택하지 않는 것

아래 항목은 기본 제품 범위 밖이다.

- 별도 long-running orchestrator daemon
- tmux worker runtime
- provider 공통 session database
- provider-native 기능을 우회하는 독자 prompt runtime
- 겹치는 파일 범위를 자동으로 여러 worker에게 나누는 기능
- 외부 PM/CRM/QA/SaaS 데이터를 로컬 state로 복제하는 기능

필요한 외부 상태는 `record-work`나 connector handoff에 링크로 남긴다.

## Pack Mapping

`gstack`식 계약은 `team-pack`에서 켜진다.

| Preset | Packs | Orchestration level |
| --- | --- | --- |
| `normal` | `delivery-pack`, `incident-pack` | 단일 lane delivery와 incident 대응 |
| `orchestrator` | `delivery-pack`, `incident-pack`, `team-pack` | gstack식 owner, handoff, review, QA gate |
| `all-in-one` | development, team, company, review automation | 개발 오케스트레이션과 회사 운영 기록을 함께 적용 |

`parallel-build`가 active harness에 있으면 `task-state gate`는 실행 전
`ownership`, review/QA/ship 전 `handoff`를 요구한다.

## Provider Rendering

같은 계약을 provider마다 native 표면으로 렌더링한다.

| Provider | Surface |
| --- | --- |
| Codex | `stackpilot-dev-kit` plugin, native agents, `team`, `ultrawork`, `workflow-gate` skills |
| Gemini | `stackpilot-dev` extension commands, `TEAM.md`, `gate`, `team`, `ultrawork` commands |
| Claude Code | native subagents, user-scope skills, official MCP, `team`, `workflow-gate` skills |

공통 source of truth는 `bootstrap.toml`의 harness/pack/preset과
`addons/stackpilot-dev-kit`의 provider별 entrypoint다.

## Gate Contract

| Signal | Meaning | Required before |
| --- | --- | --- |
| `spec` | objective와 acceptance target이 고정됨 | plan |
| `plan` | 실행 순서와 bounded files가 정해짐 | execute |
| `ownership` | lane별 owner와 write scope가 겹치지 않음 | execute with `parallel-build` |
| `handoff` | stage 전환 기록이 남음 | review, QA, ship with `parallel-build` |
| `review` | reviewer가 회귀 위험을 확인함 | ship |
| `qa` | 최소 검증 명령이나 runtime check가 실행됨 | ship |
| `verify` | acceptance target이 증거로 확인됨 | ship |

반복 실패가 있으면 `investigation-note`가 없이는 다음 단계로 넘기지 않는다.

## CLI Pattern

```bash
stack-pilot install --providers codex,gemini,claude --preset orchestrator --mode replace

stack-pilot internal task-state begin \
  --title "Build auth flow" \
  --providers codex,gemini,claude \
  --preset orchestrator \
  --phase plan \
  --owner codex \
  --summary "Auth flow objective and acceptance target are being fixed." \
  --checkpoint "Resume from owner map and failing fixture inventory."

stack-pilot internal task-state advance --complete spec,plan,ownership
stack-pilot internal gate check --target-phase execute --json
stack-pilot internal gate apply --target-phase execute --json

stack-pilot internal task-state advance --complete handoff,review,qa,verify
stack-pilot internal gate check --target-phase ship --json
```

## Handoff Format

stage 전환 handoff는 길지 않아야 한다.

```markdown
## Handoff
- Objective:
- Acceptance target:
- Owner map:
- Files touched:
- Decisions:
- Rejected options:
- Verification:
- Risks:
- Next action:
```

## Stop Rules

- owner map이 겹치면 병렬화하지 않는다
- 다음 작업이 한 unresolved detail에 막혀 있으면 main lane에서 해결한다
- 두 번째 verification 실패 후에는 blind retry 대신 investigation note를 남긴다
- destructive branch point나 외부 write는 사용자 승인 전 진행하지 않는다
- provider별 native 기능이 더 강하면 StackPilot runtime을 만들지 않고 해당 기능을 사용한다

이 계약이 `oh-my-*`나 `gstack` 대비 가져야 하는 차이는 제품 경계다.
`StackPilot`은 각 LLM의 baseline과 실행 규율을 맞추고, 실제 실행은 provider-native
도구에 맡긴다.
