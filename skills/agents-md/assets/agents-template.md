# AGENTS.md 템플릿

이 문서는 에이전트의 작업 규칙만 다룹니다. 상세 기획/설계는 PRD/TRD로 분리하고 여기에는 요약+링크만 둡니다.

## Root AGENTS.md (모노레포)

```markdown
# Agent instructions (scope: <path/> and subdirectories)

## Scope and layout
- **This AGENTS.md applies to:** `<path/>` and below.
- **Key directories:**
  - <dir/>: <what it owns>

## Modules / subprojects
- 모듈 맵은 `references/module-map-format.md` 형식을 사용.

## Cross-domain workflows
- <frontend -> backend API, auth/session, shared types 등>

## Verification (preferred commands)
- <root 또는 공통 명령 요약, 자세한 명령은 모듈 AGENTS.md로>

## Global conventions
- <전역 규칙과 금지사항>

## Links to module instructions
- `<module-a>/AGENTS.md`
- `<module-b>/AGENTS.md`

## References
- README.md
- PRD.md
- TRD.md
```

## Module AGENTS.md (모듈별)

```markdown
# Agent instructions (scope: <module-path/> and subdirectories)

## Scope and layout
- **This AGENTS.md applies to:** `<module-path/>` and below.
- **Key directories:**
  - <dir/>: <what it owns>

## Module ownership
- <owner/team, 주요 책임>

## Feature map (optional)
- `references/feature-map-format.md` 형식을 사용.

## Build · Test · Run
- <모듈 전용 명령과 전제조건>

## Module-specific rules
- <모듈 한정 규칙, 금지사항, 위험 구역>

## References
- <module docs link>
```
