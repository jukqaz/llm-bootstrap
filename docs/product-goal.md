# Product Goal

This document is the top-level source of truth for `llm-bootstrap`.

Strategy docs, capability design, backlog decisions, and renderer changes
should all be checked against this document first.

## One-line goal

`llm-bootstrap` has a two-step goal:

1. establish a safe provider-native baseline for each LLM runtime
2. add capability layers that help the user work better on top of that baseline

In other words, this is not just an installer.
It follows a `baseline first, enablement second` product rule.

## Product definition

`llm-bootstrap` reproduces a safe baseline for runtimes such as `Codex`,
`Gemini`, and `Claude Code`, then adds planning, execution, review, QA,
record, and company-operation capabilities through provider-native surfaces.

The product must answer two questions:

1. "How do I align this LLM cleanly from the start?"
2. "Once aligned, how do I help the user work better?"

## Layer 1. Baseline

Baseline is the floor contract that should exist first across providers.

It includes:

- install / replace / restore / uninstall / doctor
- backup and recovery
- provider-native config rendering
- core MCP
- env-gated MCP
- auth, session, and history preservation
- legacy artifact cleanup
- minimal workflow docs and provider-native entrypoints

It does not include:

- heavy task-state
- a large company memory database
- a runtime-first orchestration engine
- provider-specific "wow" features forced as defaults

Baseline succeeds when:

- it can be reproduced on a new machine
- it can be re-applied safely on an existing machine
- it does not break auth or history
- drift is visible through doctor
- uninstall and restore remain predictable

## Layer 2. Enablement

Enablement is the capability layer added after baseline is stable.

Examples:

- project planning
- implementation execution
- review
- QA
- incident response
- founder loop
- operating review
- record-first workflows
- optional productivity or QA tools

Enablement is not about "more features".
It is about "better outcomes with less friction".

Therefore enablement should:

- stay separate from baseline
- remain opt-in
- preserve provider-native surfaces
- keep install-state separate from task-state
- leave external-tool ownership as a handoff when SaaS or runtimes do it better

## Why the product is split into two layers

Without this split, the product drifts in two bad directions:

1. bootstrap safety gets weaker
2. the product starts copying a heavy orchestration runtime

We do not want either.

We want:

- a small and safe baseline
- strong and extensible enablement

## Product priority order

Always preserve this order:

1. baseline safety
2. provider-native fit
3. doctor and recoverability
4. record-first execution contracts
5. stronger entrypoints and thin task-state
6. optional advanced tooling
7. company-scale operating capabilities

That means orchestration depth matters, but it cannot come before baseline
integrity.

## Naming and positioning

The current name, `llm-bootstrap`, is still accurate for Layer 1.
So the immediate change should be positioning, not renaming.

Recommended positioning:

> `llm-bootstrap` first stabilizes each provider-native LLM baseline, then
> layers on optional capabilities for planning, execution, review, QA, and
> company operations.

## Implementation test

Before adding any feature, ask:

1. Is this baseline or enablement?
2. If baseline, is it safely reproducible across providers?
3. If enablement, can it stay opt-in?
4. Does it preserve provider-native surfaces?
5. Does it keep install-state separate from task-state?
6. Can doctor or a record contract make it observable?

If the answer is unclear, it should not enter the core product.

## Current conclusion

`llm-bootstrap` is already a baseline product.
The next step is to grow enablement without weakening that foundation, and to
keep all future improvements aligned to this two-layer model.
