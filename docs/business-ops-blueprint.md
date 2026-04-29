# Business Ops Blueprint

To move `StackPilot` beyond a developer bootstrap and toward a
business-capable operating layer, the product definition has to change.

This document explains the gap between the current repository and tools such as
`gstack` and `oh-my-codex`, then outlines the architecture needed to cover not
just delivery work but broader business operations.

## Current State

Today `StackPilot` is focused on:

- provider home bootstrap and recovery
- minimal MCP baseline management
- workflow docs plus lightweight skill or agent bundles
- safe operational commands such as `install`, `doctor`, `restore`,
  `uninstall`, and `wizard`

The repository intentionally does not provide:

- persistent taskboards or session memory systems
- large workflow state machines
- large default MCP bundles
- built-in business connectors
- integrated surfaces for sales, support, marketing, finance, or operations

In other words, the current product is closer to an AI tooling bootstrap than a
business operating system.

## External Comparison

### gstack

Based on its public README, `gstack` is a strong software sprint process layer.

- `office-hours -> plan -> review -> qa -> ship -> retro`
- product framing, design review, DX review, security, QA, and release in one
  chain
- artifact-first flow where one stage feeds the next
- stronger founder or product framing than a typical coding harness

Strengths:

- closes the gap between product definition and engineering execution
- enforces thinking about scope, quality, and shipping evidence together
- has a visible pre- and post-release operating loop

Limits:

- still centered on software sprint execution
- does not primarily productize connectors and permission systems
- stronger on process than on a full business data plane

### oh-my-codex

Based on its public README, `oh-my-codex` is focused on multi-agent
orchestration for OpenAI Codex CLI.

- many specialist agents
- mode-based routing
- parallel execution
- session persistence
- auto-routing for model and reasoning

Strengths:

- strong for pushing large engineering tasks through orchestration
- clear separation across executor, tester, reviewer, and specialist roles
- a thicker runtime layer than Codex by itself

Limits:

- weaker on business-wide connectors and artifact contracts
- more coding-runtime oriented than business-operations oriented
- closer to a Codex execution layer than a cross-provider home bootstrap

### Current Position of stackpilot

`StackPilot` sits below both tools in the stack.

- strength: safe baseline, backup and restore, provider consistency, lean MCP
- weakness: thin execution loop for real work
- user perception: it organizes the environment but does not run the business

That perception is accurate because the current scope is defined that way.

## Problem Statement

The user need is not just a coding bootstrap.

The actual desired surface includes:

- product definition
- engineering delivery
- design review
- QA and release
- market research
- sales follow-up
- support triage
- operational reporting
- weekly retrospectives

The target is not only "an agent that writes code" but "an operating system for
a small team."

## Target Product Definition

The next-stage product definition for `StackPilot` should be:

> A cross-provider operating system that keeps a safe developer-home bootstrap
> core while adding workflow packs and business connector packs for product,
> delivery, and operational loops.

The key move is to preserve the bootstrap core and add higher layers, not to
replace the core with a brand-new runtime.

## Design Principles

- separate core bootstrap from operational packs
- keep the default install lean
- make business features opt-in through presets or packs
- reuse provider-native surfaces wherever possible
- require explicit permission boundaries for business connectors
- minimize hidden state in automations
- keep outputs reusable as inputs to the next step
- standardize non-engineering work through artifact contracts

## Target Architecture

### Layer 0: Bootstrap Core

This is the current repository's job.

- provider install and recovery
- backup and restore
- minimal MCP wiring
- env-gated MCP
- distribution of native docs, skills, commands, and agents
- doctor checks and runtime dependency validation

This layer should remain conservative.

### Layer 1: Workflow Packs

These are playbooks for product, delivery, and operations.

Candidate packs:

- `founder-pack`
- `delivery-pack`
- `launch-pack`
- `support-pack`
- `growth-pack`
- `ops-pack`

Each pack should bundle docs, skills, commands, agent rosters, and output
contracts.

Examples:

- `founder-pack`: problem framing, market scan, positioning, weekly review
- `delivery-pack`: office hours, planning, implementation, review, QA, ship
- `launch-pack`: release brief, launch checklist, post-launch watch
- `support-pack`: ticket triage, incident summary, churn-risk digest
- `growth-pack`: campaign review, funnel diagnostics, creator outreach
- `ops-pack`: KPI review, weekly operating review, cross-function retro

### Layer 2: Connector Packs

Business-wide execution requires external systems.

Minimum connector categories:

- communication: Gmail, Calendar, Slack
- knowledge: Drive, Docs, Notion
- product delivery: GitHub, Linear, Figma
- customer: Intercom, Zendesk, HubSpot
- commerce and finance: Stripe, Shopify, accounting export
- analytics and growth: GA4, Search Console, ad platforms

Each connector needs an explicit contract:

- which MCP, app, or tool it uses
- which permission scopes are required
- whether it is read-only or write-capable
- whether automation is allowed
- what the fallback path is

### Layer 3: Business Automations

Recurring operations should close the loop.

Candidate automations:

- daily founder brief
- weekly market scan
- weekly pipeline review
- daily support digest
- release readiness check
- post-launch canary review
- monthly KPI narrative

Rules for this layer:

- use current-run evidence first
- keep explicit approval boundaries for external writes
- report through thread inbox style outputs
- prefer outputs that point to the next action

## Capability Domains

### 1. Product Strategy

Needed capabilities:

