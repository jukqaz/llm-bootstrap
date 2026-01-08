---
name: infra-handoff
description: 인프라 운영/배포/모니터링 인수인계 산출물을 정리하고 실행 절차를 전달해야 할 때 사용.
---

# 인프라 인수인계

## Overview

인프라 인수인계를 위한 핵심 산출물을 정리하라. 비밀정보는 문서에 직접 적지 말고 안전한 전달 경로를 안내하라.

## Workflow

### 1) 컨텍스트 수집
- AGENTS.md, README.md, PRD.md, TRD.md, 기존 런북/배포 문서를 확인하라.
- 대상 환경, 서비스 목록, 운영 범위를 정리하라.

### 2) 인수인계 범위 정의
- 운영 책임, 온콜/에스컬레이션, SLO/SLI 범위를 확인하라.
- 데이터/백업/복구 책임 범위를 정리하라.

### 3) 산출물 작성
- `references/handoff-template.md`를 기반으로 인수인계 문서를 작성하라.
- 실행 절차는 `references/execution-guide.md`를 사용하라.
- 배포 상세는 `infra-release-runbook` 스킬을 참고하라.

### 4) 검증
- 배포/롤백/모니터링 확인 항목을 점검하라.
- 권한/접근 경로가 정상인지 확인하라.

### 5) 전달 및 확인
- 전달 대상과 일정, 확인 항목을 기록하라.
- 추가 질문이 있으면 수집해 반영하라.

## Output checklist

- 인수인계 범위와 책임이 명확하다.
- 실행 절차와 점검 항목이 포함된다.
- 민감정보는 안전한 채널로 분리된다.

## Resources

### references/
- `handoff-template.md`
- `execution-guide.md`
