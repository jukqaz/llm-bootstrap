# External Tool Landscape

이 문서는 기존 참고 레포(`gstack`, `oh-my-*`)와 별개로,
현재 `StackPilot` 설계에 참고할 수 있는 외부 도구와 서비스들을 정리한다.

목적은 두 가지다.

- 바로 가져올 만한 실전 도구를 추려두기
- 구조는 다르지만 참고할 가치가 있는 패턴을 따로 보관하기

## 분류 기준

외부 도구는 아래 다섯 가지로 나눠서 본다.

- output/token compression
- context packing and repo ingest
- rule and task context assembly
- MCP package and profile management
- agent runtime and harness reference

## 1. Output / Token Compression

### RTK

- 링크: [rtk-ai/rtk](https://github.com/rtk-ai/rtk)
- 포지션: shell command output compression proxy
- 핵심 가치:
  - `git`, `rg`, `test`, `build` 같은 명령 출력 자체를 줄인다
  - Codex, Gemini, Copilot 등 여러 agent에 이미 맞춰져 있다
  - single binary라서 설치와 제거가 단순하다
- 참고할 점:
  - prompt보다 command output을 줄이는 접근이 실효성이 높다
  - default install에 들어가도 무게 대비 효과가 좋다
- `StackPilot` 적용 판단:
  - `core` 또는 `near-core`

### Caveman

- 링크: [JuliusBrussee/caveman](https://github.com/JuliusBrussee/caveman)
- 포지션: terse response / memory compression skill
- 핵심 가치:
  - 모델 출력 스타일을 압축한다
  - `CLAUDE.md` 같은 메모 파일을 압축하는 `caveman-compress` 패턴이 있다
  - plugin, skill, Codex plugin 경로까지 같이 제공한다
- 참고할 점:
  - "답변을 짧게 만든다"는 축에서 실험 속도가 빠르다
  - 하네스보다 style mode에 가깝다
- `StackPilot` 적용 판단:
  - `reference-only`
  - 별도 Caveman pack을 만들기보다 향후 공통 capability를 설계할 때
    terse/review/commit 출력 규칙만 참고한다

### Compresr Context Gateway

- 링크: [compresr Context Gateway](https://compresr.ai/docs/gateway)
- 포지션: LLM API 앞단 transparent compression proxy
- 핵심 가치:
  - 대화 이력과 툴 출력을 백그라운드에서 자동 압축한다
  - threshold 기반 compaction과 compression logs를 제공한다
- 참고할 점:
  - 구조상 가장 강력하지만 가장 무겁다
  - 별도 프록시, API 키, 운영 로그, 환경 설정이 필요하다
- `StackPilot` 적용 판단:
  - 기본선 제외
  - `advanced/enterprise lane` 후보

## 2. Context Packing / Repo Ingest

### Repomix

- 링크: [yamadashy/repomix](https://github.com/yamadashy/repomix)
- 포지션: AI-friendly repository packer
- 핵심 가치:
  - 저장소 전체를 LLM 친화적 단일 출력으로 패킹한다
  - `--compress`로 Tree-sitter 기반 구조 압축을 제공한다
  - token count tree, include/ignore, split output, MCP server 모드가 있다
- 참고할 점:
  - 큰 repo를 여러 agent에 나눠 전달할 때 좋다
  - `MCP`, `compression`, `repo pack`이 한 도구에 같이 있다
- `StackPilot` 적용 판단:
  - `optional context tool`
  - 대형 repo 분석용 pack 후보

### Gitingest

- 링크: [coderamp-labs/gitingest](https://github.com/coderamp-labs/gitingest)
- 포지션: prompt-friendly remote repo digest
- 핵심 가치:
  - GitHub 저장소를 빠르게 digest 형태로 바꾼다
  - 원격 repo quick ingest에 강하다
- 참고할 점:
  - Repomix보다 가볍고 단순하다
  - 기능 폭은 좁지만 onboarding 속도가 빠르다
- `StackPilot` 적용 판단:
  - 기본선 제외
  - remote repo triage 참고용

## 3. Rule / Task Context Assembly

### Coding Context CLI

- 링크: [kitproj/coding-context-cli](https://github.com/kitproj/coding-context-cli)
- 문서: [How to Use with AI Agents](https://kitproj.github.io/coding-context-cli/how-to/use-with-ai-agents.html)
- 포지션: rules + task + skills context assembler
- 핵심 가치:
  - rule, task, skill metadata를 조합해 단일 context를 만든다
  - `-w` write-rules mode로 agent별 설정 파일에 규칙을 쓰고 task만 stdout으로 내보낸다
  - token estimate와 selector 기반 rule filtering이 있다
- 참고할 점:
  - "규칙은 설정 파일에, 현재 작업은 stdout에"라는 분리가 매우 유용하다
  - 현재 `StackPilot`의 renderer 구조와 잘 맞는다
- `StackPilot` 적용 판단:
  - 도구 자체보다 아이디어 채택 가치가 높다
  - source of truth는 공통 harness spec에 두고 renderer가 provider 파일을 쓰는 방향 참고

## 4. MCP Package / Profile Management

### MCPM

- 링크: [pathintegral-institute/mcpm.sh](https://github.com/pathintegral-institute/mcpm.sh)
- 포지션: MCP package manager and registry CLI
- 핵심 가치:
  - MCP 서버를 전역 설치하고 profile/tag로 묶을 수 있다
  - client별 integration 관리와 registry search가 있다
  - 여러 클라이언트에 같은 MCP set을 공유하는 운영 모델을 제안한다
- 참고할 점:
  - `omx`식으로 MCP를 별도 관리하고 싶을 때 가장 직접적인 사례다
  - profile 기반 운영은 pack/catalog 구조와 잘 맞는다
- `StackPilot` 적용 판단:
  - `advanced MCP lane`
  - 기본선보다 optional profile management 참고용

## 5. Agent Runtime / Harness Reference

### GitHub Copilot CLI

- 링크: [github/copilot-cli](https://github.com/github/copilot-cli)
- 포지션: terminal-native coding agent
- 핵심 가치:
  - GitHub coding agent harness를 CLI로 직접 제공한다
  - MCP-powered extensibility와 LSP 설정 표면이 같이 있다
  - autopilot mode와 GitHub integration이 강하다
- 참고할 점:
  - 실제 사용자-facing agent CLI가 plugin/MCP/LSP를 어떻게 같이 묶는지 볼 수 있다
  - runtime 참고용이지 bootstrap에 그대로 가져올 대상은 아니다
- `StackPilot` 적용 판단:
  - runtime/reference only

### OpenHands

- 링크: [OpenHands/OpenHands](https://github.com/OpenHands/OpenHands)
- 포지션: AI software development agent platform
- 핵심 가치:
  - CLI, GUI, cloud, SDK까지 갖춘 agent platform
  - Slack, Jira, Linear 같은 integration 사례가 있다
- 참고할 점:
  - 우리보다 훨씬 무거운 쪽의 runtime reference다
  - multi-user, cloud, issue resolver 쪽 패턴을 볼 수 있다
- `StackPilot` 적용 판단:
  - architecture/reference only

## 우선순위

### 바로 참고할 것

- `RTK`
- `Repomix`
- `Coding Context CLI`
- `MCPM`

이 네 개는 각각 다른 문제를 풀기 때문에 중복이 적다.

- `RTK`: shell output compression
- `Repomix`: repo packing and compression
- `Coding Context CLI`: rules/task separation
- `MCPM`: MCP profile management

### 선택적으로 볼 것

- `Caveman`
- `Gitingest`

둘 다 가볍고 인사이트가 있지만, core 구조를 바꿀 정도는 아니다.

### 무거운 참고용

- `Compresr`
- `OpenHands`
- `GitHub Copilot CLI`

효과는 크지만 구조가 무겁거나 제품 범위가 넓다.
기본 bootstrap보다 상위 runtime 참고용으로 두는 편이 맞다.

## StackPilot에 반영할 원칙

- output compression은 `RTK` 축으로 유지한다
- response terseness는 별도 pack으로 늘리지 않고, 필요해지면 공통 capability
  catalog의 한 옵션으로 흡수한다
- repo ingest는 `Repomix` 또는 유사 pack으로 optional 제공한다
- rules/task 분리는 `Coding Context CLI`의 write-rules 아이디어를 흡수한다
- MCP는 `MCPM`처럼 profile과 client integration을 나눠서 본다
- gateway/proxy 계층은 기본선에 넣지 않는다

## 한 줄 결론

외부 레퍼런스는 많지만, 현재 `StackPilot` 기준으로 바로 가져올 것은
`RTK`, `Repomix`, `Coding Context CLI`, `MCPM`이고,
`Caveman`은 reference-only, `Compresr`와 `OpenHands`는 상위 runtime 참고용이다.
