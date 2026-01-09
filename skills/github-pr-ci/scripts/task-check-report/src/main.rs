use std::env;
use std::fs;
use std::fs::OpenOptions;
use std::io::Write;

fn strip_ansi(input: &str) -> String {
    let mut output = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '\u{1b}' {
            if let Some('[') = chars.peek().copied() {
                chars.next();
                while let Some(next) = chars.next() {
                    if next.is_ascii_alphabetic() {
                        break;
                    }
                }
            }
            continue;
        }
        output.push(ch);
    }
    output
}

fn read_env_usize(key: &str, default: usize) -> usize {
    env::var(key)
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(default)
}

fn main() {
    let exit_code = env::var("EXIT_CODE").unwrap_or_else(|_| "1".to_string());
    let log_path = env::var("LOG_PATH").unwrap_or_else(|_| "task-check.log".to_string());
    let report_path =
        env::var("REPORT_PATH").unwrap_or_else(|_| "task-check-report.md".to_string());
    let max_lines = read_env_usize("MAX_LINES", 120);
    let summary_lines = read_env_usize("SUMMARY_LINES", 6);

    let content = fs::read_to_string(&log_path).unwrap_or_default();
    let cleaned = strip_ansi(&content);
    let lines: Vec<&str> = cleaned.lines().collect();

    let snippet = if lines.is_empty() {
        "(no output)".to_string()
    } else {
        let start = lines.len().saturating_sub(max_lines);
        lines[start..].join("\n")
    };

    let summary_patterns = [
        "Doc lists are out of date",
        "Missing docs:",
        "ERROR:",
        "WARN:",
        "task: Failed",
        "::error",
    ];

    let mut summary = Vec::new();
    for line in &lines {
        if summary_patterns.iter().any(|pattern| line.contains(pattern)) {
            summary.push((*line).to_string());
        }
    }
    if summary.len() > summary_lines {
        summary = summary[summary.len() - summary_lines..].to_vec();
    }
    let summary_block = if summary.is_empty() {
        "- (no summary)".to_string()
    } else {
        summary
            .iter()
            .map(|line| format!("- {}", line))
            .collect::<Vec<String>>()
            .join("\n")
    };

    let status = if exit_code == "0" { "PASS" } else { "FAIL" };

    let comment_body = format!(
        "<!-- task-check-report -->\n### task check: {status}\n**Summary**\n{summary_block}\n\n**Log (last {max_lines} lines)**\n```\n{snippet}\n```\n"
    );
    fs::write(&report_path, comment_body).expect("Failed to write report");

    if let Ok(summary_path) = env::var("GITHUB_STEP_SUMMARY") {
        let summary_body = format!(
            "### task check: {status}\n**Summary**\n{summary_block}\n"
        );
        if let Ok(mut file) = OpenOptions::new()
            .create(true)
            .append(true)
            .open(summary_path)
        {
            let _ = writeln!(file, "{}", summary_body);
        }
    }
}
