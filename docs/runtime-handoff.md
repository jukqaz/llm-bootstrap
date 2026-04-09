# Runtime Handoff

This document defines where `llm-bootstrap` stops and where the target runtime
takes over.

The rule is simple:

- bootstrap aligns installation state and contracts
- the provider runtime owns account connections and actual execution

## Boundary

`llm-bootstrap` owns:

- `preset -> pack -> harness -> connectors -> MCP -> provider surface`
- installing the required docs, skills, commands, and scripts into provider homes
- comparing requested state and installed state through `doctor`
- recording provider state in `llm-bootstrap-state.json`

The runtime owns:

- app connector login and account linkage
- actual inbox, calendar, design, and project data access
- recurring automation scheduler registration and execution

## Connector handoff

### App connectors

Current default app connectors:

- `github`
- `linear`
- `gmail`
- `calendar`
- `drive`
- `figma`
- `stitch`

In `doctor`, active app connectors expose:

- `runtime_owner = provider-runtime`
- `verification_mode = manual-runtime-check`
- `connection_status = not-verified`
- `auth_state = external-runtime`

Inactive app connectors expose:

- `connection_status = not-requested`
- `next_step = null`

Meaning:

- bootstrap installs the requirement contract
- the actual login, session, and permission state belongs to the provider runtime

Operational checklist:

1. Open the target provider runtime.
2. Confirm the app connector appears in the UI or tool list.
3. Confirm the intended account is connected.
4. Perform one read action directly.
5. If writes require approval, confirm the approval boundary appears as expected.

### MCP connectors

If a future connector uses `tool_source = mcp`, the contract is:

- `runtime_owner = bootstrap`
- `verification_mode = bootstrap-check`
- `connection_status = managed`

Meaning:

- bootstrap owns the wiring
- state should be verified through `doctor` plus actual MCP invocation

### Native connectors

`tool_source = native` means the provider exposes the capability directly.

- `runtime_owner = provider-native`
- `verification_mode = native-check`
- `connection_status = ready`

## Automation handoff

Current automation contracts:

- `daily-founder-brief`
- `weekly-market-scan`
- `weekly-operating-review`
- `weekly-pipeline-review`

In `doctor`, active automation contracts expose:

- `status = rendered`
- `scheduler_owner = runtime-managed`
- `registration_status = not-registered`

Inactive automation contracts expose:

- `status = inactive`
- `registration_status = not-requested`
- `next_step = null`

Meaning:

- bootstrap installs which automation contracts are active
- the actual recurring schedule must still be registered by the runtime or an external automation layer

Operational checklist:

1. Confirm the active automation list with `doctor --json`.
2. Confirm the target runtime exposes scheduling or automation features.
3. Register the desired cadence.
4. Validate the first execution against the artifact contract.

## Handoff by preset

### `light`

- focus: delivery baseline
- connector handoff: `github`, `linear`
- automation handoff: none

### `normal`

- focus: delivery plus incident
- connector handoff: `github`, `linear`
- automation handoff: none

### `full`

- focus: delivery plus company
- connector handoff:
  - delivery: `github`, `linear`
  - company: `gmail`, `calendar`, `drive`, `figma`, `stitch`
- automation handoff:
  - `daily-founder-brief`
  - `weekly-market-scan`
  - `weekly-operating-review`
  - `weekly-pipeline-review`

### `company`

- focus: founder plus ops
- connector handoff:
  - `linear`, `gmail`, `calendar`, `drive`, `figma`, `stitch`
- automation handoff:
  - `daily-founder-brief`
  - `weekly-market-scan`
  - `weekly-operating-review`
  - `weekly-pipeline-review`

## Practical conclusion

What remains after bootstrap is not a bootstrap bug. It is an operating handoff.

- app connectors must actually be logged in
- automations must actually be registered

So the next steps after bootstrap are:

1. open the provider runtime
2. verify connector linkage
3. register the scheduler
4. validate the first run
