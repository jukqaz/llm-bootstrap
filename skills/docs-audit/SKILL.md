---
name: docs-audit
description: AGENTS.md/PRD.md/TRD.md/README.md/CHANGELOG.md 문서를 점검하고 링크/대소문자/규칙 충돌을 리포트하라. 문서 정합성 감사가 필요할 때 사용.
---

# 문서 감사

## Overview

핵심 문서를 읽고 정합성을 점검하라. 링크 경로, 대소문자, 규칙 충돌을 찾아 리포트하되, 수정은 사용자 승인 후에만 하라.

범위: 문서 정합성 감사, 링크/규칙 충돌 리포트
비범위: 문서 본문 수정, 대규모 문서 재작성, 링크 정리(doc-linker), 릴리즈 문서 갱신(release-docs)

## Workflow

### 1) 대상 문서 확인
- 루트의 `AGENTS.md`, `PRD.md`, `TRD.md`, `README.md`, `CHANGELOG.md`를 확인하라.
- 존재하지 않는 문서는 목록에 기록하라.

### 2) 링크/경로 검증
- 문서 간 링크가 실제 파일을 가리키는지 확인하라.
- 파일명 대소문자가 링크와 일치하는지 확인하라.

### 3) 규칙 충돌 점검
- 언어, 네이밍 규칙, 작업 원칙이 서로 충돌하는지 확인하라.
- 중복 규칙은 묶고 상충 규칙은 명시적으로 지적하라.

### 4) 결과 리포트
- 문제를 Severity로 분류하라 (High/Medium/Low).
- 수정 제안은 하되, 승인 없이는 수정하지 말아라.

## Output checklist

- 누락 문서 목록이 포함된다.
- 링크/대소문자 오류가 명시된다.
- 규칙 충돌과 중복이 정리되어 있다.

## Resources

### references/
- `docs-audit-checklist.md`: 문서 감사 체크리스트
### scripts/
- `docs-audit-validator/Cargo.toml`: 핵심 문서 섹션 검증(Rust)
