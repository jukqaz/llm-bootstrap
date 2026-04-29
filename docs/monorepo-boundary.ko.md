# Monorepo Boundary

이 문서는 `StackPilot` 저장소를 어떻게 나눠서 생각할지 고정한다.

결론은 단순하다.

- 저장소는 당분간 모노레포로 유지한다.
- 제품 계약은 `provider kit`, `bootstrap umbrella`, `addon`으로 분리한다.
- `StackPilot`라는 이름은 provider kit을 관리하는 umbrella CLI에 둔다.

## 결정

현재 가장 중요한 것은 repo를 쪼개는 것이 아니라 경계를 분리하는 것이다.

이 저장소는 서로 성격이 다른 세 층을 함께 담고 있다.

1. `provider-native kits`
2. `bootstrap umbrella`
3. `workflow/company addons`

셋을 같은 제품 표면으로 뭉개면 문제가 생긴다.

- provider kit은 provider 업데이트 속도에 맞춰 바뀐다.
- umbrella는 느리게 바뀌고 안정성이 우선이어야 한다.
- addon은 실험과 반복이 많아서 빨리 바뀔 수 있어야 한다.

따라서 지금은 모노레포를 유지하되, 제품 정의와 문서, 명령, 디렉터리 책임을
세 층으로 분리한다.

## Provider Kits

provider kit은 각 LLM의 native surface에 맞춘 baseline 제품이다.

- `codex-kit`
  - config, AGENTS, MCP, plugin, skills, subagents, agent TOML
- `gemini-kit`
  - settings, extension, commands, GEMINI.md, hooks, MCP
- `claude-kit`
  - CLAUDE.md, subagents, official MCP CLI, hooks, skills

kit의 성공 기준:

- provider native 장점을 잃지 않음
- provider 업데이트가 와도 설정 ownership이 명확함
- 다른 provider 때문에 release와 검증이 과하게 묶이지 않음

## Umbrella

`StackPilot` umbrella는 provider kit을 설치하고 검증하는 제품이다.

직접 책임지는 것:

- `install`
- `replace`
- `restore`
- `uninstall`
- `backup`
- `doctor`
- `probe`
- provider kit 렌더링
- auth/session/history 보존
- old-tool cleanup
- provider 공식 surface와의 호환성 점검

umbrella의 성공 기준:

- 새 머신에서 재현 가능
- 기존 머신에 다시 적용 가능
- auth/history를 깨뜨리지 않음
- drift를 `doctor`로 확인 가능
- rollback과 restore가 예측 가능

## Addon

addon은 provider kit 위에 얹는 선택 기능이다.

예:

- `stackpilot-dev-kit`
- workflow gate
- task-state
- record-work
- review / qa / ship / retro
- repo automation
- founder / company / operating review
- orchestration, team, multi-agent helper

addon의 역할:

- 더 나은 실행 흐름을 제공한다
- provider-native surface 위에 얹힌다
- opt-in이어야 한다
- umbrella 계약을 오염시키지 않는다

## 모노레포 규칙

모노레포 안에서는 다음 규칙을 따른다.

1. 각 provider kit은 독립적으로 설명되고 검증될 수 있어야 한다.
2. umbrella는 kit 설치, 복구, 검증을 담당하되 kit native surface를 숨기지 않는다.
3. addon은 kit 위에 얹히되, umbrella의 필수 성공 기준이 되면 안 된다.
4. README 첫 문단과 release 설명은 provider-native kit 모델 기준으로 쓴다.
5. addon 문서는 별도 문서군과 별도 디렉터리 경계로 분리한다.
6. preview/nightly provider 변화 추종은 addon이 아니라 compatibility check 문제로 다룬다.

강한 올인원 제품 표면이 필요하면 `all-in-one` 같은 preset으로 묶는다.
하지만 그 경우에도 kit 계약은 그대로 유지하고,
`all-in-one = provider kits + addon bundle` 원칙을 깨지 않는다.

## 문서 경계

앞으로 문서는 아래처럼 읽는다.

- `README*`
  - 제품의 기본 설명과 설치 계약
- `product-goal*`
  - provider kit 중심의 최상위 목표
- `monorepo-boundary*`
  - kit / umbrella / addon 경계
- `provider-native-kit-strategy*`
  - provider별 kit 제품 모델
- capability, company, workflow 관련 문서
  - addon 설계 문서

즉 capability 문서가 존재하더라도, 그것이 곧 umbrella 제품 계약은 아니다.

## 디렉터리 경계

즉시 repo split은 하지 않는다.
대신 논리 경계를 먼저 분명히 한다.

권장 방향:

```text
src/                       -> umbrella CLI, state, backup, doctor, probe
src/providers/             -> provider kit renderer implementation
kits/codex/                -> Codex kit source assets
kits/gemini/               -> Gemini kit source assets
kits/claude/               -> Claude kit source assets
templates/                 -> migration 중인 transitional provider renderer assets
docs/monorepo-boundary*    -> repo 경계 문서
addons/stackpilot-dev-kit/        -> addon bundle source
addons/stackpilot-dev-kit/bundles -> addon bundle output
future addons/*            -> 추후 addon 전용 공간
```

addon source는 `addons/stackpilot-dev-kit/` 아래에 둔다.
provider별 baseline 자산은 장기적으로 `kits/{provider}` 아래로 이동한다.

## 명령 경계

사용자-facing 기본 명령은 umbrella를 기준으로 설명한다.

umbrella:

- `install`
- `baseline`
- `sync`
- `restore`
- `uninstall`
- `doctor`
- `probe`
- `wizard`

provider kit alias 후보:

- `codex install`
- `gemini install`
- `claude install`

addon 또는 내부 lane:

- `task-state`
- `record`
- `internal gate`
- `internal repo-automation`
- workflow/company capability entrypoint

당장 명령을 지우지는 않는다.
하지만 문서상 기본 제품 계약과 실험적/선택 기능을 구분한다.

## 나중에 분리하는 기준

별도 repo나 별도 패키지로 나누는 조건은 아래다.

1. 특정 provider kit의 버전 주기가 다른 kit과 명확히 갈라짐
2. 특정 provider kit만 독립 배포하는 편이 검증과 release를 단순하게 만듦
3. addon이 umbrella 없이 독립 배포 가능
4. addon 인터페이스가 충분히 굳음
5. release, test, docs 파이프라인을 따로 가져가는 편이 더 단순함

이 조건이 되기 전에는 모노레포가 더 싸다.

## 현재 운영 결론

지금 `StackPilot`는 "세 provider를 한 UX로 맞추는 bootstrap + addon 실험실"처럼
보일 수 있다. 앞으로는 이를 아래처럼 다시 고정한다.

- `StackPilot` = provider kit umbrella
- `codex-kit`, `gemini-kit`, `claude-kit` = provider-native baseline 제품 표면
- `stackpilot-dev-kit` = provider-neutral workflow addon 묶음
- capability/company/orchestration 문서 = addon 설계층

즉 repo는 하나로 두되, 제품 정체성은 provider-native kit 모델로 좁힌다.
