---
name: test-orchestrator
description: 테스트 전략과 실행 순서를 정의하고 테스트 범위를 정리해야 할 때 사용.
---

# 테스트 오케스트레이션

## Overview

테스트 오케스트레이션 관련 작업을 체계화하라. 모든 결과는 한국어로 정리하고 핵심만 요약하라.

범위: 테스트 계획/범위, 실행 순서, 테스트 유형/우선순위
비범위: 리그레션 체크리스트(regression-plan), 테스트 리포트(test-report), QA 운영 기준(qa-test)

## Workflow

### 1) 컨텍스트 수집
- AGENTS.md, README.md, PRD.md, TRD.md를 확인하라.
- 대상 범위와 목표를 정리하라.
- 정보가 부족하면 먼저 질문하라.

### 2) 산출물 정의
- 필요한 산출물을 테스트 계획, 실행 순서, 테스트 범위 기준으로 정리하라.
- 우선순위와 완료 기준을 명확히 하라.

### 3) 실행 및 정리
- 필요한 경우 `references/`의 템플릿을 사용하라.
- 결과를 간결하게 요약하고 링크를 갱신하라.

## Output checklist

- 산출물이 목표와 범위를 충족한다.
- 완료 기준이 명확하다.
- 테스트 범위와 실행 순서가 명확하다.
- 공유 가능한 요약이 포함된다.
## Resources

### references/
- `test-plan-template.md`: 테스트 계획 템플릿
- `test-types.md`: 테스트 유형 요약
