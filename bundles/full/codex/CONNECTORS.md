# Connectors

Active company connectors are metadata-first and approval-aware.

- `github`: delivery context, read-write, automation allowed
- `linear`: issue, cycle, and roadmap context, read-write, write requires approval
- `gmail`: inbox context, read-only by default, write requires approval
- `calendar`: schedule context, read-only by default
- `drive`: shared document context, read-only by default
- `figma`: design file and prototype context, read-only by default
- `stitch`: design exploration and generated UI concept context, read-only by default

Before execution:

1. Name the connector you need.
2. Check whether access is read-only or read-write.
3. State whether the action needs approval.
