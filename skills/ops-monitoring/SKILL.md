---
name: ops-monitoring
description: 서비스 모니터링 기준, 알림 정책, 운영 점검 항목을 정리해야 할 때 사용.
---

# 운영 모니터링

## Overview

운영 모니터링 관련 작업을 체계화하라. 모든 결과는 한국어로 정리하고 핵심만 요약하라.

범위: 모니터링 지표/알림 정책, 운영 점검 항목, 모니터링 체크리스트
비범위: 배포 런북/체크리스트(infra-release-runbook, deploy-checklist), 인프라 운영 기준(infra-platform)

## Workflow

### 1) 컨텍스트 수집
- AGENTS.md, README.md, PRD.md, TRD.md를 확인하라.
- 대상 범위와 목표를 정리하라.
- 정보가 부족하면 먼저 질문하라.

### 2) 산출물 정의
- 필요한 산출물을 모니터링 체크리스트, 알림 기준, 운영 점검 항목 기준으로 정리하라.
- 우선순위와 완료 기준을 명확히 하라.

### 3) 실행 및 정리
- 필요한 경우 `references/`의 템플릿을 사용하라.
- 결과를 간결하게 요약하고 링크를 갱신하라.

## Output checklist

- 산출물이 목표와 범위를 충족한다.
- 완료 기준이 명확하다.
- 핵심 지표/알림 기준과 대응 루틴이 포함된다.
- 공유 가능한 요약이 포함된다.
## Resources

### references/
- `alert-brief.md`: 알림 요약 템플릿
- `monitoring-checklist.md`: 모니터링 체크리스트
