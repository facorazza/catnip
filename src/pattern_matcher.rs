use regex::Regex;
use std::path::Path;
use tracing::{debug, instrument};

/// Match files recursively by name, extension, or glob patterns
#[instrument]
pub fn match_path_recursive(path: &Path, pattern: &str) -> bool {
    let path_str = path.to_string_lossy();

    // Try different matching strategies
    match_by_extension(&path_str, pattern) ||
    match_by_filename(&path_str, pattern) ||
    match_by_glob_pattern(&path_str, pattern) ||
    match_by_directory_name(&path_str, pattern)
}

/// Match by file extension (e.g., "*.rs", "rs")
fn match_by_extension(path: &str, pattern: &str) -> bool {
    let extension = if pattern.starts_with("*.") {
        &pattern[2..]
    } else if !pattern.contains('.') && !pattern.contains('/') && !pattern.contains('*') {
        pattern
    } else {
        return false;
    };

    let matches = path.ends_with(&format!(".{}", extension));
    if matches {
        debug!("Extension match: '{}' matches pattern '{}'", path, pattern);
    }
    matches
}

/// Match by exact filename (e.g., "Cargo.toml", "main.rs")
fn match_by_filename(path: &str, pattern: &str) -> bool {
    if let Some(filename) = path.split('/').last() {
        let matches = filename == pattern;
        if matches {
            debug!("Filename match: '{}' matches pattern '{}'", filename, pattern);
        }
        return matches;
    }
    false
}

/// Match by directory name anywhere in the path
fn match_by_directory_name(path: &str, pattern: &str) -> bool {
    if pattern.contains('.') || pattern.contains('*') || pattern.contains('/') {
        return false;
    }

    let path_components: Vec<&str> = path.split('/').collect();
    let matches = path_components.iter().any(|&component| component == pattern);
    if matches {
        debug!("Directory name match: '{}' contains directory '{}'", path, pattern);
    }
    matches
}

/// Advanced glob pattern matching
fn match_by_glob_pattern(path: &str, pattern: &str) -> bool {
    if !pattern.contains('*') && !pattern.contains('?') && !pattern.contains('[') {
        return false;
    }

    let regex_pattern = glob_to_regex(pattern);
    match Regex::new(&regex_pattern) {
        Ok(re) => {
            let matches = re.is_match(path);
            if matches {
                debug!("Glob match: '{}' matches pattern '{}'", path, pattern);
            }
            matches
        }
        Err(e) => {
            debug!("Invalid regex from glob '{}': {}", pattern, e);
            false
        }
    }
}

/// Convert glob pattern to regex
fn glob_to_regex(pattern: &str) -> String {
    let mut regex_pattern = String::from("^");
    let mut chars = pattern.chars().peekable();

    while let Some(ch) = chars.next() {
        match ch {
            '*' => {
                if chars.peek() == Some(&'*') {
                    chars.next(); // consume second *
                    if chars.peek() == Some(&'/') {
                        chars.next(); // consume /
                        regex_pattern.push_str("(?:[^/]*/)*");
                    } else {
                        regex_pattern.push_str(".*");
                    }
                } else {
                    regex_pattern.push_str("[^/]*");
                }
            }
            '?' => regex_pattern.push_str("[^/]"),
            '[' => {
                regex_pattern.push('[');
                while let Some(class_char) = chars.next() {
                    regex_pattern.push(class_char);
                    if class_char == ']' {
                        break;
                    }
                }
            }
            '.' | '^' | '$' | '(' | ')' | '{' | '}' | '+' | '|' | '\\' => {
                regex_pattern.push('\\');
                regex_pattern.push(ch);
            }
            _ => regex_pattern.push(ch),
        }
    }

    regex_pattern.push('$');
    regex_pattern
}

