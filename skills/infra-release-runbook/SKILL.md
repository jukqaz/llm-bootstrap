---
name: infra-release-runbook
description: 배포 절차, 검증, 롤백을 포함한 런북을 만들고 배포 실행 체크리스트를 관리해야 할 때 사용.
---

# 배포 런북

## Overview

배포 런북 관련 작업을 체계화하라. 모든 결과는 한국어로 정리하고 핵심만 요약하라.

범위: 배포 절차/검증/롤백 상세 런북
비범위: 간단 배포 체크리스트(deploy-checklist), 모니터링 정책(ops-monitoring), 인프라 운영 기준(infra-platform)

## Workflow

### 1) 컨텍스트 수집
- AGENTS.md, README.md, PRD.md, TRD.md를 확인하라.
- 대상 범위와 목표를 정리하라.
- 정보가 부족하면 먼저 질문하라.

### 2) 산출물 정의
- 필요한 산출물을 배포 런북, 검증/롤백 절차, 단계별 체크 항목 기준으로 정리하라.
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
- `rollback-checklist.md`: 롤백 체크리스트
- `runbook-template.md`: 배포 런북 템플릿
