# llm-bootstrap

`Codex`, `Gemini`, optional `Claude Code`의 사용자 홈 설정을 정리하는 macOS용 bootstrap 저장소다.

영문 기본 문서:
- [README.md](README.md)
- [docs/codex-first-blueprint.md](docs/codex-first-blueprint.md)

한국어 보조 문서:
- [docs/codex-first-blueprint.ko.md](docs/codex-first-blueprint.ko.md)

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

## 기본 baseline

- 항상 켜짐:
  - `chrome-devtools`
- env가 있을 때만 켜짐:
  - `context7`
  - `exa`
- `Codex`: plugin, skill, workflow docs
- `Gemini`: extension, native command, workflow docs
- `Claude Code`: skill, workflow docs, lightweight agent docs

프로젝트 전용 MCP는 기본 배포물에 넣지 않는다. `merge`에서는 기존 unmanaged MCP가 유지된다.

## 빠른 시작

```bash
git clone https://github.com/jukqaz/llm-bootstrap.git
cd llm-bootstrap
./install.sh
```

예시:

```bash
cargo run -- install --providers codex,gemini
cargo run -- doctor --providers codex,gemini,claude --json
```

## wizard

```bash
cargo run -- wizard
```

wizard는 다음을 묻는다.

- provider 선택
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
