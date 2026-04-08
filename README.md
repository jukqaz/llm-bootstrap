# llm-bootstrap

macOS 개발 머신에서 `Codex`, `Gemini`, 그리고 optional `Claude Code` 홈 설정을 정리하는 Rust 기반 bootstrap 저장소다.

이 저장소는 다음 원칙으로 동작한다.

- provider별 인증 토큰과 세션 상태는 건드리지 않는다
- 개발용 baseline 문서, MCP, plugin, extension만 반영한다
- 공통 의도는 repo root `bootstrap.toml`에서 선언하고, provider renderer가 각자 최종 파일로 변환한다
- 모든 apply는 쓰기 전에 기존 상태를 provider 홈 내부 `backups/llm-bootstrap-<timestamp>/` 아래로 먼저 백업한다
- 모든 uninstall도 제거 전에 기존 상태를 provider 홈 내부 `backups/llm-bootstrap-<timestamp>/` 아래로 먼저 백업한다
- apply mode는 `merge`와 `replace`를 지원하고, 기본값은 `merge`다
- shell script는 Rust CLI를 실행하는 얇은 래퍼만 유지한다
- `rtk-ai`는 기본 포함이지만 opt-out 할 수 있다
- 외부 도구는 가능하면 해당 도구가 공식 지원하는 init 경로를 우선 사용한다

## 지원 순서

- 1순위: `Codex`
- 2순위: `Gemini`
- 3순위: `Claude Code` optional compatibility lane

현재 구현은 Codex-first다. richest workflow, plugin, agent baseline은 Codex에 먼저 들어간다.
Gemini는 extension/hook/settings merge로 등가 기능을 맞추고, Claude Code는 optional compatibility lane으로 추가한다.

설계 기준은 `docs/codex-first-blueprint.md`에 정리했다.

## baseline

단일 baseline으로 간다. preset 분기는 두지 않는다. 이 저장소는 user/home scope만 관리하고, repo/project-level 설정 파일은 만들지 않는다.

- `chrome-devtools`
- `context7` if `CONTEXT7_API_KEY` is set
- `exa` if `EXA_API_KEY` is set
- Codex `llm-dev-kit` plugin
- Gemini `llm-bootstrap-dev` extension
- Codex workflow/checklist + `office-hours` / `investigate` / `autopilot` / `retro` + browser QA skill
- Gemini workflow docs + extension command/agent pack + `autopilot` / `retro`
- Claude compatibility workflow docs + lightweight agent pack + `autopilot` / `retro`

현재 `bootstrap.toml`이 관리하는 공통 선언은 다음이다.

- 기본 provider 목록
- 기본 apply mode
- 외부 도구 enablement
- always-on MCP 목록
- env-gated MCP 목록

최종 `Codex config.toml`이나 `Gemini settings.json` 전체를 공통화하지는 않는다.

## 현재 baseline

- `Codex`: 개발용 `config.toml`, `AGENTS.md`, `RTK.md` when enabled, custom agents, MCP wrapper scripts
- `Codex`: local marketplace + `llm-dev-kit` skill plugin baseline
- `Codex`: `WORKFLOW`, `SHIP_CHECKLIST`, `OFFICE_HOURS`, `INVESTIGATE`, `AUTOPILOT`, `RETRO`
- `Gemini`: 개발용 `GEMINI.md`, `RTK` hook when enabled, MCP scripts, `settings.json` merge
- `Gemini`: `llm-bootstrap-dev` extension baseline with workflow docs, native custom commands, review/qa agents, `AUTOPILOT`, `RETRO`
- `Claude Code`: user-scope `CLAUDE.md`, MCP wrapper scripts, optional RTK compatibility lane
- `Claude Code`: `WORKFLOW`, `SHIP_CHECKLIST`, `OFFICE_HOURS`, `INVESTIGATE`, `AUTOPILOT`, `RETRO`, `REVIEW`, `QA`, `SHIP`, native personal skills

## 서브에이전트 설정 원칙

- `Codex`
  - 공식 `config.toml` / agent TOML 옵션만 사용한다
  - 역할별 모델은 `gpt-5.4`, `gpt-5.4-mini`, `gpt-5-codex`로 제한한다
  - reasoning effort는 역할별로 `medium`, `high`, `xhigh`만 사용한다
  - 전역 context window는 기본값을 유지한다
  - long-context lane만 explicit `model_context_window = 1000000`과 `model_auto_compact_token_limit = 900000`을 pin 한다
  - 현재 `planner-1m`, `architect-1m`, `reviewer-1m`만 1M lane이다
- `Gemini`
  - 공식 설정 표면이 global `model.name` 중심이므로 extension agent에는 per-agent model pin을 넣지 않는다
  - 역할 차이는 문서와 workflow contract로만 분리한다
