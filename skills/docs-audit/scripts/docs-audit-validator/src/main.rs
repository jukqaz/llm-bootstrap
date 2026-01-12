use std::collections::HashSet;
use std::fs;
use std::path::Path;

struct DocSpec<'a> {
    name: &'a str,
    path: &'a str,
    required_groups: Vec<Vec<&'a str>>,
    optional: bool,
    require_subsections: bool,
    require_h1: bool,
}

fn read_file(path: &str) -> Result<String, String> {
    fs::read_to_string(path).map_err(|err| format!("Failed to read {path}: {err}"))
}

fn collect_headers(content: &str) -> HashSet<String> {
    let mut headers = HashSet::new();
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('#') {
            headers.insert(trimmed.to_lowercase());
        }
    }
    headers
}

fn has_any(headers: &HashSet<String>, options: &[&str]) -> bool {
    options
        .iter()
        .any(|option| headers.contains(&option.to_lowercase()))
}

fn validate_doc(spec: &DocSpec) -> (Vec<String>, Vec<String>) {
    let mut errors = Vec::new();
    let mut warnings = Vec::new();
    let path = Path::new(spec.path);
    if !path.exists() {
        if spec.optional {
            warnings.push(format!("WARN: Missing optional doc: {}", spec.path));
            return (errors, warnings);
        }
        errors.push(format!("ERROR: Missing required doc: {}", spec.path));
        return (errors, warnings);
    }

    let content = match read_file(spec.path) {
        Ok(content) => content,
        Err(err) => {
            errors.push(format!("ERROR: {err}"));
            return (errors, warnings);
        }
    };
    let headers = collect_headers(&content);
    if spec.require_h1 {
        let has_h1 = content.lines().any(|line| line.trim_start().starts_with("# "));
        if !has_h1 {
            errors.push(format!("ERROR: {} missing top-level header", spec.name));
        }
    }

    for group in &spec.required_groups {
        if !has_any(&headers, group) {
            errors.push(format!(
                "ERROR: {} missing required section: {}",
                spec.name,
                group.join(" / ")
            ));
        }
    }

    if spec.require_subsections {
        let has_sub = content.lines().any(|line| line.trim_start().starts_with("## "));
        if !has_sub {
            errors.push(format!(
                "ERROR: {} missing subsection entries (## ...)",
                spec.name
            ));
        }
    }

    (errors, warnings)
}

fn main() {
    let specs = vec![
        DocSpec {
            name: "AGENTS.md",
            path: "AGENTS.md",
            required_groups: vec![vec!["## 질문 답변 요약"]],
            optional: false,
            require_subsections: false,
            require_h1: true,
        },
        DocSpec {
            name: "PRD.md",
            path: "PRD.md",
            required_groups: vec![
                vec!["## 요약"],
                vec!["## 목표"],
                vec!["## 비목표"],
                vec!["## 사용자 및 사용 사례"],
                vec!["## 요구사항", "## 요구 사항"],
                vec!["## 성공 지표"],
                vec!["## 리스크 및 가정"],
                vec!["## 열린 질문"],
            ],
            optional: false,
            require_subsections: false,
            require_h1: true,
        },
        DocSpec {
            name: "TRD.md",
            path: "TRD.md",
            required_groups: vec![
                vec!["## 요약"],
                vec!["## 아키텍처"],
                vec!["## 데이터"],
                vec!["## 구현 계획"],
                vec!["## 빌드 / 실행 / 테스트", "## 빌드/실행/테스트"],
                vec!["## 보안 및 개인정보"],
                vec!["## 리스크 및 대응"],
                vec!["## 열린 질문"],
            ],
            optional: false,
            require_subsections: false,
            require_h1: true,
        },
        DocSpec {
            name: "README.md",
            path: "README.md",
            required_groups: vec![
                vec!["## 목적", "## 개요", "## summary"],
                vec!["## 사용 가이드", "## 사용 방법", "## 사용", "## usage"],
                vec!["## 문서", "## documentation"],
            ],
            optional: false,
            require_subsections: false,
            require_h1: true,
        },
        DocSpec {
            name: "ROLE-MAP.md",
            path: "ROLE-MAP.md",
            required_groups: vec![vec!["## 기본 플레이북", "## 기본 플레이북 (자주 쓰는 흐름)"]],
            optional: false,
            require_subsections: false,
            require_h1: true,
        },
        DocSpec {
            name: "CHANGELOG.md",
            path: "CHANGELOG.md",
            required_groups: vec![vec!["# changelog"]],
            optional: true,
            require_subsections: true,
            require_h1: true,
        },
    ];

    let mut errors = Vec::new();
    let mut warnings = Vec::new();

    for spec in specs {
        let (errs, warns) = validate_doc(&spec);
        errors.extend(errs);
        warnings.extend(warns);
    }

    for warning in &warnings {
        eprintln!("{warning}");
    }
    for error in &errors {
        eprintln!("{error}");
    }

    if !errors.is_empty() {
        std::process::exit(1);
    }

    println!("Docs audit validation passed.");
}
