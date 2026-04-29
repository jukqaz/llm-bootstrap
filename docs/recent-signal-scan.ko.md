# Recent Signal Scan

기준일: `2026-04-13`

이 문서는 `X`, `Reddit`, `Hacker News`, `GitHub`, 제품 블로그에서 최근에 반복해서 보이는 흐름을
`StackPilot` 관점으로 정리한 메모다.

목적은 "뭐가 시끄러운가"를 적는 것이 아니라, 실제로 다음 설계에 영향을 줄 만한 신호만 분리하는 것이다.

## 한 줄 결론

최근 흐름은 크게 여덟 가지다.

1. markdown이나 frontmatter 기반 workflow-as-code
2. 멀티에이전트 orchestration과 shared workspace
3. context bloat를 줄이기 위한 on-demand docs / codebase index MCP
4. 여러 coding agent를 한 API나 control plane으로 다루는 adapter 계층
5. cloud branch agent, remote sandbox, Kubernetes 같은 실행 분리
6. 거대한 scaffold보다 단순한 bash-first agent 재평가
7. open-source/local-first coding runtime 확산
8. MCP 보안과 token waste에 대한 회의감 증가

## 채널별 핵심 신호

### X

#### 1. workflow-as-code가 빠르게 제품화되고 있다

- GitHub는 `2026-02-27`에 Agentic Workflows를 X에서 공개했고, markdown으로 작성한 내용을 executable workflow로 바꾼다고 설명했다.
- GitHub는 `2026-04-01` changelog에서 Copilot cloud agent가 PR 없이 branch에서 research, plan, code를 수행할 수 있다고 공개했다.

의미:

- 단순 prompt pack보다 workflow contract가 제품 표면으로 올라오고 있다
- local bootstrap도 나중에 remote automation lane과 연결될 수 있게 설계해야 한다

#### 2. terminal coding agent 경쟁이 더 넓어지고 있다

- OpenCode 계열은 `2026-02-25`에 `OpenCode Go`를 공개하며 저비용 agentic coding 흐름을 밀었다.
- 다만 `2026-04-13` 현재 GitHub 기준 `opencode-ai/opencode`는 archived 상태이고, 유지 중인 후속 프로젝트는 `charmbracelet/crush`다.
- JetBrains는 `2026-02-05`에 `Junie CLI` early access를 X에서 알렸다.

의미:

- provider 수는 계속 늘어날 가능성이 높다
- 지금부터도 runtime을 직접 통합하기보다 `source catalog + provider renderer` 구조가 맞다

### Reddit

#### 1. context bloat를 줄이는 doc/codebase MCP가 강한 관심사다

- `2026-03-10`에는 project docs를 on-demand 검색하게 하는 MCP가 올라왔고, 큰 `AGENTS.md`/`CLAUDE.md`를 매번 context에 넣는 방식의 한계를 직접 지적했다.
- `2026-03-09`에는 codebase knowledge graph MCP가 coding agent 탐색 토큰을 크게 줄였다는 사례가 올라왔다.
- `2026-03-05`에는 call graph와 symbol index를 주는 local MCP가 올라왔다.

의미:

- "규칙은 파일에 넣고 task는 별도 전달" 방향이 계속 강화되고 있다
- optional lane으로 `project-doc search`, `code-index MCP`, `repo pack` 계열을 고려할 가치가 크다

#### 2. shared workspace와 multi-agent control plane 수요가 올라오고 있다

- `2026-03-31`에는 Claude Code 에이전트끼리 파일, 브라우저, MCP를 공유하는 shared workspace 사례가 올라왔다.
- `2026-02-09`에는 MCP Orchestrator처럼 sub-agent를 병렬로 다루는 MCP server 사례가 올라왔다.

의미:

- 멀티에이전트는 "역할 추가"보다 "handoff와 공유 자원 모델"이 더 중요해지고 있다
- `StackPilot`도 role 수보다 harness contract와 shared context 전략을 먼저 잡는 편이 맞다

#### 3. local/open-source coding stack도 계속 실험 중이다

- `2026-04-06`에는 OpenCode 계열 runtime을 self-hosted LLM과 테스트한 글이 올라왔고 반응이 컸다.
- 반대로 `2026-03` 이후 goose/opencode의 버그나 token opacity에 대한 불만 글도 보인다.
- 그래서 이제는 `OpenCode` 자체보다 `Crush`를 현재형 runtime reference로 보는 편이 맞다.

의미:

- open-source runtime은 중요하지만 아직 품질 편차가 크다
- 기본선에 넣기보다 optional/reference lane으로 두는 판단이 맞다

### Hacker News

#### 1. sandboxed remote orchestration이 별도 제품군으로 커지고 있다

- `Kelos`는 최근 Show HN에서 coding agent를 ephemeral pod 위에서 declarative workflow로 돌리는 구조를 소개했다.

