---
name: code-polish-simplify
description: 오버엔지니어링을 줄이고 코드 폴리싱, 구조 단순화, 베이스라인 얼라인이 필요할 때 사용.
---

# Code Polish Simplify

구조가 과하게 복잡해졌거나 팀의 베이스라인과 어긋난 구현을 단순화할 때 사용한다.

## Overview

범위: 오버엔지니어링 제거, 책임 축소, 구조 단순화, naming/polish, 베이스라인 얼라인
비범위: 공식 문서 감사(library-guideline-audit), 변경 후 후속 점검(change-completeness)

## Workflow

### 1) 현재 복잡도 파악
- 상태, 분기, 추상화 층, helper 수, indirection을 빠르게 확인하라.
- “왜 이 복잡도가 생겼는지”보다 “지금 정말 필요한가”를 먼저 판단하라.

### 2) 제거 우선순위
- single-use abstraction
- 중복 wrapper
- 이유 없는 config 분기
- 파일 간 책임 분산으로만 보이는 helper
- 이름만 복잡한 thin adapter

### 3) 단순화 적용
- 더 적은 상태와 더 적은 경로를 우선하라.
- 공통 베이스라인이 있는 모듈이면 그 패턴에 먼저 맞춰라.
- 동작을 바꾸는 리팩터링보다 읽기 비용을 줄이는 리팩터링을 우선하라.

### 4) 정리와 설명
- 무엇을 제거했고 왜 단순해졌는지 짧게 설명하라.
- 후속 테스트나 CI/docs 영향이 있으면 change-completeness 연계를 제안하라.

## Output checklist

- 제거된 추상화/분기가 명확하다.
- 결과 구조가 기존보다 읽기 쉽다.
- 베이스라인과 어긋나는 지점이 줄었다.
- 남긴 복잡성은 이유가 설명된다.

## Resources

### references/
- `simplify-checklist.md`: 단순화 우선 패턴과 stop rules
