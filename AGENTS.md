# llm-bootstrap 저장소 지침

이 저장소는 새 macOS 머신에서 `Codex`와 `Gemini` 개발 baseline을 한 번에 재현하는 bootstrap source다.
결과물은 각 provider의 홈 디렉터리(`~/.codex`, `~/.gemini`)에 반영된다.

## 목적

- macOS 개발 머신에서 LLM 도구 baseline을 빠르게 재현한다
- provider별 런타임 상태와 인증 정보는 보존하고, 개발용 설정만 관리한다
- 선언형 템플릿, plugin bundle, Rust CLI를 source of truth로 유지한다
- 공통 baseline 선택은 repo root `bootstrap.toml`이 담당하고, provider renderer가 최종 파일을 만든다
- bootstrap 범위는 user/home 설정으로 제한하고, repo/project-level 설정은 만들지 않는다

## 작업 원칙

- 응답과 문서는 한국어를 기본으로 한다
- 설치기는 idempotent 하게 유지한다
- 기존 홈 상태는 provider별 backup 디렉터리로 항상 먼저 보존한다
- `apply`는 `merge`와 `replace` 두 mode를 지원한다
- shell wrapper는 최소화하고, 실질 로직은 Rust CLI에 둔다
- 외부 도구가 공식 init 경로를 제공하면 템플릿 복제보다 그 init 결과를 우선 사용한다
- Codex plugin은 홈 local marketplace와 installed cache까지 함께 맞춘다
- Gemini는 `merge`에서 `settings.json`을 JSON merge로 반영하고, `replace`에서는 auth/session 키를 보존한 채 bootstrap baseline으로 다시 쓴다
- `rtk-ai`와 MCP wrapper에 필요한 로컬 툴 설치는 Rust CLI가 보장한다
- RTK는 기본 enabled지만 `--without-rtk`로 끌 수 있고, enabled일 때는 `rtk init -g --codex` / `rtk init -g --gemini --auto-patch` 결과를 기준으로 맞춘다
- 절대경로가 필요한 항목은 실행 시점의 `$HOME`으로 계산한다
- 단일 baseline을 유지하고, 같은 기능을 plugin/MCP 양쪽에 중복해서 넣지 않는다
- env가 없는 선택 MCP는 경고만 남기고 생성하지 않는다
- plugin bundle은 skills 중심으로 유지하고, 공통 MCP는 provider home 설정이 전담한다
- secret manager 연계는 기본 제공하지 않고, bootstrap은 export된 env만 소비한다
- `doctor`는 blocking missing과 disabled warning을 분리해 보여준다

## 검증 명령

- `bash -n install.sh`
- `cargo check`
- `cargo test`
- `cargo run -- doctor`
- `cargo run -- --help`

## 완료 기준

- `install.sh` 문법 검사가 통과한다
- Rust CLI 빌드가 통과한다
- README의 사용법과 실제 명령 구조가 일치한다
- 두 provider 경로, backup 규칙, apply mode 의미가 문서에 명확히 적혀 있다
- plugin, MCP, extension baseline이 문서와 코드에서 일치한다
