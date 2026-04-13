# Product Goal

이 문서는 `llm-bootstrap`의 최상위 목표를 고정하는 source of truth다.

세부 전략, capability 설계, backlog, renderer 구현은 모두 이 문서를 기준으로
판단한다.

## 한 줄 목표

`llm-bootstrap`의 목표는 두 단계다.

1. 각 LLM의 provider-native baseline을 처음부터 안정적으로 맞춘다
2. 그 baseline 위에 더 잘 일하게 해주는 capability를 얹는다

즉 이 제품은 단순 설치기가 아니라,
`baseline first, enablement second` 원칙으로 움직이는 사용자 홈 레이어다.

## 제품 정의

`llm-bootstrap`는 `Codex`, `Gemini`, `Claude Code` 같은 LLM 런타임을
처음부터 다시 세팅할 때 안전한 baseline을 재현하고,
그 다음 planning, execution, review, QA, record, company operations 같은
능력을 provider-native surface에 맞게 추가하는 도구다.

핵심은 다음 두 질문에 답하는 것이다.

1. "이 LLM을 처음 쓸 때 baseline을 어떻게 안 꼬이게 맞출까?"
2. "맞춘 baseline 위에서 어떻게 더 잘 일하게 만들까?"

## Layer 1. Baseline

baseline은 모든 provider에서 공통으로 먼저 성립해야 하는 바닥 계약이다.

포함:

- install / replace / restore / uninstall / doctor
- backup과 복구 가능성
- provider-native config 렌더링
- core MCP
- env-gated MCP
- auth, session, history 보존
- legacy artifact cleanup
- 최소 workflow 문서와 provider-native entrypoint

포함하지 않음:

- 무거운 task-state
- 거대한 회사 memory DB
- runtime-first orchestration engine
- provider-specific wow feature를 기본값으로 강제하는 것

baseline의 성공 기준은 다음과 같다.

- 새 머신에서도 재현 가능하다
- 이미 쓰던 머신에도 덮어쓸 수 있다
- history와 auth를 망가뜨리지 않는다
- doctor로 drift를 판별할 수 있다
- uninstall과 restore가 예측 가능하다

## Layer 2. Enablement

enablement는 baseline이 안정화된 뒤, 더 잘 쓰기 위해 얹는 능력 계층이다.

예:

- project planning
- implementation execution
- review
- QA
- incident response
- founder loop
- operating review
- record-first workflows
- optional productivity or QA tools

enablement의 목적은 "더 많은 기능"이 아니라
"더 적은 마찰로 더 좋은 결과"다.

따라서 enablement는 다음 원칙을 따른다.

- baseline과 분리된다
- opt-in 가능해야 한다
- provider-native surface를 유지한다
- install-state와 task-state를 섞지 않는다
- 외부 SaaS나 runtime이 더 잘하는 일은 handoff로 남긴다

## 왜 두 레이어로 나누는가

이 분리가 없으면 제품이 두 방향으로 동시에 흔들린다.

1. bootstrap safety가 약해진다
2. orchestration runtime을 무겁게 복제하게 된다

우리가 원하는 것은 둘 중 하나가 아니다.

원하는 것은:

- baseline은 작고 안전하게
- enablement는 강하고 확장 가능하게

## 제품 우선순위

언제나 아래 순서를 지킨다.

1. baseline 안정성
2. provider-native 적합성
3. doctor와 recoverability
4. record-first execution contract
5. stronger entrypoints and task-state
6. optional advanced tooling
7. company-scale operating capabilities

즉 "더 강한 orchestration"은 중요하지만,
baseline integrity보다 앞설 수 없다.

## 이름과 포지셔닝

현재 이름 `llm-bootstrap`는 Layer 1에는 정확하다.
따라서 당장은 이름보다 설명을 먼저 바꾼다.

권장 포지셔닝:

> `llm-bootstrap`는 각 LLM의 provider-native baseline을 먼저 안정화하고,
> 그 위에 planning, execution, review, QA, and company operations capability를
> 선택적으로 얹는 도구다.

즉 현재 단계의 제품은 `rename`보다 `repositioning`이 먼저다.

## 구현 판단 기준

새 기능을 넣기 전에 항상 아래를 묻는다.

1. 이것이 baseline인가, enablement인가?
2. baseline이라면 모든 provider에 안전하게 재현 가능한가?
3. enablement라면 opt-in 가능한가?
4. provider-native surface를 유지하는가?
5. install-state와 task-state를 섞지 않는가?
6. doctor나 record contract로 추적 가능한가?

이 질문에 답하지 못하면 core로 넣지 않는다.

## 현재 기준 결론

현재 `llm-bootstrap`는 이미 baseline 제품이다.
앞으로는 enablement를 강하게 키우되,
제품 설명과 설계 기준을 이 문서의 두 레이어 모델로 고정한다.
