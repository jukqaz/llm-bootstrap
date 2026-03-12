# Codex Agent 지침

이 저장소는 Codex 스킬의 curated source를 보관하는 용도다. runtime 기준선은 `~/.codex/skills`이며, 이 저장소는 검증을 마친 스킬만 선별적으로 반영한다.

## 목적

- 재사용 가능한 스킬을 정리하고 보관한다
- 로컬에서 검증한 스킬을 curated 형태로 유지한다
- 저장소 안에서 문서/인덱스/구조 검증이 가능하도록 유지한다

## 언어

- 응답과 문서는 한국어를 기본으로 한다
- 스킬 이름과 파일명은 영어를 사용한다

## 질문 답변 요약

- runtime source of truth: `~/.codex/skills`
- 이 저장소 역할: curated source / archive / authoring
- 유지 대상: current reusable skills + 최소 메타 스킬
- 검증 명령: `task skills-index`, `task doc-links`, `task validate-skills`, `task validate-docs`, `task check`

## 작업 원칙

- 스킬은 `skills/<skill-name>/` 아래에 둔다
- 각 스킬에는 `SKILL.md`와 필요한 `references/`, `assets/`, `scripts/`만 둔다
- runtime에서 이미 정리한 범용/중복 스킬은 다시 추가하지 않는다
- 새 스킬은 로컬 `~/.codex/skills`에서 먼저 검증한 뒤, 재사용 가치가 확인되면 이 저장소로 올린다
- 이 저장소는 live runtime source of truth가 아니므로, 로컬 스킬 전체를 기계적으로 미러링하지 않는다

## 빌드 · 테스트 · 실행

- `task skills-index`
- `task doc-links`
- `task validate-skills`
- `task validate-docs`
- `task bootstrap-docs -- ROOT=.`
- `task check`

## 안전 & 품질

- 민감정보와 토큰은 저장하지 않는다
- 삭제/정리 작업은 현재 runtime 기준선과 충돌하지 않는지 먼저 확인한다
- 문서와 인덱스가 현재 skill set과 어긋나지 않게 함께 갱신한다

## Git & 협업

- 커밋 메시지는 Conventional Commits를 권장한다
- 큰 정리는 제거/추가/문서 갱신을 가능한 범위에서 한 세트로 묶는다

## 참고 문서

- PRD.md
- TRD.md
- CHANGELOG.md
- SKILLS.md
- ROLE-MAP.md
- README.md
