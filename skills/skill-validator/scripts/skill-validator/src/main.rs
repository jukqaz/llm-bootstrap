use regex::Regex;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

const SECTION_HEADERS: &[(&str, &str)] = &[
    ("references/", "references"),
    ("assets/", "assets"),
    ("scripts/", "scripts"),
];

fn valid_skill_name(name: &str) -> bool {
    !name.is_empty()
        && name
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
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

fn extract_resource_refs(lines: &[&str]) -> HashMap<&'static str, Vec<String>> {
    let mut refs: HashMap<&'static str, Vec<String>> = HashMap::new();
    refs.insert("references", Vec::new());
    refs.insert("assets", Vec::new());
    refs.insert("scripts", Vec::new());

    let mut section: Option<&'static str> = None;
    for line in lines {
        if let Some(header) = line.strip_prefix("### ") {
            section = SECTION_HEADERS
                .iter()
                .find(|(key, _)| *key == header.trim())
                .map(|(_, value)| *value);
            continue;
        }
        if let Some(active) = section {
            if line.trim_start().starts_with("- ") {
                if let Some(start) = line.find('`') {
                    if let Some(end) = line[start + 1..].find('`') {
                        let value = &line[start + 1..start + 1 + end];
                        if let Some(list) = refs.get_mut(active) {
                            list.push(value.to_string());
                        }
                    }
                }
            }
        }
    }
    refs
}

fn normalize_link_target(target: &str) -> Option<String> {
    let trimmed = target.trim();
    if trimmed.is_empty() || trimmed.starts_with('#') {
        return None;
    }
    if trimmed.starts_with("http://")
        || trimmed.starts_with("https://")
        || trimmed.starts_with("mailto:")
        || trimmed.starts_with("tel:")
    {
        return None;
    }
    if trimmed.contains("{{") || trimmed.contains("}}") {
        return None;
    }
    if trimmed.contains('<') || trimmed.contains('>') {
        return None;
    }
    if trimmed.contains("://") {
        return None;
    }
    Some(trimmed.to_string())
}

fn resolve_link_path(target: &str, file_path: &Path, root_dir: &Path) -> PathBuf {
    if target.starts_with('/') {
        return root_dir.join(target.trim_start_matches('/'));
    }
    file_path
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join(target)
}

fn validate_markdown_links(file_path: &Path, root_dir: &Path, link_re: &Regex) -> Vec<String> {
    let mut errors = Vec::new();
    let content = match fs::read_to_string(file_path) {
        Ok(content) => content,
        Err(err) => {
            errors.push(format!("Failed to read {}: {err}", file_path.display()));
            return errors;
        }
    };
    for caps in link_re.captures_iter(&content) {
        let raw_target = caps.get(1).map(|m| m.as_str()).unwrap_or("");
        let raw_target = raw_target.split('#').next().unwrap_or("");
        let raw_target = raw_target.split('?').next().unwrap_or("");
        let Some(target) = normalize_link_target(raw_target) else {
            continue;
        };
        let resolved = resolve_link_path(&target, file_path, root_dir);
        if !resolved.exists() {
            errors.push(format!(
                "Missing link target in {}: {}",
                file_path.display(),
                raw_target
            ));
        }
    }
    errors
}

fn collect_md_files(dir: &Path, files: &mut Vec<PathBuf>) {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                collect_md_files(&path, files);
            } else if path.extension().and_then(|s| s.to_str()) == Some("md") {
                files.push(path);
            }
        }
    }
}

fn validate_skill(skill_dir: &Path, link_re: &Regex) -> (Vec<String>, Vec<String>) {
    let mut errors = Vec::new();
    let mut warnings = Vec::new();
    let root_dir = skill_dir.parent().unwrap_or_else(|| Path::new("."));

    if !valid_skill_name(&skill_dir.file_name().unwrap().to_string_lossy()) {
        errors.push(format!("Invalid skill folder name: {}", skill_dir.display()));
    }

    let skill_md = skill_dir.join("SKILL.md");
    if !skill_md.exists() {
        errors.push(format!("Missing SKILL.md: {}", skill_dir.display()));
        return (errors, warnings);
    }

    let text = match fs::read_to_string(&skill_md) {
        Ok(text) => text,
        Err(err) => {
            errors.push(format!("Failed to read {}: {err}", skill_md.display()));
            return (errors, warnings);
        }
    };
    let fm = parse_frontmatter(&text);
    let name = fm.get("name");
    let desc = fm.get("description");
    if name.is_none() || desc.is_none() {
        errors.push(format!("Missing frontmatter fields in {}", skill_md.display()));
    } else if name.unwrap() != skill_dir.file_name().unwrap().to_string_lossy().as_ref() {
        errors.push(format!(
            "name mismatch in {}: {} != {}",
            skill_md.display(),
            name.unwrap(),
            skill_dir.file_name().unwrap().to_string_lossy()
        ));
    }

    if let Ok(entries) = fs::read_dir(skill_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                continue;
            }
            let filename = path.file_name().unwrap().to_string_lossy();
            if filename == "SKILL.md" || filename.starts_with('.') {
                continue;
            }
            if filename.to_lowercase().starts_with("readme") {
                errors.push(format!("Unexpected README in {}", skill_dir.display()));
            } else {
                warnings.push(format!(
                    "Unexpected file in {}: {}",
                    skill_dir.display(),
                    filename
                ));
            }
        }
    }

    let refs = extract_resource_refs(&text.lines().collect::<Vec<_>>());
    for (section, files) in refs {
        if files.is_empty() {
            continue;
        }
        let section_dir = skill_dir.join(section);
        if !section_dir.exists() {
            errors.push(format!("Missing {section}/ dir in {}", skill_dir.display()));
            continue;
        }
        for rel_path in files {
            let target = section_dir.join(rel_path);
            if !target.exists() {
                errors.push(format!("Missing {section} file: {}", target.display()));
            }
        }
    }

    errors.extend(validate_markdown_links(&skill_md, root_dir, link_re));

    let references_dir = skill_dir.join("references");
    if references_dir.exists() {
        let mut md_files = Vec::new();
        collect_md_files(&references_dir, &mut md_files);
        for md in md_files {
            errors.extend(validate_markdown_links(&md, root_dir, link_re));
        }
    }

    let assets_dir = skill_dir.join("assets");
    if assets_dir.exists() {
        let mut md_files = Vec::new();
        collect_md_files(&assets_dir, &mut md_files);
        for md in md_files {
            errors.extend(validate_markdown_links(&md, root_dir, link_re));
        }
    }

    (errors, warnings)
}

fn main() {
    let skills_dir = PathBuf::from("skills");
    if !skills_dir.exists() {
        eprintln!("skills dir not found: {}", skills_dir.display());
        std::process::exit(2);
    }

    let link_re = Regex::new(r"!?\[[^\]]+\]\(([^)]+)\)").expect("Invalid regex");

    let mut all_errors = Vec::new();
    let mut all_warnings = Vec::new();

    let entries = match fs::read_dir(&skills_dir) {
        Ok(entries) => entries,
        Err(err) => {
            eprintln!("Failed to read skills dir: {err}");
            std::process::exit(2);
        }
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let (errors, warnings) = validate_skill(&path, &link_re);
        all_errors.extend(errors);
        all_warnings.extend(warnings);
    }

    for warning in &all_warnings {
        eprintln!("WARN: {warning}");
    }
    for error in &all_errors {
        eprintln!("ERROR: {error}");
    }

    if !all_errors.is_empty() {
        std::process::exit(1);
    }
    println!("Skill validation passed.");
}
