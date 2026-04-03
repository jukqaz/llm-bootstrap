# codex-bootstrap

새 macOS 머신에서 Codex 전역 baseline을 한 번에 재현하는 bootstrap 저장소다.

현재 baseline:

- `oh-my-codex` 설치
- `rtk` 설치 및 Codex global init
- `icm` 설치 및 MCP 등록
- `omx setup --scope user --force`

## 대상 환경

- macOS
- Homebrew 사용 가능
- Codex CLI 또는 Codex Desktop이 이미 설치된 환경

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

## 검증

```bash
omx doctor
codex mcp list
rtk gain
```

정상 상태라면 `omx_*` MCP들과 `icm`이 보이고, `omx doctor`가 통과해야 한다.
