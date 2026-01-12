use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

const AGENTS_TEMPLATE: &str = include_str!("../../../assets/AGENTS-template.md");
const PRD_TEMPLATE: &str = include_str!("../../../assets/PRD-template.md");
const TRD_TEMPLATE: &str = include_str!("../../../assets/TRD-template.md");
const README_TEMPLATE: &str = include_str!("../../../assets/README-template.md");
const ROLE_MAP_TEMPLATE: &str = include_str!("../../../assets/ROLE-MAP-template.md");
const CHANGELOG_TEMPLATE: &str = include_str!("../../../assets/CHANGELOG-template.md");

fn usage() -> ! {
    eprintln!("Usage: bootstrap-docs [--root <path>] [--force] [--dry-run]");
    std::process::exit(2);
}

fn detect_stack(root: &Path) -> String {
    let mut items = Vec::new();
    let checks = [
        ("Cargo.toml", "Rust"),
        ("package.json", "Node"),
        ("pyproject.toml", "Python"),
        ("requirements.txt", "Python"),
        ("go.mod", "Go"),
        ("Gemfile", "Ruby"),
        ("pom.xml", "Java"),
        ("build.gradle", "Java"),
        ("build.gradle.kts", "Kotlin"),
        ("composer.json", "PHP"),
    ];
    for (file, label) in checks {
        if root.join(file).exists() {
            items.push(label.to_string());
        }
    }
    if items.is_empty() {
        "TODO".to_string()
    } else {
        items.sort();
        items.dedup();
        items.join(", ")
    }
}

fn detect_commands(root: &Path) -> String {
    let mut commands = Vec::new();
    if root.join("Taskfile.yml").exists() {
        commands.push("task check".to_string());
    }
    if root.join("Makefile").exists() {
        commands.push("make test".to_string());
    }
    if root.join("Cargo.toml").exists() {
        commands.push("cargo test".to_string());
    }
    if root.join("package.json").exists() {
        commands.push("npm test".to_string());
    }
    if commands.is_empty() {
        "TODO".to_string()
    } else {
        commands.join(", ")
    }
}

fn detect_module_scope(root: &Path) -> (String, String) {
    let monorepo_dirs = ["apps", "packages", "services", "modules"];
    let mut hits = Vec::new();
    for dir in monorepo_dirs {
        if root.join(dir).is_dir() {
            hits.push(format!("{dir}/"));
        }
    }
    if hits.is_empty() {
        ("모노레포 아님".to_string(), "루트만 사용".to_string())
    } else {
        let scope = format!("모노레포 후보: {}", hits.join(", "));
        let nested = format!("모듈별 AGENTS.md 후보: {}", hits.join(", "));
        (scope, nested)
    }
}

fn render_template(template: &str, ctx: &HashMap<&str, String>) -> String {
    let mut output = template.to_string();
    for (key, value) in ctx {
        let token = format!("{{{{{key}}}}}");
        output = output.replace(&token, value);
    }
    output
}

fn write_doc(root: &Path, name: &str, content: &str, force: bool, dry_run: bool) {
    let path = root.join(name);
    if path.exists() && !force {
        println!("Skip (exists): {}", path.display());
        return;
    }
    if dry_run {
        println!("Would write: {}", path.display());
        return;
    }
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    fs::write(&path, content).expect("Failed to write doc");
    println!("Wrote: {}", path.display());
}

fn main() {
    let mut root = PathBuf::from(".");
    let mut force = false;
    let mut dry_run = false;

    let args: Vec<String> = env::args().collect();
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--root" => {
                i += 1;
                if i >= args.len() {
                    usage();
                }
                root = PathBuf::from(&args[i]);
            }
            "--force" => force = true,
            "--dry-run" => dry_run = true,
            _ => usage(),
        }
        i += 1;
    }

    let root = root.canonicalize().unwrap_or(root);
    let project_name = root
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("프로젝트")
        .to_string();

    let tech_stack = detect_stack(&root);
    let commands = detect_commands(&root);
    let (module_scope, nested_scope) = detect_module_scope(&root);

    let mut ctx = HashMap::new();
    ctx.insert("PROJECT_NAME", project_name);
    ctx.insert("TECH_STACK", tech_stack);
    ctx.insert("COMMANDS", commands);
    ctx.insert("MODULE_SCOPE", module_scope);
    ctx.insert("NESTED_SCOPE", nested_scope);

    let agents = render_template(AGENTS_TEMPLATE, &ctx);
    let prd = render_template(PRD_TEMPLATE, &ctx);
    let trd = render_template(TRD_TEMPLATE, &ctx);
    let readme = render_template(README_TEMPLATE, &ctx);
    let role_map = render_template(ROLE_MAP_TEMPLATE, &ctx);
    let changelog = render_template(CHANGELOG_TEMPLATE, &ctx);

    write_doc(&root, "AGENTS.md", &agents, force, dry_run);
    write_doc(&root, "PRD.md", &prd, force, dry_run);
    write_doc(&root, "TRD.md", &trd, force, dry_run);
    write_doc(&root, "README.md", &readme, force, dry_run);
    write_doc(&root, "ROLE-MAP.md", &role_map, force, dry_run);
    write_doc(&root, "CHANGELOG.md", &changelog, force, dry_run);
}
