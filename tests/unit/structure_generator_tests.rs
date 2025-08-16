use catnip::structure_generator::generate_directory_structure;
use std::path::PathBuf;

#[test]
fn test_generate_directory_structure_simple() {
    let files = vec![
        PathBuf::from("main.rs"),
        PathBuf::from("lib.rs"),
        PathBuf::from("Cargo.toml"),
    ];

    let structure = generate_directory_structure(&files);

    assert!(!structure.is_empty());
    assert!(structure.iter().any(|line| line.contains("main.rs")));
    assert!(structure.iter().any(|line| line.contains("lib.rs")));
    assert!(structure.iter().any(|line| line.contains("Cargo.toml")));
}

#[test]
fn test_generate_directory_structure_nested() {
    let files = vec![
        PathBuf::from("src/main.rs"),
        PathBuf::from("src/lib.rs"),
        PathBuf::from("tests/integration.rs"),
        PathBuf::from("Cargo.toml"),
    ];

    let structure = generate_directory_structure(&files);

    assert!(!structure.is_empty());

    // Should contain directory names
    assert!(structure.iter().any(|line| line.contains("src")));
    assert!(structure.iter().any(|line| line.contains("tests")));

    // Should contain file names
    assert!(structure.iter().any(|line| line.contains("main.rs")));
    assert!(structure.iter().any(|line| line.contains("lib.rs")));
    assert!(structure.iter().any(|line| line.contains("integration.rs")));
    assert!(structure.iter().any(|line| line.contains("Cargo.toml")));

    // Should have proper tree structure with connectors
    assert!(
        structure
            .iter()
            .any(|line| line.contains("├──") || line.contains("└──"))
    );
}

#[test]
fn test_generate_directory_structure_deep_nesting() {
    let files = vec![
        PathBuf::from("src/utils/helper.rs"),
        PathBuf::from("src/main.rs"),
        PathBuf::from("docs/api/README.md"),
        PathBuf::from("tests/unit/mod.rs"),
    ];

    let structure = generate_directory_structure(&files);

    // Should have proper tree structure with indentation
    assert!(
        structure
            .iter()
            .any(|line| line.contains("├──") || line.contains("└──"))
    );
    assert!(
        structure
            .iter()
            .any(|line| line.contains("│") || line.contains("    "))
    );

    // Should contain all directories and files
    assert!(structure.iter().any(|line| line.contains("src")));
    assert!(structure.iter().any(|line| line.contains("utils")));
    assert!(structure.iter().any(|line| line.contains("docs")));
    assert!(structure.iter().any(|line| line.contains("api")));
    assert!(structure.iter().any(|line| line.contains("tests")));
    assert!(structure.iter().any(|line| line.contains("unit")));

    assert!(structure.iter().any(|line| line.contains("helper.rs")));
    assert!(structure.iter().any(|line| line.contains("main.rs")));
    assert!(structure.iter().any(|line| line.contains("README.md")));
    assert!(structure.iter().any(|line| line.contains("mod.rs")));
}

#[test]
fn test_generate_directory_structure_alphabetical_ordering() {
    let files = vec![
        PathBuf::from("z_last.rs"),
        PathBuf::from("a_first.rs"),
        PathBuf::from("m_middle.rs"),
        PathBuf::from("src/z_file.rs"),
        PathBuf::from("src/a_file.rs"),
    ];

    let structure = generate_directory_structure(&files);

    // Find positions of files in the structure
    let structure_string = structure.join("\n");
    let first_pos = structure_string.find("a_first.rs").unwrap();
    let middle_pos = structure_string.find("m_middle.rs").unwrap();
    let last_pos = structure_string.find("z_last.rs").unwrap();

    // Should be in alphabetical order
    assert!(first_pos < middle_pos);
    assert!(middle_pos < last_pos);

    // Same for nested files
    let a_file_pos = structure_string.find("a_file.rs").unwrap();
    let z_file_pos = structure_string.find("z_file.rs").unwrap();
    assert!(a_file_pos < z_file_pos);
}

