---
name: role-dispatcher
description: 작업 성격에 맞는 역할을 빠르게 선택하고 해당 역할 스킬로 안내해야 할 때 사용.
---

# 역할 디스패처

## Overview

역할 디스패처 관련 작업을 체계화하라. 모든 결과는 한국어로 정리하고 핵심만 요약하라.

범위: 작업 성격에 맞는 역할/스킬 선택 안내
비범위: 개별 산출물 작성(각 역할 스킬)

## Workflow

### 1) 컨텍스트 수집
- AGENTS.md, README.md, PRD.md, TRD.md를 확인하라.
- 대상 범위와 목표를 정리하라.
- 정보가 부족하면 먼저 질문하라.

### 2) 산출물 정의
- 필요한 산출물을 역할 선택 결과, 추천 스킬 목록 기준으로 정리하라.
- 키워드 기반 추천이 필요하면 `references/keyword-map.json` 또는 `scripts/recommend_skills.py`를 사용하라.
- 우선순위와 완료 기준을 명확히 하라.

### 3) 실행 및 정리
- 필요한 경우 `references/`의 템플릿을 사용하라.
- 결과를 간결하게 요약하고 링크를 갱신하라.

## Output checklist

- 산출물이 목표와 범위를 충족한다.
- 완료 기준이 명확하다.
- 추천 역할/스킬과 선택 이유가 명확하다.
- 공유 가능한 요약이 포함된다.
## Resources

### references/
- `role-dispatch-questions.md`: 역할 선택 질문
- `keyword-map.json`: 키워드-스킬 매핑
### scripts/
- `recommend_skills.py`: 키워드 기반 스킬 추천 스크립트
