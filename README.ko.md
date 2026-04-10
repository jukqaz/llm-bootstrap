# llm-bootstrap

`llm-bootstrap`는 `Codex`, `Gemini`, optional `Claude Code`의 provider-native baseline을
먼저 안정화하고, 그 위에 planning, execution, review, QA, company operations capability를
선택적으로 얹는 사용자 홈 레이어다.

## 설치

현재 release: `v0.2.2`

기본 경로는 wizard 실행이다.

```bash
curl -fsSL https://github.com/jukqaz/llm-bootstrap/releases/latest/download/install-release.sh | bash
```

release 자산:
- [GitHub Releases](https://github.com/jukqaz/llm-bootstrap/releases)

비대화식 설치가 필요하면 이렇게 실행한다.

```bash
curl -fsSL https://github.com/jukqaz/llm-bootstrap/releases/latest/download/install-release.sh | bash -s -- --providers codex,gemini
```

## 릴리스 규칙

GitHub Release는 `v*` 태그로 배포한다. patch 번호가 `10`을 넘을 상황이면
`x.y.11+`을 내지 않고 다음 minor로 올린 뒤 patch를 `0`으로 초기화한다.

## 문서

영문:
- [README.md](README.md)
- [docs/product-goal.md](docs/product-goal.md)
- [docs/codex-first-blueprint.md](docs/codex-first-blueprint.md)

한국어:
- [docs/codex-first-blueprint.ko.md](docs/codex-first-blueprint.ko.md)
- [docs/product-goal.ko.md](docs/product-goal.ko.md)
- [docs/capability-os-strategy.ko.md](docs/capability-os-strategy.ko.md)
- [docs/solo-company-flow.ko.md](docs/solo-company-flow.ko.md)
- [docs/operating-record-model.ko.md](docs/operating-record-model.ko.md)
- [docs/direction-review.ko.md](docs/direction-review.ko.md)
- [docs/business-ops-blueprint.ko.md](docs/business-ops-blueprint.ko.md)
- [docs/dev-company-operating-model.ko.md](docs/dev-company-operating-model.ko.md)
- [docs/external-tool-landscape.ko.md](docs/external-tool-landscape.ko.md)
- [docs/official-best-practices.ko.md](docs/official-best-practices.ko.md)
- [docs/recent-signal-scan.ko.md](docs/recent-signal-scan.ko.md)
- [docs/provider-surface-strategy.ko.md](docs/provider-surface-strategy.ko.md)
- [docs/oh-my-comparison-report.ko.md](docs/oh-my-comparison-report.ko.md)
- [docs/reference-repo-backlog.ko.md](docs/reference-repo-backlog.ko.md)
- [docs/reference-surface-matrix.ko.md](docs/reference-surface-matrix.ko.md)
- [docs/superset-strategy.ko.md](docs/superset-strategy.ko.md)
- [docs/runtime-handoff.ko.md](docs/runtime-handoff.ko.md)
- [docs/runtime-risk-register.ko.md](docs/runtime-risk-register.ko.md)

참고 데이터:
- [catalog/sources/README.md](catalog/sources/README.md)
- [catalog/sources/index.toml](catalog/sources/index.toml)

## 핵심 원칙

- provider 인증 토큰과 세션 상태는 직접 관리하지 않는다
- user/home 범위만 다룬다
- 쓰기 전과 제거 전 모두 backup을 만든다
- 기본 모드는 `merge`
- 공식 init 경로가 있으면 우선 사용한다

우선순위:

1. `Codex`
2. `Gemini`
3. `Claude Code`

현재 `bootstrap.toml` 기본 provider는 다음 둘이다.

- `codex`
- `gemini`

`claude`는 지원하지만 명시적으로 고를 때만 설치된다.

## 기본 baseline

- 항상 켜짐:
  - `chrome-devtools`
- env가 있을 때만 켜짐:
  - `context7`
  - `exa`
- `Codex`: plugin, skill, workflow docs
- `Codex`: plugin, skill, workflow docs, `workflow-gate` skill
- `Gemini`: extension, native command, workflow docs, `gate` command
- `Claude Code`: official MCP registration, subagent docs, workflow skill pack, `workflow-gate` skill

프로젝트 전용 MCP는 기본 배포물에 넣지 않는다. `merge`에서는 기존 unmanaged MCP가 유지된다.

## 안전 모델

- `install`, `replace`, `restore`, `uninstall` 전에 항상 backup을 만든다
- `merge`는 unmanaged 자산을 유지한다
- `replace`는 managed 자산을 다시 만들고, 알려진 `omx`, `omc`, `omg`,
  `oh-my-*` 사용자 레벨 흔적을 backup한 뒤 제거한다
- `replace`와 `uninstall`은 알려진 legacy `launchctl` env key와
  `~/.zshrc.d/llm-bootstrap-env.zsh`의 legacy key도 제거한다
- `restore`는 현재 상태를 다시 backup한 뒤 선택한 backup을 복구한다
- env가 없는 선택 MCP는 disabled 상태로 남는다

## 실행 예시

가장 빠른 설치 경로:

```bash
curl -fsSL https://github.com/jukqaz/llm-bootstrap/releases/latest/download/install-release.sh | bash
```

일반 사용자에게는 release archive 설치가 더 낫다. 이 경로는 Rust가 필요 없다.

1. [GitHub Releases](https://github.com/jukqaz/llm-bootstrap/releases)에서 최신 압축 파일을 받는다.
2. 압축을 푼다.
3. 다음 둘 중 하나로 실행한다.

```bash
./llm-bootstrap install --providers codex,gemini
```

또는

```bash
./install.sh --providers codex,gemini
```

특정 release를 고정해서 설치하려면:

```bash
curl -fsSL https://github.com/jukqaz/llm-bootstrap/releases/latest/download/install-release.sh | \
  LLM_BOOTSTRAP_VERSION=v0.2.2 bash -s -- --providers codex,gemini
```

소스 기반 개발이 필요할 때만 저장소를 clone해서 실행하면 된다.

```bash
git clone https://github.com/jukqaz/llm-bootstrap.git
cd llm-bootstrap
./install.sh
```

자주 쓰는 예시:

```bash
cargo run -- install --providers codex,gemini
cargo run -- baseline --providers codex,gemini
cargo run -- install --providers codex,gemini --preset light
cargo run -- install --providers codex,gemini,claude --preset full
cargo run -- install --providers codex,gemini,claude --preset orchestrator
cargo run -- sync --providers codex,gemini --preset full
cargo run -- doctor --providers codex,gemini,claude --json
cargo run -- probe --providers codex,gemini,claude --preset normal
cargo run -- install --providers codex,gemini --mode replace --dry-run
cargo run -- uninstall --providers codex,gemini --dry-run
```

`doctor --json`에는 현재 요청한 preset/pack 외에 실제 홈에 기록된
`installed_preset`, `installed_packs`, `installed_record_surface`,
`requested_record_surface`, `state_mismatch`도 나온다. 새 preset을 아직 설치하지
않은 홈이라면 이 값으로 드리프트를 먼저 확인할 수 있다. provider runtime 체크도
공용 prerequisite와 분리해서 본다.

- Codex: `codex` CLI 또는 `/Applications/Codex.app`
- Gemini: `gemini` CLI
- Claude Code: `claude` CLI

## 세트메뉴 preset

`oh-my` 계열처럼 빠르게 고를 수 있는 세트메뉴는 `preset`으로 제공한다.
내부 source of truth는 여전히 `pack`이고, preset은 pack 묶음 alias다.

- `light`
  - `delivery-pack`
- `normal`
  - `delivery-pack`, `incident-pack`
- `full`
  - `delivery-pack`, `incident-pack`, `founder-pack`, `ops-pack`
- `orchestrator`
  - `delivery-pack`, `incident-pack`, `team-pack`
- `company`
  - `founder-pack`, `ops-pack`
- `review-automation`
  - `review-automation-pack`

`company`와 `full`은 이제 개발 surface를 끄는 수준이 아니라,
실제로 아래 회사운영 자산을 provider native surface에 렌더링한다.

- `RALPH_PLAN.md`
- `FOUNDER_LOOP.md`
- `OPERATING_REVIEW.md`
- `OPERATING_RECORDS.md`
- `CONNECTORS.md`
- `AUTOMATIONS.md`
- Codex skill / Gemini command / Claude skill 진입점
- `record-work` Codex/Claude skill, Gemini command

예시:

```bash
cargo run -- install --providers codex,gemini --preset normal
cargo run -- install --providers codex,gemini,claude --preset full
cargo run -- install --providers codex,gemini,claude --preset orchestrator
cargo run -- doctor --providers codex,gemini --preset company --json
cargo run -- record --type project --title "MVP scope" --next-action "create first issue"
cargo run -- record --type task --title "Build auth flow" --surface both --github-repo owner/repo
```

orchestrator lane에는 얇은 workflow gate도 같이 들어간다.

```bash
llm-bootstrap internal task-state begin --title "Review auth flow" --providers codex,gemini,claude --preset orchestrator --phase execute
llm-bootstrap internal gate check --target-phase plan|execute|review|qa|ship --json
llm-bootstrap internal task-state advance --complete spec,plan,ownership,handoff,review,qa,verify
llm-bootstrap internal gate apply --target-phase ship --json
```

세밀 제어가 필요하면 기존처럼 `--packs delivery-pack,incident-pack`를 직접 써도 된다.
`--preset`과 `--packs`는 함께 쓰지 않는다.

### preset별 연결 성격

각 preset은 단순 문서 묶음이 아니라
`pack -> connectors -> connector apps -> MCP -> provider surface`
조합으로 동작한다.

- `light`
  - packs: `delivery-pack`
  - connector apps: `github`, `linear`
  - MCP: `chrome-devtools`, `context7`
  - surfaces:
    - Codex: `llm-dev-kit`, `delivery-skills`
    - Gemini: `llm-bootstrap-dev`, `delivery-commands`
    - Claude: `claude-skills`, `delivery-skills`
- `normal`
  - packs: `delivery-pack`, `incident-pack`
  - connector apps: `github`, `linear`
  - MCP: `chrome-devtools`, `context7`
  - surfaces:
    - Codex: `delivery-skills`, `incident-skills`
    - Gemini: `delivery-commands`, `incident-commands`
    - Claude: `delivery-skills`, `incident-skills`
- `full`
  - packs: `delivery-pack`, `incident-pack`, `founder-pack`, `ops-pack`
  - connector apps: `github`, `linear`, `gmail`, `calendar`, `drive`, `figma`, `stitch`
  - MCP: `chrome-devtools`, `context7`, `exa`
  - surfaces:
    - Codex: development + company skills
    - Gemini: development + company commands
    - Claude: development + company skills
- `orchestrator`
  - packs: `delivery-pack`, `incident-pack`, `team-pack`
  - connector apps: `github`, `linear`
  - MCP: `chrome-devtools`, `context7`
  - surfaces:
    - Codex: delivery + incident + team skills
    - Gemini: delivery + incident + team commands
    - Claude: delivery + incident + team skills
- `company`
  - packs: `founder-pack`, `ops-pack`
  - connector apps: `linear`, `gmail`, `calendar`, `drive`, `figma`, `stitch`
  - MCP: `exa`
  - surfaces:
    - Codex: company skills
    - Gemini: company commands
    - Claude: company skills
- `review-automation`
  - packs: `review-automation-pack`
  - connector apps: `github`, `linear`
  - MCP: `chrome-devtools`, `context7`
  - automations: `pr-review-gate`, `release-readiness-gate`
  - surfaces:
    - Codex: `review-automation-skills`
    - Gemini: `review-automation-commands`
    - Claude: `review-automation-skills`

`doctor --json`은 이 조합을 pack별로 그대로 노출한다. 이제 provider별로
설치된 preset state 안에 connectors, automations, surfaces, managed paths도 같이
기록하고 비교한다.

runtime 경계:

- app connector auth는 provider runtime이 소유하므로 `runtime-managed`로 보고한다
- runtime scheduler automation contract는 설치 state에 렌더링하지만 반복 스케줄 등록은 runtime이 맡는다
- repo automation contract는 설치 state에 렌더링하지만 repository workflow와 branch protection 등록은 repo가 맡는다

`doctor --json`은 active connector와 automation에 대해 runtime handoff 힌트도
같이 보여준다.

- connector: `runtime_owner`, `verification_mode`, `connection_status`, `next_step`
- automation: `lane`, `scheduler_owner`, `registration_status`, `next_step`
- runtime queue: `runtime_handoff.connector_queue`, `runtime_handoff.automation_queue`, `runtime_handoff.next_steps`
- repo automation queue: `runtime_handoff.repo_automation_queue`, `runtime_handoff.pending_repo_registration_count`
- records: `active_record_templates`, `record_templates`, `record_readiness`

선택형 repo automation scaffold:

```bash
cargo run -- internal repo-automation scaffold --repo-root /path/to/repo
cargo run -- internal repo-automation scaffold --repo-root /path/to/repo --pr-required-check check --release-required-check "check,pr-review-gate / gate"
```

이 명령은 target repo에 `.github/workflows/pr-review-gate.yml`,
`.github/workflows/release-readiness-gate.yml`,
`.github/llm-bootstrap/BRANCH_PROTECTION.md`를 생성한다. 기본 home bootstrap 경로에는
repo-level workflow 생성을 섞지 않는다.

## wizard

```bash
cargo run -- wizard
```

wizard는 다음을 묻는다.

- provider 선택
- preset 선택
- record surface 선택
- `merge` / `replace`
- RTK 포함 여부
- `EXA_API_KEY`
- `CONTEXT7_API_KEY`
- 키 저장 대상
  - GUI 앱: `launchctl setenv`
  - CLI 셸: `~/.zshrc.d/llm-bootstrap-env.zsh`

env 재사용 순서:

1. 현재 프로세스 env
2. `launchctl getenv`
3. managed CLI env 파일

## 공개 저장소 기준

- 실제 키값은 커밋하지 않는다
- 로컬에서 생성되는 managed env 파일은 홈 디렉터리에만 둔다
- 테스트 fixture도 일반 이름만 쓴다

## 검증

```bash
bash -n install.sh
bash -n uninstall.sh
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
```

## 모드

`merge`
- 이전 unmanaged 자산을 보존한다
- bootstrap baseline만 덮어쓴다

`replace`
- bootstrap 관리 자산을 다시 쓴다
- 알려진 `omx`, `omc`, `omg`, `oh-my-*` 사용자 레벨 흔적을 backup한 뒤 제거한다
- 알려진 legacy env key를 `launchctl`과 managed CLI env 파일에서 제거한다

## backup과 복구

모든 `install`, `replace`, `uninstall`은 provider별 backup을 먼저 만든다.
home 레벨 legacy cleanup은 `~/.llm-bootstrap-legacy-backups/*`에 backup을 만든다.

선택한 provider의 최신 backup을 복구하려면:

```bash
cargo run -- restore --providers codex,gemini,claude
```

특정 backup 디렉터리를 복구하려면:

```bash
cargo run -- restore --providers codex --backup llm-bootstrap-1712550000
```

먼저 backup 목록을 보고 싶다면:

```bash
cargo run -- backups --providers codex,gemini,claude
cargo run -- restore --providers codex,gemini --list --json
```

실제 복구 전에 계획만 보려면:

```bash
cargo run -- restore --providers codex,gemini --backup llm-bootstrap-1712550000 --dry-run
```

`restore`는 현재 상태를 한 번 더 backup한 뒤, 선택한 backup 안의
bootstrap 관리 자산을 복구한다.
