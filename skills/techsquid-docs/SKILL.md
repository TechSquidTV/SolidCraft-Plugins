---
name: techsquid-docs
description: Use when writing or changing a TechSquid plugin's README.md or REFERENCE.md, DEVELOPMENT.md, a shared doc, a source ledger under docs/sources, or release notes.
---

# Documentation

## One home for each fact

| File | Owns |
|---|---|
| `README.md` (root) | The short index: what the plugins do, the Plugin List, showcase examples |
| `plugins/<id>/README.md` | A short user guide: a native picture, what it makes, Get started, a link to the reference |
| `plugins/<id>/REFERENCE.md` | Every field, the Standards table with sources, naming, limitations, Older field names |
| `DEVELOPMENT.md` | Build and test instructions, architecture notes, example rendering commands |
| `docs/catalog-contract.md` | The internal descriptor contract |
| `docs/parts-library.md` | Shared reuse, BOM rules and host requirements |
| `docs/sources/` | Source evidence and independent measurements |
| `release-notes/v<version>.md` | What changed in a release, breaking changes and migration |
| `AGENTS.md`, `skills/` | Agent routing and the short form of each rule |
| `CONTRIBUTING.md` | The human contributor guide |

Link to the owner instead of restating it. `build.sh` copies each plugin's README, REFERENCE, pictures and the shared docs into its package and rewrites `../../docs/` links, so keep plugin guides linking with relative paths.

## Style

Write in plain, direct sentences like the existing docs: one idea per sentence, active voice, the user's words (Size, Create, Sketch point) for dialog fields and the keys (`size`) in tables. Use a plain `x` in sizes. Don't keep a static file inventory in docs; describe the layout and let readers look.

## When to update

- A field, default or limit changes: REFERENCE.md, and README.md if Get started mentions it.
- A standards table changes: the Standards table in REFERENCE.md, with its source.
- A new plugin: both guides, an example with a picture, a row in the root Plugin List and the DEVELOPMENT.md table.
- A new rule or workflow for contributors: the matching skill and, if it is a hard rule, AGENTS.md and CONTRIBUTING.md. Run `python3 tests/skills_test.py`.
