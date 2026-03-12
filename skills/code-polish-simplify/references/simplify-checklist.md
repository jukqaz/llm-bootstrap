# Simplify Checklist

## Prefer removing

- 한 번만 쓰는 adapter, bridge, coordinator
- 값 전달만 하는 wrapper
- runtime에서 의미 없는 feature flag
- 동일 책임의 helper가 여러 파일로 찢어진 구조
- 설명 없이는 이해되지 않는 naming

## Keep only if justified

- 테스트 격리 때문에 필요한 seam
- 플랫폼 차이 때문에 유지해야 하는 분기
- 롤백/운영 안전성을 위한 최소 guard
- 향후 확장보다 현재 운영상 꼭 필요한 abstraction

## Stop rules

- 줄 수가 줄어도 책임 경계가 더 흐려지면 중단
- 공통 패턴과 맞지 않는데 이유를 설명 못하면 더 줄이지 말고 정리부터 한다
- 단순화가 runtime behavior를 바꾸면 change-completeness 검증을 붙인다
