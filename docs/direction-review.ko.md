# Direction Review

> 최신 제품 기준선은 [capability-os-strategy.ko.md](capability-os-strategy.ko.md)를 따른다.
> 이 문서는 당시 방향 검토의 근거와 위험 판단을 보관한다.

이 문서는 지금까지 검토한 내부 문서, 외부 레퍼런스, 현재 구현을 한 번에 묶어서
`StackPilot`의 다음 방향이 맞는지 판단한 체크포인트다.

목적은 두 가지다.

- 이미 합의한 방향을 흩어지지 않게 남기기
- 구현 우선순위를 잘못 잡지 않도록 기준선을 고정하기

## 검토 범위

이번 판단은 아래를 함께 보고 내렸다.

- [docs/codex-first-blueprint.ko.md](docs/codex-first-blueprint.ko.md)
- [docs/provider-surface-strategy.ko.md](docs/provider-surface-strategy.ko.md)
- [docs/external-tool-landscape.ko.md](docs/external-tool-landscape.ko.md)
- [docs/business-ops-blueprint.ko.md](docs/business-ops-blueprint.ko.md)
- [bootstrap.toml](../bootstrap.toml)
- [src/manifest.rs](../src/manifest.rs)
- [src/providers/codex.rs](../src/providers/codex.rs)
- [src/providers/gemini.rs](../src/providers/gemini.rs)
- [src/providers/claude.rs](../src/providers/claude.rs)

즉 이번 평가는 "좋은 아이디어 모음"이 아니라, 문서와 코드가 실제로 어느 정도 맞물리는지까지 같이 본 결과다.

## 현재 판단

한 줄 결론은 이렇다.

> 방향은 대체로 맞다. 다만 제품 우선순위와 구현 순서를 더 엄격하게 고정해야 한다.

구체적으로는 아래 순서가 맞다.

1. bootstrap core를 계속 작고 안전하게 유지
2. 멀티에이전트 하네스와 Ralph-loop를 공통 스펙으로 정의
3. provider별 공식 표면에 맞는 renderer로 배포
4. `gstack`은 runtime이 아니라 workflow contract source로 흡수
5. 외부 도구는 rich capability를 보강하되 기본 비용은 낮게 유지
6. business ops는 위 기반이 안정화된 뒤 상위 pack으로 올림

즉 지금 시점에서 맞는 중심축은 `business ops`보다 `harness + provider-native distribution + optional external tooling`이다.

## 이미 맞는 부분

### 1. bootstrap core 포지셔닝

현재 저장소는 여전히 bootstrap core로 정의돼 있고, 이 판단은 맞다.

- `bootstrap.toml`은 provider, default mode, MCP baseline만 안다
- `src/manifest.rs`도 아직 `bootstrap`, `external`, `mcp`만 관리한다
- provider renderer는 user-home state, backup/restore, official init path 쪽에 집중한다

이 중심은 유지해야 한다.

## 2. provider-native distribution 방향

이 방향도 맞다.

- Codex는 `config.toml`, `AGENTS.md`, plugin, agent TOML, MCP 표면이 강하다
- Gemini는 extension, `settings.json`, command TOML 중심이 자연스럽다
- Claude는 official MCP CLI와 subagent/hook/settings 축을 우선해야 한다

공통 스펙을 강제로 한 파일 형식으로 통일하는 것보다, 같은 intent를 provider-native surface로 렌더링하는 쪽이 유지비가 낮다.

### 3. `gstack`를 contract source로 보는 판단

이 판단도 맞다.

`gstack`의 강점은 runtime보다 workflow discipline이다.

따라서 가져와야 하는 것은:

- `office-hours`
- `review`
- `qa`
- `ship`
- `retro`

반대로 그대로 복제하면 안 되는 것은:

- 제품 전체 runtime 구조
- 별도 taskboard/state layer
- provider-native surface를 무시한 통합 추상화

### 4. 외부 도구를 별도 landscape로 관리한 것

이것도 맞다.

특히 아래 네 축 구분이 실용적이다.

- `RTK`: shell output compression
- `Repomix`: repo/context packing
- `Coding Context CLI`: rule/task separation
- `MCPM`: MCP profile management

즉 "토큰 절약"을 prompt terse만으로 보지 않고, output path와 context path에서 푸는 방향이 맞다.

