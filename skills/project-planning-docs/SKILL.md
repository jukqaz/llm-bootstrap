---
name: project-planning-docs
description: 프로젝트 셋업 단계에서 질문-답변으로 기획을 구체화하고 PRD.md와 TRD.md로 문서화하라. 기획 대화를 구조화해 문서로 남겨야 할 때 사용.
---

# 프로젝트 기획 문서화

## Overview

프로젝트 셋업 단계에서 질문-답변으로 기획을 구체화하고 PRD.md와 TRD.md로 문서화하라. 모든 문서는 한국어로 작성하고 파일명 대소문자를 엄격히 지켜라.

## Workflow

### 1) 컨텍스트 수집
- `AGENTS.md`, `README.md`, 기존 `PRD.md`/`TRD.md`를 확인하라.
- AGENTS가 없으면 먼저 `agents-md` 스킬로 생성할지 물어라.

### 2) 질문 게이트 (필수)
- `references/project-planning-question-bank.md`의 필수 질문을 먼저 묻고 답변을 받기 전에는 문서 작업을 시작하지 말아라.
- 필요 최소 질문만 하라. 선택 질문은 필수 답변 이후에만 묻는다.
- 답변이 부족하면 추가 질문을 이어가라.

### 3) 범위 결정
- `references/project-planning-decision-criteria.md`로 라이트/표준/확장 범위를 결정하라.
- 모호하면 사용자에게 선택을 요청하라.

### 4) 문서 작성
- 새 문서는 `assets/PRD.md`, `assets/TRD.md` 템플릿을 사용하라.
- 기존 문서가 있으면 내용을 보존하고 누락된 섹션만 추가하라.
- 합의되지 않은 가정은 반드시 표시하라.

### 5) 링크 정리
- AGENTS가 있으면 `PRD.md`/`TRD.md` 링크를 추가하거나 갱신하라.
- README 갱신이 필요하면 `readme-maintainer` 스킬을 제안하라.

### 6) 리뷰 및 승인
- 변경 사항을 요약하고 사용자 승인 후 반영하라.

## 네이밍/대소문자 규칙
- 파일명은 `AGENTS.md`, `PRD.md`, `TRD.md`, `CHANGELOG.md`, `ROLE-MAP.md`를 사용하라.
- 기존 파일이 다른 대소문자라면 변경 전에 승인받아라.
- 이 스킬은 `PRD.md`, `TRD.md` 생성/갱신에 초점을 둔다.

## Output checklist

- PRD/TRD가 질문 답변을 반영하고 범위에 맞는지 확인하라.
- 링크가 정확하고 대소문자가 일관적인지 확인하라.
- 미확정 사항은 명시되어 있다.

## Resources

### references/
- `project-planning-decision-criteria.md`: 범위 결정 기준
- `project-planning-question-bank.md`: 기획 질문 목록
### assets/
- `PRD.md`: PRD 템플릿.
- `TRD.md`: TRD 템플릿.
