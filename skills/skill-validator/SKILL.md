---
name: skill-validator
description: 저장소의 스킬 구조와 SKILL.md 프론트매터를 검사하고 누락/불일치를 리포트하라. 스킬 품질 점검이 필요할 때 사용.
---

# 스킬 검증

## Overview

스킬 디렉터리 구조, SKILL.md 프론트매터, 리소스 참조를 검증하라. 수정은 사용자 승인 후에만 하라.

범위: 스킬 구조/프론트매터/참조 검증
비범위: 자동 수정, 패키징/배포

## Workflow

### 1) 스킬 탐색
- `skills/` 아래의 `SKILL.md`를 모두 찾으라.

### 2) 구조 검사
- 각 스킬 디렉터리에 `SKILL.md`가 있는지 확인하라.
- 불필요한 문서(README 등)가 있는지 확인하라.
- `references/`, `assets/`, `scripts/`는 필요할 때만 존재해야 한다.

### 3) 프론트매터 검사
- `name`과 `description`이 존재하는지 확인하라.
- `name`이 폴더명과 일치하는지 확인하라.
- 대소문자 규칙(소문자+하이픈)을 벗어나면 리포트하라.

### 4) 참조 검증
- SKILL.md 본문에서 참조한 파일이 실제로 존재하는지 확인하라.
- SKILL.md와 references/ 내 Markdown 링크가 유효한지 확인하라.

### 5) 리포트
- 오류/경고를 구분해 제시하라.
- 수정은 승인 후에만 진행하라.

### 6) 자동화(선택)
- `scripts/validate_skills.py`로 구조/참조 검증을 수행하라.

## Output checklist

- 스킬 목록과 검사 결과가 포함된다.
- 프론트매터/구조/참조 오류가 구분된다.
- 깨진 Markdown 링크가 리포트된다.

## Resources

### references/
- `skill-validator-checklist.md`: 스킬 검증 체크리스트
### scripts/
- `validate_skills.py`: 스킬 구조 검증 스크립트
