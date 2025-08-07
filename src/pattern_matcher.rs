use std::collections::HashSet;
use std::path::Path;
use tracing::{debug, instrument};

#[derive(Debug)]
pub struct PatternMatcher {
    // Fast lookups for exact matches
    exact_filenames: HashSet<String>,
    exact_extensions: HashSet<String>,
    exact_directories: HashSet<String>,

    // Simple glob patterns
    glob_patterns: Vec<GlobPattern>,
}

#[derive(Debug)]
struct GlobPattern {
    pattern: String,
    parts: Vec<GlobPart>,
}

#[derive(Debug)]
enum GlobPart {
    Literal(String),
    Star,       // *
    DoubleStar, // **
    Question,   // ?
}

impl PatternMatcher {
    pub fn new(patterns: &[String]) -> Self {
        let mut exact_filenames = HashSet::new();
        let mut exact_extensions = HashSet::new();
        let mut exact_directories = HashSet::new();
        let mut glob_patterns = Vec::new();

        for pattern in patterns {
            Self::categorize_pattern(
                pattern.trim(),
                &mut exact_filenames,
                &mut exact_extensions,
                &mut exact_directories,
                &mut glob_patterns,
            );
        }

        debug!(
            "PatternMatcher created: {} exact filenames, {} extensions, {} directories, {} globs",
            exact_filenames.len(),
            exact_extensions.len(),
            exact_directories.len(),
            glob_patterns.len()
        );

        Self {
            exact_filenames,
            exact_extensions,
            exact_directories,
            glob_patterns,
        }
    }

    fn categorize_pattern(
        pattern: &str,
        exact_filenames: &mut HashSet<String>,
        exact_extensions: &mut HashSet<String>,
        exact_directories: &mut HashSet<String>,
        glob_patterns: &mut Vec<GlobPattern>,
    ) {
        // Extension patterns (*.rs, *.py, etc.)
        if let Some(ext) = pattern.strip_prefix("*.") {
            if !ext.contains('*') && !ext.contains('?') && !ext.contains('/') {
                exact_extensions.insert(ext.to_string());
                return;
            }
        }

        // Exact filename patterns (Cargo.toml, main.rs, etc.)
        if !pattern.contains('*') && !pattern.contains('?') && !pattern.contains('/') {
            exact_filenames.insert(pattern.to_string());
            return;
        }

        // Simple directory patterns (node_modules, target, etc.)
        if !pattern.contains('*')
            && !pattern.contains('?')
            && !pattern.contains('/')
            && !pattern.contains('.')
        {
            exact_directories.insert(pattern.to_string());
            return;
        }

        // Everything else becomes a glob pattern
        glob_patterns.push(Self::parse_glob_pattern(pattern));
    }

    fn parse_glob_pattern(pattern: &str) -> GlobPattern {
        let mut parts = Vec::new();
        let mut current_literal = String::new();
        let mut chars = pattern.chars().peekable();

        while let Some(ch) = chars.next() {
            match ch {
                '*' => {
                    if chars.peek() == Some(&'*') {
                        chars.next(); // consume second *
                        if !current_literal.is_empty() {
                            parts.push(GlobPart::Literal(current_literal.clone()));
                            current_literal.clear();
                        }
                        parts.push(GlobPart::DoubleStar);
                    } else {
                        if !current_literal.is_empty() {
                            parts.push(GlobPart::Literal(current_literal.clone()));
                            current_literal.clear();
                        }
                        parts.push(GlobPart::Star);
                    }
                }
                '?' => {
                    if !current_literal.is_empty() {
                        parts.push(GlobPart::Literal(current_literal.clone()));
                        current_literal.clear();
                    }
                    parts.push(GlobPart::Question);
                }
                _ => current_literal.push(ch),
            }
        }

        if !current_literal.is_empty() {
            parts.push(GlobPart::Literal(current_literal));
        }

        GlobPattern {
            pattern: pattern.to_string(),
            parts,
        }
    }

    #[instrument(skip(self))]
    pub fn matches_path(&self, path: &Path) -> bool {
        let path_str = path.to_string_lossy();
        let filename = path
            .file_name()
            .map(|n| n.to_string_lossy())
            .unwrap_or_default();

        // Fast exact filename check
        if self.exact_filenames.contains(filename.as_ref()) {
            debug!("Exact filename match: {}", filename);
            return true;
        }

        // Fast extension check
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            if self.exact_extensions.contains(ext) {
                debug!("Extension match: .{}", ext);
                return true;
            }
        }

        // Fast directory check
        for component in path.components() {
            if let Some(dir_name) = component.as_os_str().to_str() {
                if self.exact_directories.contains(dir_name) {
                    debug!("Directory match: {}", dir_name);
                    return true;
                }
            }
        }

