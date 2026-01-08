---
name: skill-indexer
description: 저장소 내 스킬 목록을 SKILLS.md에 정리하라. 스킬 인덱스 문서가 필요할 때 사용.
---

# 스킬 인덱스

## Overview

스킬 목록을 수집해 `SKILLS.md`에 정리하라. 기존 내용은 보존하고 필요한 섹션만 갱신하라.

## Workflow

### 1) 스킬 수집
- `skills/` 아래의 `SKILL.md`를 모두 찾으라.
- 각 스킬의 `name`과 `description`을 추출하라.

### 2) 정렬 기준
- 기본값은 name 오름차순이다.
- 사용자가 다른 기준을 원하면 따른다.

### 3) 인덱스 작성
- `assets/SKILLS-template.md`를 사용하라.
- 기존 `SKILLS.md`가 있으면 표 섹션만 갱신하라.

### 4) 검증
- 스킬 경로가 실제로 존재하는지 확인하라.

## Output checklist

- 모든 스킬이 포함된다.
- name/description/path가 정확하다.

## Resources

### assets/
- `SKILLS-template.md`: 스킬 인덱스 템플릿.
