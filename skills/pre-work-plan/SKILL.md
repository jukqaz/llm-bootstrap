---
name: pre-work-plan
description: 작업 직전에 계획을 세우고 간단한 작업은 파일로, 복잡한 작업은 Linear 이슈로 관리하라. 계획 수립, 범위/리스크 확인, 작업 시작 전 체크리스트가 필요할 때 사용.
---

# 작업 전 계획

## Overview

작업을 시작하기 전에 계획을 수립하라. 단순 작업은 로컬 계획 문서로 기록하고, 복잡한 작업은 Linear 이슈로 관리하라. 모든 문서는 한국어로 작성하고 파일명 대소문자를 엄격히 지켜라.

범위: 작업 시작 전 계획/범위/리스크 정리, 실행 체크리스트
비범위: 진행 상태 보고(status-report), 리스크 레지스터 유지(risk-register)

## Workflow

### 1) 컨텍스트 수집
- `AGENTS.md`, `README.md`, `PRD.md`, `TRD.md`를 확인하라.
- 작업 목표, 범위, 테스트 요구사항을 파악하라.

### 2) 질문 게이트
- `references/pre-work-plan-question-bank.md`의 필수 질문에 답이 없으면 먼저 질문하라.
- 답변 없이 계획을 작성하지 말아라.
- 계획 초안 작성 후 사용자 승인을 받기 전에는 작업을 시작하지 말아라.

### 3) 복잡도 분기
- `references/pre-work-plan-decision-criteria.md`로 단순/복잡을 판단하라.
- 판단이 모호하면 사용자에게 확인하라.
 - 복잡 작업인데 PRD/TRD가 없으면 `project-planning-docs` 사용을 제안하라.

### 4) 단순 작업: 로컬 계획 문서
- 인덱스는 `PLAN.md`에 유지하라.
- 세부 계획은 `plans/YYYY-MM-DD-<slug>.md`로 생성하라.
- 템플릿은 `assets/PLAN-template.md`를 사용하라.
- `PLAN.md`에는 최신 5~10개 항목만 유지하고, 오래된 항목은 `plans/archive/`로 이동하라.

### 5) 복잡 작업: Linear 이슈
- Linear가 설정되어 있으면 이슈를 생성하라.
- 팀/프로젝트, 우선순위, 마감일, 라벨을 확인하고 반영하라.
- 이슈 본문은 `assets/PLAN-template.md` 구조를 따르되, 수용 기준과 테스트 계획을 명확히 작성하라.
- Linear가 설정되지 않았으면 사용자에게 로컬 문서로 진행할지 확인하라.

### 6) 작업 종료 업데이트
- 작업 완료 시 계획 상태를 Done 또는 Abandoned로 갱신하라.
- 로컬 문서라면 `PLAN.md` 인덱스와 해당 계획 파일을 함께 업데이트하라.
 - Linear 이슈가 있으면 상태를 업데이트하고 완료 여부를 기록하라.

## 네이밍/대소문자 규칙
- 파일명은 `PLAN.md`를 사용하라.
- 계획 파일은 `plans/` 아래에 `YYYY-MM-DD-<slug>.md`로 저장하라.
- 기존 파일의 대소문자를 변경할 경우 사용자 승인 후 진행하라.
 - 같은 날짜에 여러 계획이 있으면 `YYYY-MM-DD-<slug>-2`처럼 번호를 붙여라.

## Output checklist

- 계획이 범위/리스크/테스트를 포함한다.
- 단순/복잡 분기가 명확하다.
- 인덱스와 계획 파일의 링크가 일치한다.
 - 작업 시작 전 승인 기록이 있다.

## Resources

### references/
- `pre-work-plan-decision-criteria.md`: 복잡도 분기 기준
- `pre-work-plan-question-bank.md`: 계획 수립 질문 목록
### assets/
- `PLAN-template.md`: 계획 템플릿.
- `PLAN-index-template.md`: 인덱스 템플릿.
