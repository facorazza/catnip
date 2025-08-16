use catnip::patterns::{DEFAULT_EXCLUDE_PATTERNS, DEFAULT_INCLUDE_PATTERNS};

#[test]
fn test_default_exclude_patterns_not_empty() {
    assert!(!DEFAULT_EXCLUDE_PATTERNS.is_empty());
    assert!(
        DEFAULT_EXCLUDE_PATTERNS.len() > 50,
        "Should have a reasonable number of exclusion patterns"
    );
}

#[test]
fn test_default_include_patterns_not_empty() {
    assert!(!DEFAULT_INCLUDE_PATTERNS.is_empty());
    assert!(
        DEFAULT_INCLUDE_PATTERNS.len() > 30,
        "Should have a reasonable number of inclusion patterns"
    );
}

#[test]
fn test_common_exclusion_patterns() {
    // Compiled files
    assert!(DEFAULT_EXCLUDE_PATTERNS.contains(&"*.pyc"));
    assert!(DEFAULT_EXCLUDE_PATTERNS.contains(&"*.o"));
    assert!(DEFAULT_EXCLUDE_PATTERNS.contains(&"*.so"));
    assert!(DEFAULT_EXCLUDE_PATTERNS.contains(&"*.dll"));
    assert!(DEFAULT_EXCLUDE_PATTERNS.contains(&"*.exe"));

    // Version control
    assert!(DEFAULT_EXCLUDE_PATTERNS.contains(&".git/*"));
    assert!(DEFAULT_EXCLUDE_PATTERNS.contains(&".svn/*"));

    // Development artifacts
    assert!(DEFAULT_EXCLUDE_PATTERNS.contains(&"node_modules/*"));
    assert!(DEFAULT_EXCLUDE_PATTERNS.contains(&"__pycache__/*"));
    assert!(DEFAULT_EXCLUDE_PATTERNS.contains(&"target/*"));

    // System files
    assert!(DEFAULT_EXCLUDE_PATTERNS.contains(&".DS_Store"));
    assert!(DEFAULT_EXCLUDE_PATTERNS.contains(&"Thumbs.db"));

    // Media files
    assert!(DEFAULT_EXCLUDE_PATTERNS.contains(&"*.jpg"));
    assert!(DEFAULT_EXCLUDE_PATTERNS.contains(&"*.png"));
    assert!(DEFAULT_EXCLUDE_PATTERNS.contains(&"*.mp4"));

    // Archives
    assert!(DEFAULT_EXCLUDE_PATTERNS.contains(&"*.zip"));
    assert!(DEFAULT_EXCLUDE_PATTERNS.contains(&"*.tar"));

    // Logs
    assert!(DEFAULT_EXCLUDE_PATTERNS.contains(&"*.log"));
    assert!(DEFAULT_EXCLUDE_PATTERNS.contains(&"logs/*"));
}

#[test]
fn test_common_inclusion_patterns() {
    // Programming languages
    assert!(DEFAULT_INCLUDE_PATTERNS.contains(&"*.rs"));
    assert!(DEFAULT_INCLUDE_PATTERNS.contains(&"*.py"));
    assert!(DEFAULT_INCLUDE_PATTERNS.contains(&"*.js"));
    assert!(DEFAULT_INCLUDE_PATTERNS.contains(&"*.ts"));
    assert!(DEFAULT_INCLUDE_PATTERNS.contains(&"*.java"));
    assert!(DEFAULT_INCLUDE_PATTERNS.contains(&"*.cpp"));
    assert!(DEFAULT_INCLUDE_PATTERNS.contains(&"*.c"));
    assert!(DEFAULT_INCLUDE_PATTERNS.contains(&"*.go"));

    // Web technologies
    assert!(DEFAULT_INCLUDE_PATTERNS.contains(&"*.html"));
    assert!(DEFAULT_INCLUDE_PATTERNS.contains(&"*.css"));
    assert!(DEFAULT_INCLUDE_PATTERNS.contains(&"*.scss"));

    // Configuration files
    assert!(DEFAULT_INCLUDE_PATTERNS.contains(&"*.json"));
    assert!(DEFAULT_INCLUDE_PATTERNS.contains(&"*.yaml"));
    assert!(DEFAULT_INCLUDE_PATTERNS.contains(&"*.yml"));
    assert!(DEFAULT_INCLUDE_PATTERNS.contains(&"*.toml"));

    // Documentation
    assert!(DEFAULT_INCLUDE_PATTERNS.contains(&"*.md"));
    assert!(DEFAULT_INCLUDE_PATTERNS.contains(&"*.txt"));
    assert!(DEFAULT_INCLUDE_PATTERNS.contains(&"README*"));

    // Build files
    assert!(DEFAULT_INCLUDE_PATTERNS.contains(&"Makefile"));
    assert!(DEFAULT_INCLUDE_PATTERNS.contains(&"Cargo.toml"));
    assert!(DEFAULT_INCLUDE_PATTERNS.contains(&"package.json"));
    assert!(DEFAULT_INCLUDE_PATTERNS.contains(&"Dockerfile*"));

    // Scripts
    assert!(DEFAULT_INCLUDE_PATTERNS.contains(&"*.sh"));
    assert!(DEFAULT_INCLUDE_PATTERNS.contains(&"*.bash"));

    // Database
    assert!(DEFAULT_INCLUDE_PATTERNS.contains(&"*.sql"));
}

