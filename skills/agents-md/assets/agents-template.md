# AGENTS.md 템플릿

이 문서는 에이전트의 작업 규칙만 다룹니다. 상세 기획/설계는 PRD/TRD로 분리하고 여기에는 요약+링크만 둡니다.

## Root AGENTS.md (모노레포)

```markdown
# Agent instructions (scope: <path/> and subdirectories)

## 질문 답변 요약
- 응답 언어/톤: <예: 한국어 고정, 간결한 톤>
- 프로젝트 타입/기술 스택: <예: 문서/스킬 저장소, Rust 스크립트>
- 모노레포 여부/모듈 경계: <예: 모노레포 아님>
- 중첩 AGENTS.md 범위: <예: 루트만 사용>
- 필수 섹션: <언어, 커뮤니케이션, 작업 범위, 빌드/테스트 등>
- 빌드/실행/테스트/코드생성 명령: <예: task check>
- 교차 워크플로우: <예: ROLE-MAP 기본 플레이북>
- Git/브랜치/커밋/PR 규칙: <예: 간결한 명령형 커밋>
- 보안/민감정보/파괴적 명령 규칙: <예: 파괴적 명령은 사전 확인>
- MCP/외부 도구 사용 규칙: <예: 승인 후 사용, 민감정보 금지>
- 참조 문서: <README/PRD/TRD 등>

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
