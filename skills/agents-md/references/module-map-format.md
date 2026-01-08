# Module map format (monorepo)

짧고 스캔하기 쉽게 유지하라. 5-20개 항목을 권장한다.

```markdown
## Modules / subprojects

| Module | Type | Path | What it owns | How to run | Tests | Docs | AGENTS |
|--------|------|------|--------------|------------|-------|------|--------|
| backend | <spring> | `backend/` | APIs, DB, jobs | `...` | `...` | `docs/backend/...` | `backend/AGENTS.md` |
| frontend | <nextjs> | `frontend/` | UI, web client | `...` | `...` | `docs/frontend/...` | `frontend/AGENTS.md` |
```

Notes:
- "Type"은 빠른 라우팅 힌트만 적는다.
- "How to run"은 요약만 두고 자세한 명령은 모듈 AGENTS.md로 보낸다.
