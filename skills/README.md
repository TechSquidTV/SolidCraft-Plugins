# Agent skills

Skills for contributing to the TechSquid SolidCraft plugins. Each folder is one skill in the [Agent Skills](https://agentskills.io) format: a `SKILL.md` with a name and a description that tell the agent when to load it. [AGENTS.md](../AGENTS.md) routes to them. `python3 tests/skills_test.py` fails when the router table and this folder disagree, or a file a skill names is gone.

| Skill | Loads when the agent |
|---|---|
| [`techsquid-contributing`](techsquid-contributing/SKILL.md) | branches, commits, opens a PR, reads CI or prepares a release |
| [`techsquid-plugins`](techsquid-plugins/SKILL.md) | needs to know where a plugin or crate lives, or adds a plugin or a family crate |
| [`techsquid-parts`](techsquid-parts/SKILL.md) | adds or changes a part: its geometry, standards table, name, component, site or reuse |
| [`techsquid-fields`](techsquid-fields/SKILL.md) | changes dialog fields, `plugin.json`, error messages or the host commands a plugin runs |
| [`techsquid-tests`](techsquid-tests/SKILL.md) | writes or runs unit tests, host tests or examples, or renders pictures |
| [`techsquid-docs`](techsquid-docs/SKILL.md) | writes a plugin guide or reference, a source ledger or release notes |

To use them in a session, open this repository with an agent that reads `AGENTS.md` (Claude Code reads it through `CLAUDE.md`). They need no install. When a command, field or file that a skill names changes, update the skill in the same PR.
