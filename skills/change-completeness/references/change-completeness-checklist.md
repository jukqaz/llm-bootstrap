# Change Completeness Checklist

## Runtime change

- 주요 사용자 플로우 smoke 확인
- 관련 regression 경로 확인
- 로그/모니터링/오류 메시지 영향 확인

## Dependency change

- lockfile / version policy 반영
- deprecated API 정리
- CI 캐시나 빌드 단계 영향 확인

## Config or env change

- `.env.example` 또는 샘플 문서 반영
- bootstrap/setup 문서 반영
- fallback, default, failure mode 확인

## Docs and commands

- README
- CHANGELOG
- AGENTS
- quality/check/help command
- deploy/runbook/checklist

## Completion rule

- 처리 완료
- deferred with reason
- not applicable