- `Claude Code`
  - 공식 subagent frontmatter `model`과 `tools`만 사용한다
  - `triage`는 `haiku`, `reviewer`와 `verifier`는 `sonnet`, `planner`와 `executor`는 `inherit`로 둔다
  - subagent별 context window pin은 별도 설정하지 않는다

## MCP 원칙

- 기본 포함:
  - `chrome-devtools`
- env가 있을 때만 포함:
  - `context7`
  - `exa`
- 제외:
  - `filesystem`: Codex/Gemini 기본 파일 작업과 중복
  - `playwright`: `chrome-devtools`와 역할이 겹치고 baseline 대비 비용이 커서 제외
- plugin은 skills-only bundle로 유지하고, MCP는 provider home config가 전담한다

`Exa`는 일반 웹/코드 검색 lane으로 두고 `EXA_API_KEY`가 있을 때만 활성화한다.
`Context7`도 `CONTEXT7_API_KEY`가 있을 때만 활성화한다.
baseline wrapper는 공급망 drift를 줄이기 위해 검증된 npm 버전으로 pin 한다.
secret manager SDK/CLI 연계는 기본 제공하지 않는다. 필요한 값은 사용자가 직접 env로 넣는 방식을 기준으로 한다.

## 앱 / 플러그인 연동

- `Figma`, `Linear`는 Codex 쪽에서 MCP보다 official curated app/plugin lane이 더 자연스럽다
- Codex 소스 기준 discoverable curated plugin allowlist에 `figma@openai-curated`, `linear@openai-curated`가 포함되어 있다
- bootstrap은 이 두 integration을 강제 설치하지 않고, baseline 준비 후 Codex UI 또는 plugin flow에서 enable 하는 쪽을 권장한다
- Gemini 쪽은 app lane보다 MCP/extension 구성이 더 현실적이므로 baseline에는 포함하지 않는다

## 어떤 repo에서 무엇을 가져오나

- `oh-my-codex`: Codex home baseline, agent roster, `AGENTS.md` 운영 규율
- `oh-my-gemini-cli` / `oh-my-gemini`: Gemini extension + settings merge + hook 구조
- `oh-my-claudecode`: Claude Code compatibility lane
- `oh-my-openagent`: category routing, fanout, 계층형 context injection
- `oh-my-agent`: opinionated team workflow pack
- `OpenHarness`: 모듈형 harness 관점
- `gstack`: `plan -> review -> qa -> ship` delivery loop
- `harness/harness`: artifact/release/verification discipline만 참고

반대로 giant state machine, 과한 기본 MCP, 거대한 agent catalog는 들여오지 않는다.

## 전제조건

- macOS
- `brew` 설치
- 대상 도구가 이미 설치되어 있어야 한다
  - Codex
  - Gemini CLI/Desktop
- bootstrap이 다음 도구는 자동으로 맞춘다
  - `node` / `npx`
- RTK가 켜져 있으면 `rtk-ai/tap/rtk`를 설치하고 공식 init를 사용한다
  - Codex: `rtk init -g --codex`
  - Gemini: `rtk init -g --gemini --auto-patch`
- 권장 환경 변수
  - `EXA_API_KEY`
  - `CONTEXT7_API_KEY`
- `Bitwarden`/`Vaultwarden`나 `Infisical`을 쓰더라도, bootstrap은 최종적으로 export된 env만 소비한다.

이 저장소는 provider 앱 자체를 설치하지 않는다. 앱 위에 개발용 baseline만 올린다.

## 빠른 사용법

```bash
git clone https://github.com/jukqaz/llm-bootstrap.git
cd llm-bootstrap
./install.sh
```

기본값은 `codex,gemini` 전체 적용이다. `Claude Code`는 opt-in provider다.

우선순위는 항상 provider 선택이다.

```bash
cargo run -- apply --providers codex
cargo run -- apply --providers gemini
cargo run -- apply --providers claude
cargo run -- apply --providers codex,gemini
cargo run -- apply --providers codex,gemini,claude
```

RTK:

- 기본값은 enabled
- `bootstrap.toml`의 `external.rtk.enabled = true`가 기준이다
- 임시로 빼고 싶으면 `cargo run -- apply --without-rtk`
- uninstall에서도 RTK를 건드리지 않으려면 `cargo run -- uninstall --without-rtk`
- RTK 자산은 직접 템플릿으로 복제하지 않고 공식 `rtk init`가 생성하는 경로를 우선 사용한다

apply mode:

