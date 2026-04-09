# Source Catalog

This directory stores structured reference data for `llm-bootstrap`.

It is intentionally separate from runtime manifests.

The goal is to answer these questions in a reusable form:

- which official provider surfaces should we follow first
- which reference repos are worth learning from
- which external tools improve capability without bloating the baseline
- what should be adopted, and what should explicitly not be copied

## Layout

```text
catalog/sources/
├── README.md
├── index.toml
├── official/
├── reference/
└── tool/
```

- `official/`: provider-native documentation and guidance
- `reference/`: repos that influence workflow or harness design
- `tool/`: external tools that improve output, context, or MCP operations

## Schema

Each source file uses the same top-level shape:

```toml
id = "openai_codex"
name = "OpenAI Codex"
source_type = "official"
status = "active"
provider_scope = ["codex"]
categories = ["runtime", "provider-surface"]
weight = "core"
summary = "Short explanation"

urls = [
  "https://example.com"
]

native_surfaces = ["config.toml", "AGENTS.md"]
capabilities = ["mcp", "subagents"]
adoptable_parts = ["provider-native rendering"]
do_not_copy = ["replace native surfaces with a fake abstraction"]

[[evidence]]
label = "Primary docs"
url = "https://example.com"
note = "What this source proves"

[[recommended_article]]
title = "Best practices page"
url = "https://example.com/best-practices"
focus = "security"
why_it_matters = "Explains what should influence llm-bootstrap design"
```
```

## Field Rules

- `source_type`: `official`, `reference`, or `tool`
- `status`: lifecycle marker such as `active`
- `provider_scope`: `codex`, `gemini`, `claude`, or `cross-provider`
- `categories`: use broad capability buckets only
- `weight`: `core`, `optional`, or `advanced`
- `summary`: one-sentence operational value
- `native_surfaces`: only for sources that define provider-native surfaces
- `capabilities`: what the source is actually good at
- `adoptable_parts`: what `llm-bootstrap` should reuse
- `do_not_copy`: what should stay outside this repo
- `[[evidence]]`: one or more concrete URLs with short proof notes
- `[[recommended_article]]`: optional curated official reading list for design
  decisions

## Intended Use

This catalog is the input layer for future:

- harness catalogs
- pack catalogs
- provider renderers
- optional external-tool lanes

It is not yet the install manifest.

That boundary is deliberate:

- source catalog answers "what did we learn?"
- runtime manifest answers "what do we install?"
