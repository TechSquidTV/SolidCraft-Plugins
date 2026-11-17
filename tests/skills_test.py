#!/usr/bin/env python3
"""Check the agent docs: AGENTS.md, CLAUDE.md, CONTRIBUTING.md and skills/.

    python3 tests/skills_test.py

Needs no SolidCraft. It fails when:
- CLAUDE.md doesn't import AGENTS.md,
- the skill tables in AGENTS.md and skills/README.md don't list exactly the folders in skills/,
- a SKILL.md lacks a `name` equal to its folder or a `description`,
- a relative link, or a backticked repository path, in any of these files names nothing.
"""
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
SKILLS = ROOT / "skills"
# Backticked paths under these folders must exist. Placeholders (<id>, *) are skipped.
PREFIXES = ("plugins/", "crates/", "tests/", "skills/", "docs/", "examples/", "tools/", ".github/", "release-notes/")

failures = []


def fail(where, what):
    failures.append(f"{where}: {what}")


def rel(path):
    return path.relative_to(ROOT).as_posix()


folders = sorted(p.name for p in SKILLS.iterdir() if p.is_dir())

claude = (ROOT / "CLAUDE.md").read_text()
if not claude.startswith("@AGENTS.md"):
    fail("CLAUDE.md", "must start with @AGENTS.md")

agents = (ROOT / "AGENTS.md").read_text()
routed = sorted(set(re.findall(r"`skills/([a-z0-9-]+)/SKILL\.md`", agents.split("## Working on X")[-1])))
if routed != folders:
    fail("AGENTS.md", f"router lists {routed}, skills/ has {folders}")

indexed = sorted(set(re.findall(r"\]\(([a-z0-9-]+)/SKILL\.md\)", (SKILLS / "README.md").read_text())))
if indexed != folders:
    fail("skills/README.md", f"index lists {indexed}, skills/ has {folders}")

for name in folders:
    skill = SKILLS / name / "SKILL.md"
    if not skill.exists():
        fail(rel(skill.parent), "has no SKILL.md")
        continue
    head = skill.read_text().split("---")
    meta = head[1] if len(head) > 2 else ""
    if not re.search(rf"^name: {re.escape(name)}$", meta, re.M):
        fail(rel(skill), f"front matter needs name: {name}")
    if not re.search(r"^description: \S", meta, re.M):
        fail(rel(skill), "front matter needs a description")

docs = [ROOT / "AGENTS.md", ROOT / "CONTRIBUTING.md", *sorted(SKILLS.rglob("*.md"))]
for doc in docs:
    text = doc.read_text()
    for target in re.findall(r"\]\(([^)\s]+)\)", text):
        if re.match(r"[a-z]+:", target) or target.startswith("#"):
            continue
        path = target.split("#")[0]
        if not (doc.parent / path).exists():
            fail(rel(doc), f"link to {target} names nothing")
    for token in re.findall(r"`([^`\s]+)`", text):
        path = token.split(":")[0].rstrip(".,")
        if not path.startswith(PREFIXES) or re.search(r"[<>*…]", path):
            continue
        if not (ROOT / path).exists():
            fail(rel(doc), f"`{token}` names nothing")

for line in failures:
    print(f"FAIL {line}")
print(f"{len(failures)} failures" if failures else f"ok: {len(folders)} skills, {len(docs)} files")
sys.exit(1 if failures else 0)
