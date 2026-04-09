# Security Policy

## Supported scope

This repository contains bootstrap logic, templates, and workflows.
It must not contain live credentials, tokens, or user-specific local state.

## Reporting

If you find a security issue:

- do not open a public issue with live secrets
- redact any credential-like material
- prefer a private GitHub security advisory if available

If a public issue is the only available path, report the minimum reproducible
details without including secrets, personal paths, or machine-specific state.

## Expected secret handling

- secrets are provided through environment variables only
- the repository does not commit secret values
- wizard persistence writes to user home, not to the repository

## Trust boundaries

- `llm-bootstrap` manages provider home configuration and install state only
- provider login state, app connector auth, and recurring scheduler registration remain runtime-managed
- bootstrap verification can confirm requested install state, but it does not prove that external accounts are logged in

## Local filesystem safety

- backup and restore operate inside the local user trust boundary
- bootstrap skips symbolic links and unsupported special files while copying backup or rendered assets
- absolute backup restore paths should be treated as trusted local operator input
