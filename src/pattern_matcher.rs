use regex::Regex;
use std::path::Path;
use tracing::{debug, instrument};

#[instrument]
pub fn advanced_pattern_match(path: &str, pattern: &str) -> bool {
    let regex_pattern = format!(
        "^{}$",
        pattern
            .replace('.', r"\.")
            .replace('*', ".*")
            .replace('?', ".")
    );

    match Regex::new(&regex_pattern) {
        Ok(re) => {
            let matches = re.is_match(path);
            debug!("Pattern '{}' vs path '{}': {}", pattern, path, matches);
            matches
        }
        Err(e) => {
            debug!("Invalid regex pattern '{}': {}", regex_pattern, e);
            false
        }
    }
}

#[instrument(skip(patterns))]
pub fn matches_any_pattern(path: &Path, patterns: &[String]) -> bool {
    let path_str = path.to_string_lossy();
    let filename = path
        .file_name()
        .map(|n| n.to_string_lossy())
        .unwrap_or_default();

    let matches = patterns.iter().any(|pattern| {
        advanced_pattern_match(&path_str, pattern) || advanced_pattern_match(&filename, pattern)
    });

    if matches {
        debug!(
            "Path '{}' matched one of {} patterns",
            path_str,
            patterns.len()
        );
    }

    matches
}
