---
name: outsourcing-handoff
description: 외주/협력사에 작업 범위, 실행 절차, 산출물 기준을 인수인계해야 할 때 사용.
---

# 외주 인수인계

## Overview

외주 협업을 위해 작업 범위와 실행 절차를 명확히 전달하라. 계약/보안/권한 관련 사항은 별도 문서로 분리하라.

범위: 외주/협력사 인수인계, 범위/절차/산출물 기준
비범위: 벤더 조달(procurement-vendor), 내부 프로젝트 운영(project-ops), 인프라 인수인계(infra-handoff)

## Workflow

### 1) 컨텍스트 수집
- AGENTS.md, README.md, PRD.md, TRD.md를 확인하라.
- 외주 범위, 일정, 책임 구분을 정리하라.

### 2) 산출물 정의
- `references/handoff-package.md`를 기반으로 인수인계 패키지를 작성하라.
- 실행 절차는 `references/outsourcing-execution-guide.md`에 정리하라.
- 수용 기준은 `references/acceptance-checklist.md`를 사용하라.

### 3) 전달 및 확인
- 커뮤니케이션 채널/보고 주기를 명시하라.
- 전달 후 질문을 수집해 보완하라.

## Output checklist

- 범위/일정/수용 기준이 명확하다.
- 실행 절차와 환경 정보가 포함된다.
- 보안/권한 전달 경로가 분리된다.

## Resources

### references/
- `acceptance-checklist.md`: 수용 기준 체크리스트
- `handoff-package.md`: 외주 인수인계 패키지
- `outsourcing-execution-guide.md`: 외주 실행 가이드
