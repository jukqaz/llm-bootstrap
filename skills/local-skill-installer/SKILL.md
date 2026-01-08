---
name: local-skill-installer
description: 로컬 `skills/` 디렉터리에서 Codex 스킬을 `~/.codex/skills`로 설치하라. 네트워크 없이 설치가 필요할 때 사용.
---

# 로컬 스킬 설치

## Overview

로컬 저장소의 스킬을 Codex 홈 디렉터리로 복사하라. 기존 스킬이 있으면 덮어쓰기 전에 반드시 확인하라.

## Workflow

### 1) 대상 확인
- 설치할 스킬 이름과 경로를 확인하라.
- `CODEX_HOME`이 설정되어 있으면 그 경로를 사용하라.

### 2) 충돌 확인
- 대상 경로에 동일한 스킬이 있으면 덮어쓰기 여부를 확인하라.

### 3) 복사 수행
- `rsync -a` 또는 `cp -R`로 복사하라.
- 복사 후 `SKILL.md`가 있는지 확인하라.

### 4) 안내
- 설치 후 Codex 재시작을 안내하라.

## Output checklist

- 설치 경로와 설치된 스킬 목록이 포함된다.
- 덮어쓰기 여부가 명시된다.

## Resources

### references/
- `checklist.md`: 로컬 설치 체크리스트.
