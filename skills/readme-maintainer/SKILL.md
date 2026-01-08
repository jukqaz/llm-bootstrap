---
name: readme-maintainer
description: README.md를 작업 종료 시점에 생성하거나 갱신하라. 문서 링크 정리, 변경 요약 반영, 대소문자 일관성을 유지해야 할 때 사용.
---

# README 유지관리

## Overview

작업을 마무리할 때 README.md를 생성하거나 갱신하라. 기존 내용을 보존하고, 변경 내용을 요약하며, 문서 링크를 일관되게 유지하라. 큰 재작성은 피하고 필요한 섹션만 수정하라.

## Workflow

### 1) 컨텍스트 수집
- `README.md`, `AGENTS.md`, `PRD.md`, `TRD.md`를 읽어라.
- README가 없으면 생성 여부를 사용자에게 확인하라.

### 2) 네이밍/대소문자 규칙
- 정확한 파일명 `README.md`를 사용하라.
- 다른 대소문자 파일이 있으면 변경 전에 사용자 확인을 받으라.

### 3) 내용 갱신
- `assets/README-template.md`를 기본 템플릿으로 사용하라.
- 기존 README가 있으면 중복을 피하고 필요한 섹션만 추가하라.
- 변경 사항 요약, 프로젝트 목적, 문서 링크를 포함하라.
 - 기존 헤딩을 유지해 링크 깨짐을 방지하라.

### 4) 링크 정리
- 문서 링크는 `AGENTS.md`, `PRD.md`, `TRD.md` 순서로 유지하라.
- 링크 텍스트와 파일명이 일치하는지 확인하라.

### 5) 리뷰 및 승인
- 변경 사항을 요약하고 사용자 승인 후 반영하라.

## Output checklist

- README 내용이 프로젝트 목적과 사용법을 간단히 설명한다.
- 문서 링크가 정확하고 대소문자가 일관적이다.
- 기존 README 내용이 불필요하게 삭제되지 않았다.

## Resources

### references/
- `question-bank.md`: README 갱신에 필요한 질문 목록.

### assets/
- `README-template.md`: 기본 README 템플릿.