#[test]
fn test_generate_directory_structure_empty_list() {
    let files: Vec<PathBuf> = vec![];
    let structure = generate_directory_structure(&files);
    assert!(structure.is_empty());
}

#[test]
fn test_generate_directory_structure_single_file() {
    let files = vec![PathBuf::from("main.rs")];
    let structure = generate_directory_structure(&files);

    assert_eq!(structure.len(), 1);
    assert!(structure[0].contains("main.rs"));
    assert!(structure[0].contains("└──")); // Should be the only item
}

#[test]
fn test_generate_directory_structure_mixed_depths() {
    let files = vec![
        PathBuf::from("root.rs"),
        PathBuf::from("src/main.rs"),
        PathBuf::from("src/utils/helper.rs"),
        PathBuf::from("src/utils/deep/nested.rs"),
        PathBuf::from("tests/test.rs"),
    ];

    let structure = generate_directory_structure(&files);

    // Should handle mixed depths properly
    assert!(structure.iter().any(|line| line.contains("root.rs")));
    assert!(structure.iter().any(|line| line.contains("src")));
    assert!(structure.iter().any(|line| line.contains("utils")));
    assert!(structure.iter().any(|line| line.contains("deep")));
    assert!(structure.iter().any(|line| line.contains("tests")));

    // Check that tree structure is valid
    let has_branches = structure.iter().any(|line| line.contains("├──"));
    let has_last_items = structure.iter().any(|line| line.contains("└──"));
    assert!(has_branches || has_last_items); // Should have proper tree connectors
}

#[test]
fn test_generate_directory_structure_duplicate_directories() {
    let files = vec![
        PathBuf::from("src/file1.rs"),
        PathBuf::from("src/file2.rs"),
        PathBuf::from("src/utils/util1.rs"),
        PathBuf::from("src/utils/util2.rs"),
    ];

    let structure = generate_directory_structure(&files);

    // Should only show each directory once
    let src_count = structure.iter().filter(|line| line.contains("src")).count();
    let utils_count = structure
        .iter()
        .filter(|line| line.contains("utils"))
        .count();

    assert_eq!(src_count, 1, "src directory should appear exactly once");
    assert_eq!(utils_count, 1, "utils directory should appear exactly once");

    // But should show both files under each directory
    assert!(structure.iter().any(|line| line.contains("file1.rs")));
    assert!(structure.iter().any(|line| line.contains("file2.rs")));
    assert!(structure.iter().any(|line| line.contains("util1.rs")));
    assert!(structure.iter().any(|line| line.contains("util2.rs")));
}

#[test]
fn test_generate_directory_structure_relative_paths() {
    // Test with paths that might be relative to different working directories
    let files = vec![
        PathBuf::from("./src/main.rs"),
        PathBuf::from("src/../tests/test.rs"),
        PathBuf::from("src/./lib.rs"),
    ];

    let structure = generate_directory_structure(&files);

    // Should normalize and handle relative path components
    assert!(!structure.is_empty());
    assert!(structure.iter().any(|line| line.contains("main.rs")));
    assert!(structure.iter().any(|line| line.contains("test.rs")));
    assert!(structure.iter().any(|line| line.contains("lib.rs")));
}

#[test]
fn test_generate_directory_structure_formatting() {
    let files = vec![
        PathBuf::from("src/main.rs"),
        PathBuf::from("src/lib.rs"),
        PathBuf::from("tests/test.rs"),
    ];

    let structure = generate_directory_structure(&files);

    // Check for proper formatting characters
    let structure_string = structure.join("\n");

    // Should have tree characters
    assert!(structure_string.contains("├──") || structure_string.contains("└──"));

    // Should have proper indentation for nested items
    let has_indented_lines = structure
        .iter()
        .any(|line| line.starts_with("    ") || line.starts_with("│   "));
    assert!(
        has_indented_lines,
        "Should have properly indented nested items"
    );

    // Should not have malformed tree structure
    assert!(!structure_string.contains("├──├──")); // No double connectors
    assert!(!structure_string.contains("└──└──")); // No double connectors
}
