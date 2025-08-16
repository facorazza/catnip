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

    // Test that directory matching works for any component in the path
    assert!(matcher.matches_path(&PathBuf::from("target")));
    assert!(matcher.matches_path(&PathBuf::from("node_modules")));
    assert!(matcher.matches_path(&PathBuf::from("project/target")));
    assert!(matcher.matches_path(&PathBuf::from("src/node_modules")));

    assert!(!matcher.matches_path(&PathBuf::from("src/main.rs")));
    assert!(!matcher.matches_path(&PathBuf::from("tests/unit.rs")));
}

#[test]
fn test_glob_patterns() {
    let matcher = PatternMatcher::new(&["src/*.rs".to_string(), "**/*.test.js".to_string()]);

    // src/*.rs - single level only
    assert!(matcher.matches_path(&PathBuf::from("src/main.rs")));
    assert!(!matcher.matches_path(&PathBuf::from("tests/main.rs")));
    assert!(!matcher.matches_path(&PathBuf::from("src/nested/deep.rs")));
}

#[test]
fn test_question_mark_pattern() {
    let matcher = PatternMatcher::new(&["*.?s".to_string()]);

    assert!(matcher.matches_path(&PathBuf::from("main.rs")));
    assert!(matcher.matches_path(&PathBuf::from("main.js")));
    assert!(matcher.matches_path(&PathBuf::from("main.ts")));
    assert!(!matcher.matches_path(&PathBuf::from("main.txt")));
}

#[test]
fn test_performance_with_patterns() {
    let patterns: Vec<String> = (0..100).map(|i| format!("pattern_{}.rs", i)).collect();
    let matcher = PatternMatcher::new(&patterns);

    let start = std::time::Instant::now();
    for i in 0..100 {
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
