# Ralph Plan

Use this file as the active delivery contract for the current `llm-bootstrap`
program, not as a product surface template.

## Goal

Finish the current enablement backlog through bounded harness lanes until the
remaining work is reduced to routine follow-through, not repeated replanning.

## Requested Outcome

- keep the plan durable in-repo
- group the remaining work into a small number of harness lanes
- run each lane through the same Ralph loop until its exit criteria are met
- preserve baseline safety while increasing execution depth

## Bounded Scope

- current enablement backlog only
- no product rename
- no heavy runtime-first orchestration engine
- no repo-level workflow generation by default outside the optional lane

## Owners

- primary owner: `llm-bootstrap` core delivery loop
- active harness lanes:
  - `workflow-control-plane`
  - `repo-automation-lane`
  - `entrypoint-layer`
  - `precision-loop`
  - `company-live-loop`

## Connectors and Evidence Inputs

- source docs: [`docs/product-goal.ko.md`](docs/product-goal.ko.md),
  [`docs/superset-strategy.ko.md`](docs/superset-strategy.ko.md),
  [`docs/reference-repo-backlog.ko.md`](docs/reference-repo-backlog.ko.md)
- current implementation evidence: `bootstrap.toml`, `src/main.rs`,
  `.github/workflows/*`
- runtime evidence: `cargo test`, `install`, `doctor`, targeted CLI smoke

## Approval Boundary

- keep baseline integrity first
- keep provider-native surfaces first
- keep advanced lanes opt-in
- stop and re-evaluate if a lane starts forcing repo/runtime behavior into the
  default bootstrap path

## Verification Rule

Every harness lane closes only when all of the following are true:

1. code and docs agree
2. the narrowest credible tests pass
3. local install or runtime smoke proves the behavior
4. the next slice is explicitly rewritten before moving on

## Next Slice

Use [`docs/ralph-loop-program-plan.ko.md`](docs/ralph-loop-program-plan.ko.md)
as the detailed execution map, then continue in this order:

1. `workflow-control-plane`
2. `repo-automation-lane`
3. `entrypoint-layer`
4. `precision-loop`
5. `company-live-loop`
