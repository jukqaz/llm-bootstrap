# Provider Config Ownership

`Codex`, `Gemini`, `Claude Code`는 업데이트 주기가 다르고 설정 파일 구조도 다르다.
따라서 `StackPilot`은 설정 전체를 하나의 소유물처럼 덮어쓰지 않는다.

이 문서는 provider 업데이트 뒤 설정이 꼬이지 않게 하기 위한 소유권 기준이다.

## 원칙

- `merge`는 사용자와 provider runtime이 만든 선호 설정을 보존한다.
- `replace`는 명시적인 재baseline 동작으로 보고 bootstrap baseline을 다시 적용한다.
- auth, session, history, credentials는 bootstrap이 소유하지 않는다.
- MCP script, bootstrap docs, plugin, extension, skill, command surface는 bootstrap이 소유한다.
- provider가 새 버전에서 추가한 알 수 없는 설정은 기본적으로 보존한다.

## 소유권

`runtime-owned`:

- 로그인, auth, OAuth, account, credential store
- 세션, 대화 기록, history
- provider가 자동으로 추가한 marketplace, project, cache, telemetry 형태의 상태
- provider별 UI, output, approval mode 같은 사용자 선호값

`bootstrap-owned`:

- managed MCP script와 해당 MCP 등록
- `stackpilot-state.json`
- workflow docs
- Codex plugin, Gemini extension, Claude skills
- RTK hook과 RTK 문서
- known old-tool artifact cleanup

`user-owned`:

- provider별 UI 취향
- project trust, manual MCP, custom hook

## Provider별 기준

### Codex

`merge`에서도 최신 baseline을 고정해야 하는 `model`,
`model_reasoning_effort`, `plan_mode_reasoning_effort`, native memories는
bootstrap이 소유한다. verbosity, history, tools, 대부분의 feature toggle,
project trust, marketplace 설정은 기존 사용자 값을 보존한다.

bootstrap은 `AGENTS.md`, agents, managed MCP, plugin surface, workflow docs를
맞춘다.

`replace`, `merge`, 신규 설치 기준 Codex baseline은 다음 방향으로 둔다.

- 기본 모델은 `gpt-5.5`
- 기본 reasoning effort는 `xhigh`
- 신규 설치와 `replace`에서는 `[features].fast_mode = true`와
  `service_tier = "fast"`로 Fast mode 경로를 명시적으로 켠다. `merge`에서는
  사용자가 명시한 `service_tier` opt-down을 보존한다.
- custom agent는 역할별 model tier를 쓴다.
  - 고위험 판단/설계/리뷰: `planner`, `architect`, `reviewer`,
    `security-reviewer`, `platform-infra`는 `gpt-5.5` + `xhigh`
  - 구현 실무: `executor`, `backend-service`, `frontend-app`, `mobile-app`,
    `test-engineer`는 `gpt-5.5` + `high`
  - 반복/저비용 보조: `triage`, `explore`, `docs-researcher`, `git-master`,
    `verifier`는 `gpt-5.4-mini`를 쓰고 역할 위험도에 따라
    `low`/`medium`/`high` effort를 나눈다. `gpt-5.4` 일반 모델은 기본 agent
    template에 쓰지 않는다.
- 1M context가 필요한 `*-1m` agent만 `gpt-5.5` + `xhigh`에
  `model_context_window = 1000000`과
  `model_auto_compact_token_limit = 900000`을 붙인다. 이 PC의 CLI smoke는
  이 조합을 수용했지만, 로컬 Codex catalog는 여전히 `gpt-5.5`의
  `max_context_window`를 `272000`으로 표시하므로 global default에는 1M
  metadata를 강제하지 않는다.
- native memory는 `[features].memories = true`,
  `[memories].generate_memories = true`, `[memories].use_memories = true`,
  `[memories].disable_on_external_context = false`로 활성화한다.
- 예전 `gpt-5-codex` alias는 새 Codex 모델 surface 기준으로 쓰지 않는다.
- `deep-init`, `team`, `ultrawork`는 Codex plugin skill로 제공한다.

### Gemini

`merge`에서는 UI 표시 방식과 output format 같은 사용자 선호값을 대부분 보존한다.
단, 다음 항목은 Gemini baseline 안정성 정책으로 보고 bootstrap이 소유한다.