- founder office hours
- problem statement refinement
- opportunity sizing
- competitor comparison
- pricing hypothesis review
- roadmap slicing

Representative artifacts:

- `Opportunity Brief`
- `Wedge Decision`
- `Next Slice Plan`

### 2. Delivery

Needed capabilities:

- office hours
- implementation plan
- engineering review
- design review
- QA
- ship checklist

Representative artifacts:

- `Build Plan`
- `Risk Register`
- `QA Report`
- `Ship Decision`

### 3. Go-To-Market

Needed capabilities:

- launch planning
- campaign asset checklist
- channel experiment review
- creator or partner outreach drafting
- landing page critique

Representative artifacts:

- `Launch Brief`
- `Campaign Review`
- `Landing Page Audit`

### 4. Sales and Customer Success

Needed capabilities:

- lead digest
- meeting prep
- follow-up drafts
- pipeline health review
- churn-risk review

Representative artifacts:

- `Pipeline Review`
- `Meeting Brief`
- `Follow-up Draft`

### 5. Support and Operations

Needed capabilities:

- issue triage
- incident summary
- refund or dispute prep
- VOC summarization
- weekly operating review

Representative artifacts:

- `Support Digest`
- `Incident Review`
- `VOC Summary`
- `Weekly Ops Review`

## Pack-Centered Product Structure

The repository already prefers preset-driven layering, so packs are the right
expansion unit.

### Recommended Presets

- `light`
  - bootstrap core only
- `normal`
  - core plus delivery baseline
- `full`
  - core plus delivery plus founder baseline
- `business`
  - full plus connector packs plus automation templates

`business` should not become the default preset because it adds cost,
permission, and environment complexity.

### Recommended Pack Layout

- `packs/delivery`
- `packs/founder`
- `packs/growth`
- `packs/support`
- `packs/ops`
- `packs/connectors`

Each pack should contain:

- docs
- skills
- commands
- agents
- automation templates
- permission manifest

## Proposed Artifact Contracts

Non-engineering work also needs strong output schemas.

Example contracts:

### Founder Office Hours

- `Problem:`
- `User Pain:`
- `Current Workaround:`
- `Why Now:`
- `Narrowest Wedge:`
- `Next Experiment:`

### Market Scan

- `Signal:`
- `Why It Matters:`
- `Implication:`
- `Recommended Move:`
- `Evidence:`

### Pipeline Review

- `Top Deals:`
- `Stalled Deals:`
- `Risks:`
- `Actions This Week:`

### Support Digest

- `Top Issues:`
- `Affected Users:`
- `Severity:`
- `Immediate Action:`
- `Longer-term Fix:`

## Permissions and Security

The wider the business scope, the more important the security model becomes.

Required rules:

- split read and write permissions per connector
- define approval boundaries per tool
- classify customer and finance data as high sensitivity
- make automatic external writes opt-in
- never store live secrets in the bootstrap repository
- teach `doctor` to report permission gaps separately from missing connectors

## Competitive Positioning

### Against gstack

`gstack` wins on process strength.
`StackPilot` should not try to compete by merely adding more skills.

The better differentiation is:

- cross-provider baseline and home-state safety
- lean default with pack opt-in
- connector and permission manifest as first-class product surfaces
- the same structure across founder, growth, support, and ops
- a work OS that includes automation and connectors

### Against oh-my-codex

`oh-my-codex` wins on orchestration runtime depth.

The better differentiation is:

- provider-neutral bootstrap rather than Codex-only runtime
- focus on operating model plus connector packs over multi-agent spectacle
- built-in business artifacts and recurring automations
- high-quality home install, migration, and restore behavior

## Roadmap

### Phase 0

Document and packaging baseline.

- add this blueprint
- lock preset and pack naming
- separate current scope from future scope

### Phase 1

Add the delivery plus founder layer.

- draft `founder-pack`
- restructure `delivery-pack`
- define the `business` preset
- add founder artifact contracts

### Phase 2

Add connector packs.

- start with Gmail, Calendar, Drive, and GitHub
- add a permission manifest format
- add connector doctor and reporting

### Phase 3

Add support, growth, and ops packs.

- support digest
- pipeline review
- weekly ops review
- launch brief

### Phase 4

Productize automations.

- recurring templates
- approval boundaries
- run history summary
- recommended automation bundles per pack

## Repository Impact

This direction changes the packaging model more than the installer core.

Expected changes:

- extend `bootstrap.toml` with pack and preset metadata
- make `src/manifest.rs` and provider renderers pack-aware
- add business-pack artifacts under `templates/` and `plugins/`
- clarify preset behavior in README and wizard
- teach `doctor` to report connector and permission state

The installer core remains in place.
The move is to expand distribution units, not to build a brand-new runtime from
scratch.

## Success Criteria

- users can run non-engineering work from the same system
- the default install remains lightweight and safe
- connector permissions and automation boundaries are explicit
- artifacts can be reused by the next step
- the product is recognized as a business-capable operating layer rather than
  only a bootstrap

## Recommended Next Steps

1. Design the `business` preset and pack manifest schema first.
2. Build `founder-pack` as the first vertical slice.
3. Add only `Gmail`, `Calendar`, `Drive`, and `GitHub` first.
4. Start with `weekly market scan`, `pipeline review`, and `support digest`
   automations.

## References

- [gstack](https://github.com/garrytan/gstack)
- [oh-my-codex](https://github.com/junghwaYang/oh-my-codex)
