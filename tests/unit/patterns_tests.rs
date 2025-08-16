use catnip::patterns::{DEFAULT_EXCLUDE_PATTERNS, DEFAULT_INCLUDE_PATTERNS};
use std::collections::HashSet;

#[test]
fn test_default_patterns_exist() {
    assert!(!DEFAULT_EXCLUDE_PATTERNS.is_empty());
    assert!(!DEFAULT_INCLUDE_PATTERNS.is_empty());
    assert!(DEFAULT_EXCLUDE_PATTERNS.len() > 10);
    assert!(DEFAULT_INCLUDE_PATTERNS.len() > 10);
}

#[test]
fn test_common_exclusion_patterns() {
    // Test for patterns that should definitely exist
    let has_pyc = DEFAULT_EXCLUDE_PATTERNS.contains(&"*.pyc");
    let has_git = DEFAULT_EXCLUDE_PATTERNS.iter().any(|p| p.contains(".git"));
    let has_target = DEFAULT_EXCLUDE_PATTERNS
        .iter()
        .any(|p| p.contains("target"));
    let has_node_modules = DEFAULT_EXCLUDE_PATTERNS
        .iter()
        .any(|p| p.contains("node_modules"));

    assert!(
        has_pyc || has_git || has_target || has_node_modules,
        "Should have some common exclusion patterns"
    );
}

#[test]
fn test_common_inclusion_patterns() {
    // Test for patterns that should definitely exist
    let has_rust = DEFAULT_INCLUDE_PATTERNS.contains(&"*.rs");
    let has_python = DEFAULT_INCLUDE_PATTERNS.contains(&"*.py");
    let has_js = DEFAULT_INCLUDE_PATTERNS.contains(&"*.js");
    let has_md = DEFAULT_INCLUDE_PATTERNS.contains(&"*.md");

    assert!(
        has_rust || has_python || has_js || has_md,
        "Should have some common inclusion patterns"
    );
}

#[test]
fn test_no_duplicate_patterns() {
    let mut exclude_set = HashSet::new();
    for pattern in DEFAULT_EXCLUDE_PATTERNS {
        assert!(
            exclude_set.insert(pattern),
            "Duplicate exclude pattern: {}",
            pattern
        );
    }

    let mut include_set = HashSet::new();
    for pattern in DEFAULT_INCLUDE_PATTERNS {
        assert!(
            include_set.insert(pattern),
            "Duplicate include pattern: {}",
            pattern
        );
    }
}

#[test]
fn test_patterns_are_strings() {
    // Basic validation that patterns are non-empty strings
    for pattern in DEFAULT_EXCLUDE_PATTERNS {
        assert!(!pattern.is_empty(), "Empty exclude pattern found");
    }

    for pattern in DEFAULT_INCLUDE_PATTERNS {
        assert!(!pattern.is_empty(), "Empty include pattern found");
    }
}
