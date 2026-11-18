# Agent skills

Skills for contributing to the TechSquid SolidCraft plugins. Each folder is one skill in the [Agent Skills](https://agentskills.io) format: a `SKILL.md` with a name and a description that tell the agent when to load it. [AGENTS.md](../AGENTS.md) routes to them. `python3 tests/skills_test.py` checks them against the [specification](https://agentskills.io/specification) and fails when the router table and this folder disagree, or a file a skill names is gone.

| Skill | Loads when the agent |
|---|---|
| [`techsquid-contributing`](techsquid-contributing/SKILL.md) | commits, pushes, opens or updates a PR, or prepares a release |
| [`techsquid-plugins`](techsquid-plugins/SKILL.md) | looks for where something lives, or adds a new plugin or family of parts |
| [`techsquid-parts`](techsquid-parts/SKILL.md) | adds or changes the hardware a plugin builds: sizes, variants, tables, names, part numbers, reuse |
| [`techsquid-fields`](techsquid-fields/SKILL.md) | changes a dialog, `plugin.json`, an error message or the host commands a plugin runs |
| [`techsquid-icons`](techsquid-icons/SKILL.md) | draws, redraws or reviews a command icon, or hears that one doesn't look right |
| [`techsquid-tests`](techsquid-tests/SKILL.md) | proves a change works: unit tests, host tests, examples, pictures |
| [`techsquid-docs`](techsquid-docs/SKILL.md) | writes or updates guides, references, ledgers, release notes or these skills |

To use them in a session, open this repository with an agent that reads `AGENTS.md` (Claude Code reads it through `CLAUDE.md`). They need no install. When a command, field or file that a skill names changes, update the skill in the same PR.

## Writing a skill

- The `description` decides whether a skill loads. Start it with "Use this skill when", describe what the user is trying to do, name requests that don't use the repository's words, and say which neighbouring skill to use instead.
- Put what an agent would get wrong without being told: commands, file names, and a Gotchas section of real failures from this repository's history. Leave out what any agent knows.
- Use a checklist for multi-step work and end it with the command that proves it worked.
- When an agent makes a mistake a skill should have prevented, add it to that skill's Gotchas.
