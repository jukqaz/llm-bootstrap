---
name: agents-md
description: Create or update root and nested AGENTS.md by asking targeted questions and merging the answers into the existing files. Use when users want to bootstrap, regenerate, or refine AGENTS.md for single or monorepo projects. AGENTS.md 생성/갱신 요청이 있을 때 사용.
---

# Make AGENTS.md

## Overview

AGENTS.md를 처음 만들거나 갱신할 때, 사용자에게 필요한 정보를 단계적으로 질문하고 기존 내용과 병합하도록 안내하라. 루트와 중첩 AGENTS.md를 구분해 적용 범위를 명확히 하고, 문서를 비대하게 만들지 않도록 요약+링크 전략을 기본값으로 유지하라.

범위: AGENTS.md 생성/갱신, 중첩 AGENTS 규칙 정리
비범위: PRD/TRD 작성(agents-followup-docs, project-planning-docs), 문서 링크 정리(doc-linker)

## Workflow

### 1) 컨텍스트 수집
- 프로젝트 루트의 `README.md`, `AGENTS.md`, `PRD.md`, `TRD.md`를 확인하라. 파일이 없으면 건너뛰고 필요한 내용을 질문으로 보완하라.
- 기존 `AGENTS.md`가 있으면 섹션 구조와 톤을 파악하고, 없는 경우 템플릿을 사용하라.
- 리포 구조를 훑고 루트와 모듈 경계를 파악하라. 모노레포면 모듈별 중첩 AGENTS.md 후보 위치를 식별하라.
- AGENTS.md에 포함할 내용과 분리할 내용을 구분하라. 상세 기획·설계·API는 `PRD.md`/`TRD.md`로 보내고 AGENTS.md에는 요약과 링크만 남겨라.

### 2) 질문 플로우 실행
- `references/agents-md-question-bank.md`를 열고 필수 질문부터 순서대로 물어보라.
- 사용자가 이미 제공한 정보는 다시 묻지 말고, 누락된 섹션만 채우도록 질문을 이어가라.
- 모노레포 여부, 모듈 목록, 교차 워크플로우, 중첩 AGENTS.md 범위를 반드시 확인하라.
- “질문 후 병합/갱신” 원칙을 고수하라. 기존 문장을 임의로 삭제하거나 대체하지 말고, 사용자 확인 후 반영하라.

### 3) 구조 설계 및 병합
- 루트 AGENTS.md에는 전역 규칙과 모듈 맵(옵션)을 유지하고, 기술/명령은 모듈 AGENTS.md로 내려라.
- 모노레포라면 `references/module-map-format.md`를 사용해 모듈 맵을 작성하라.
- 기능 맵이 필요하면 모듈 AGENTS.md에 `references/feature-map-format.md`를 사용해 추가하라.
- 기존 섹션 제목과 포맷을 유지하라.
- 중복되는 규칙은 하나로 합치고, 의미가 다르면 병렬로 유지하라.
- 목록 중심으로 정리하고, 문단 길이를 최소화하라.
- AGENTS.md가 길어지면 “요약 + 링크”로 축약하고 상세는 다른 문서로 이동하라.

### 4) 출력 검증
- 변경된 섹션을 간단히 요약하고 사용자 확인을 받으라.
- 필요하면 `assets/agents-template.md`를 기반으로 전체 구조를 재정렬하라.

## Output Checklist

- AGENTS.md가 행동 규칙 위주인지 확인하라.
- 문서 링크가 최신이며 중복이 없는지 확인하라.
- 보안/민감정보 규칙이 누락되지 않았는지 확인하라.
- 루트/모듈 AGENTS.md 간 규칙 충돌이 없는지 확인하라.

## Resources

### references/
- `agents-md-question-bank.md`: AGENTS.md 질문 시퀀스
- `feature-map-format.md`: Feature map format
- `module-map-format.md`: Module map format (monorepo)
### assets/
- `agents-template.md`: 루트/모듈 AGENTS.md 기본 템플릿(요약형).
