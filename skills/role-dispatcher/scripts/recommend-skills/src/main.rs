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
    keywords: Vec<String>,
    #[serde(default)]
    skills: Vec<String>,
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

    let mut matches = Vec::new();
    let mut skill_list = Vec::new();
    let mut seen = HashSet::new();

    for rule in map.rules {
        let mut matched = Vec::new();
        for keyword in &rule.keywords {
            let keyword_lower = keyword.to_lowercase();
            let hit = if is_simple_word(&keyword_lower) {
                contains_word(&input_lower, &keyword_lower)
            } else {
                input_lower.contains(&keyword_lower)
            };
            if hit {
                matched.push(keyword.clone());
            }
        }
        if matched.is_empty() {
            continue;
        }
        matches.push(MatchOutput {
            label: rule.label.clone(),
            keywords: matched,
        });
        for skill in rule.skills {
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
                println!("- {}: {}", m.label, keywords);
            }
        }
    }
}
