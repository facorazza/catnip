use catnip::pattern_matcher::PatternMatcher;
use std::path::PathBuf;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exact_filename_matching() {
        let matcher = PatternMatcher::new(&[
            "Cargo.toml".to_string(),
            "README.md".to_string(),
            "main.rs".to_string(),
        ]);

        assert!(matcher.matches_path(&PathBuf::from("Cargo.toml")));
        assert!(matcher.matches_path(&PathBuf::from("project/Cargo.toml")));
        assert!(matcher.matches_path(&PathBuf::from("src/main.rs")));
        assert!(matcher.matches_path(&PathBuf::from("README.md")));

        assert!(!matcher.matches_path(&PathBuf::from("cargo.toml"))); // case sensitive
        assert!(!matcher.matches_path(&PathBuf::from("lib.rs")));
    }

    #[test]
    fn test_extension_matching() {
        let matcher =
            PatternMatcher::new(&["*.rs".to_string(), "*.py".to_string(), "*.js".to_string()]);

        assert!(matcher.matches_path(&PathBuf::from("main.rs")));
        assert!(matcher.matches_path(&PathBuf::from("src/lib.rs")));
        assert!(matcher.matches_path(&PathBuf::from("script.py")));
        assert!(matcher.matches_path(&PathBuf::from("app.js")));

        assert!(!matcher.matches_path(&PathBuf::from("file.txt")));
        assert!(!matcher.matches_path(&PathBuf::from("file.cpp")));
    }

    #[test]
    fn test_directory_matching_optimized() {
        let matcher = PatternMatcher::new(&[
            "target".to_string(),
            "node_modules".to_string(),
            "build".to_string(),
        ]);

        assert!(matcher.matches_path(&PathBuf::from("target/debug/main")));
        assert!(matcher.matches_path(&PathBuf::from("src/target/file")));
        assert!(matcher.matches_path(&PathBuf::from("project/node_modules/lib")));
        assert!(matcher.matches_path(&PathBuf::from("build/output.exe")));
        assert!(matcher.matches_path(&PathBuf::from("target")));
        assert!(matcher.matches_path(&PathBuf::from("node_modules")));

        assert!(!matcher.matches_path(&PathBuf::from("src/main.rs")));
        assert!(!matcher.matches_path(&PathBuf::from("tests/unit.rs")));
    }

    #[test]
    fn test_directory_patterns_with_and_without_suffix() {
        let matcher1 = PatternMatcher::new(&["target".to_string()]);
        let matcher2 = PatternMatcher::new(&["target/*".to_string()]);

        let test_paths = [
            "target/debug/main",
            "src/target/file",
            "target",
            "project/target/release",
        ];

        for path in &test_paths {
            let path_buf = PathBuf::from(path);
            assert_eq!(
                matcher1.matches_path(&path_buf),
                matcher2.matches_path(&path_buf),
                "Mismatch for path: {}",
                path
            );
        }
    }

    #[test]
    fn test_simple_glob_patterns() {
        let matcher = PatternMatcher::new(&[
            "src/*.rs".to_string(),
            "tests/*.py".to_string(),
            "*.?s".to_string(),
        ]);

        // src/*.rs
        assert!(matcher.matches_path(&PathBuf::from("src/main.rs")));
        assert!(!matcher.matches_path(&PathBuf::from("tests/main.rs")));
        assert!(!matcher.matches_path(&PathBuf::from("src/nested/deep.rs")));

        // tests/*.py
        assert!(matcher.matches_path(&PathBuf::from("tests/test.py")));
        assert!(!matcher.matches_path(&PathBuf::from("src/test.py")));

        // *.?s (question mark)
        assert!(matcher.matches_path(&PathBuf::from("main.rs")));
        assert!(matcher.matches_path(&PathBuf::from("main.js")));
        assert!(matcher.matches_path(&PathBuf::from("main.ts")));
        assert!(!matcher.matches_path(&PathBuf::from("main.txt"))); // too many chars
        assert!(!matcher.matches_path(&PathBuf::from("main.s"))); // too few chars
    }

    #[test]
    fn test_double_star_patterns() {
        let matcher = PatternMatcher::new(&[
            "**/target/**".to_string(),
            "src/**/*.rs".to_string(),
            "**/*.test.js".to_string(),
        ]);

        // **/target/**
        assert!(matcher.matches_path(&PathBuf::from("target/debug")));
        assert!(matcher.matches_path(&PathBuf::from("project/target/release")));
        assert!(matcher.matches_path(&PathBuf::from("deep/nested/target/file")));

        // src/**/*.rs
        assert!(matcher.matches_path(&PathBuf::from("src/main.rs")));
        assert!(matcher.matches_path(&PathBuf::from("src/utils/helper.rs")));
        assert!(matcher.matches_path(&PathBuf::from("src/deep/nested/lib.rs")));
        assert!(!matcher.matches_path(&PathBuf::from("tests/main.rs")));

        // **/*.test.js
        assert!(matcher.matches_path(&PathBuf::from("app.test.js")));
        assert!(matcher.matches_path(&PathBuf::from("src/app.test.js")));
        assert!(matcher.matches_path(&PathBuf::from("tests/unit/helper.test.js")));
        assert!(!matcher.matches_path(&PathBuf::from("app.js")));
    }

    #[test]
    fn test_performance_fast_paths() {
        let patterns = vec![
            "*.rs".to_string(),
            "node_modules".to_string(),
            "Cargo.toml".to_string(),
            "complex/**/pattern/**/*.test.js".to_string(),
        ];

        let matcher = PatternMatcher::new(&patterns);

        // Fast paths
        assert!(matcher.matches_path(&PathBuf::from("main.rs")));
        assert!(matcher.matches_path(&PathBuf::from("Cargo.toml")));
        assert!(matcher.matches_path(&PathBuf::from("node_modules/lib.js")));
        assert!(!matcher.matches_path(&PathBuf::from("file.txt")));

        // Glob matching
        assert!(matcher.matches_path(&PathBuf::from("complex/deep/pattern/nested/file.test.js")));
    }

    #[test]
    fn test_case_sensitivity() {
        let matcher = PatternMatcher::new(&["*.RS".to_string(), "Makefile".to_string()]);

        assert!(matcher.matches_path(&PathBuf::from("main.RS")));
        assert!(!matcher.matches_path(&PathBuf::from("main.rs")));
        assert!(matcher.matches_path(&PathBuf::from("Makefile")));
        assert!(!matcher.matches_path(&PathBuf::from("makefile")));
    }

    #[test]
    fn test_edge_cases() {
        let matcher = PatternMatcher::new(&[
            "*".to_string(),
            "**".to_string(),
            "?".to_string(),
            "".to_string(),
        ]);

        // Single star
        assert!(matcher.matches_path(&PathBuf::from("file")));
        assert!(matcher.matches_path(&PathBuf::from("file.ext")));

        // Double star
        assert!(matcher.matches_path(&PathBuf::from("any/path/file")));

        // Single question mark
        assert!(matcher.matches_path(&PathBuf::from("a")));
        assert!(matcher.matches_path(&PathBuf::from("1")));
        assert!(!matcher.matches_path(&PathBuf::from("ab")));
    }

    #[test]
    fn test_path_separator_handling() {
        let matcher = PatternMatcher::new(&["src/*".to_string(), "src/**/*".to_string()]);

        // src/* should not cross directory boundaries
        assert!(matcher.matches_path(&PathBuf::from("src/main.rs")));
        assert!(!matcher.matches_path(&PathBuf::from("src/utils/helper.rs")));

        // src/**/* should cross directory boundaries
        assert!(matcher.matches_path(&PathBuf::from("src/main.rs")));
        assert!(matcher.matches_path(&PathBuf::from("src/utils/helper.rs")));
        assert!(matcher.matches_path(&PathBuf::from("src/deep/nested/file.rs")));
    }

    #[test]
    fn test_performance_with_many_patterns() {
        let patterns: Vec<String> = (0..1000).map(|i| format!("pattern_{}.rs", i)).collect();
        let matcher = PatternMatcher::new(&patterns);

        let start = std::time::Instant::now();
        for i in 0..1000 {
            let path = PathBuf::from(format!("pattern_{}.rs", i));
            assert!(matcher.matches_path(&path));
        }
        let duration = start.elapsed();

        assert!(
            duration.as_millis() < 100,
            "Pattern matching took too long: {:?}",
            duration
        );
    }

    #[test]
    fn test_common_exclusion_patterns() {
        let matcher = PatternMatcher::new(&[
            "node_modules".to_string(),
            "target".to_string(),
            "__pycache__".to_string(),
            ".git".to_string(),
            "*.pyc".to_string(),
            "*.log".to_string(),
        ]);

        // Directory exclusions
        assert!(matcher.matches_path(&PathBuf::from("node_modules/react/index.js")));
        assert!(matcher.matches_path(&PathBuf::from("project/node_modules/lib")));
        assert!(matcher.matches_path(&PathBuf::from("target/debug/main")));
        assert!(matcher.matches_path(&PathBuf::from("src/__pycache__/module.pyc")));
        assert!(matcher.matches_path(&PathBuf::from(".git/config")));

        // File extensions
        assert!(matcher.matches_path(&PathBuf::from("module.pyc")));
        assert!(matcher.matches_path(&PathBuf::from("app.log")));

        // No match
        assert!(!matcher.matches_path(&PathBuf::from("src/main.rs")));
        assert!(!matcher.matches_path(&PathBuf::from("README.md")));
    }
}