의미:

- local bootstrap 이후의 상위 실행층은 sandbox-first 흐름으로 갈 가능성이 높다
- 다만 이건 bootstrap core보다 advanced lane에 가깝다

#### 2. 단순한 agent scaffold가 다시 주목받고 있다

- `mini-swe-agent`는 HN과 GitHub 양쪽에서 "100줄 수준의 bash-first agent"가 여전히 강력하다는 메시지로 소비된다.

의미:

- 하네스가 복잡해져도 기본 실행 루프는 단순하게 유지해야 한다
- 더 많은 abstraction이 항상 더 나은 건 아니다

#### 3. MCP에 대한 회의도 늘고 있다

- HN에서는 MCP가 context를 과도하게 먹고, 결국 code-first tool use가 더 낫다는 논의가 반복된다.
- CLI adapter나 output schema, code execution 기반 접근이 더 실용적이라는 흐름도 보인다.

의미:

- MCP 수를 늘리기보다 quality와 output shape를 관리하는 게 더 중요하다
- `doctor`와 source catalog에도 security/cost 관점을 같이 넣어야 한다

### GitHub / 공식 블로그

#### 1. GitHub는 agent workflow와 cloud delegation을 공식 축으로 밀고 있다

- [GitHub Agentic Workflows](https://github.github.com/gh-aw)
- [Research, plan, and code with Copilot cloud agent](https://github.blog/changelog/2026-04-01-research-plan-and-code-with-copilot-cloud-agent)
- [Copilot CLI is now generally available](https://github.blog/changelog/2026-02-25-github-copilot-cli-is-now-generally-available)

의미:

- workflow gate와 background delegation은 일시적 유행이 아니라 제품 방향이다
- 다만 bootstrap은 여기서 runtime을 직접 복제하지 말고 contract만 가져와야 한다

#### 2. GitHub 쪽 오픈소스는 spec/workflow 레이어를 빠르게 쌓고 있다

- `github/spec-kit`
- `githubnext/agentics`

의미:

- plan-first, spec-first, workflow-as-code는 참고 가치가 높다
- 이건 `gstack`과 별개로 또 다른 contract source다

#### 3. universal adapter와 evaluation도 별도 축으로 뜨고 있다

- `coder/agentapi`는 여러 coding agent를 한 HTTP API로 다룬다
- `SalesforceAIResearch/MCP-Universe`는 MCP-heavy agent 평가 프레임워크를 표방한다

의미:

- 장기적으로는 install catalog와 control-plane catalog가 분리될 가능성이 크다
- 지금은 이 둘을 core가 아니라 advanced lane 참고로 두는 것이 맞다

## StackPilot에 직접 영향 주는 판단

### 지금 바로 가져갈 신호

- source catalog를 더 풍부하게 유지
- workflow contract를 explicit data로 올리기
- optional `repo/context` lane 설계
- MCP는 수보다 output shape와 profile 관리 중심으로 보기
- provider renderer 우선 원칙 유지

### 곧 고려할 것

- `spec-kit` 계열의 spec/plan/tasks 계약
- `agentapi` 같은 universal control plane 참고
- `mini-swe-agent`식 minimal loop 교훈
- remote sandbox/branch agent 연동 가능성

### 아직 core에 넣지 말 것

- Kubernetes orchestration
- shared workspace runtime
- full universal agent API
- 무거운 benchmark harness
- proxy/gateway 방식 compression

## 추가된 source catalog 항목

이번 스캔을 반영해 아래 source를 카탈로그에 추가했다.

- [catalog/sources/reference/github_agentic_workflows.toml](../catalog/sources/reference/github_agentic_workflows.toml)
- [catalog/sources/reference/spec_kit.toml](../catalog/sources/reference/spec_kit.toml)
- [catalog/sources/tool/goose.toml](../catalog/sources/tool/goose.toml)
- [catalog/sources/tool/agentapi.toml](../catalog/sources/tool/agentapi.toml)
- [catalog/sources/tool/mini_swe_agent.toml](../catalog/sources/tool/mini_swe_agent.toml)
- [catalog/sources/tool/kelos.toml](../catalog/sources/tool/kelos.toml)
- [catalog/sources/tool/mcp_universe.toml](../catalog/sources/tool/mcp_universe.toml)

## 최종 판단

최근 흐름은 분명히 "더 많은 agent" 쪽으로 가고 있다.
하지만 더 중요한 흐름은 아래 셋이다.

- context를 필요할 때만 불러오는 구조
- workflow를 데이터와 contract로 다루는 구조
- runtime과 control plane을 분리하는 구조

즉 `StackPilot` 기준으로는 "모든 유행 도구를 넣자"가 아니라,
"유행하는 구조 신호를 source catalog로 흡수하고 core/optional/advanced로 나누자"가 맞다.
