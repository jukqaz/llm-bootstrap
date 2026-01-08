---
name: release-docs
description: 배포/릴리즈 시 README.md와 CHANGELOG.md를 함께 갱신하라. 릴리즈 노트 작성, 변경 로그 정리, 배포 전 문서 정리가 필요할 때 사용.
---

# 배포 문서 묶음

## Overview

릴리즈 전에 README.md와 CHANGELOG.md를 정리하라. 변경 사항을 요약하고 링크를 일관되게 유지하며, 문서 파일명과 대소문자를 정확히 맞춰라.

## Workflow

### 1) 컨텍스트 수집
- `AGENTS.md`와 `README.md`, `CHANGELOG.md`를 읽어라.
- 기존 릴리즈 프로세스가 있으면(버전 규칙, 태그, 릴리즈 노트 형식) 확인하라.

### 2) 릴리즈 범위 확인
- 릴리즈 버전과 날짜를 확인하라.
- 릴리즈 범위(이전 태그..현재 태그/HEAD)를 확인하라.
- 필요 시 최근 변경 사항 요약을 위해 `git log`를 사용하라.

### 3) 네이밍/대소문자 규칙
- 정확한 파일명을 사용하라: `README.md`, `CHANGELOG.md`.
- 다른 대소문자 파일이 있으면 변경 전에 사용자 확인을 받으라.

### 4) CHANGELOG 갱신
- `assets/CHANGELOG-template.md` 형식을 사용하라.
- 최신 릴리즈 항목을 최상단에 추가하라.
- 변경 사항을 Added/Changed/Fixed/Removed로 분류하라.
 - 변경 범위가 불명확하면 사용자에게 질문하고 멈춰라.

### 5) README 갱신
- README에 변경 로그 링크가 없으면 추가하라.
- 릴리즈 관련 정보(버전/업데이트일)가 있으면 최신으로 갱신하라.
- README가 없으면 생성 여부를 사용자에게 확인하라.

### 6) 리뷰 및 승인
- 변경 사항을 요약하고 사용자 승인 후 반영하라.

## Output checklist

- README/CHANGELOG 링크가 정확하고 대소문자가 일관적이다.
- CHANGELOG 최신 항목이 상단에 있다.
- 기존 README/CHANGELOG 내용을 불필요하게 삭제하지 않았다.

## Resources

### references/
- `release-docs-question-bank.md`: 릴리즈 문서 질문 목록
### assets/
- `CHANGELOG-template.md`: 변경 로그 템플릿.
- `README-release-snippet.md`: README에 추가할 릴리즈 섹션 스니펫.
