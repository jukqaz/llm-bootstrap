use std::collections::HashSet;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

struct Args {
    write: bool,
    check: bool,
    fail_on_missing: bool,
}

fn usage() -> ! {
    eprintln!("Usage: doc-linker [--write | --check] [--fail-on-missing]");
    std::process::exit(2);
}

fn parse_args() -> Args {
    let mut write = false;
    let mut check = false;
    let mut fail_on_missing = false;
    let mut it = env::args().skip(1);
    while let Some(arg) = it.next() {
        match arg.as_str() {
            "--write" => write = true,
            "--check" => check = true,
            "--fail-on-missing" => fail_on_missing = true,
            "--help" => usage(),
            _ => {
                eprintln!("Unknown arg: {arg}");
                usage();
            }
        }
    }
    if write && check {
        eprintln!("Use either --write or --check, not both.");
        std::process::exit(2);
    }
    Args {
        write,
        check,
        fail_on_missing,
    }
}

fn repo_root() -> PathBuf {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let root = manifest_dir
        .ancestors()
        .nth(4)
        .expect("Failed to resolve repo root");
    root.to_path_buf()
}

fn load_order(path: &Path) -> Vec<String> {
    let content = fs::read_to_string(path).expect("Failed to read link-order.md");
    let mut order = Vec::new();
    for line in content.lines() {
        let trimmed = line.trim_start();
        let Some(pos) = trimmed.find(')') else {
            continue;
        };
        let (left, right) = trimmed.split_at(pos);
        if left.chars().all(|c| c.is_ascii_digit()) {
            let doc = right[1..].trim();
            if !doc.is_empty() {
                order.push(doc.to_string());
            }
        }
    }
    order
}

fn ordered_docs(order: &[String], root: &Path, exclude: &HashSet<&str>) -> Vec<String> {
    let mut docs = Vec::new();
    for name in order {
        if exclude.contains(name.as_str()) {
            continue;
        }
        if root.join(name).exists() {
            docs.push(name.clone());
        }
    }
    docs
}

fn replace_list(text: &str, header: &str, docs: &[String]) -> Result<(String, bool), String> {
    let lines: Vec<&str> = text.lines().collect();
    let mut header_idx = None;
    for (idx, line) in lines.iter().enumerate() {
        if line.trim() == header {
            header_idx = Some(idx);
            break;
        }
    }
    let Some(header_idx) = header_idx else {
        return Err(format!("Header not found: {header}"));
    };

    let mut i = header_idx + 1;
    while i < lines.len() && lines[i].trim().is_empty() {
        i += 1;
    }
    let list_start = i;
    while i < lines.len() && lines[i].trim_start().starts_with("- ") {
        i += 1;
    }
    let list_end = i;

    let mut new_lines: Vec<String> = Vec::new();
    new_lines.extend(lines[..list_start].iter().map(|s| s.to_string()));
    new_lines.extend(docs.iter().map(|doc| format!("- {doc}")));
    new_lines.extend(lines[list_end..].iter().map(|s| s.to_string()));

    let changed = new_lines
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>()
        != lines;
    Ok((new_lines.join("\n") + "\n", changed))
}

fn process_file(path: &Path, header: &str, docs: &[String], write: bool) -> Result<bool, String> {
    let text = fs::read_to_string(path)
        .map_err(|err| format!("Failed to read {}: {err}", path.display()))?;
    let (updated, changed) = replace_list(&text, header, docs)?;
    if changed && write {
        fs::write(path, updated).map_err(|err| format!("Failed to write {}: {err}", path.display()))?;
    }
    Ok(changed)
}

fn main() {
    let args = parse_args();
    let root = repo_root();
    let order_path = root
        .join("skills")
        .join("doc-linker")
        .join("references")
        .join("link-order.md");
    let order = load_order(&order_path);

    let agents_path = root.join("AGENTS.md");
    let readme_path = root.join("README.md");

    let mut changes = Vec::new();
    let missing: Vec<String> = order
        .iter()
        .filter(|name| !root.join(name).exists())
        .cloned()
        .collect();

    let mut exclude = HashSet::new();
    exclude.insert("AGENTS.md");
    let agents_docs = ordered_docs(&order, &root, &exclude);
    if agents_path.exists() {
        match process_file(&agents_path, "## 참고 문서", &agents_docs, args.write) {
            Ok(true) => changes.push("AGENTS.md".to_string()),
            Ok(false) => {}
            Err(err) => {
                eprintln!("{err}");
                std::process::exit(1);
            }
        }
    }

    let mut exclude = HashSet::new();
    exclude.insert("README.md");
    let readme_docs = ordered_docs(&order, &root, &exclude);
    if readme_path.exists() {
        match process_file(&readme_path, "## 문서", &readme_docs, args.write) {
            Ok(true) => changes.push("README.md".to_string()),
            Ok(false) => {}
            Err(err) => {
                eprintln!("{err}");
                std::process::exit(1);
            }
        }
    }

    if !missing.is_empty() && (args.check || args.fail_on_missing) {
        eprintln!("Missing docs: {}", missing.join(", "));
    }

    if !changes.is_empty() && !args.write {
        eprintln!("Doc lists are out of date: {}", changes.join(", "));
        eprintln!("Run: task doc-links");
        std::process::exit(1);
    }

    if !missing.is_empty() && args.fail_on_missing {
        eprintln!("Create missing docs or update link-order.md");
        std::process::exit(1);
    }

    if args.write {
        if !changes.is_empty() {
            println!("Updated: {}", changes.join(", "));
        } else {
            println!("No changes.");
        }
    } else if !changes.is_empty() {
        println!("Updated: {}", changes.join(", "));
    } else {
        println!("Doc lists are up to date.");
    }
}
