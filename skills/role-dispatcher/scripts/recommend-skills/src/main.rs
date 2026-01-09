use serde::Deserialize;
use std::collections::HashSet;
use std::env;
use std::fs;
use std::path::PathBuf;

#[derive(Deserialize)]
struct KeywordMap {
    #[serde(default)]
    default: Vec<String>,
    #[serde(default)]
    rules: Vec<Rule>,
}

#[derive(Deserialize)]
struct Rule {
    #[serde(default)]
    label: String,
    #[serde(default)]
    priority: i32,
    #[serde(default)]
    keywords: Vec<KeywordEntry>,
    #[serde(default)]
    skills: Vec<String>,
    #[serde(default)]
    flow: Vec<String>,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum KeywordEntry {
    Simple(String),
    Weighted { term: String, weight: i32 },
}

impl KeywordEntry {
    fn term(&self) -> &str {
        match self {
            KeywordEntry::Simple(value) => value,
            KeywordEntry::Weighted { term, .. } => term,
        }
    }

    fn weight(&self) -> i32 {
        match self {
            KeywordEntry::Simple(_) => 1,
            KeywordEntry::Weighted { weight, .. } => *weight,
        }
    }
}

#[derive(serde::Serialize)]
struct JsonOutput {
    skills: Vec<String>,
    matches: Vec<MatchOutput>,
}

#[derive(serde::Serialize)]
struct MatchOutput {
    label: String,
    keywords: Vec<String>,
    score: i32,
    flow: Vec<String>,
}

struct MatchedRule {
    label: String,
    keywords: Vec<String>,
    score: i32,
    flow: Vec<String>,
    skills: Vec<String>,
}

fn is_simple_word(word: &str) -> bool {
    !word.is_empty() && word.chars().all(|c| c.is_ascii_alphanumeric())
}

fn is_word_char(byte: u8) -> bool {
    byte.is_ascii_alphanumeric()
}

fn contains_word(text: &str, word: &str) -> bool {
    let text_bytes = text.as_bytes();
    let word_bytes = word.as_bytes();
    if word_bytes.is_empty() || text_bytes.len() < word_bytes.len() {
        return false;
    }
    for i in 0..=text_bytes.len() - word_bytes.len() {
        if &text_bytes[i..i + word_bytes.len()] == word_bytes {
            let left_ok = i == 0 || !is_word_char(text_bytes[i - 1]);
            let right_ok = i + word_bytes.len() == text_bytes.len()
                || !is_word_char(text_bytes[i + word_bytes.len()]);
            if left_ok && right_ok {
                return true;
            }
        }
    }
    false
}

fn usage() -> ! {
    eprintln!("Usage: recommend-skills --text <text> | --file <path> [--map <path>] [--json]");
    std::process::exit(2);
}

fn match_rules(input_lower: &str, rules: Vec<Rule>) -> Vec<MatchedRule> {
    let mut matched_rules = Vec::new();

    for rule in rules {
        let mut matched = Vec::new();
        let mut keyword_score = 0;
        for keyword in &rule.keywords {
            let keyword_lower = keyword.term().to_lowercase();
            let hit = if is_simple_word(&keyword_lower) {
                contains_word(input_lower, &keyword_lower)
            } else {
                input_lower.contains(&keyword_lower)
            };
            if hit {
                matched.push(keyword.term().to_string());
                keyword_score += keyword.weight();
            }
        }
        if matched.is_empty() {
            continue;
        }
        let score = keyword_score + rule.priority;
        let flow = if rule.flow.is_empty() {
            rule.skills.clone()
        } else {
            rule.flow.clone()
        };
        matched_rules.push(MatchedRule {
            label: rule.label,
            keywords: matched,
            score,
            flow,
            skills: rule.skills,
        });
    }

    matched_rules.sort_by(|a, b| b.score.cmp(&a.score).then_with(|| a.label.cmp(&b.label)));
    matched_rules
}

fn main() {
    let mut map_path = PathBuf::from("skills/role-dispatcher/references/keyword-map.json");
    let mut text: Option<String> = None;
    let mut file: Option<PathBuf> = None;
    let mut json_output = false;

    let args: Vec<String> = env::args().collect();
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--map" => {
                i += 1;
                if i >= args.len() {
                    usage();
                }
                map_path = PathBuf::from(&args[i]);
            }
            "--text" => {
                i += 1;
                if i >= args.len() {
                    usage();
                }
                text = Some(args[i].clone());
            }
            "--file" => {
                i += 1;
                if i >= args.len() {
                    usage();
                }
                file = Some(PathBuf::from(&args[i]));
            }
            "--json" => {
                json_output = true;
            }
            _ => usage(),
        }
        i += 1;
    }

    if text.is_none() && file.is_none() {
        usage();
    }
    if text.is_some() && file.is_some() {
        eprintln!("Provide only one of --text or --file");
        std::process::exit(2);
    }

    let input = if let Some(value) = text {
        value
    } else {
        let path = file.expect("file path");
        fs::read_to_string(path).expect("Failed to read input file")
    };
    let input_lower = input.to_lowercase();

    let map_raw = fs::read_to_string(&map_path).expect("Failed to read keyword map");
    let map: KeywordMap = serde_json::from_str(&map_raw).expect("Invalid keyword map JSON");

    let matched_rules = match_rules(&input_lower, map.rules);
    let mut skill_list = Vec::new();
    let mut seen = HashSet::new();
    let mut matches = Vec::new();
    for matched in matched_rules {
        matches.push(MatchOutput {
            label: matched.label,
            keywords: matched.keywords,
            score: matched.score,
            flow: matched.flow,
        });
        for skill in matched.skills {
            if seen.insert(skill.clone()) {
                skill_list.push(skill);
            }
        }
    }

    if skill_list.is_empty() {
        for skill in map.default {
            if seen.insert(skill.clone()) {
                skill_list.push(skill);
            }
        }
    }

    if json_output {
        let output = JsonOutput {
            skills: skill_list,
            matches,
        };
        println!("{}", serde_json::to_string_pretty(&output).unwrap());
    } else {
        println!("추천 스킬:");
        for skill in &skill_list {
            println!("- {}", skill);
        }
        if !matches.is_empty() {
            println!("\n매칭 키워드:");
            for m in matches {
                let keywords = m.keywords.join(", ");
                println!("- {} (점수 {}): {}", m.label, m.score, keywords);
                if !m.flow.is_empty() {
                    println!("  흐름: {}", m.flow.join(" → "));
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_simple_word_boundaries() {
        assert!(contains_word("run ci checks", "ci"));
        assert!(!contains_word("cicd pipeline", "ci"));
    }

    #[test]
    fn sorts_by_weighted_score() {
        let map = KeywordMap {
            default: Vec::new(),
            rules: vec![
                Rule {
                    label: "first".to_string(),
                    priority: 0,
                    keywords: vec![KeywordEntry::Weighted {
                        term: "alpha".to_string(),
                        weight: 1,
                    }],
                    skills: vec!["skill-a".to_string()],
                    flow: vec!["skill-a".to_string()],
                },
                Rule {
                    label: "second".to_string(),
                    priority: 0,
                    keywords: vec![KeywordEntry::Weighted {
                        term: "beta".to_string(),
                        weight: 3,
                    }],
                    skills: vec!["skill-b".to_string()],
                    flow: vec!["skill-b".to_string()],
                },
            ],
        };
        let input_lower = "alpha and beta".to_lowercase();
        let matched_rules = match_rules(&input_lower, map.rules);
        assert_eq!(matched_rules[0].label, "second");
    }
}
