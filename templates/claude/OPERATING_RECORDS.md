# Operating Records

Use this lane when work should continue across sessions instead of ending in one chat.

Default record surfaces:

- local docs
- GitHub issues, PRs, releases, and repo docs

CLI entrypoint:

```bash
stack-pilot record --type project --title "MVP scope" --next-action "create first issue"
stack-pilot record --type task --title "Build auth flow" --surface both --github-repo owner/repo
```

External systems such as Linear, Gmail, Calendar, Drive, Figma, CRM, helpdesk,
analytics, and accounting remain source of truth. Link to them instead of
copying their data.

Minimum record:

```yaml
id: ""
type: ""
title: ""
status: "draft | active | blocked | decided | shipped | closed"
source: ""
owner: ""
updated_at: ""
next_action: ""
linked_tools:
  github: ""
  linear: ""
  figma: ""
  docs: ""
  calendar: ""
  crm: ""
  helpdesk: ""
  analytics: ""
context:
  summary: ""
  assumptions: []
decision:
  chosen: ""
  alternatives: []
  rationale: ""
evidence:
  links: []
  notes: []
approvals:
  required: false
  reason: ""
  approver: ""
handoff:
  runtime_owner: ""
  external_object_id: ""
  next_step: ""
```

Record types:

- `OpportunityRecord`: market, customer pain, wedge, why now
- `DecisionRecord`: options, chosen path, rationale, review date
- `ProjectRecord`: brief, scope, milestone, artifact links
- `TaskRecord`: issue-sized work, owner, status, verification
- `SupportRecord`: inquiry, severity, reply draft, repro, resolution
- `GrowthRecord`: campaign, channel, hypothesis, result, next experiment
- `OpsRecord`: KPI, pipeline, risks, decisions, next bets
- `RiskRecord`: legal, security, privacy, finance, release approval boundary
- `HandoffRecord`: external runtime owner, object id, next step

Rules:

1. Keep the record small enough to update.
2. Put only decisions, next actions, evidence, and links in the record.
3. Use GitHub for code/project execution records when available.
4. Leave CRM, helpdesk, calendar, and analytics details in their own tools.
5. Require approval before customer sends, legal/finance decisions, or external writes.
