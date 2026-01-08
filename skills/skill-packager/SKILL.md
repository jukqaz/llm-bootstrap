---
name: skill-packager
description: 스킬을 패키징(.skill)하고 검증 결과를 요약하라. 배포 전 패키징이 필요할 때 사용.
---

# 스킬 패키징

## Overview

스킬을 패키징하고 검증 결과를 요약하라. 스크립트가 없으면 사용자에게 다음 단계(설치/복사)를 확인하라.

범위: 로컬 패키징 실행, 결과 요약
비범위: 릴리즈 문서 갱신, 원격 배포/게시

## Workflow

### 1) 스크립트 찾기
- `package_skill.py` 위치를 찾으라.
- 우선순위: 레포 내 `scripts/` → `$CODEX_HOME/skills/.system/skill-creator/scripts/`.

### 2) 패키징 실행
- `package_skill.py <skill-dir> [output-dir]`로 실행하라.
- 실패 시 에러 메시지를 요약하고 수정 제안을 하라.

### 3) 결과 공유
- 생성된 `.skill` 파일 경로를 알려라.
- 검증 경고/오류를 요약하라.

## Output checklist

- 패키징 결과 파일 경로가 포함된다.
- 실패 시 원인과 수정 제안이 포함된다.

## Resources

### references/
- `skill-packager-checklist.md`: 패키징 체크리스트
