# codex-skills

개인용 Codex 스킬을 관리하는 저장소다.

## 목적
- 스킬 구조를 표준화하고 재사용성을 높인다.
- 문서 네이밍과 링크 규칙을 일관되게 유지한다.

## 구성
- `skills/`: 스킬 디렉터리 모음
- 각 스킬은 `SKILL.md`와 필요한 `references/`, `assets/`, `scripts/`를 가진다.
- 스크립트는 Rust로 작성하고 기존 Python 스크립트는 순차 전환한다.

## 사용 가이드
- 작업 시작 시 `ROLE-MAP.md`의 기본 플레이북(자주 쓰는 흐름)을 따라 스킬을 연계한다.
- 작업 성격이 애매하면 `role-dispatcher`로 역할을 먼저 정리한다.
- 기본 플레이북 바로가기: [ROLE-MAP.md](ROLE-MAP.md#기본-플레이북-자주-쓰는-흐름)

## 문서
- AGENTS.md
- PRD.md
- TRD.md
- CHANGELOG.md
- SKILLS.md
- ROLE-MAP.md

## 자동화
- `task skills-index`: SKILLS.md 갱신
- `task doc-links`: AGENTS/README 문서 목록 갱신
- `task validate-skills`: 스킬 구조 검증
- `task check`: 전체 점검
- GitHub Actions에서 `task check`를 자동 실행한다.
