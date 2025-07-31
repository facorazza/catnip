use regex::Regex;
use std::path::Path;

pub fn advanced_pattern_match(path: &str, pattern: &str) -> bool {
    let regex_pattern = format!(
        "^{}$",
        pattern
            .replace('.', r"\.")
            .replace('*', ".*")
            .replace('?', ".")
    );

    if let Ok(re) = Regex::new(&regex_pattern) {
        re.is_match(path)
    } else {
        false
    }
}

pub fn matches_any_pattern(path: &Path, patterns: &[String]) -> bool {
    let path_str = path.to_string_lossy();
    let filename = path.file_name()
        .map(|n| n.to_string_lossy())
        .unwrap_or_default();

    patterns.iter().any(|pattern| {
        advanced_pattern_match(&path_str, pattern) ||
        advanced_pattern_match(&filename, pattern)
    })
}
