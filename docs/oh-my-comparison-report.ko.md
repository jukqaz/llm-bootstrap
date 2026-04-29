# oh-my 시리즈 비교 리포트

이 문서는 현재 `StackPilot`와 공개 `oh-my-*` 시리즈를 같은 질문으로 비교한다.

질문은 단순하다.

> "지금 이 저장소는 무엇에 강하고, `oh-my-codex`, `oh-my-claudecode`, `oh-my-gemini`는 무엇에 강한가?"

비교 기준은 두 층으로 나눈다.

- 현재 저장소의 실제 구조와 문서
- 각 `oh-my-*` 저장소의 공개 README

## 비교 대상

- `StackPilot`
- `oh-my-codex`
- `oh-my-claudecode`
- `oh-my-gemini`

## 비교 전제

이 비교는 "로컬 개발자가 지금 바로 어떤 제품을 고를까" 관점이 아니라,
"이 저장소를 어디까지 밀고 가야 `oh-my` 계열과 다른 강점을 가지는가"를 보는 관점이다.

핵심 전제:

- `StackPilot`는 설치기이자 기준선 관리자다
- `oh-my-*`는 실행기이자 orchestration runtime이다
- 따라서 1:1 대체재 비교보다 "제품 계층이 어떻게 다르나"를 먼저 봐야 한다

## 한눈에 보는 분석표

| 항목 | `StackPilot` | `oh-my-codex` | `oh-my-claudecode` | `oh-my-gemini` |
|---|---|---|---|---|
| 제품 중심축 | provider-native bootstrap | Codex orchestration runtime | Claude team runtime | Gemini hook-enforced workflow runtime |
| 주 사용자 질문 | "환경을 어떻게 맞출까?" | "여러 agent를 어떻게 굴릴까?" | "Claude 팀 실행을 어떻게 표준화할까?" | "Hook과 context를 어떻게 강제할까?" |
| source of truth | `preset -> pack -> harness -> surface` | mode, agent, skill, session | team workflow, skill, plugin/runtime | hook, command, Conductor state |
| 실행 깊이 | 중간 | 매우 강함 | 매우 강함 | 강함 |
| provider-native 보존 | 매우 강함 | 중간 | 중간 | 중간 |
| 설치/복구 안전성 | 매우 강함 | 보통 | 보통 | 보통 |
| preset/세트메뉴 | 강함 | 강함 | 강함 | 강함 |
| 멀티에이전트 실행력 | Codex 중심, 부분 구현 | 매우 강함 | 매우 강함 | 부분 구현, roadmap 포함 |
| session persistence | 설치 상태 중심 | 강함 | 강함 | Conductor 중심으로 강함 |
| 회사운영 확장성 | 의도적으로 큼 | 약함 | 약함 | 약함 |
| 앱/커넥터 메타 | 강함 | 제한적 | 제한적 | 제한적 |
| runtime opinionation | 낮음 | 높음 | 높음 | 높음 |
| 유지보수 비용 | 낮음~중간 | 높음 | 높음 | 높음 |

## 항목별 상세 평가

### 1. 제품 계층

| 항목 | 평가 |
|---|---|
| `StackPilot` 강점 | `Codex`, `Gemini`, `Claude`를 각자 native surface로 맞추면서도 공통 pack 구조를 유지한다 |
| `StackPilot` 약점 | 바로 일을 "대신 굴리는" 첫 체감은 약하다 |
| `oh-my-*` 강점 | 사용 직후 `autopilot`, `team`, `ralph`, `review` 같은 실행 모드 체감이 매우 강하다 |
| `oh-my-*` 약점 | provider-native surface보다 자체 runtime UX가 중심이어서, 설치기 관점에서는 더 두껍고 무겁다 |

판단:

- `StackPilot`는 기반 레이어에 더 가깝다
- `oh-my-*`는 사용자 체감 실행 레이어에 더 가깝다

### 2. 멀티에이전트와 랄프 계열 실행력

