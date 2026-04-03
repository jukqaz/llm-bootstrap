# codex-bootstrap

새 macOS 머신에서 Codex 전역 baseline을 한 번에 재현하는 bootstrap 저장소다.

현재 baseline:

- `oh-my-codex` 설치
- `rtk` 설치 및 Codex global init
- `icm` 설치 및 MCP 등록
- `omx setup --scope user --force`

## 전제조건

- macOS
- `brew`가 이미 설치되어 있어야 한다
- `codex` CLI 또는 Codex Desktop이 이미 설치되어 있어야 한다

가이드:

- Homebrew: [brew.sh](https://brew.sh)
- Codex 설치: [Codex Quickstart](https://developers.openai.com/codex/quickstart/#setup)

이 저장소는 `brew`와 `codex`를 직접 설치하지 않는다. 두 도구가 이미 준비된 머신에서 그 위에 `OMX + RTK + ICM` baseline을 올리는 역할만 한다.

## 설치 방식

- `rtk`, `icm`: Homebrew로 설치
- `oh-my-codex`: `npm install -g oh-my-codex`
- `node`/`npm`: 없으면 스크립트가 `brew install node`로 보완

즉 `npm`은 OMX 설치 경로로만 사용하고, 나머지 핵심 툴은 Homebrew와 Codex 기존 설치를 전제로 한다.

## 사용법

```bash
git clone https://github.com/jukqaz/codex-bootstrap.git
cd codex-bootstrap
./install.sh
```

## 스크립트가 하는 일

1. 기존 `~/.codex` 핵심 파일을 타임스탬프 백업 디렉터리로 복사한다.
2. Homebrew로 `rtk`와 `icm`을 설치한다.
3. `npm install -g oh-my-codex`로 OMX를 최신 상태로 맞춘다.
4. `omx setup --scope user --force`를 실행한다.
5. `rtk init -g --codex`를 실행한다.
6. `~/.codex/config.toml`에 `icm` MCP block이 없으면 추가한다.
7. `omx doctor`와 `codex mcp list`로 기본 검증을 수행한다.

## 설치 후 기대 상태

- `~/.codex/config.toml`에 `icm`과 `omx_*` MCP block이 존재한다
- `~/.codex/AGENTS.md`에 `@RTK.md`가 연결된다
- `omx doctor`가 통과한다
- 현재 clone한 저장소에는 `.omx/` 상태 파일이 생기지 않는다

## 검증

```bash
omx doctor
codex mcp list
rtk gain
```

정상 상태라면 `omx_*` MCP들과 `icm`이 보이고, `omx doctor`가 통과해야 한다.
