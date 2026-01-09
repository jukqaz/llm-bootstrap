---
name: dependency-upgrade
description: 의존성 업데이트 범위와 리스크를 정리해야 할 때 사용.
---

# 의존성 업그레이드

## Overview

의존성 업그레이드 관련 작업을 체계화하라. 모든 결과는 한국어로 정리하고 핵심만 요약하라.

범위: 의존성 업데이트 범위/영향 분석, 리스크 정리
비범위: 코드 변경 실행(engineering), 리뷰 기준(review-checklist), 배포 준비(deploy-checklist)

## Workflow

### 1) 컨텍스트 수집
- AGENTS.md, README.md, PRD.md, TRD.md를 확인하라.
- 대상 범위와 목표를 정리하라.
- 정보가 부족하면 먼저 질문하라.

### 2) 산출물 정의
- 필요한 산출물을 업그레이드 계획, 리스크 요약 기준으로 정리하라.
- 우선순위와 완료 기준을 명확히 하라.

### 3) 실행 및 정리
- 필요한 경우 `references/`의 템플릿을 사용하라.
- 결과를 간결하게 요약하고 링크를 갱신하라.

## Output checklist

- 산출물이 목표와 범위를 충족한다.
- 완료 기준이 명확하다.
- 업그레이드 범위/리스크/롤백 계획이 포함된다.
- 공유 가능한 요약이 포함된다.
## Resources

### references/
- `dependency-upgrade-template.md`: 의존성 업그레이드 템플릿
