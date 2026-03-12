# codex-skills

로컬에서 검증한 Codex 스킬을 선별 보관하는 저장소다.

## 목적

- 이 저장소는 **runtime source of truth가 아니다**
- 실제 runtime 기준선은 `~/.codex/skills`와 `home-baseline`에서 관리한다
- 여기에는 재사용 가치가 있는 스킬과 저장소 유지에 필요한 최소 메타 스킬만 남긴다

## 현재 범위

- live에서 쓰는 핵심 스킬의 curated copy
- 아이디어 탐색/MVP 루프 스킬
- 엔지니어링 품질/문서/배포/보안 스킬
- 저장소 유지용 메타 스킬
  - `doc-linker`
  - `skill-indexer`
  - `skill-validator`

## 사용 가이드

- 새 스킬은 먼저 로컬 `~/.codex/skills`에서 검증한다
- 실제로 반복 사용 가치가 확인된 것만 이 저장소로 승격한다
- runtime에서 이미 archive된 예전 범용 스킬은 다시 되살리지 않는다
- 스킬 구조와 문서 인덱스는 이 저장소 안에서 자체 검증 가능해야 한다

## 자주 쓰는 명령

- `task skills-index`: `SKILLS.md` 재생성
- `task doc-links`: README/AGENTS 문서 링크 갱신
- `task validate-skills`: 스킬 구조 검증
- `task validate-docs`: 핵심 문서 검증
- `task bootstrap-docs -- ROOT=.`: 문서 세트 부트스트랩
- `task check`: 전체 점검

## 문서

- AGENTS.md
- PRD.md
- TRD.md
- CHANGELOG.md
- SKILLS.md
- ROLE-MAP.md
