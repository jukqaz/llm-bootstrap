---
name: doc-linker
description: AGENTS.md와 README.md에 문서 링크를 추가/정리하라. 새 문서 생성 후 링크 정리가 필요할 때 사용.
---

# 문서 링크 정리

## Overview

문서 생성/갱신 후 AGENTS.md와 README.md의 링크를 정리하라. 링크 추가는 승인 후에만 수행하라.

범위: 핵심 문서 링크 정리, 링크 순서/대소문자 점검
비범위: 문서 본문 수정, 문서 신규 작성, 문서 스타일 점검(doc-style-enforcer), 문서 정합성 감사(docs-audit)

## Workflow

### 1) 대상 문서 확인
- `AGENTS.md`, `README.md`, `PRD.md`, `TRD.md`, `CHANGELOG.md`를 확인하라.
- 문서가 없으면 생성 여부를 사용자에게 확인하라.

### 2) 링크 규칙
- 링크 순서는 `AGENTS.md` → `PRD.md` → `TRD.md` → `CHANGELOG.md` → `SKILLS.md` → `ROLE-MAP.md` → `README.md`를 기본값으로 한다.
- 파일명 대소문자는 실제 파일과 일치해야 한다.

### 3) 링크 추가/갱신
- 이미 링크가 있으면 중복 추가하지 말아라.
- 링크 변경은 승인 후에만 적용하라.

### 4) 자동화(선택)
- `scripts/update_doc_links.py`로 링크 목록을 점검하라.
- 수정이 필요하면 `--write` 옵션 사용 전에 승인 여부를 확인하라.
- `--fail-on-missing` 사용 시 누락 문서를 오류로 보고한다.

## Output checklist

- 링크 순서가 규칙에 맞다.
- 대소문자 불일치가 없다.
- 승인 없이 링크를 변경하지 않았다.

## Resources

### references/
- `link-order.md`: 링크 순서 규칙
- `doc-linker-checklist.md`: 문서 링크 정리 체크리스트
### scripts/
- `update_doc_links.py`: 문서 링크 목록 자동 정리 스크립트
