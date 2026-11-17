---
name: techsquid-contributing
description: Use this skill when taking a TechSquid plugin change from a working tree to a merged pull request - choosing a branch name, writing commits and the PR title and description, running the checks CI runs, handling a stacked PR, or preparing a release and its notes. Use it whenever the user says commit, push, open a PR, ship it or release, even for a one-line change.
---

# Contribute a change

[CONTRIBUTING.md](../../CONTRIBUTING.md) is the full human guide; this is the short form.

## Workflow

- [ ] `gh pr list --state open`: comment on overlapping work instead of starting a second copy.
- [ ] Branch from the latest `origin/main` as `<type>/<area>/<topic>`, such as `fix/nuts/pocket-depth`. One topic per branch.
- [ ] Run the checks below until they pass.
- [ ] Commit with a conventional, scoped title: `fix(screws): …`, `feat(bearings)!: …`, `docs(extrusions): …`, `chore(release): …`. `!` marks a break: a retired field, a changed part number, changed generated history.
- [ ] Open the PR with the template below. The title becomes the squash commit on `main`.

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo clippy --workspace --locked --target wasm32-unknown-unknown -- -D warnings
cargo test --workspace --locked
./build.sh
python3 tests/skills_test.py
```

CI runs the cargo checks and `./build.sh`. It doesn't run the host tests, so a change to what a plugin builds also needs the suites in `skills/techsquid-tests/SKILL.md`.

## PR template

```markdown
<One or two plain sentences: which plugin changes and what a user now sees or can do.>

## Changes
- **<Plugin or crate>:** <what changed and why>

## Behaviour changes
- <refused inputs, changed part numbers, regenerated examples; omit if none>

## Not in this PR
- <known gaps left on purpose; omit if none>

## Checks
- `cargo fmt`, native and wasm clippy, `cargo test --workspace` (<N> passed), `./build.sh`.
- Host tests on an unsigned <version> CLI: `<suite>` <N> passed, 0 failed, 0 skipped; …
```

For geometry, add a before/after table of measured values and the hand-worked expected value. For a visible change, show a native `solidcraft-cli snapshot` capture.

## Releases

1. Set the version in `Cargo.toml`, `Cargo.lock` and every `plugins/*/plugin.json`.
2. Write `release-notes/v<version>.md`: changes, breaking changes and migration steps.
3. Merge, then push the `v*` tag. `.github/workflows/release.yml` refuses a tag that doesn't match, builds `<id>-<version>.zip` and `.sha256` for each plugin, and drafts the release for a maintainer to publish.

## Gotchas

- **An agent's push token usually lacks the `workflow` scope**, so a push that touches `.github/workflows/` is rejected. Keep workflow edits out of the branch, and put the change in the PR description for a maintainer to apply.
- **A stacked PR's base is the branch below it.** Say so in the description, and retarget it to `main` once the lower PR merges.
- **Don't regenerate examples in a PR that doesn't change geometry.** An unexplained diff in `examples/*.json` hides real changes.