- `merge`:
  - 기본값
  - 기존 홈의 추가 파일은 남겨두고 bootstrap 관리 범위만 갱신한다
  - 세 provider 모두 기존 non-baseline MCP는 유지하고, bootstrap baseline MCP만 추가/갱신한다
  - provider별로 알려진 legacy bootstrap 산출물은 migration cleanup으로 정리한다
  - Gemini `settings.json`과 extension enablement는 기존 상태에 dev baseline만 merge한다
- `replace`:
  - bootstrap이 관리하는 경로를 먼저 비우고 다시 생성한다
  - 세 provider 모두 기존 MCP를 baseline 기준으로 다시 맞추고 non-baseline MCP는 제거한다
  - 하드리셋에 가깝지만, provider 토큰/세션 파일까지 직접 지우지는 않는다
  - Gemini `settings.json`과 extension enablement는 bootstrap baseline 기준으로 다시 쓰되, 알려진 auth/session 키는 보존한다

특정 provider만 적용하려면:

```bash
cargo run -- apply --providers codex
cargo run -- apply --providers gemini
cargo run -- apply --providers claude
cargo run -- apply --providers codex,gemini
cargo run -- apply --providers codex,gemini,claude
```

그 다음에 mode나 RTK 옵션을 붙인다:

```bash
cargo run -- apply --providers codex --mode replace
cargo run -- apply --providers gemini --mode merge --without-rtk
cargo run -- apply --providers claude --without-rtk
```

상태 점검:

```bash
cargo run -- doctor --providers codex
cargo run -- doctor --providers gemini --without-rtk
cargo run -- doctor --providers claude --json
```

제거:

```bash
cargo run -- uninstall --providers codex
cargo run -- uninstall --providers gemini --without-rtk
cargo run -- uninstall --providers claude
./uninstall.sh
```

CLI 도움말:

```bash
cargo run -- --help
```

wizard:

```bash
cargo run -- wizard
```

wizard는 다음을 순서대로 묻는다.

- 적용할 provider
  - 스페이스로 체크하고 엔터로 확정
- `merge` 또는 `replace`
- RTK 포함 여부
- `EXA_API_KEY`
- `CONTEXT7_API_KEY`
- 입력한 키를 어디에 저장할지 여부
  - GUI 앱용 `launchctl setenv`
  - CLI 셸용 `~/.zshrc.d/llm-bootstrap-env.zsh`
  - 필요하면 `~/.zshrc`에 source block도 추가
- 바로 `install`과 `doctor`까지 실행할지 여부

즉 line prompt를 길게 치지 않아도, 체크박스와 confirm 중심으로 필요한 순간에 한 번에 입력하고 적용할 수 있다.

## 디렉터리 구조

- `src/main.rs`: CLI 진입점과 provider dispatch
- `src/cli.rs`: 명령/인자 타입
- `src/manifest.rs`: `bootstrap.toml` 타입과 baseline MCP 선언
- `src/layout.rs`: provider별 관리 경로 목록
- `src/runtime.rs`: 런타임 명령과 환경 helper
- `src/json_ops.rs`: Gemini / Claude settings JSON 정리 로직
- `src/fs_ops.rs`: backup, copy, render helper
- `src/providers/`: Codex/Gemini/Claude install, uninstall, doctor 구현
- `bootstrap.toml`: 공통 manifest
- `templates/codex/`: Codex baseline 템플릿
- `templates/gemini/`: Gemini 문서, 스크립트, extension 템플릿
- `plugins/llm-dev-kit/`: Codex local plugin bundle
- `.agents/plugins/marketplace.json`: Codex local marketplace manifest
- `install.sh`: Rust CLI 실행 래퍼
- `uninstall.sh`: bootstrap 제거 래퍼

## apply가 하는 일

1. provider별 대상 파일을 backup 디렉터리로 복사한다.
2. Homebrew로 `node`를 보장하고, RTK가 enabled면 `rtk`도 보장한다.
3. `merge`면 기존 추가 파일은 유지하고, `replace`면 bootstrap 관리 경로를 먼저 제거한다.
4. RTK가 enabled면 공식 `rtk init`로 Codex/Gemini RTK 자산을 먼저 반영하고, disabled면 bootstrap 관리 범위의 RTK 산출물을 제거한다.
5. 활성 조건을 만족한 MCP wrapper scripts와 `config.toml`을 `~/.codex`로 반영한다.
6. Codex local marketplace와 `llm-dev-kit` skill plugin을 배치한다.
7. Codex workflow/checklist와 `office-hours`, `investigate`, `autopilot`, `retro`, browser QA skill을 추가한다.
8. Gemini용 `GEMINI.md`, MCP wrapper scripts를 반영한다.
9. Gemini extension과 native custom commands, review/qa agent pack, `AUTOPILOT`, `RETRO`를 반영한다.
10. Gemini `settings.json`은 `merge`면 기존 상태에 dev baseline만 합치고, `replace`면 bootstrap baseline 기준으로 다시 쓰되 auth/session 상태 키는 보존한다.
11. Claude provider가 선택되면 `CLAUDE.md`, workflow docs, lightweight agent pack, `AUTOPILOT`, `RETRO`, MCP wrapper scripts를 반영하고, 공식 `claude mcp add --scope user`로 baseline MCP를 등록한다.

