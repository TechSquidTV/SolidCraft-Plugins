---
name: techsquid-docs
description: Use this skill when writing or updating any prose in the TechSquid SolidCraft plugins - a plugin's README.md or REFERENCE.md, DEVELOPMENT.md, docs/, a source ledger under docs/sources, release notes, AGENTS.md or a skill - and after any change to fields, defaults, limits, standards tables or examples that users read about. Use it to decide which file a fact belongs in. Not for code comments.
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

Write in plain, direct sentences like the existing docs: one idea per sentence, active voice, the user's words (Size, Create, Placement) for dialog fields and the keys (`size`) in tables. Use a plain `x` in sizes. Don't keep a static file inventory in docs; describe the layout and let readers look.

## When to update

- A field, default or limit changes: REFERENCE.md, and README.md if Get started mentions it.
- A standards table changes: the Standards table in REFERENCE.md, with its source.
- A new plugin: both guides, an example with a picture, a row in the root Plugin List and the DEVELOPMENT.md table.
- A new rule or workflow for contributors: the matching skill and, if it is a hard rule, AGENTS.md and CONTRIBUTING.md. Run `python3 tests/skills_test.py`.

## Gotchas

- **Packaged guides are rewritten.** `build.sh` turns `../../docs/` into `docs/` and `../../DEVELOPMENT.md#components` into `docs/parts-library.md`. A plugin guide that links elsewhere in the repository breaks once installed; link to GitHub for those.
- **Generated tables aren't hand-edited.** The Bearings tables come from `catalog-tool`; `./build.sh` fails when they drift.
- **Counts go stale.** Preset counts, size ranges and test counts in prose were wrong after changes more than once. Prefer a link to the table over a number, and grep for the old number when one changes.
- **Evidence links must survive branch deletion.** Pin implementation evidence and PR preview images to a commit; use relative or default-branch links for current guides. Do not leave permanent documentation pointing to a temporary feature branch.
