use std::collections::{BTreeMap, HashMap};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

struct Args {
    skills_dir: PathBuf,
    output: PathBuf,
    check: bool,
}

struct SkillRow {
    group: &'static str,
    name: String,
    desc: String,
    path: String,
}

fn usage() -> ! {
    eprintln!("Usage: skill-indexer [--skills-dir <dir>] [--output <file>] [--check]");
    std::process::exit(2);
}

fn parse_args() -> Args {
    let mut skills_dir = PathBuf::from("skills");
    let mut output = PathBuf::from("SKILLS.md");
    let mut check = false;
    let mut it = env::args().skip(1);
    while let Some(arg) = it.next() {
        match arg.as_str() {
            "--skills-dir" => {
                let Some(val) = it.next() else { usage() };
                skills_dir = PathBuf::from(val);
            }
            "--output" => {
                let Some(val) = it.next() else { usage() };
                output = PathBuf::from(val);
            }
            "--check" => check = true,
            "--help" => usage(),
            _ => {
                eprintln!("Unknown arg: {arg}");
                usage();
            }
        }
    }
    Args {
        skills_dir,
        output,
        check,
    }
}

fn parse_frontmatter(text: &str) -> HashMap<String, String> {
    let normalized = text.replace("\r\n", "\n");
    if !normalized.starts_with("---\n") {
        return HashMap::new();
    }
    let rest = &normalized[4..];
    let Some(end) = rest.find("\n---\n") else {
        return HashMap::new();
    };
    let block = &rest[..end];
    let mut data = HashMap::new();
    for line in block.lines() {
        if let Some(pos) = line.find(':') {
            let key = line[..pos].trim();
            let value = line[pos + 1..].trim();
            if !key.is_empty() {
                data.insert(key.to_string(), value.to_string());
            }
        }
    }
    data
}

fn skill_group(name: &str) -> &'static str {
    match name {
        "doc-linker" | "docs-audit" | "readme-maintainer" | "release-docs"
        | "repo-doc-bootstrap" | "skill-indexer" | "skill-validator" => "01-Docs-SkillOps",
        "change-completeness" | "code-polish-simplify" | "github-pr-ci"
        | "library-guideline-audit" | "parallel-work" | "regression-plan" => {
            "02-Engineering-Quality"
        }
        "idea-harvest" | "idea-triage" | "market-scan" | "mvp-scope-cut"
        | "experiment-review" => "03-Ideation-MVP",
        "access-audit" | "deploy-checklist" | "infra-platform"
        | "infra-release-runbook" | "security-review" | "threat-model" => {
            "04-Infra-Security"
        }

        _ => "98-Unclassified",
    }
}

fn collect_skills(skills_dir: &Path) -> Result<Vec<SkillRow>, Vec<String>> {
    let mut rows = Vec::new();
    let mut errors = Vec::new();
    let entries = match fs::read_dir(skills_dir) {
        Ok(entries) => entries,
        Err(_) => {
            return Err(vec![format!("skills dir not found: {}", skills_dir.display())]);
        }
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let skill_md = path.join("SKILL.md");
        if !skill_md.exists() {
            errors.push(format!("Missing SKILL.md: {}", path.display()));
            continue;
        }
        let text = match fs::read_to_string(&skill_md) {
            Ok(text) => text,
            Err(err) => {
                errors.push(format!("Failed to read {}: {err}", skill_md.display()));
                continue;
            }
        };
        let fm = parse_frontmatter(&text);
        let Some(name) = fm.get("name") else {
            errors.push(format!("Missing frontmatter fields in {}", skill_md.display()));
            continue;
        };
        let Some(desc) = fm.get("description") else {
            errors.push(format!("Missing frontmatter fields in {}", skill_md.display()));
            continue;
        };
        let escaped_desc = desc.replace('|', "\\|");
        let rel_path = format!("`skills/{}`", path.file_name().unwrap().to_string_lossy());
        rows.push(SkillRow {
            group: skill_group(name),
            name: name.to_string(),
            desc: escaped_desc,
            path: rel_path,
        });
    }
    if errors.is_empty() {
        Ok(rows)
    } else {
        Err(errors)
    }
}

fn build_table(mut rows: Vec<SkillRow>) -> String {
    rows.sort_by(|a, b| a.group.cmp(b.group).then_with(|| a.name.cmp(&b.name)));

    let mut group_counts: BTreeMap<&str, usize> = BTreeMap::new();
    for row in &rows {
        *group_counts.entry(row.group).or_insert(0) += 1;
    }

    let mut lines = vec![
        "# SKILLS".to_string(),
        "".to_string(),
        format!("- Total: {}", rows.len()),
        format!("- Groups: {}", group_counts.len()),
        "".to_string(),
        "## Group Summary".to_string(),
        "".to_string(),
        "| Group | Count |".to_string(),
        "|------|------:|".to_string(),
    ];

    for (group, count) in &group_counts {
        lines.push(format!("| {group} | {count} |"));
    }

    lines.extend([
        "".to_string(),
        "## Skills".to_string(),
        "".to_string(),
        "| Group | Name | Description | Path |".to_string(),
        "|------|------|-------------|------|".to_string(),
    ]);

    for row in rows {
        lines.push(format!(
            "| {} | {} | {} | {} |",
            row.group, row.name, row.desc, row.path
        ));
    }
    lines.join("\n") + "\n"
}

fn main() {
    let args = parse_args();
    if !args.skills_dir.exists() {
        eprintln!("skills dir not found: {}", args.skills_dir.display());
        std::process::exit(2);
    }
    let rows = match collect_skills(&args.skills_dir) {
        Ok(rows) => rows,
        Err(errors) => {
            for err in errors {
                eprintln!("{err}");
            }
            std::process::exit(1);
        }
    };

    let count = rows.len();
    let content = build_table(rows);
    if args.check {
        if !args.output.exists() {
            eprintln!(
                "Missing {}. Run without --check to create it.",
                args.output.display()
            );
            std::process::exit(1);
        }
        let current = fs::read_to_string(&args.output)
            .unwrap_or_else(|_| String::from(""));
        if current != content {
            eprintln!("{} is out of date.", args.output.display());
            eprintln!("Run: task skills-index");
            std::process::exit(1);
        }
        println!("{} is up to date.", args.output.display());
        return;
    }

    if let Err(err) = fs::write(&args.output, content) {
        eprintln!("Failed to write {}: {err}", args.output.display());
        std::process::exit(1);
    }
    println!("Wrote {} skills to {}", count, args.output.display());
}
