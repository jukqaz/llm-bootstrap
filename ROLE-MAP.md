# ROLE-MAP

한 사람이 여러 역할을 맡을 때 필요한 작업 흐름과 스킬 구성을 정리한다. 역할은 크게 나누되, 스킬은 역할 단위로 분리해 재사용성을 높인다.

## 역할 축

### 공통/메타
- 산출물: 역할 선택, 의사결정 기록, 사후 분석, 상태 업데이트, KPI 요약
- 이미 구현됨: role-dispatcher, decision-log, postmortem, stakeholder-update, kpi-dashboard-brief
- 추가 예정: 없음

### 법무/계약
- 산출물: 계약/약관 요약, 변경 이력
- 이미 구현됨: legal-contracts
- 추가 예정: 없음

### 재무/회계
- 산출물: 재무 요약, 정산/예산 템플릿
- 이미 구현됨: finance-accounting
- 추가 예정: 없음

### HR/피플 운영
- 산출물: 채용/온보딩 산출물
- 이미 구현됨: hr-people-ops
- 추가 예정: 없음

### 문서/스킬 운영
- 산출물: 문서 정합성 점검, 스킬 패키징/배포, 스킬 목록 관리
- 이미 구현됨: doc-linker, docs-audit, readme-maintainer, skill-indexer, skill-packager, skill-release, skill-template-sync, skill-validator, local-skill-installer
- 추가 예정: 없음

### 조달/벤더
- 산출물: 벤더 브리프, 조달 체크리스트
- 이미 구현됨: procurement-vendor
- 추가 예정: 없음

### IT/내부 시스템
- 산출물: IT 운영 체크리스트, IT 자산 로그
- 이미 구현됨: it-internal-systems
- 추가 예정: 없음

### DevRel/커뮤니티
- 산출물: 커뮤니티 브리프, 이벤트 체크리스트
- 이미 구현됨: devrel-community
- 추가 예정: 없음

### 로컬라이징/국제화
- 산출물: 로컬라이징 가이드, 체크리스트
- 이미 구현됨: localization-i18n
- 추가 예정: 없음

### 시설/총무
- 산출물: 시설 운영 체크리스트, 시설 자산 로그
- 이미 구현됨: facilities-admin
- 추가 예정: 없음

### 제품/전략
- 산출물: PRD, TRD, 로드맵, 기능 맵
- 이미 구현됨: project-planning-docs, agents-followup-docs, agents-md, product-strategy, roadmap-planner, feature-map-builder, requirements-review
- 추가 예정: 없음

### 프로젝트/운영
- 산출물: 작업 계획, 일정, 리스크, 진행 상태 기록
- 이미 구현됨: pre-work-plan, plan-archive, project-ops, status-report, risk-register, outsourcing-handoff
- 추가 예정: 없음

### 디자인/UX
- 산출물: IA, 와이어프레임, UX 카피, 디자인 가이드
- 이미 구현됨: design-ux, ux-research, ux-copy, design-spec, wireframe-brief
- 추가 예정: 없음

### 엔지니어링
- 산출물: 코드 변경, 리팩터링, 기술 부채 정리
- 이미 구현됨: github-pr-ci, parallel-work, doc-style-enforcer, engineering, dev-cycle, dependency-upgrade, review-checklist
- 추가 예정: 없음

### 인프라/플랫폼
- 산출물: 배포 체크리스트, 환경 설정, 모니터링 기준
- 이미 구현됨: release-docs, infra-platform, infra-release-runbook, ops-monitoring, deploy-checklist, infra-handoff
- 추가 예정: 없음

### QA/테스트
- 산출물: 테스트 계획/결과, 리그레션 체크리스트
- 이미 구현됨: qa-test, test-orchestrator, regression-plan, test-report
- 추가 예정: 없음

### 데이터/분석
- 산출물: 지표 정의, 분석 요약, 리포트
- 이미 구현됨: data-analytics, analytics-summary, metric-definition, experiment-report
- 추가 예정: 없음

### 보안/컴플라이언스
- 산출물: 보안 체크리스트, 위협 모델, 권한 점검
- 이미 구현됨: security-compliance, security-review, access-audit, threat-model
- 추가 예정: 없음

### 마케팅/브랜드
- 산출물: 블로그, 광고 카피, SEO 초안, 캠페인 요약
- 이미 구현됨: marketing-brand, marketing-content, seo-research, campaign-plan, brand-voice
- 추가 예정: 없음

### 세일즈/BD
- 산출물: 제안서, 파트너십 문서, 세일즈 메시지, 가격/세일즈 FAQ
- 이미 구현됨: sales-bd, sales-proposal, partner-brief, pricing-faq
- 추가 예정: 없음

### 고객지원/CS
- 산출물: 지원 FAQ, 고객 응대 템플릿, 장애 공지
- 이미 구현됨: customer-support, support-response, incident-brief, faq-builder
- 추가 예정: 없음

### 경영/운영
- 산출물: 운영 비용/예산 기록, 운영 규정, 내부 가이드
- 이미 구현됨: management-ops, ops-admin, cost-tracking, policy-docs
- 추가 예정: 없음

## 스킬 현황

### 이미 구현된 스킬
- access-audit
- agents-followup-docs
- agents-md
- analytics-summary
- brand-voice
- campaign-plan
- cost-tracking
- customer-support
- data-analytics
- decision-log
- dependency-upgrade
- deploy-checklist
- design-spec
- design-ux
- dev-cycle
- devrel-community
- doc-linker
- doc-style-enforcer
- docs-audit
- engineering
- experiment-report
- facilities-admin
- faq-builder
- feature-map-builder
- finance-accounting
- github-pr-ci
- hr-people-ops
- incident-brief
- infra-handoff
- infra-platform
- infra-release-runbook
- it-internal-systems
- kpi-dashboard-brief
- legal-contracts
- local-skill-installer
- localization-i18n
- management-ops
- marketing-brand
- marketing-content
- metric-definition
- ops-admin
- ops-monitoring
- outsourcing-handoff
- parallel-work
- partner-brief
- plan-archive
- policy-docs
- postmortem
- pre-work-plan
- pricing-faq
- procurement-vendor
- product-strategy
- project-ops
- project-planning-docs
- qa-test
- readme-maintainer
- regression-plan
- release-docs
- requirements-review
- review-checklist
- risk-register
- roadmap-planner
- role-dispatcher
- sales-bd
- sales-proposal
- security-compliance
- security-review
- seo-research
- skill-indexer
- skill-packager
- skill-release
- skill-template-sync
- skill-validator
- stakeholder-update
- status-report
- support-response
- test-orchestrator
- test-report
- threat-model
- ux-copy
- ux-research
- wireframe-brief

### 우선 확장 후보
- 없음
