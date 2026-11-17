---
name: techsquid-contributing
description: Use when branching, committing, opening or updating a pull request, reading CI, or preparing a release of the TechSquid SolidCraft plugins.
---

# Contribute a change

Read [CONTRIBUTING.md](../../CONTRIBUTING.md) for the full human guide. This skill is the short form.

## Before you start

- Look for overlapping work: `gh pr list --state open` and `gh issue list --state open`. Comment on an existing PR instead of opening a second one.
- Branch from the latest `origin/main`. Name the branch `<type>/<area>/<topic>`, such as `fix/nuts/pocket-depth` or `feat/v-wheels/add-plugin`.
- One topic per PR. A big change goes as stacked PRs: say in the description which PR it sits on and the order to merge them.

## Commits and titles

- Conventional commits, scoped by plugin or crate: `fix(screws): …`, `feat(bearings)!: …`, `refactor: …`, `docs(extrusions): …`, `chore(release): …`. `!` marks a breaking change, such as a retired field key or changed generated history.
- The PR title becomes the squash commit on `main`, so it follows the same form. Write it and the first sentence for a reader who hasn't seen your chat or the issue: which plugin changes and what a user now sees or can do.

## Checks before every push

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo clippy --workspace --locked --target wasm32-unknown-unknown -- -D warnings
cargo test --workspace --locked
./build.sh
python3 tests/skills_test.py
```

CI (`.github/workflows/ci.yml`) runs the cargo checks and `./build.sh`. Run the host tests yourself when you change what a plugin builds, and list the results in the PR: see `skills/techsquid-tests/SKILL.md`.

## The PR description

- Start with what the change does, in plain words, for someone new to the repository.
- `## Changes`: one bullet per change. `## Behaviour changes`: anything a user or a saved design sees differently, including refused inputs. `## Not in this PR`: known gaps left on purpose. `## Checks`: each command you ran and, for host tests, each file with its pass count and no SKIPs.
- For geometry, give measured numbers and the expected ones. For a visible change, show a native capture from `solidcraft-cli snapshot`; regenerated pictures beside a README count.
- Breaking changes say how to migrate, and the plugin's REFERENCE.md lists retired fields under Older field names.

## Releases

All plugins share `workspace.package.version`. To release, set the version in `Cargo.toml`, `Cargo.lock` and every `plugins/*/plugin.json`, add `release-notes/v<version>.md` with breaking changes and migration steps, and merge. A pushed `v*` tag runs `.github/workflows/release.yml`, which refuses a tag that doesn't match, builds `<id>-<version>.zip` with a `.sha256` for every plugin, and drafts the GitHub release. A maintainer publishes the draft.
