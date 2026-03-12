---
name: idea-triage
description: 수집한 아이디어 후보를 keep, maybe, drop으로 빠르게 걸러내고 우선순위를 정해야 할 때 사용.
---

# Idea Triage

이미 모인 아이디어 후보를 빠르게 걸러서 “더 볼 것”만 남길 때 사용한다.

## Overview

범위: 후보 평가, keep/maybe/drop 분류, 우선순위 결정, 탈락 이유 기록
비범위: 시장조사, 가격 검증, MVP 설계, 구현 계획

## Workflow

### 1) 평가 단위 정리
- 후보를 한 줄 가치 제안 단위로 다시 정리하라.
- 중복 후보나 이름만 다른 후보는 합쳐라.

### 2) 빠른 점수화
- 다음 기준으로 빠르게 보라:
  - 문제 강도
  - 반복 빈도
  - 1주 내 프로토타입 가능성
  - 직접 접근 가능한 사용자 존재 여부
  - 본인 흥미와 역량 적합도
- 정밀 점수보다 탈락 이유가 분명한지에 집중하라.

### 3) 분류
- `keep`: 바로 더 볼 가치가 있는 후보
- `maybe`: 아이디어는 괜찮지만 근거가 더 필요한 후보
- `drop`: 너무 넓거나, 약하거나, 지금 만들 이유가 약한 후보

### 4) shortlist 확정
- `keep`은 1~3개만 남겨라.
- 각 `keep` 후보에 다음 검증 질문을 붙여라.

## Output checklist

- keep / maybe / drop이 구분되어 있다.
- keep 후보가 과하게 많지 않다.
- 탈락 이유가 남아 있다.
- 다음 검증 질문이 있다.

## Next Flow

- keep 후보가 정해졌으면 `market-scan`으로 넘겨라.
- 근거가 애매하면 built-in 대화로 기준을 다시 다듬어라.

## Resources

### references/
- `triage-rules.md`: keep, maybe, drop 판단 기준
- `triage-output-format.md`: shortlist 정리 포맷
