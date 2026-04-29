# 참고 소스 커버리지

> 제품 경계는 [product-goal.ko.md](product-goal.ko.md),
> 모노레포 경계는 [monorepo-boundary.ko.md](monorepo-boundary.ko.md),
> source catalog 형식은 [../catalog/sources/README.md](../catalog/sources/README.md)를 따른다.

이 문서는 "레포를 다 찾아서 다 커버한다"를 실제로 무엇으로 볼지 고정한다.

여기서 `커버`는 "전부 복사한다"는 뜻이 아니다.

- 최신 또는 중요한 소스를 빠뜨리지 않고 catalog에 넣는다
- 각 소스를 `core`, `addon`, `reference-only`, `retired` 중 어디에 둘지 고정한다
- 무엇을 채택하고 무엇을 복사하지 않을지도 같이 남긴다

즉 이 문서는 `레퍼런스 수집`이 아니라 `레퍼런스 triage` 문서다.

## 커버리지 규칙

모든 tracked source는 아래 네 상태 중 하나로 귀결한다.

- `implemented`
  - 현재 repo 구조나 renderer, docs, install flow에 이미 반영됐다
- `partial`
  - 일부 개념이나 표면은 반영됐지만 full adoption은 아니다
- `reference-only`
  - 참고 가치는 크지만 의도적으로 설치/구현 대상으로 삼지 않는다
- `retired`
  - 역사적 신호는 남기되 새 채택 대상은 아니다

그리고 모든 source는 아래 네 레이어 중 하나에 들어간다.

- `core`
  - bootstrap baseline과 직접 연결
- `addon`
  - `stackpilot-dev-kit` 같은 opt-in capability 층
- `advanced`
  - 상위 runtime, repo automation, remote lane 참고
- `archive-watch`
  - 교체되었거나 archived 된 항목

## 현재 기준

### 1. 공식 provider source

| id | 상태 | 레이어 | 현재 판단 |
| --- | --- | --- | --- |
| `openai_codex` | `implemented` | `core` | Codex renderer와 home baseline의 최우선 source다. |
| `gemini_cli` | `implemented` | `core` | Gemini extension surface와 settings merge 기준 source다. |
| `claude_code` | `implemented` | `core` | Claude user-scope skill/MCP 등록 기준 source다. |

### 2. workflow / harness 참고 레포

| id | 상태 | 레이어 | 현재 판단 |
| --- | --- | --- | --- |
| `gstack` | `partial` | `addon` | explicit plan/review/ship loop 참고는 반영했지만 full workflow parity는 목표가 아니다. |
| `oh_my_codex` | `partial` | `addon` | entrypoint naming, skill pack, mode alias만 일부 흡수한다. |
| `github_agentic_workflows` | `partial` | `advanced` | workflow contract와 repo automation 발상은 반영했지만 runtime 복제는 하지 않는다. |
| `spec_kit` | `partial` | `addon` | spec-first, task-state 계약은 참고하되 bootstrap core에는 넣지 않는다. |
| `bmad_method` | `partial` | `addon` | planning and role decomposition 발상만 얇게 가져간다. |
| `superclaude` | `partial` | `addon` | Claude skill/catalog 감각은 참고하지만 자체 runtime은 만들지 않는다. |
| `openclaw` | `partial` | `advanced` | company/inbox/control-plane 관점은 남기되 gateway runtime은 배제한다. |
| `openai_agentkit` | `reference-only` | `advanced` | orchestration 참고용이다. baseline이나 기본 addon으로는 두지 않는다. |
| `n8n_ai_workflows` | `reference-only` | `advanced` | external automation lane 참고용이다. bootstrap install 대상은 아니다. |

### 3. 실전 도구와 runtime 참고 소스

| id | 상태 | 레이어 | 현재 판단 |
| --- | --- | --- | --- |
| `rtk` | `implemented` | `core` | near-core shell output compression 축으로 실제 설치/검증에 들어간다. |
| `cline` | `partial` | `addon` | browser validation과 tool-onboarding UX만 참고한다. |
| `roo_code` | `partial` | `addon` | custom mode naming과 packaging 감각만 반영한다. |
| `continue` | `partial` | `advanced` | repo automation contract 참고용이고 기본 bootstrap에는 넣지 않는다. |
| `aider` | `partial` | `addon` | 얇은 precision loop와 git-centric ergonomics만 가져간다. |
| `repomix` | `reference-only` | `advanced` | large repo ingest lane 참고용이다. |
| `gitingest` | `reference-only` | `advanced` | remote repo triage 참고용이고 기본 의존성으로 두지 않는다. |
| `coding_context_cli` | `partial` | `addon` | rules/task 분리 아이디어를 현재 renderer 구조에 참고한다. |
| `mcpm` | `reference-only` | `advanced` | profile-based MCP management 참고용이다. |
| `goose` | `reference-only` | `advanced` | local agent runtime 참고용이다. bootstrap core에는 넣지 않는다. |
| `github_copilot_cli` | `reference-only` | `advanced` | GitHub-native agent CLI 참고용이다. |
| `openhands` | `reference-only` | `advanced` | remote execution platform 참고용이다. |
| `crush` | `reference-only` | `advanced` | 유지 중인 terminal runtime 참고용이다. |
| `opencode` | `retired` | `archive-watch` | archived source로 남기고, 후속 추적은 `crush`로 넘긴다. |
| `caveman` | `reference-only` | `addon` | 별도 pack으로 설치하지 않고, 향후 공통 capability의 terse 출력 규칙 참고용으로만 둔다. |
| `agentapi` | `reference-only` | `advanced` | universal adapter 참고용이다. |
| `mini_swe_agent` | `reference-only` | `advanced` | 단순 loop 교훈 참고용이다. |
| `kelos` | `reference-only` | `advanced` | remote sandbox orchestration 참고용이다. |
| `mcp_universe` | `reference-only` | `advanced` | MCP-heavy evaluation 참고용이다. |

## 지금 기준으로 빠진 항목

이번 패스에서는 기존 문서에 반복 등장했지만 catalog에 없던 주요 항목을 채웠다.

- `caveman`
- `gitingest`
- `github_copilot_cli`
- `openhands`
- `crush`
- `opencode`

이제 현재 문서군에서 반복적으로 참고하는 핵심 GitHub source는 모두 catalog에 들어가 있다.

남은 long-tail은 많지만, 다음부터는 아래 기준으로만 추가한다.

1. 제품 경계를 바꿀 수 있는가
2. core/addon/reference-only 판단을 흔들 만큼 강한가
3. 공식 문서나 README에서 재현 가능한 근거가 있는가

이 셋을 넘지 못하면 catalog에 억지로 넣지 않는다.

## 한 줄 결론

이 저장소에서 `레포를 다 커버한다`는 말은
`모든 유명 레포를 다 복사한다`가 아니라,
`중요한 source를 catalog에 빠짐없이 올리고 core/addon/reference/retired로 분류한다`
는 뜻으로 고정한다.
