---
name: security-review
description: 보안 체크리스트 기반으로 변경 사항을 점검하고 리스크를 요약해야 할 때 사용.
---

# 보안 리뷰

## Overview

보안 리뷰 관련 작업을 체계화하라. 모든 결과는 한국어로 정리하고 핵심만 요약하라.

범위: 변경 사항 보안 체크리스트 점검, 리스크 요약
비범위: 컴플라이언스 체계(security-compliance), 위협 모델(threat-model), 접근 감사(access-audit)

## Workflow

### 1) 컨텍스트 수집
- AGENTS.md, README.md, PRD.md, TRD.md를 확인하라.
- 대상 범위와 목표를 정리하라.
- 정보가 부족하면 먼저 질문하라.

### 2) 산출물 정의
- 필요한 산출물을 보안 체크리스트 결과, 리스크 요약 기준으로 정리하라.
- 우선순위와 완료 기준을 명확히 하라.

### 3) 실행 및 정리
- 필요한 경우 `references/`의 템플릿을 사용하라.
- 결과를 간결하게 요약하고 링크를 갱신하라.

## Output checklist

- 산출물이 목표와 범위를 충족한다.
- 완료 기준이 명확하다.
- 공유 가능한 요약이 포함된다.

## Resources

### references/
- `risk-brief.md`: 보안 리스크 요약 템플릿
- `security-checklist.md`: 보안 체크리스트
