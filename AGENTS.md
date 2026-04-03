# Codex Bootstrap 저장소 지침

이 저장소는 새 macOS 머신에서 Codex 전역 기준선을 한 번에 재현하기 위한 bootstrap source다.
현재 기준선은 `OMX + RTK + ICM`이며, 설치 결과는 각 머신의 `~/.codex`에 생성된다.

## 목적

- Codex 전역 baseline을 빠르게 재현한다
- 개인 머신별 절대경로를 저장소에 직접 박아 넣지 않는다
- generated output보다 재현 절차를 source of truth로 유지한다

## 작업 원칙

- 응답과 문서는 한국어를 기본으로 한다
- 설치 스크립트는 idempotent 하게 유지한다
- 사용자 기존 `~/.codex` 상태는 먼저 백업한다
- `omx setup --scope user --force`를 기준 설치 단계로 사용한다
- `rtk init -g --codex`로 RTK overlay를 다시 붙인다
- ICM MCP 등록은 스크립트가 명시적으로 보장한다
- 새 머신마다 달라지는 절대경로는 실행 시점에 계산한다

## 검증 명령

- `bash -n install.sh`
- `task check`

## 완료 기준

- `install.sh` 문법 검사가 통과한다
- README의 설치 순서와 스크립트 동작이 일치한다
- 설치 후 검증 명령이 문서에 명확히 적혀 있다
