#!/usr/bin/env python3
import argparse
import json
import re
from pathlib import Path


def load_map(path: Path) -> dict:
    return json.loads(path.read_text(encoding="utf-8"))


def read_text(args: argparse.Namespace) -> str:
    if args.text:
        return args.text
    if args.file:
        return Path(args.file).read_text(encoding="utf-8")
    raise SystemExit("Provide --text or --file")


def main() -> int:
    parser = argparse.ArgumentParser(description="Recommend skills by keywords")
    parser.add_argument(
        "--map",
        default="skills/role-dispatcher/references/keyword-map.json",
        help="Path to keyword map JSON",
    )
    parser.add_argument("--text", help="Input text to analyze")
    parser.add_argument("--file", help="File path to analyze")
    parser.add_argument("--json", action="store_true", help="Output as JSON")
    args = parser.parse_args()

    text = read_text(args)
    text_lower = text.lower()

    data = load_map(Path(args.map))
    matches = []
    skill_list = []

    for rule in data.get("rules", []):
        keywords = rule.get("keywords", [])
        matched = []
        for kw in keywords:
            kw_lower = kw.lower()
            if re.fullmatch(r"[a-z0-9]+", kw_lower):
                if re.search(rf"\b{re.escape(kw_lower)}\b", text_lower):
                    matched.append(kw)
            else:
                if kw_lower in text_lower:
                    matched.append(kw)
        if not matched:
            continue
        matches.append({"label": rule.get("label", ""), "keywords": matched})
        for skill in rule.get("skills", []):
            if skill not in skill_list:
                skill_list.append(skill)

    if not skill_list:
        skill_list = data.get("default", [])

    if args.json:
        output = {
            "skills": skill_list,
            "matches": matches,
        }
        print(json.dumps(output, ensure_ascii=False, indent=2))
    else:
        print("추천 스킬:")
        for skill in skill_list:
            print(f"- {skill}")
        if matches:
            print("\n매칭 키워드:")
            for match in matches:
                label = match["label"]
                keywords = ", ".join(match["keywords"])
                print(f"- {label}: {keywords}")

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