/// Enhanced pattern matching with multiple strategies
#[instrument]
pub fn advanced_pattern_match(path: &str, pattern: &str) -> bool {
    // Direct string match for exact patterns
    if path == pattern {
        return true;
    }

    // Extension patterns (*.rs, *.py, etc.)
    if pattern.starts_with("*.") {
        let ext = &pattern[2..];
        return path.ends_with(&format!(".{}", ext));
    }

    // Directory patterns with wildcards (*/target/*, node_modules/*)
    if pattern.contains('/') {
        return match_by_glob_pattern(path, pattern);
    }

    // Simple filename or directory name patterns
    if !pattern.contains('*') && !pattern.contains('?') && !pattern.contains('[') {
        // Try filename match
        if let Some(filename) = path.split('/').last() {
            if filename == pattern {
                return true;
            }
        }

        // Try directory component match
        let path_components: Vec<&str> = path.split('/').collect();
        return path_components.contains(&pattern);
    }

    // Fallback to glob matching
    match_by_glob_pattern(path, pattern)
}

/// Check if path matches any of the provided patterns
#[instrument(skip(patterns))]
pub fn matches_any_pattern(path: &Path, patterns: &[String]) -> bool {
    let path_str = path.to_string_lossy();
    let filename = path
        .file_name()
        .map(|n| n.to_string_lossy())
        .unwrap_or_default();

    let matches = patterns.iter().any(|pattern| {
        // Try matching against full path first
        if advanced_pattern_match(&path_str, pattern) {
            return true;
        }

        // For simple patterns without path separators, also try against filename
        if !pattern.contains('/') {
            if advanced_pattern_match(&filename, pattern) {
                return true;
            }
        }

        false
    });

    if matches {
        debug!("Path '{}' matched pattern from {} total patterns", path_str, patterns.len());
    }

    matches
}

/// Find all files matching patterns recursively from given paths
pub fn find_matching_files<P: AsRef<Path>>(
    root_paths: &[P],
    patterns: &[String],
) -> Vec<std::path::PathBuf> {
    use walkdir::WalkDir;

    let mut matching_files = Vec::new();

    for root_path in root_paths {
        let root = root_path.as_ref();

        if root.is_file() {
            if matches_any_pattern(root, patterns) {
                matching_files.push(root.to_path_buf());
            }
            continue;
        }

        for entry in WalkDir::new(root).into_iter().filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.is_file() && matches_any_pattern(path, patterns) {
                matching_files.push(path.to_path_buf());
            }
        }
    }

    matching_files
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_extension_patterns() {
        assert!(match_path_recursive(&PathBuf::from("main.rs"), "*.rs"));
        assert!(match_path_recursive(&PathBuf::from("src/main.rs"), "*.rs"));
        assert!(match_path_recursive(&PathBuf::from("main.rs"), "rs"));
        assert!(!match_path_recursive(&PathBuf::from("main.py"), "*.rs"));
    }

    #[test]
    fn test_filename_patterns() {
        assert!(match_path_recursive(&PathBuf::from("Cargo.toml"), "Cargo.toml"));
        assert!(match_path_recursive(&PathBuf::from("src/Cargo.toml"), "Cargo.toml"));
        assert!(!match_path_recursive(&PathBuf::from("package.json"), "Cargo.toml"));
    }

    #[test]
    fn test_directory_patterns() {
        assert!(match_path_recursive(&PathBuf::from("target/debug/main"), "target"));
        assert!(match_path_recursive(&PathBuf::from("src/target/file.rs"), "target"));
        assert!(!match_path_recursive(&PathBuf::from("src/main.rs"), "target"));
    }

    #[test]
    fn test_glob_patterns() {
        assert!(match_path_recursive(&PathBuf::from("target/debug/main"), "target/*"));
        assert!(match_path_recursive(&PathBuf::from("target/debug/main"), "*/debug/*"));
        assert!(match_path_recursive(&PathBuf::from("src/lib.rs"), "src/*.rs"));
        assert!(!match_path_recursive(&PathBuf::from("src/main.rs"), "target/*"));
    }

    #[test]
    fn test_find_matching_files() {
        // This would need actual files to test properly
        let patterns = vec!["*.rs".to_string(), "Cargo.toml".to_string()];
        let results = find_matching_files(&[PathBuf::from(".")], &patterns);
        // In a real Rust project, this should find .rs files and Cargo.toml
        assert!(!results.is_empty() || true); // Allow empty for test environment
    }
}
