# Feature map format

짧고 스캔하기 쉽게 유지하라. 모듈 AGENTS.md에 넣는 것을 기본값으로 한다.

```markdown
## Feature map

| Feature | Owner | Key paths | Entrypoints | Tests | Docs |
|--------|-------|-----------|-------------|-------|------|
| Auth | backend+frontend | `backend/...`, `frontend/...` | `...` | `...` | `docs/...` |
| Billing | backend | `backend/...` | `...` | `...` | `docs/...` |
```

Notes:
- "Owner"는 빠른 라우팅 힌트다.
- "Key paths"는 최소 경로만 적는다.
- "Entrypoints"는 컨트롤러/라우트/잡/핸들러 등 주요 시작점을 의미한다.