## 아직 구현보다 앞서 있는 부분

### 1. harness catalog와 pack catalog

문서에서는 이미 `harness`, `pack`, `ralph-loop`, `parallel-build`, `review-gate`를 공통 정의로 다루지만,
코드는 아직 그 구조를 모른다.

현재 구현 상태:

- `bootstrap.toml`에는 `harnesses`, `packs`가 없다
- `src/manifest.rs`도 이를 deserialize하지 않는다
- provider renderer도 공통 harness metadata를 읽어 렌더링하지 않는다

즉 지금 문서는 "맞는 방향"이지만 아직 "구현된 구조"는 아니다.

### 2. Claude 설명의 일부 drift

Claude 전략은 최종 판단상 `subagent/MCP/hook-first`가 맞다.

하지만 일부 문서 문단과 README 표현은 아직 `skill` 중심 서술이 남아 있다.
이건 방향이 틀렸다기보다, 문서가 단계적으로 수정되면서 표현이 덜 정리된 상태에 가깝다.

### 3. doctor 범주화

문서에서는 `runtime`, `mcp`, `harness`, `agent parity`, `workflow gates` 같은 category-aware 진단을 제안하지만,
현재 doctor는 여전히 파일 존재 확인 비중이 높다.

이 역시 방향은 맞지만 구현은 아직 초기 단계다.

## 지금 기준에서 잘못 잡으면 안 되는 것

### 1. business ops를 먼저 구현 우선순위로 두는 것

`docs/business-ops-blueprint.ko.md`는 장기 확장 방향으로는 유효하다.
하지만 지금 즉시 구현 우선순위로 끌어올리면 기준선이 흔들릴 가능성이 크다.

이유:

- 아직 공통 harness spec이 없다
- provider renderer가 shared harness를 읽는 구조가 없다
- doctor가 pack/harness parity를 볼 수 없다

즉 business layer는 "금지"가 아니라 "한 단계 뒤"가 맞다.

### 2. provider surface를 억지로 통합 abstraction으로 덮는 것

이건 피해야 한다.

`plugin`, `extension`, `skill`, `subagent`, `command`를 같은 설치 단위로 강제하면,
결국 provider 공식 경로와 충돌하고 유지보수가 무거워진다.

맞는 방향은:

- source of truth는 공통 `harness`와 `pack`
- 실제 설치물은 provider-native renderer output

### 3. 가볍게를 기능 축소로 오해하는 것

이건 이번 검토에서 분명히 정리된 부분이다.

목표는 기능 제거가 아니다.

목표는:

- 풍부한 기능
- 얇은 기본선
- 빠른 활성화
- 작은 컨텍스트 경로

즉 heavy capability도 유지하되 항상 로드하지 않는 방향이 맞다.

## 권장 우선순위

다음 순서가 가장 안전하다.

1. `provider-surface-strategy` 기준 문구 정리
2. `bootstrap.toml`과 `src/manifest.rs`에 `harnesses`, `packs` 설계 추가
3. `delivery`, `parallel-build`, `incident`, `review-gate`, `ralph-loop` 공통 정의 추가
4. Codex/Gemini/Claude renderer가 같은 하네스를 각자 native surface로 렌더링하게 변경
5. `doctor`를 `runtime -> mcp -> harness -> agent parity` 순으로 category-aware화
6. 그다음 외부 도구를 `core / optional / advanced` lane으로 연결
7. business ops pack은 마지막에 상위 레이어로 추가

## 최종 판단

현재 방향은 "틀렸다"기보다 "순서를 더 좁혀야 한다"가 맞다.

올바른 방향:

- bootstrap core 유지
- multi-agent harness를 제품의 기본 실행 모델로 승격
- Ralph-loop를 기본 control flow로 정의
- `gstack`은 workflow contract source로만 흡수
- 외부 도구는 rich-but-lean 원칙으로 연결
- provider별 공식 문서를 최우선으로 따름

보류해야 할 것:

- business-wide pack를 먼저 구현하는 것
- provider-native surface를 덮는 통합 runtime abstraction
- 새 영속 상태 계층이나 무거운 gateway를 기본선에 넣는 것

즉 지금의 최적 문장은 이것이다.

> `StackPilot`는 작은 bootstrap이 아니라, provider-native 표면 위에 공통 하네스를 얹는 설치기여야 한다.
