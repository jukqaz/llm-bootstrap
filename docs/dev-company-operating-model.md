# Dev + Company Operating Model

This document defines how `StackPilot` should evolve from a
"developer bootstrap" into a bootstrap for both
"software delivery and company operations."

It must satisfy both of these constraints:

- development execution has to stay strong
- company operations must run on the same foundation

But those two concerns should not be flattened into one giant layer.
They should share a common control plane while staying separate at the pack and
connector layers.

## Final Goal

The target state should be fixed as:

> install shared harnesses onto provider-native surfaces, then compose delivery
> packs and company-operation packs on top so that a small team can run both
> engineering and operating loops from the same base.

So this is not only a coding bootstrap.
But it is also not a giant universal runtime.

The intended stack is:

- small bootstrap core
- shared harness layer
- development packs
- company-operation packs
- connector packs
- automation layer

## Shared Principles

### 1. Keep the core small and safe

The core should still handle only:

- user-home installation
- backup and restore
- provider-native config rendering
- minimal MCP baseline
- doctor

It should not directly contain business-domain logic.

### 2. Development and operations share the same control layer

Development and company operations are not separate products.
They are work streams for the same team and should share:

- harnesses
- artifact contracts
- approval boundaries
- verification gates
- automation metadata

### 3. Functional capability should stay pack-based

Even on one runtime base, capability should remain separated into packs:

- `delivery-pack`
- `founder-pack`
- `growth-pack`
- `support-pack`
- `ops-pack`
- `finance-pack`

That is how the baseline stays lean while the capability stays rich.

### 4. External systems should enter through connector packs

Company operations are meaningless without external systems.
But shipping all of them by default would make the baseline too heavy.

So:

- the default install should include very few connectors
- connectors should activate only when required by a pack
- metadata should distinguish read-only, write-capable, and approval-required
  behavior

## Target Structure

### Layer 0. Bootstrap Core

Responsibilities:

- provider home baseline install
- backup and restore
- minimal MCP wiring
- near-core tools like RTK
- provider renderer execution

Representative files:

- [bootstrap.toml](../bootstrap.toml)
- [src/manifest.rs](../src/manifest.rs)
- [src/providers/codex.rs](../src/providers/codex.rs)
- [src/providers/gemini.rs](../src/providers/gemini.rs)
- [src/providers/claude.rs](../src/providers/claude.rs)

### Layer 1. Harness Layer

This is the common control plane for all work.

Minimum harness set:

- `ralph-loop`
- `delivery`
- `parallel-build`
- `incident`
- `review-gate`
- `founder-loop`
- `operating-review`

Each harness should define at least:

- team topology
- role ownership
- handoff schema
- stop rule
- verification rule
- final artifact contract

### Layer 2. Work Packs

This is where work begins to split.

#### Development packs

- `delivery-pack`
- `release-pack`
- `incident-pack`

Examples:

- office hours
- build plan
- implement
- review
- QA
- ship
- retro

#### Company-operation packs

- `founder-pack`
- `growth-pack`
- `support-pack`
- `ops-pack`
- `finance-pack`

Examples:

- founder review
- market scan
- pipeline review
- support digest
- KPI review
- operating retro
- finance checks

### Layer 3. Connector Packs

This layer connects work packs to real systems.

Minimum connector categories:

- communication
  - Gmail
  - Calendar
  - Slack
- knowledge
  - Drive
  - Docs
  - Notion
- product delivery
  - GitHub
  - Linear
  - Figma
- customer
  - CRM
  - helpdesk
- commerce / finance
  - Stripe
  - billing
  - accounting export
- analytics
  - GA4
  - Search Console
  - ads / attribution

Each connector metadata record should define:

- tool source
- required scope
- read or write level
- approval requirement
- whether automation is allowed
- fallback

### Layer 4. Automation Layer

This closes recurring operations.

Minimum automation set:

- daily founder brief
- weekly operating review
- weekly market scan
- weekly pipeline review
- daily support digest
- release readiness
- KPI summary

Rules:

- current-run evidence first
- write actions require explicit approval boundaries
- results should land in a thread inbox
- outputs should preserve a clear next action

## Pack Breakdown

### Development packs

#### `delivery-pack`

Goal:

- close the software-delivery loop from intent to ship

Representative artifacts:

- `Build Plan`
- `Risk Register`
- `QA Report`
- `Ship Decision`

Required harnesses:

- `ralph-loop`
- `delivery`
- `review-gate`

#### `incident-pack`

Goal:

- triage regressions and incidents quickly

Representative artifacts:

- `Incident Summary`
- `Root Cause Note`
- `Fix Verification`

Required harnesses:

- `incident`
- `review-gate`

### Company-operation packs

#### `founder-pack`

Goal:

- narrow product direction, market understanding, and priority decisions

Representative artifacts:

- `Opportunity Brief`
- `Wedge Decision`
- `Next Slice Plan`

Required harnesses:

- `founder-loop`
- `operating-review`

#### `growth-pack`

Goal:

- analyze acquisition and funnel issues

Representative artifacts:

- `Growth Review`
- `Funnel Diagnosis`
- `Outreach Plan`

#### `support-pack`

Goal:

- summarize customer issues and churn risk quickly

Representative artifacts:

- `Support Digest`
- `Escalation Note`
- `Churn Risk Summary`

#### `ops-pack`

Goal:

- bind together team operations, KPIs, and cross-functional review

Representative artifacts:

- `Weekly Operating Review`
- `KPI Narrative`
- `Ops Retro`

#### `finance-pack`

Goal:

- bring revenue, cost, and settlement checks into the operating loop

Representative artifacts:

- `Finance Check`
- `Revenue Snapshot`
- `Collection Risk Note`

## Implementation Order

The order matters and should stay fixed:

### Phase 1. Lock the shared control layer

- keep the source catalog
- design the harness catalog
- design the pack catalog
- prepare provider renderers to consume shared metadata

No company-operation connector work should happen yet.

### Phase 2. Implement development packs first

Start with:

- `ralph-loop`
- `delivery-pack`
- `review-gate`
- `incident-pack`

This phase stabilizes quality gates and doctor output.

### Phase 3. Add founder and ops packs

Then add the minimum company-operation packs:

- `founder-pack`
- `ops-pack`

At that point, the product begins to support both development and company
operations on one base.

### Phase 4. Add connector metadata

Priority order:

1. GitHub
2. Gmail
3. Calendar
4. Drive
5. CRM and helpdesk
6. finance and analytics

### Phase 5. Connect automation

Priority order:

1. weekly operating review
2. daily founder brief
3. support digest
4. pipeline review
5. release readiness

## Immediate Next Steps

Given the current repository, the correct next moves are:

1. define the `harness catalog` schema and document
2. define the `pack catalog` schema and document
3. lock `delivery-pack` and `founder-pack` as the first implementation targets
4. add `harness` and `pack` state concepts to doctor

## What Not To Do

- do not ship business connectors in the default install
- do not build a provider-agnostic runtime that overwrites native surfaces
- do not merge development and company-operation packs into one unit
- do not start with a state-heavy taskboard
- do not build automation before the harness and pack contracts exist

## Final Judgment

`StackPilot` should evolve into:

- a small bootstrap core
- a shared harness layer
- development packs
- company-operation packs
- connector packs
- an automation layer

In other words, it should become an operating base for both engineering and
company operations, not just a developer tool.

