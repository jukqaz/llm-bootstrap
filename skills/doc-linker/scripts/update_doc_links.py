#!/usr/bin/env python3
import argparse
import re
import sys
from pathlib import Path


def load_order(path):
    order = []
    for line in path.read_text(encoding="utf-8").splitlines():
        match = re.match(r"\s*\d+\)\s+(.+)$", line)
        if match:
            order.append(match.group(1).strip())
    return order


def ordered_docs(order, root, exclude):
    docs = []
    for name in order:
        if name in exclude:
            continue
        if (root / name).exists():
            docs.append(name)
    return docs


def replace_list(text, header, docs):
    lines = text.splitlines()
    header_idx = None
    for idx, line in enumerate(lines):
        if line.strip() == header:
            header_idx = idx
            break
    if header_idx is None:
        raise ValueError(f"Header not found: {header}")

    i = header_idx + 1
    while i < len(lines) and lines[i].strip() == "":
        i += 1
    list_start = i
    while i < len(lines) and lines[i].lstrip().startswith("- "):
        i += 1
    list_end = i

    new_lines = lines[:list_start] + [f"- {doc}" for doc in docs] + lines[list_end:]
    changed = new_lines != lines
    return "\n".join(new_lines) + "\n", changed


def process_file(path, header, docs, write):
    text = path.read_text(encoding="utf-8")
    updated, changed = replace_list(text, header, docs)
    if changed and write:
        path.write_text(updated, encoding="utf-8")
    return changed


def main():
    parser = argparse.ArgumentParser(description="Update document link lists")
    parser.add_argument("--write", action="store_true")
    parser.add_argument("--check", action="store_true")
    parser.add_argument("--fail-on-missing", action="store_true")
    args = parser.parse_args()

    if args.write and args.check:
        print("Use either --write or --check, not both.", file=sys.stderr)
        return 2

    root = Path(__file__).resolve().parents[3]
    order_path = root / "skills" / "doc-linker" / "references" / "link-order.md"
    order = load_order(order_path)

    agents_path = root / "AGENTS.md"
    readme_path = root / "README.md"

    changes = []
    missing = [name for name in order if not (root / name).exists()]
    agents_docs = ordered_docs(order, root, exclude={"AGENTS.md"})
    if agents_path.exists():
        changed = process_file(agents_path, "## 참고 문서", agents_docs, args.write)
        if changed:
            changes.append("AGENTS.md")

    readme_docs = ordered_docs(order, root, exclude={"README.md"})
    if readme_path.exists():
        changed = process_file(readme_path, "## 문서", readme_docs, args.write)
        if changed:
            changes.append("README.md")

    if missing:
        print("Missing docs: " + ", ".join(missing), file=sys.stderr)

    if changes and not args.write:
        print("Doc lists are out of date: " + ", ".join(changes), file=sys.stderr)
        print("Run: task doc-links", file=sys.stderr)
        return 1

    if missing and args.fail_on_missing:
        print("Create missing docs or update link-order.md", file=sys.stderr)
        return 1

    if changes:
        print("Updated: " + ", ".join(changes))
    else:
        print("Doc lists are up to date.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