apply를 실행하면 provider별 backup 경로를 항상 출력한다.

## uninstall이 하는 일

1. provider별 bootstrap 관리 범위를 backup 디렉터리로 먼저 복사한다.
2. RTK가 enabled면 공식 uninstall을 먼저 실행한다.
   - Codex: `rtk init -g --codex --uninstall`
   - Gemini: `rtk init -g --gemini --uninstall --auto-patch`
3. Codex에서는 bootstrap이 관리하는 `config.toml`, `AGENTS.md`, agents, scripts, plugin, workflow 자산을 제거한다.
4. Gemini에서는 `GEMINI.md`, scripts, extension, QA agent를 제거한다.
5. Gemini `settings.json`은 파일 전체를 지우지 않고 bootstrap baseline MCP만 제거한다. RTK uninstall이 켜져 있으면 RTK hook도 제거한다.
6. Gemini `extension-enablement.json`에서는 `llm-bootstrap-dev` 항목만 제거하고 다른 extension 설정은 남긴다.
7. Claude provider가 선택되면 user-scope baseline MCP를 공식 `claude mcp remove --scope user`로 제거하고, bootstrap이 관리하는 `CLAUDE.md`와 scripts만 걷어낸다.

정확히 이전 상태로 복원해야 하면 uninstall 뒤 provider backup에서 필요한 파일만 되돌리는 방식이 기준이다.

## doctor가 확인하는 것

- `node`, `npx`
- `rtk` when enabled
- `EXA_API_KEY`, `CONTEXT7_API_KEY` 유무
- `~/.codex/config.toml`
- `~/.codex/AGENTS.md`
- `~/.codex/agents/*.toml`
- `~/.codex/scripts/*.sh`
- `~/.codex/.agents/plugins/marketplace.json`
- `~/.codex/plugins/llm-dev-kit/.codex-plugin/plugin.json`
- `~/.codex/OFFICE_HOURS.md`
- `~/.codex/INVESTIGATE.md`
- `~/.codex/AUTOPILOT.md`
- `~/.codex/RETRO.md`
- `~/.gemini/GEMINI.md`
- `~/.gemini/WORKFLOW.md`
- `~/.gemini/SHIP_CHECKLIST.md`
- `~/.gemini/settings.json`
- `~/.gemini/hooks/rtk-hook-gemini.sh` when RTK is enabled
- `~/.gemini/scripts/*.sh`
- `~/.gemini/extensions/llm-bootstrap-dev/gemini-extension.json`
- `~/.gemini/extensions/llm-bootstrap-dev/AUTOPILOT.md`
- `~/.gemini/extensions/llm-bootstrap-dev/RETRO.md`
- `~/.gemini/extensions/llm-bootstrap-dev/commands/*.toml`
- `~/.claude/CLAUDE.md` when the claude provider is selected
- `~/.claude/WORKFLOW.md`
- `~/.claude/AUTOPILOT.md`
- `~/.claude/RETRO.md`
- `~/.claude/REVIEW.md`
- `~/.claude/QA.md`
- `~/.claude/SHIP.md`
- `~/.claude/skills/*/SKILL.md`
- `~/.claude/scripts/*.sh` when the claude provider is selected
- `~/.claude/RTK.md` and `~/.claude/hooks/rtk-rewrite.sh` when RTK is enabled for claude
- `~/.claude.json` when the claude provider is selected

해석 기준:

- `[missing]`: bootstrap 결과물이나 필수 명령이 실제로 없어서 apply 또는 환경 보강이 필요함
- `[warn]`: 실행은 가능하지만 일부 MCP가 비활성화된 상태임
- `EXA_API_KEY`, `CONTEXT7_API_KEY` warning: 해당 MCP는 생성되지 않고 disabled 상태로 남음

Optional follow-up:

- `Figma`, `Linear`는 Codex curated app/plugin enable 권장

## 검증

```bash
bash -n install.sh
bash -n uninstall.sh
cargo check
cargo test
cargo run -- doctor
cargo run -- doctor --json
```

추가로 필요하면 provider별 네이티브 명령으로 점검한다.

```bash
codex features list
codex exec --skip-git-repo-check 'print hello'
```
