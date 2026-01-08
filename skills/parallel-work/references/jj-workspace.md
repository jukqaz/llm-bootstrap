# Jujutsu workspace 치트시트

공식 문서: https://docs.jj-vcs.dev/latest/cli-reference/

## 기본 확인

```bash
jj workspace list
```

## 새 워크스페이스 추가

```bash
jj workspace add <path>
```

## 워크스페이스 제거

```bash
# 이름은 list로 확인 후 입력
jj workspace forget <name>
```

## 주의사항

- 정확한 옵션/플래그는 `jj help workspace`로 확인하라.
- 제거 작업은 사용자 확인 후 진행하라.