| 항목 | 평가 |
|---|---|
| `StackPilot` 강점 | `ralph-loop`, `ralph-plan`, `delivery`, `incident`, `founder-loop`, `operating-review` 같은 공통 하네스를 preset과 pack에 넣었다 |
| `StackPilot` 약점 | 멀티에이전트 runtime 자체를 강하게 소유하지 않는다. 실행 품질은 provider-native lane에 의존한다 |
| `oh-my-codex` 강점 | README 기준 32 agents, mode routing, parallel execution, session persistence를 전면에 둔다 |
| `oh-my-claudecode` 강점 | `team-plan -> team-prd -> team-exec -> team-verify -> team-fix` 파이프라인과 tmux worker runtime이 강하다 |
| `oh-my-gemini` 강점 | hook-enforced workflow, phase gate, `ralph-retry`, Conductor로 실행을 강제한다 |
| `oh-my-gemini` 약점 | README 기준 multi-agent orchestration은 아직 roadmap 성격이 섞여 있다 |

판단:

- 실행력만 보면 `oh-my-*`가 강하다
- 하네스 정의와 설치 안전성은 현재 저장소가 더 정리돼 있다

### 3. provider-native 적합성

| 항목 | 평가 |
|---|---|
| `StackPilot` 강점 | Codex는 plugin/agent, Gemini는 extension/command, Claude는 skill/subagent doc/MCP 흐름으로 분리한다 |
| `StackPilot` 약점 | 공통 UX가 약해서 provider마다 체감이 조금 다를 수 있다 |
| `oh-my-*` 강점 | 각 제품은 자기 주력 provider에서 매우 높은 체감을 만든다 |
| `oh-my-*` 약점 | 공통 모드와 런타임을 강하게 밀기 때문에, provider native UX를 완전히 존중하는 구조는 아니다 |

판단:

- provider를 오래 안정적으로 운영하려면 `StackPilot` 쪽이 더 관리하기 쉽다
- 한 provider에서 빠르게 생산성을 극대화하려면 `oh-my-*` 쪽이 더 자극적이다

### 4. 설치, backup, doctor, restore

| 항목 | 평가 |
|---|---|
| `StackPilot` 강점 | install/replace/uninstall/restore/doctor를 제품 핵심 계약으로 둔다 |
| `StackPilot` 약점 | 런타임이 아니라 설치기이므로, 실행 중 관측 기능은 상대적으로 약하다 |
| `oh-my-*` 강점 | setup UX는 빠르다 |
| `oh-my-*` 약점 | 공개 README 기준으로는 home-state drift, backup, restore, uninstall 예측 가능성보다 runtime 사용성이 더 앞선다 |

판단:

- 홈 설정이 자주 꼬이는 환경에서는 `StackPilot`가 더 강하다
- 빠르게 agent 경험을 올리는 데는 `oh-my-*`가 더 강하다

### 5. 회사운영 확장성

| 항목 | 평가 |
|---|---|
| `StackPilot` 강점 | `founder-pack`, `ops-pack`, `linear`, `gmail`, `calendar`, `drive`, `figma`, `stitch`, company automations 메타가 이미 있다 |
| `StackPilot` 약점 | connector auth와 recurring scheduler는 아직 runtime-managed 경계 밖이다 |
| `oh-my-*` 강점 | 개발 실행력은 높다 |
| `oh-my-*` 약점 | 공개 README 기준으로는 거의 전부 개발 orchestration 중심이다 |

판단:

- "개발 + 회사운영"을 같이 보는 방향성은 현재 저장소가 더 분명하다
- 하지만 실제 운영 자동화까지 닫히려면 connector health와 scheduler 연동이 더 필요하다

### 6. 유지보수 비용

| 항목 | 평가 |
|---|---|
| `StackPilot` 강점 | 카탈로그와 renderer가 분리돼 있어 drift 관리가 상대적으로 쉽다 |
| `StackPilot` 약점 | 기능 체감이 runtime 제품보다 느릴 수 있다 |
| `oh-my-*` 강점 | 사용 경험이 강하고 mode가 곧 제품이어서 이해가 쉽다 |
| `oh-my-*` 약점 | mode, skill, hook, persistence, analytics까지 포함하면 유지보수 비용이 커진다 |

