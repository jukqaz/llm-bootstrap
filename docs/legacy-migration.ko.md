# 이전 oh-my 또는 OMC 설치에서 마이그레이션하기

`llm-bootstrap`는 `merge` 모드에서 unmanaged 자산을 보존한다. 기존 머신에
바로 적용할 때는 이게 기본값으로 맞지만, 예전 harness 흔적도 같이 남을 수
있다.

## 빠른 판단 기준

- unmanaged MCP, skill, command, extension을 유지하려면 `merge`
- 선택한 provider를 기준으로 깔끔하게 다시 맞추려면 `replace`
- 전체 reset은 원치 않고 known legacy 흔적만 정리하려면 `merge --cleanup legacy`

## `merge`를 써야 하는 경우

현재 unmanaged MCP, commands, skills, extensions를 유지하면서 bootstrap
baseline만 얹고 싶다면 `merge`를 쓴다.

```bash
cargo run -- install --providers codex,gemini,claude --mode merge
```

다만 예전 설치가 충돌하는 command, skill, extension을 남겨뒀다면 수동으로
지우거나 `replace`로 전환해야 한다.

## `replace`를 써야 하는 경우

선택한 provider를 기준으로 baseline을 깔끔하게 다시 맞추고 싶다면 `replace`
를 쓴다.

```bash
cargo run -- install --providers codex,gemini,claude --mode replace
```

`replace`는 다음을 함께 수행한다.

1. 선택한 provider의 bootstrap 관리 파일 제거
2. 현재 baseline MCP만 남기도록 재구성
3. 선택한 provider의 known legacy oh-my/OMC 흔적 제거

provider가 허용하는 범위에서는 auth 또는 session 상태를 계속 보존한다.

## `merge`에서만 legacy cleanup을 따로 쓰는 경우

`merge`를 유지하면서 known legacy 흔적만 지우고 싶다면 다음처럼 실행한다.

```bash
cargo run -- install --providers codex,gemini,claude --mode merge --cleanup legacy
```

이건 known legacy 위치만 정리하고, 임의의 사용자 자산 전체를 지우지는
않는다.

## 안전 원칙

- 모든 install은 provider별 backup을 먼저 만든다.
- 최신 backup은 `cargo run -- restore --providers ...`로 복구할 수 있다.
- 자동 legacy cleanup은 의도적으로 좁은 범위만 다룬다.
- known cleanup 목록 바깥의 custom 자산은 backup을 확인한 뒤 수동으로
  정리하는 편이 맞다.
