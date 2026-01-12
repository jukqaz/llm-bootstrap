use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

const REQUIRED_ITEMS: &[&str] = &[
    "응답 언어/톤",
    "프로젝트 타입/기술 스택",
    "모노레포 여부/모듈 경계",
    "중첩 AGENTS.md 범위",
    "필수 섹션",
    "빌드/실행/테스트/코드생성 명령",
    "교차 워크플로우",
    "Git/브랜치/커밋/PR 규칙",
    "보안/민감정보/파괴적 명령 규칙",
    "MCP/외부 도구 사용 규칙",
    "참조 문서",
];

const INVALID_ANSWERS: &[&str] = &[
    "",
    "-",
    "tbd",
    "todo",
    "n/a",
    "na",
    "unknown",
    "미정",
    "미기입",
    "확인 필요",
];

const ALLOWED_EMPTY: &[&str] = &["없음", "해당 없음"];

fn is_invalid_answer(value: &str) -> bool {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return true;
    }
    let lower = trimmed.to_lowercase();
    if ALLOWED_EMPTY.iter().any(|allowed| lower == *allowed) {
        return false;
    }
    if INVALID_ANSWERS.iter().any(|item| lower == *item) {
        return true;
    }
    trimmed.contains('<') && trimmed.contains('>')
}

fn extract_answer_section(content: &str, heading: &str) -> Option<String> {
    let normalized = content.replace("\r\n", "\n");
    let mut lines = normalized.lines().peekable();
    while let Some(line) = lines.next() {
        if line.trim() == heading {
            let mut section = Vec::new();
            while let Some(next) = lines.peek() {
                if next.starts_with("## ") {
                    break;
                }
                section.push(lines.next().unwrap());
            }
            return Some(section.join("\n"));
        }
    }
    None
}

fn parse_answers(section: &str) -> HashMap<String, String> {
    let mut answers = HashMap::new();
    for line in section.lines() {
        let trimmed = line.trim_start();
        if !trimmed.starts_with("- ") {
            continue;
        }
        let item = trimmed.trim_start_matches("- ").trim();
        if let Some((label, value)) = item.split_once(':') {
            answers.insert(label.trim().to_string(), value.trim().to_string());
        }
    }
    answers
}

fn main() {
    let path = PathBuf::from("AGENTS.md");
    let content = match fs::read_to_string(&path) {
        Ok(content) => content,
        Err(err) => {
            eprintln!("ERROR: Failed to read {}: {err}", path.display());
            std::process::exit(1);
        }
    };

    let section = match extract_answer_section(&content, "## 질문 답변 요약") {
        Some(section) => section,
        None => {
            eprintln!("ERROR: Missing section: ## 질문 답변 요약");
            std::process::exit(1);
        }
    };

    let answers = parse_answers(&section);
    let mut errors = Vec::new();

    for required in REQUIRED_ITEMS {
        match answers.get(*required) {
            Some(value) => {
                if is_invalid_answer(value) {
                    errors.push(format!("AGENTS.md: 답변 누락 또는 미기입: {required}"));
                }
            }
            None => errors.push(format!("AGENTS.md: 누락된 항목: {required}")),
        }
    }

    for error in &errors {
        eprintln!("ERROR: {error}");
    }

    if !errors.is_empty() {
        std::process::exit(1);
    }

    println!("AGENTS.md question summary passed.");
}
