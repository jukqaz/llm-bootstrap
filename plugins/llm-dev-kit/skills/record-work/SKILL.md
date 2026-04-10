---
name: record-work
description: Create or update a compact operating record before complex work continues across sessions or external tools.
---

# record-work

Use this skill when work should leave a durable record before continuing.

Workflow:

1. Identify the record type: opportunity, decision, project, task, support, growth, ops, risk, or handoff.
2. Capture only current state, decision, next action, evidence, and links.
3. Prefer local docs or GitHub issue/PR/release links as the record surface.
4. Leave external source-of-truth data in Linear, Gmail, Calendar, Drive, Figma, CRM, helpdesk, or analytics.
5. Mark approval boundaries before external writes, customer sends, legal/finance decisions, or security/privacy changes.

CLI:

```bash
llm-bootstrap record --type project --title "MVP scope" --next-action "create first issue"
llm-bootstrap record --type task --title "Build auth flow" --surface both --github-repo owner/repo
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
