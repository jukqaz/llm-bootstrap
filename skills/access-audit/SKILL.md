---
name: access-audit
description: 접근 권한과 역할 구성을 점검하고 변경 이력을 정리해야 할 때 사용.
---

# 권한 점검

## Overview

권한 점검 관련 작업을 체계화하라. 모든 결과는 한국어로 정리하고 핵심만 요약하라.

범위: 접근 권한/역할 구성 점검, 권한 변경 이력, 접근 로그 요약
비범위: 보안 정책/컴플라이언스 수립(security-compliance), 코드/설계 보안 리뷰(security-review)

## Workflow

### 1) 컨텍스트 수집
- AGENTS.md, README.md, PRD.md, TRD.md를 확인하라.
- 대상 범위와 목표를 정리하라.
- 정보가 부족하면 먼저 질문하라.

### 2) 산출물 정의
- 필요한 산출물을 권한 점검 결과, 변경 이력 요약 기준으로 정리하라.
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
- `access-checklist.md`: 권한 점검 체크리스트
- `access-log-template.md`: 권한 변경 로그 템플릿
