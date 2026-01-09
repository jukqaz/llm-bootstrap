#!/usr/bin/env python3
import argparse
import re
import sys
from pathlib import Path


VALID_NAME_RE = re.compile(r"^[a-z0-9-]+$")
LINK_RE = re.compile(r"!?\[[^\]]+\]\(([^)]+)\)")
SECTION_HEADERS = {
    "references/": "references",
    "assets/": "assets",
    "scripts/": "scripts",
}


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


def extract_resource_refs(lines):
    refs = {"references": [], "assets": [], "scripts": []}
    section = None
    for line in lines:
        if line.startswith("### "):
            header = line[4:].strip()
            section = SECTION_HEADERS.get(header)
            continue
        if section and line.strip().startswith("- "):
            match = re.search(r"`([^`]+)`", line)
            if match:
                refs[section].append(match.group(1))
    return refs


def validate_skill(skill_dir):
    errors = []
    warnings = []
    root_dir = skill_dir.parent

    if not VALID_NAME_RE.match(skill_dir.name):
        errors.append(f"Invalid skill folder name: {skill_dir}")

    skill_md = skill_dir / "SKILL.md"
    if not skill_md.exists():
        errors.append(f"Missing SKILL.md: {skill_dir}")
        return errors, warnings

    text = skill_md.read_text(encoding="utf-8")
    fm = parse_frontmatter(text)
    name = fm.get("name")
    desc = fm.get("description")
    if not name or not desc:
        errors.append(f"Missing frontmatter fields in {skill_md}")
    elif name != skill_dir.name:
        errors.append(f"name mismatch in {skill_md}: {name} != {skill_dir.name}")

    # Warn on unexpected root files
    for item in skill_dir.iterdir():
        if item.is_dir():
            continue
        if item.name == "SKILL.md":
            continue
        if item.name.startswith("."):
            continue
        if item.name.lower().startswith("readme"):
            errors.append(f"Unexpected README in {skill_dir}")
        else:
            warnings.append(f"Unexpected file in {skill_dir}: {item.name}")

    refs = extract_resource_refs(text.splitlines())
    for section, files in refs.items():
        if not files:
            continue
        section_dir = skill_dir / section
        if not section_dir.exists():
            errors.append(f"Missing {section}/ dir in {skill_dir}")
            continue
        for rel_path in files:
            target = section_dir / rel_path
            if not target.exists():
                errors.append(f"Missing {section} file: {target}")

    errors.extend(validate_markdown_links(skill_md, root_dir))
    references_dir = skill_dir / "references"
    if references_dir.exists():
        for md_file in references_dir.rglob("*.md"):
            errors.extend(validate_markdown_links(md_file, root_dir))

    return errors, warnings


def normalize_link_target(target):
    target = target.strip()
    if not target or target.startswith("#"):
        return None
    if target.startswith(("http://", "https://", "mailto:", "tel:")):
        return None
    if "{{" in target or "}}" in target:
        return None
    if "://" in target:
        return None
    return target


def resolve_link_path(target, file_path, root_dir):
    if target.startswith("/"):
        return (root_dir / target.lstrip("/")).resolve()
    return (file_path.parent / target).resolve()


def validate_markdown_links(file_path, root_dir):
    errors = []
    text = file_path.read_text(encoding="utf-8")
    for match in LINK_RE.finditer(text):
        raw_target = match.group(1).split("#", 1)[0].split("?", 1)[0]
        target = normalize_link_target(raw_target)
        if not target:
            continue
        resolved = resolve_link_path(target, file_path, root_dir)
        if not resolved.exists():
            errors.append(f"Missing link target in {file_path}: {raw_target}")
    return errors


def main():
    parser = argparse.ArgumentParser(description="Validate skills structure")
    parser.add_argument("--skills-dir", default="skills")
    args = parser.parse_args()

    skills_dir = Path(args.skills_dir)
    if not skills_dir.exists():
        print(f"skills dir not found: {skills_dir}", file=sys.stderr)
        return 2

    all_errors = []
    all_warnings = []
    for skill_dir in sorted(skills_dir.iterdir()):
        if not skill_dir.is_dir():
            continue
        errors, warnings = validate_skill(skill_dir)
        all_errors.extend(errors)
        all_warnings.extend(warnings)

    for warning in all_warnings:
        print(f"WARN: {warning}", file=sys.stderr)
    for error in all_errors:
        print(f"ERROR: {error}", file=sys.stderr)

    if all_errors:
        return 1

    print("Skill validation passed.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
