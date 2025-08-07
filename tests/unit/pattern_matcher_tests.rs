use catnip::pattern_matcher::PatternMatcher;
use std::path::PathBuf;

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
fn test_directory_matching() {
    let matcher = PatternMatcher::new(&[
        "target".to_string(),
        "node_modules".to_string(),
        "build".to_string(),
    ]);

    assert!(matcher.matches_path(&PathBuf::from("target/debug/main")));
    assert!(matcher.matches_path(&PathBuf::from("src/target/file")));
    assert!(matcher.matches_path(&PathBuf::from("project/node_modules/lib")));
    assert!(matcher.matches_path(&PathBuf::from("build/output.exe")));

    assert!(!matcher.matches_path(&PathBuf::from("src/main.rs")));
    assert!(!matcher.matches_path(&PathBuf::from("tests/unit.rs")));
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
    assert!(matcher.matches_path(&PathBuf::from("project/src/lib.rs")));
    assert!(!matcher.matches_path(&PathBuf::from("tests/main.rs")));
    assert!(!matcher.matches_path(&PathBuf::from("src/nested/deep.rs")));

    // tests/*.py
    assert!(matcher.matches_path(&PathBuf::from("tests/test.py")));
    assert!(matcher.matches_path(&PathBuf::from("project/tests/unit.py")));
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
        "**/target/*".to_string(),
        "src/**/*.rs".to_string(),
        "**/*.test.js".to_string(),
    ]);

    // **/target/*
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
fn test_complex_patterns() {
    let matcher = PatternMatcher::new(&[
        "**/.git/*".to_string(),
        "**/node_modules/**".to_string(),
        "build/*/output".to_string(),
    ]);

    // **/.git/*
    assert!(matcher.matches_path(&PathBuf::from(".git/config")));
    assert!(matcher.matches_path(&PathBuf::from("project/.git/HEAD")));
    assert!(matcher.matches_path(&PathBuf::from("nested/project/.git/objects/abc")));

    // **/node_modules/**
    assert!(matcher.matches_path(&PathBuf::from("node_modules/react/index.js")));
    assert!(matcher.matches_path(&PathBuf::from("project/node_modules/lodash/lib.js")));
    assert!(matcher.matches_path(&PathBuf::from(
        "deep/project/node_modules/nested/lib/file.js"
    )));

    // build/*/output
    assert!(matcher.matches_path(&PathBuf::from("build/debug/output")));
    assert!(matcher.matches_path(&PathBuf::from("build/release/output")));
    assert!(matcher.matches_path(&PathBuf::from("project/build/test/output")));
    assert!(!matcher.matches_path(&PathBuf::from("build/output"))); // missing middle part
    assert!(!matcher.matches_path(&PathBuf::from("build/debug/release/output"))); // too many parts
}

#[test]
fn test_non_matching_patterns() {
    let matcher = PatternMatcher::new(&[
        "*.rs".to_string(),
        "src/*".to_string(),
        "target".to_string(),
    ]);

    assert!(!matcher.matches_path(&PathBuf::from("file.txt")));
    assert!(!matcher.matches_path(&PathBuf::from("README.md")));
    assert!(!matcher.matches_path(&PathBuf::from("tests/main.rs"))); // doesn't match src/*
    assert!(!matcher.matches_path(&PathBuf::from("docs/guide.md")));
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
        "".to_string(), // empty pattern
    ]);

    // Single star should match anything in same directory
    assert!(matcher.matches_path(&PathBuf::from("file")));
    assert!(matcher.matches_path(&PathBuf::from("file.ext")));

    // Double star should match anything anywhere
    assert!(matcher.matches_path(&PathBuf::from("any/path/file")));

    // Single question mark should match single character
    assert!(matcher.matches_path(&PathBuf::from("a")));
    assert!(matcher.matches_path(&PathBuf::from("1")));
    assert!(!matcher.matches_path(&PathBuf::from("ab"))); // too long
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

    // Should still be fast with many patterns
    let start = std::time::Instant::now();
    for i in 0..1000 {
        let path = PathBuf::from(format!("pattern_{}.rs", i));
        assert!(matcher.matches_path(&path));
    }
    let duration = start.elapsed();

    // Should complete in reasonable time (adjust threshold as needed)
    assert!(
        duration.as_millis() < 100,
        "Pattern matching took too long: {:?}",
        duration
    );
}
