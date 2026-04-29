# Runtime Risk Register

This document turns the remaining runtime-managed work into an explicit queue.

It does not redefine bootstrap scope.
It tells the operator what still needs to be verified after `install` and
`doctor` already succeeded.

## Risk classes

### R1. App connector linkage

Examples:

- `github`
- `linear`
- `gmail`
- `calendar`
- `drive`
- `figma`
- `stitch`

What bootstrap already proves:

- the active preset selected the connector
- the connector contract is rendered into provider homes
- `doctor --json` exposes the expected handoff fields

What bootstrap does not prove:

- the external account is logged in
- the intended workspace or account is selected
- reads and writes actually work in the provider runtime

Operational action:

1. Open the target runtime.
2. Verify the connector is visible.
3. Confirm the intended account or workspace is selected.
4. Execute one real read action.
5. Confirm write approval boundaries if writes are expected.

### R2. Runtime scheduler registration

Examples:

- `daily-founder-brief`
- `weekly-market-scan`
- `weekly-operating-review`
- `weekly-pipeline-review`

What bootstrap already proves:

- the automation contract is active for the selected preset
- the artifact name and connector dependencies are known
- `doctor --json` exposes scheduler handoff fields

What bootstrap does not prove:

- the recurring schedule is registered
- the first run actually executes
- the produced artifact is delivered where the operator expects it

Operational action:

1. Open the runtime that will own scheduling.
2. Register the cadence explicitly.
3. Run one manual or first scheduled execution.
4. Verify the output artifact.

### R3. Provider runtime drift

Examples:

- Codex state DB migration notices
- provider-specific plugin notices outside `StackPilot` managed paths
- session-specific MCP visibility differences

What bootstrap already proves:

- managed files and state match the requested preset
- managed MCP scripts and settings are present

What bootstrap does not prove:

- the provider runtime has no internal state corruption
- unrelated third-party plugins are healthy
- the runtime shows every MCP the same way in every session mode

Operational action:

1. Treat provider runtime notices as runtime issues first, not bootstrap issues.
2. Reproduce with a minimal runtime command.
3. Only change bootstrap when the notice is caused by a managed asset.

## Priority order

1. Verify active app connectors.
2. Register active automations.
3. Investigate runtime-specific notices only after bootstrap state is clean.

## Mapping to `doctor --json`

Use these fields first:

- `catalog.runtime_handoff.connector_queue`
- `catalog.runtime_handoff.automation_queue`
- `catalog.runtime_handoff.next_steps`
- `catalog.connectors[].connection_status`
- `catalog.automations[].registration_status`

## Practical rule

If `doctor` is green and the remaining queue is only runtime handoff, the next
fix belongs in the provider runtime or the operator workflow, not in
`StackPilot`.
