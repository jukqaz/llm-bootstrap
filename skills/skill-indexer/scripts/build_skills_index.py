#!/usr/bin/env python3
import argparse
import re
import sys
from pathlib import Path


def parse_frontmatter(text):
    text = text.replace("\r\n", "\n")
    match = re.match(r"---\n(.*?)\n---\n", text, re.S)
    if not match:
        return {}
    data = {}
    for line in match.group(1).splitlines():
        if ":" not in line:
            continue
        key, value = line.split(":", 1)
        data[key.strip()] = value.strip()
    return data


def collect_skills(skills_dir):
    rows = []
    errors = []
    for skill_path in sorted(skills_dir.iterdir()):
        if not skill_path.is_dir():
            continue
        skill_md = skill_path / "SKILL.md"
        if not skill_md.exists():
            errors.append(f"Missing SKILL.md: {skill_path}")
            continue
        text = skill_md.read_text(encoding="utf-8")
        fm = parse_frontmatter(text)
        name = fm.get("name")
        desc = fm.get("description")
        if not name or not desc:
            errors.append(f"Missing frontmatter fields in {skill_md}")
            continue
        rows.append((name, desc.replace("|", "\\|"), f"`skills/{skill_path.name}`"))
    return rows, errors


def build_table(rows):
    rows = sorted(rows, key=lambda r: r[0])
    lines = [
        "# SKILLS",
        "",
        "| Name | Description | Path |",
        "|------|-------------|------|",
    ]
    for name, desc, path in rows:
        lines.append(f"| {name} | {desc} | {path} |")
    return "\n".join(lines) + "\n"


def main():
    parser = argparse.ArgumentParser(description="Generate SKILLS.md index")
    parser.add_argument("--skills-dir", default="skills")
    parser.add_argument("--output", default="SKILLS.md")
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()

    skills_dir = Path(args.skills_dir)
    if not skills_dir.exists():
        print(f"skills dir not found: {skills_dir}", file=sys.stderr)
        return 2

    rows, errors = collect_skills(skills_dir)
    if errors:
        for err in errors:
            print(err, file=sys.stderr)
        return 1

    content = build_table(rows)
    output_path = Path(args.output)
    if args.check:
        if not output_path.exists():
            print(f"Missing {output_path}. Run without --check to create it.", file=sys.stderr)
            return 1
        current = output_path.read_text(encoding="utf-8")
        if current != content:
            print(f"{output_path} is out of date.", file=sys.stderr)
            print("Run: task skills-index", file=sys.stderr)
            return 1
        print(f"{output_path} is up to date.")
        return 0

    output_path.write_text(content, encoding="utf-8")
    print(f"Wrote {len(rows)} skills to {output_path}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