        // Glob pattern matching
        for glob in &self.glob_patterns {
            if self.matches_glob(&path_str, glob) {
                debug!("Glob match: {} -> {}", glob.pattern, path_str);
                return true;
            }
        }

        false
    }

    fn matches_glob(&self, path: &str, glob: &GlobPattern) -> bool {
        self.match_parts(path, &glob.parts, 0, 0)
    }

    fn match_parts(
        &self,
        path: &str,
        parts: &[GlobPart],
        path_pos: usize,
        part_idx: usize,
    ) -> bool {
        // If we've consumed all parts
        if part_idx >= parts.len() {
            return path_pos == path.len();
        }

        // If we've consumed all of the path but have parts left
        if path_pos >= path.len() {
            // Only OK if all remaining parts are stars
            return parts[part_idx..]
                .iter()
                .all(|p| matches!(p, GlobPart::Star | GlobPart::DoubleStar));
        }

        match &parts[part_idx] {
            GlobPart::Literal(lit) => {
                if path[path_pos..].starts_with(lit) {
                    self.match_parts(path, parts, path_pos + lit.len(), part_idx + 1)
                } else {
                    false
                }
            }
            GlobPart::Question => {
                let next_char_boundary = path[path_pos..]
                    .char_indices()
                    .nth(1)
                    .map(|(i, _)| path_pos + i)
                    .unwrap_or(path.len());

                if path_pos < path.len() && !path.chars().nth(path_pos).unwrap_or('\0').eq(&'/') {
                    self.match_parts(path, parts, next_char_boundary, part_idx + 1)
                } else {
                    false
                }
            }
            GlobPart::Star => {
                // Try matching zero characters
                if self.match_parts(path, parts, path_pos, part_idx + 1) {
                    return true;
                }

                // Try matching one or more characters (but not path separator)
                for i in path_pos + 1..=path.len() {
                    if path[path_pos..i].contains('/') {
                        break;
                    }
                    if self.match_parts(path, parts, i, part_idx + 1) {
                        return true;
                    }
                }
                false
            }
            GlobPart::DoubleStar => {
                // Try matching zero characters
                if self.match_parts(path, parts, path_pos, part_idx + 1) {
                    return true;
                }

                // Try matching one or more characters (including path separator)
                for i in path_pos + 1..=path.len() {
                    if self.match_parts(path, parts, i, part_idx + 1) {
                        return true;
                    }
                }
                false
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_exact_matching() {
        let matcher = PatternMatcher::new(&[
            "*.rs".to_string(),
            "Cargo.toml".to_string(),
            "target".to_string(),
        ]);

        // Extension matching
        assert!(matcher.matches_path(&PathBuf::from("main.rs")));
        assert!(matcher.matches_path(&PathBuf::from("src/lib.rs")));

        // Filename matching
        assert!(matcher.matches_path(&PathBuf::from("Cargo.toml")));
        assert!(matcher.matches_path(&PathBuf::from("project/Cargo.toml")));

        // Directory matching
        assert!(matcher.matches_path(&PathBuf::from("target/debug/main")));
        assert!(matcher.matches_path(&PathBuf::from("src/target/file")));
    }

    #[test]
    fn test_glob_matching() {
        let matcher = PatternMatcher::new(&[
            "src/*.rs".to_string(),
            "tests/**/*.rs".to_string(),
            "*.?s".to_string(),
        ]);

        // Simple glob
        assert!(matcher.matches_path(&PathBuf::from("src/main.rs")));
        assert!(!matcher.matches_path(&PathBuf::from("tests/main.rs")));

        // Double star
        assert!(matcher.matches_path(&PathBuf::from("tests/unit/test.rs")));
        assert!(matcher.matches_path(&PathBuf::from("tests/integration/deep/test.rs")));

        // Question mark
        assert!(matcher.matches_path(&PathBuf::from("main.rs")));
        assert!(matcher.matches_path(&PathBuf::from("main.js")));
        assert!(!matcher.matches_path(&PathBuf::from("main.txt")));
    }

    #[test]
    fn test_non_matches() {
        let matcher = PatternMatcher::new(&["*.rs".to_string(), "src/*".to_string()]);

        assert!(!matcher.matches_path(&PathBuf::from("file.txt")));
        assert!(!matcher.matches_path(&PathBuf::from("README.md")));
        assert!(!matcher.matches_path(&PathBuf::from("tests/main.rs"))); // doesn't match src/*
    }

    #[test]
    fn test_directory_patterns() {
        let matcher =
            PatternMatcher::new(&["target/*".to_string(), "**/node_modules/*".to_string()]);

        assert!(matcher.matches_path(&PathBuf::from("target/debug/main")));
        assert!(matcher.matches_path(&PathBuf::from("project/node_modules/lib/index.js")));
        assert!(matcher.matches_path(&PathBuf::from("deep/project/node_modules/lib.js")));
    }
}
