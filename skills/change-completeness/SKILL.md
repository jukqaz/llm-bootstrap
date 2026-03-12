---
name: change-completeness
description: runtime, dependency, config 변경 후 테스트, CI, 문서, 운영 영향까지 누락 없이 점검하고 후속 수정 항목을 정리해야 할 때 사용.
---

# Change Completeness

코드 변경이 끝난 뒤 후속 수정과 검증 누락을 막기 위해 사용한다.

## Overview

범위: 테스트, CI, 문서, help/check command, 운영 영향, 릴리즈 영향 점검
비범위: 구조 단순화(code-polish-simplify), 공식 문서 감사(library-guideline-audit)

## Workflow

### 1) 변경 성격 분류
- runtime 변경
- dependency 변경
- config/env 변경
- docs-only 변경
- build/test tooling 변경

### 2) 후속 영향 점검
- 테스트 범위와 추가 검증이 필요한지 확인하라.
- CI나 local quality command가 바뀌는지 확인하라.
- README, CHANGELOG, AGENTS, runbook, checklist에 반영할 게 있는지 확인하라.
- 운영 시나리오나 롤백 경로가 바뀌는지 확인하라.

### 3) 누락 항목 정리
- “이미 했다”와 “해야 한다”를 분리해서 적어라.
- runtime 영향이 있으면 최소 smoke/regression 경로를 제시하라.
- dependency/config 영향이 있으면 환경 변수, 샘플 파일, bootstrap 절차 반영 여부를 확인하라.

### 4) 마감 기준
- 코드 diff만 끝난 상태를 완료로 보지 말아라.
- 테스트, CI, docs, ops 영향이 모두 처리되거나 명시적으로 deferred 되어야 한다.

## Output checklist

- 변경 유형이 명확하다.
- 테스트/CI/docs/ops 후속 항목이 정리되었다.
- 누락 없이 처리됐거나 deferred 이유가 남아 있다.

## Resources

### references/
- `change-completeness-checklist.md`: 변경 유형별 후속 점검 체크리스트