- `general.defaultApprovalMode = "plan"`
- `general.checkpointing.enabled = true`
- `general.plan.enabled = true`
- `general.plan.modelRouting = true`
- `general.retryFetchErrors = true`
- `general.maxAttempts = 10`
- `general.sessionRetention.enabled = false`는 대화 기록 자동 삭제를 막기 위해
  유지한다.
- `general.enableAutoUpdate = false`
- `general.enableAutoUpdateNotification = false`
- `model.name = "auto"`를 쓴다. Gemini CLI 쪽은 모델/라우팅 변화가 빠르므로
  bootstrap baseline에서는 특정 preview 모델을 고정하지 않는다.
- `hooksConfig.enabled = true`
- `skills.enabled = true`
- `experimental.memoryV2 = true`
- `experimental.autoMemory = true`
- `experimental.contextManagement = true`
- `experimental.modelSteering = true`
- `experimental.topicUpdateNarration = true`

bootstrap은 managed MCP, extension surface, RTK hook, workflow docs를 맞춘다.
auth shape는 `replace`에서도 보존한다.

Gemini는 Claude/Codex처럼 provider-native per-agent model routing을 고정하지
않는다. 공식 Gemini CLI extension surface는 prompt, MCP server, custom command
패키징이므로 `extensions/stackpilot-dev/agents/*.md`는 role prompt 자산으로 두고,
실제 실행은 `/team`, `/review`, `/qa`, `/ship` 같은 extension command contract가
담당한다.

### Claude Code

Claude는 설정 파일보다 `CLAUDE.md`, agents, skills, MCP registration 쪽이 더
중요하다.

bootstrap은 skills, docs, RTK hook, official MCP registration을 맞춘다.
conversation history와 project logs는 건드리지 않는다.

`merge`에서도 다음 값은 Claude baseline 안정성 정책으로 보고 bootstrap이
소유한다.

- `model = "opus[1m]"`. Claude Code 공식 모델 설정은 1M 지원 계정에서
  `opus[1m]`, `sonnet[1m]` alias를 허용하므로 지원되는 곳에서는 1M을 쓴다.
- `effortLevel` 파일 값은 제거하고 `env.CLAUDE_CODE_EFFORT_LEVEL = "max"`로
  max reasoning을 기본 적용한다.
- `autoMemoryEnabled = true`
- `autoUpdatesChannel = "stable"`
- `cleanupPeriodDays = 365`
- `includeGitInstructions = true`
- `awaySummaryEnabled = true`
- `fastModePerSessionOptIn = true`
- Claude subagent는 공식 `~/.claude/agents/*.md` frontmatter를 사용한다.
  `planner`/`reviewer`는 `opus[1m]`, `executor`/`verifier`는 `sonnet[1m]`,
  `triage`는 `haiku`로 역할별 비용을 나눈다.
- `showThinkingSummaries = true`
- `useAutoModeDuringPlan = false`
- `env.CLAUDE_CODE_EXPERIMENTAL_AGENT_TEAMS = "1"`
- `permissions.defaultMode = "auto"`
- 민감 파일 읽기와 파괴적 git/shell 명령은 `permissions.deny`에 추가
- git push, curl/wget, ssh, kubectl, terraform apply는 `permissions.ask`에 추가

`merge`에서는 사용자가 명시한 `model`, `env.CLAUDE_CODE_EFFORT_LEVEL`,
`fastModePerSessionOptIn` opt-down을 보존한다. 다만 기존 StackPilot 값인
`opus`는 `opus[1m]`으로 마이그레이션한다.
`replace`는 다시 full max baseline을 적용한다.

## 운영 규칙

설정이 꼬였을 때 바로 `replace`를 쓰지 않는다.

1. `doctor --json`으로 missing과 mismatch를 확인한다.
2. provider CLI가 없거나 auth가 깨진 경우 runtime을 먼저 복구한다.
3. surface 누락은 `install --mode merge`로 재렌더링한다.
4. provider 설정을 완전히 다시 맞춰야 할 때만 `--mode replace`를 쓴다.

이 기준을 지키면 provider가 빠르게 업데이트되어도 bootstrap은 baseline과
addon surface만 고치고, 사용자의 실제 runtime 선호와 세션 상태는 계속 보존된다.
