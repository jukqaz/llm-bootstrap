---
name: record-work
description: Create or update a compact operating record before complex work continues across sessions or external tools.
---

# record-work

Use this skill when work should leave a durable record before continuing.

For deeper reference, load `../../OPERATING_RECORDS.md`.

Workflow:

1. Identify the record type: opportunity, decision, project, task, support, growth, ops, risk, or handoff.
2. If an active local task-state already exists, prefer attaching it with `--from-task-state` so owner, next action, and lane context stay aligned.
3. Capture only current state, decision, next action, evidence, and links.
4. Prefer local docs or GitHub issue/PR/release links as the record surface.
5. Leave external source-of-truth data in Linear, Gmail, Calendar, Drive, Figma, CRM, helpdesk, or analytics.
6. Mark approval boundaries before external writes, customer sends, legal/finance decisions, or security/privacy changes.

CLI:

```bash
llm-bootstrap record --type project --title "MVP scope" --next-action "create first issue"
llm-bootstrap internal task-state begin --title "Build auth flow" --phase execute --owner codex --next-action "capture resumable record"
llm-bootstrap record --type task --title "Build auth flow" --from-task-state
llm-bootstrap record --type task --title "Build auth flow" --surface both --github-repo owner/repo
```
