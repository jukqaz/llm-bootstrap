# Git worktree 치트시트

공식 문서: https://git-scm.com/docs/git-worktree

## 기본 확인

```bash
git worktree list
```

## 새 작업 디렉터리 추가

```bash
# 새 브랜치 생성 + 워크트리 추가
git worktree add -b <branch> <path>
```

```bash
# 기존 브랜치에 워크트리 추가
git worktree add <path> <branch>
```

## 정리

```bash
# 워크트리 제거(디렉터리 제거)
git worktree remove <path>
```

```bash
# 오래된 참조 정리
git worktree prune
```

## 주의사항

- 워크트리를 제거해도 브랜치는 남는다. 브랜치 삭제는 별도로 진행하라.
- `remove`는 디렉터리를 삭제하므로 반드시 사용자 확인을 받아라.
