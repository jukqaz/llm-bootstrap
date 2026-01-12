---
name: repo-doc-bootstrap
description: 기존 프로젝트에 Codex 문서 세트(AGENTS.md/PRD.md/TRD.md/README.md/ROLE-MAP.md/CHANGELOG.md)를 자동 생성해 초기 세팅을 빠르게 끝낼 때 사용. 기존 코드/파일을 스캔해 기술 스택과 기본 실행 명령을 채우고, 누락 문서만 생성하거나 필요 시 덮어쓴다.
---

# Repo Doc Bootstrap

## Overview

기존 프로젝트에 Codex 문서 세트를 빠르게 부트스트랩하라. 기본 템플릿을 생성하고, 스택/명령 힌트를 자동으로 채운 뒤 사람 검토로 마무리한다.

범위: 문서 부트스트랩 생성, 누락 문서 자동 생성
비범위: 상세 내용 완성, 문서 정합성 감사(docs-audit)

## Workflow

### 1) 컨텍스트 수집
- 대상 리포 루트 경로를 확인하라.
- 기존 문서(AGENTS/PRD/TRD/README/ROLE-MAP/CHANGELOG) 유무를 점검하라.
- 덮어쓰기 여부(`--force`)를 사용자에게 확인하라.

### 2) 자동 생성 실행
- `scripts/bootstrap-docs`로 문서 초안을 생성하라.
- 기본은 누락 문서만 생성하고, 덮어쓰기는 `--force`로만 수행하라.
- 미리보기만 필요하면 `--dry-run`으로 실행하라.

### 3) 보완 및 검증
- 생성된 문서의 TODO 항목을 채우도록 안내하라.
- `task validate-agents`, `task validate-docs`, `task check`를 통해 검증하라.

## Usage

- `cargo run --manifest-path skills/repo-doc-bootstrap/scripts/bootstrap-docs/Cargo.toml -- --root .`
- `cargo run --manifest-path skills/repo-doc-bootstrap/scripts/bootstrap-docs/Cargo.toml -- --root . --dry-run`
- `cargo run --manifest-path skills/repo-doc-bootstrap/scripts/bootstrap-docs/Cargo.toml -- --root . --force`

## Output checklist

- AGENTS/PRD/TRD/README/ROLE-MAP/CHANGELOG 초안이 생성되었다.
- 기술 스택/명령 힌트가 자동 채워졌다.
- TODO 항목이 표시되어 있다.
- 문서 검증이 통과한다.

## Resources

### assets/
- `AGENTS-template.md`: AGENTS.md 기본 템플릿
- `PRD-template.md`: PRD 템플릿
- `TRD-template.md`: TRD 템플릿
- `README-template.md`: README 템플릿
- `ROLE-MAP-template.md`: ROLE-MAP 템플릿
- `CHANGELOG-template.md`: CHANGELOG 템플릿
### scripts/
- `bootstrap-docs/Cargo.toml`: 문서 부트스트랩 생성(Rust)
