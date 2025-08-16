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
    assert!(structure.iter().any(|line| line.contains("src")));
    assert!(structure.iter().any(|line| line.contains("tests")));
    assert!(structure.iter().any(|line| line.contains("main.rs")));
    assert!(structure.iter().any(|line| line.contains("Cargo.toml")));
}

#[test]
fn test_generate_directory_structure_empty() {
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
}
