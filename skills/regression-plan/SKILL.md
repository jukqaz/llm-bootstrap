---
name: regression-plan
description: 변경에 따른 리그레션 위험을 정리하고 반복 검증 항목을 설계해야 할 때 사용.
---

# 리그레션 플랜

## Overview

리그레션 플랜 관련 작업을 체계화하라. 모든 결과는 한국어로 정리하고 핵심만 요약하라.

범위: 리그레션 범위/위험, 반복 검증 체크리스트
비범위: 전체 테스트 계획(test-orchestrator), 테스트 리포트(test-report), QA 운영 기준(qa-test)

## Workflow

### 1) 컨텍스트 수집
- AGENTS.md, README.md, PRD.md, TRD.md를 확인하라.
- 대상 범위와 목표를 정리하라.
- 정보가 부족하면 먼저 질문하라.

### 2) 산출물 정의
- 필요한 산출물을 리그레션 체크리스트, 위험 범위 기준으로 정리하라.
- 우선순위와 완료 기준을 명확히 하라.

### 3) 실행 및 정리
- 필요한 경우 `references/`의 템플릿을 사용하라.
- 결과를 간결하게 요약하고 링크를 갱신하라.

## Output checklist

- 산출물이 목표와 범위를 충족한다.
- 완료 기준이 명확하다.
- 리그레션 범위와 핵심 체크 항목이 구분된다.
- 공유 가능한 요약이 포함된다.
## Resources

### references/
- `regression-checklist.md`: 리그레션 체크리스트
