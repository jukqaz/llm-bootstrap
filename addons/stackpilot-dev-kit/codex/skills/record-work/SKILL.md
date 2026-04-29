---
name: record-work
description: Create or update a compact operating record before complex work continues across sessions or external tools.
---

# record-work

Use this skill when work should leave a durable record before continuing.

Workflow:

1. Identify the record type: opportunity, decision, project, task, support, growth, ops, risk, or handoff.
2. If an active local task-state already exists, prefer attaching it with `--from-task-state` so owner, summary, checkpoint, next action, and lane context stay aligned.
3. Capture only current state, decision, next action, evidence, and links.
4. Prefer local docs or GitHub issue/PR/release links as the record surface.
5. Leave external source-of-truth data in Linear, Gmail, Calendar, Drive, Figma, CRM, helpdesk, or analytics.
6. Mark approval boundaries before external writes, customer sends, legal/finance decisions, or security/privacy changes.

CLI:

```bash
stack-pilot record --type project --title "MVP scope" --next-action "create first issue"
stack-pilot internal task-state begin --title "Build auth flow" --phase execute --owner codex --summary "Auth flow is wired and waiting on review." --checkpoint "Resume from the oauth fixture repro and capture the failing output." --next-action "capture resumable record"
stack-pilot record --type task --title "Build auth flow" --from-task-state
stack-pilot record --type task --title "Build auth flow" --surface both --github-repo owner/repo
```

Minimum output:

```yaml
type: ""
title: ""
status: ""
owner: ""
next_action: ""
linked_tools: {}
context:
  summary: ""
  task_state:
    source: ""
    checkpoint: ""
decision:
  chosen: ""
  rationale: ""
evidence:
  links: []
approvals:
  required: false
handoff:
  runtime_owner: ""
  next_step: ""
```
