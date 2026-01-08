---
name: agents-followup-docs
description: AGENTS.md에 참조된 PRD.md와 TRD.md를 프로젝트 규모 분기 기준으로 생성/갱신하고 링크를 정리하라. AGENTS.md 작성 이후 PRD/TRD가 필요할 때 사용.
---

# AGENTS 후속 문서

## Overview

AGENTS.md에서 참조하는 PRD.md와 TRD.md를 생성하거나 갱신하라. 프로젝트 규모에 따라 문서 범위를 분기하고, AGENTS.md는 가볍게 유지하며, 변경 후 링크를 갱신하라. 모든 문서는 한국어로 작성하라. README 갱신은 `readme-maintainer` 스킬로 분리한다.

## Workflow

### 1) Collect context
- 루트와 중첩 `AGENTS.md`가 있으면 읽어라.
- `README.md`와 기존 `PRD.md` / `TRD.md`를 읽어라.
- `AGENTS.md`가 없으면 먼저 생성할지, 문서만 진행할지 물어라.

### 2) Decide scope (branching)
- `references/agents-followup-decision-criteria.md`로 라이트/표준/확장 중 하나를 선택하라.
- 규모가 불명확하면 선택안을 사용자에게 확인하라.
- 핵심 질문에 답변이 없으면 문서 작업을 중단하고 답변을 기다려라.

### 3) Enforce naming and casing
- 정확한 파일명을 사용하라: `AGENTS.md`, `PRD.md`, `TRD.md`.
- 기존 파일이 다른 대소문자(예: `prd.md`)를 쓰면, 변경이나 링크 수정 전에 확인을 받으라.
- AGENTS.md의 링크 대상은 선택한 대소문자로 일관되게 유지하라.

### 4) Draft or update documents
- 새 문서는 `assets/PRD.md`와 `assets/TRD.md` 템플릿을 사용하라.
- 갱신 시 기존 내용을 보존하고 누락된 섹션만 추가하라.
- `references/agents-followup-question-bank.md`에서 누락 정보만 질문하라.

### 5) Update AGENTS.md references
- 관련 `AGENTS.md`에 `PRD.md` / `TRD.md` 링크를 추가 또는 갱신하라.
- 중첩 AGENTS가 있으면 적절한 범위에만 링크를 추가하라.

### 6) Review and confirm
- 변경 사항을 요약하고 최종 반영 전에 승인받아라.

## Output checklist

- PRD/TRD 내용이 선택한 범위에 맞는지 확인하라.
- AGENTS.md 링크가 정확하고 대소문자가 일관적인지 확인하라.
- 사용자가 승인하지 않으면 기존 문서 내용을 변경하지 말아라.
 - README 변경이 필요한 경우 `readme-maintainer`로 넘긴다.

## Resources

### references/
- `agents-followup-decision-criteria.md`: 범위 결정 기준
- `agents-followup-question-bank.md`: PRD/TRD 질문 목록
### assets/
- `PRD.md`: PRD 템플릿.
- `TRD.md`: TRD 템플릿.