판단:

- 이 저장소는 장기 운영형
- `oh-my-*`는 체감 주도형

## 현재 저장소의 강점

1. provider-native surface를 깨지 않고 공통 구조를 입힌다
2. `preset -> pack -> harness -> apps/MCP/surface` 모델이 명확하다
3. backup, restore, uninstall, doctor 같은 운영 안전장치가 강하다
4. 회사운영 pack까지 의도적으로 포함한다
5. 문서, catalog, installer, doctor가 한 저장소 안에서 연결된다

## 현재 저장소의 단점

1. 즉시 체감되는 orchestration runtime은 `oh-my-*`보다 약하다
2. session persistence와 실행 중 memory 계층은 얇다
3. connector auth와 recurring scheduler는 아직 runtime 경계 밖이다
4. provider-native를 유지하는 만큼, 공통 UX의 강도가 낮다
5. "바로 일을 시킨다"는 인상은 `oh-my-*`보다 약하다

## `oh-my-*` 시리즈의 강점

1. 실행 모드가 곧 제품 표면이라 학습이 빠르다
2. `autopilot`, `team`, `ralph`, `review`, `ultrawork` 같은 진입점이 강하다
3. 병렬 실행과 persistence 체감이 크다
4. 개발자 입장에서 첫 사용 감도가 매우 높다

## `oh-my-*` 시리즈의 단점

1. 자체 runtime이 두꺼워질수록 provider-native 표면과 거리가 생긴다
2. 설치/복구/드리프트 관리보다 실행 UX가 우선이다
3. 회사운영용 pack, connector, automation 모델은 상대적으로 약하다
4. 장기적으로는 mode와 runtime 유지비가 커질 수 있다

## 최종 판단

가장 정확한 표현은 이렇다.

- `StackPilot`는 `oh-my`의 대체품이 아니라, 더 아래층의 기반 레이어다
- `oh-my-*`는 즉시 생산성을 올리는 orchestration 제품이다
- 현재 저장소가 잘하는 것은 "안전하게 설치하고, provider별로 맞게 배포하고, 회사운영까지 확장 가능한 구조를 갖는 것"이다
- 현재 저장소가 아직 약한 것은 "한 번 켜면 agent 팀이 바로 굴러가는 체감"이다

따라서 전략은 다음이 맞다.

1. `oh-my-*`처럼 runtime 전체를 복제하지 않는다
2. 대신 `preset`, `pack`, `ralph-loop`, `ralph-plan`, `founder-pack`, `ops-pack`을 더 다듬는다
3. provider-native surface와 설치 안전성은 끝까지 유지한다
4. 필요한 곳에서만 `autopilot`, `team`, `ralph` 같은 강한 실행 진입점을 얹는다

한 줄 결론:

> `oh-my-*`는 빠르게 일하게 만드는 팀장이고, `StackPilot`는 환경과 정책을 안 꼬이게 유지하는 운영 기반이다.

## 근거

현재 저장소:

- [README.md](../README.md)
- [README.ko.md](../README.ko.md)
- [bootstrap.toml](../bootstrap.toml)
- [docs/provider-surface-strategy.ko.md](provider-surface-strategy.ko.md)
- [docs/dev-company-operating-model.ko.md](dev-company-operating-model.ko.md)

외부 README:

- [oh-my-codex README](https://raw.githubusercontent.com/junghwaYang/oh-my-codex/refs/heads/main/README.md)
- [oh-my-claudecode README](https://raw.githubusercontent.com/Yeachan-Heo/oh-my-claudecode/refs/heads/main/README.md)
- [oh-my-gemini README](https://raw.githubusercontent.com/richardcb/oh-my-gemini/refs/heads/main/README.md)