#[test]
fn test_no_duplicate_patterns() {
    // Check exclude patterns for duplicates
    let mut exclude_set = std::collections::HashSet::new();
    for pattern in DEFAULT_EXCLUDE_PATTERNS {
        assert!(
            exclude_set.insert(pattern),
            "Duplicate exclude pattern: {}",
            pattern
        );
    }

    // Check include patterns for duplicates
    let mut include_set = std::collections::HashSet::new();
    for pattern in DEFAULT_INCLUDE_PATTERNS {
        assert!(
            include_set.insert(pattern),
            "Duplicate include pattern: {}",
            pattern
        );
    }
}

#[test]
fn test_patterns_are_glob_format() {
    // All patterns should be valid glob patterns (no regex syntax)
    for pattern in DEFAULT_EXCLUDE_PATTERNS {
        // Should not contain regex-specific characters
        assert!(
            !pattern.contains('\\'),
            "Pattern should not contain backslashes: {}",
            pattern
        );
        assert!(
            !pattern.contains('^'),
            "Pattern should not contain regex anchors: {}",
            pattern
        );
        assert!(
            !pattern.contains('$'),
            "Pattern should not contain regex anchors: {}",
            pattern
        );
        assert!(
            !pattern.contains('('),
            "Pattern should not contain regex groups: {}",
            pattern
        );
        assert!(
            !pattern.contains(')'),
            "Pattern should not contain regex groups: {}",
            pattern
        );
        assert!(
            !pattern.contains('+'),
            "Pattern should not contain regex quantifiers: {}",
            pattern
        );
        assert!(
            !pattern.contains('|'),
            "Pattern should not contain regex alternation: {}",
            pattern
        );
    }

    for pattern in DEFAULT_INCLUDE_PATTERNS {
        // Should not contain regex-specific characters
        assert!(
            !pattern.contains('\\'),
            "Pattern should not contain backslashes: {}",
            pattern
        );
        assert!(
            !pattern.contains('^'),
            "Pattern should not contain regex anchors: {}",
            pattern
        );
        assert!(
            !pattern.contains('$'),
            "Pattern should not contain regex anchors: {}",
            pattern
        );
        assert!(
            !pattern.contains('('),
            "Pattern should not contain regex groups: {}",
            pattern
        );
        assert!(
            !pattern.contains(')'),
            "Pattern should not contain regex groups: {}",
            pattern
        );
        assert!(
            !pattern.contains('+'),
            "Pattern should not contain regex quantifiers: {}",
            pattern
        );
        assert!(
            !pattern.contains('|'),
            "Pattern should not contain regex alternation: {}",
            pattern
        );
    }
}

#[test]
fn test_exclude_include_overlap() {
    let exclude_set: std::collections::HashSet<_> = DEFAULT_EXCLUDE_PATTERNS.iter().collect();
    let include_set: std::collections::HashSet<_> = DEFAULT_INCLUDE_PATTERNS.iter().collect();

    // There shouldn't be exact overlaps between exclude and include patterns
    let overlap: Vec<_> = exclude_set.intersection(&include_set).collect();
    assert!(
        overlap.is_empty(),
        "Found overlapping patterns: {:?}",
        overlap
    );
}

#[test]
fn test_pattern_consistency() {
    // Test that patterns follow consistent conventions

    // Extension patterns should start with *.
    let extension_exclude: Vec<_> = DEFAULT_EXCLUDE_PATTERNS
        .iter()
        .filter(|p| p.starts_with("*."))
        .collect();
    let extension_include: Vec<_> = DEFAULT_INCLUDE_PATTERNS
        .iter()
        .filter(|p| p.starts_with("*."))
        .collect();

    assert!(
        !extension_exclude.is_empty(),
        "Should have extension exclusion patterns"
    );
    assert!(
        !extension_include.is_empty(),
        "Should have extension inclusion patterns"
    );

    // Directory patterns should end with /* or contain /
    let dir_patterns: Vec<_> = DEFAULT_EXCLUDE_PATTERNS
        .iter()
        .filter(|p| p.contains('/'))
        .collect();

    assert!(!dir_patterns.is_empty(), "Should have directory patterns");
}

#[test]
fn test_language_coverage() {
    // Test that major programming languages are covered
    let languages = [
        "rs", "py", "js", "ts", "java", "cpp", "c", "go", "php", "rb", "swift", "kt", "scala",
        "clj", "cs", "fs", "dart", "lua", "pl", "r",
    ];

    for lang in &languages {
        let pattern = format!("*.{}", lang);
        assert!(
            DEFAULT_INCLUDE_PATTERNS.contains(&pattern.as_str()),
            "Missing include pattern for language: {}",
            lang
        );
    }
}

#[test]
fn test_build_tool_coverage() {
    // Test that major build tools and config files are covered
    let build_files = [
        "Cargo.toml",
        "package.json",
        "pom.xml",
        "build.gradle",
        "Makefile",
        "CMakeLists.txt",
        "setup.py",
        "pyproject.toml",
        "Gemfile",
        "go.mod",
    ];

    for file in &build_files {
        assert!(
            DEFAULT_INCLUDE_PATTERNS.contains(file)
                || DEFAULT_INCLUDE_PATTERNS.iter().any(|p| p.contains(file)),
            "Missing include pattern for build file: {}",
            file
        );
    }
}
