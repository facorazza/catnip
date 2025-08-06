use catnip::exclusion_patterns::{DEFAULT_EXCLUDE_PATTERNS, DEFAULT_INCLUDE_PATTERNS};
use catnip::file_processor::{concatenate_files, get_files_recursively};
use catnip::pattern_matcher::{advanced_pattern_match, matches_any_pattern};
use catnip::structure_generator::generate_directory_structure;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use tempfile::TempDir;

#[tokio::test]
async fn test_get_files_recursively_basic() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();

    // Create test files
    fs::write(temp_path.join("test.rs"), "fn main() {}").unwrap();
    fs::write(temp_path.join("test.py"), "print('hello')").unwrap();
    fs::write(temp_path.join("README.md"), "# Test").unwrap();

    let files = get_files_recursively(&[temp_path.to_path_buf()], &[], &[], false, false, 10)
        .await
        .unwrap();

    assert_eq!(files.len(), 3);
    assert!(files.iter().any(|f| f.file_name().unwrap() == "test.rs"));
    assert!(files.iter().any(|f| f.file_name().unwrap() == "test.py"));
    assert!(files.iter().any(|f| f.file_name().unwrap() == "README.md"));
}

#[tokio::test]
async fn test_get_files_recursively_with_exclusions() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();

    // Create test files
    fs::write(temp_path.join("test.rs"), "fn main() {}").unwrap();
    fs::write(temp_path.join("test.log"), "log data").unwrap();
    fs::create_dir_all(temp_path.join("target")).unwrap();
    fs::write(temp_path.join("target").join("binary"), "binary").unwrap();

    let files = get_files_recursively(
        &[temp_path.to_path_buf()],
        &["*.log".to_string(), "target/*".to_string()],
        &[],
        false,
        false,
        10,
    )
    .await
    .unwrap();

    assert_eq!(files.len(), 1);
    assert!(files.iter().any(|f| f.file_name().unwrap() == "test.rs"));
    assert!(!files.iter().any(|f| f.file_name().unwrap() == "test.log"));
}

#[tokio::test]
async fn test_get_files_recursively_with_inclusions() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();

    // Create test files
    fs::write(temp_path.join("test.rs"), "fn main() {}").unwrap();
    fs::write(temp_path.join("test.py"), "print('hello')").unwrap();
    fs::write(temp_path.join("test.txt"), "text file").unwrap();

    let files = get_files_recursively(
        &[temp_path.to_path_buf()],
        &[],
        &["*.rs".to_string()],
        false,
        false,
        10,
    )
    .await
    .unwrap();

    assert_eq!(files.len(), 1);
    assert!(files.iter().any(|f| f.file_name().unwrap() == "test.rs"));
    assert!(!files.iter().any(|f| f.file_name().unwrap() == "test.py"));
}

#[tokio::test]
async fn test_get_files_recursively_nested_directories() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();

    // Create nested structure
    fs::create_dir_all(temp_path.join("src").join("utils")).unwrap();
    fs::write(temp_path.join("src").join("main.rs"), "fn main() {}").unwrap();
    fs::write(
        temp_path.join("src").join("utils").join("helper.rs"),
        "fn helper() {}",
    )
    .unwrap();

    let files = get_files_recursively(&[temp_path.to_path_buf()], &[], &[], false, false, 10)
        .await
        .unwrap();

    assert_eq!(files.len(), 2);
    assert!(files.iter().any(|f| f.file_name().unwrap() == "main.rs"));
    assert!(files.iter().any(|f| f.file_name().unwrap() == "helper.rs"));
}

#[tokio::test]
async fn test_get_files_recursively_size_limit() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();

    // Create small and large files
    fs::write(temp_path.join("small.rs"), "fn main() {}").unwrap();
    fs::write(temp_path.join("large.rs"), "x".repeat(2 * 1024 * 1024)).unwrap(); // 2MB

    let files = get_files_recursively(
        &[temp_path.to_path_buf()],
        &[],
        &[],
        false,
        false,
        1, // 1MB limit
    )
    .await
    .unwrap();

    assert_eq!(files.len(), 1);
    assert!(files.iter().any(|f| f.file_name().unwrap() == "small.rs"));
    assert!(!files.iter().any(|f| f.file_name().unwrap() == "large.rs"));
}

#[tokio::test]
async fn test_concatenate_files() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();

    // Create test files
    fs::write(temp_path.join("test1.rs"), "fn test1() {}").unwrap();
    fs::write(temp_path.join("test2.py"), "def test2(): pass").unwrap();

    let files = vec![temp_path.join("test1.rs"), temp_path.join("test2.py")];

    let result = concatenate_files(&files, None).await.unwrap();

    assert!(result.contains("# Project Structure"));
    assert!(result.contains("# File Contents"));
    assert!(result.contains("test1.rs"));
    assert!(result.contains("test2.py"));
    assert!(result.contains("fn test1() {}"));
    assert!(result.contains("def test2(): pass"));
    assert!(result.contains("```rust"));
    assert!(result.contains("```python"));
}

#[tokio::test]
async fn test_concatenate_files_with_output() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();

    // Create test file
    fs::write(temp_path.join("test.rs"), "fn main() {}").unwrap();

    let files = vec![temp_path.join("test.rs")];
    let output_file = temp_path.join("output.md");

    let result = concatenate_files(&files, Some(output_file.to_str().unwrap()))
        .await
        .unwrap();

    assert!(output_file.exists());
    let file_content = fs::read_to_string(&output_file).unwrap();
    assert_eq!(result, file_content);
    assert!(file_content.contains("fn main() {}"));
}

#[test]
fn test_advanced_pattern_match() {
    assert!(advanced_pattern_match("test.rs", "*.rs"));
    assert!(advanced_pattern_match("src/main.rs", "src/*.rs"));
    assert!(advanced_pattern_match("test.py", "test.?y"));
    assert!(!advanced_pattern_match("test.rs", "*.py"));
    assert!(!advanced_pattern_match("test.rs", "src/*.rs"));
}

#[test]
fn test_matches_any_pattern() {
    let path = PathBuf::from("src/main.rs");
    let patterns = vec!["*.rs".to_string(), "*.py".to_string()];

    assert!(matches_any_pattern(&path, &patterns));

    let patterns = vec!["*.py".to_string(), "*.js".to_string()];
    assert!(!matches_any_pattern(&path, &patterns));
}

#[test]
fn test_matches_any_pattern_directory() {
    let path = PathBuf::from("target/debug/main");
    let patterns = vec!["target/*".to_string()];

    assert!(matches_any_pattern(&path, &patterns));
}

#[test]
fn test_generate_directory_structure() {
    let files = vec![
        PathBuf::from("src/main.rs"),
        PathBuf::from("src/lib.rs"),
        PathBuf::from("tests/integration.rs"),
        PathBuf::from("Cargo.toml"),
    ];

    let structure = generate_directory_structure(&files);

    assert!(!structure.is_empty());
    assert!(structure.iter().any(|line| line.contains("src")));
    assert!(structure.iter().any(|line| line.contains("main.rs")));
    assert!(structure.iter().any(|line| line.contains("lib.rs")));
    assert!(structure.iter().any(|line| line.contains("tests")));
    assert!(structure.iter().any(|line| line.contains("Cargo.toml")));
}

#[test]
fn test_generate_directory_structure_nested() {
    let files = vec![
        PathBuf::from("src/utils/helper.rs"),
        PathBuf::from("src/main.rs"),
        PathBuf::from("docs/README.md"),
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
}

#[test]
fn test_default_exclude_patterns() {
    assert!(!DEFAULT_EXCLUDE_PATTERNS.is_empty());
    assert!(DEFAULT_EXCLUDE_PATTERNS.contains(&"*.pyc"));
    assert!(DEFAULT_EXCLUDE_PATTERNS.contains(&".git/*"));
    assert!(DEFAULT_EXCLUDE_PATTERNS.contains(&"*/node_modules*"));
    assert!(DEFAULT_EXCLUDE_PATTERNS.contains(&"*.log"));
}

#[test]
fn test_default_include_patterns() {
    assert!(!DEFAULT_INCLUDE_PATTERNS.is_empty());
    assert!(DEFAULT_INCLUDE_PATTERNS.contains(&"*.rs"));
    assert!(DEFAULT_INCLUDE_PATTERNS.contains(&"*.py"));
    assert!(DEFAULT_INCLUDE_PATTERNS.contains(&"*.js"));
    assert!(DEFAULT_INCLUDE_PATTERNS.contains(&"Cargo.toml"));
}

#[tokio::test]
async fn test_binary_file_detection() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();

    // Create a binary file (with null bytes)
    let binary_content = vec![0u8, 1u8, 2u8, 0u8, 255u8];
    fs::write(temp_path.join("binary.bin"), binary_content).unwrap();

    // Create a text file
    fs::write(temp_path.join("text.txt"), "This is text content").unwrap();

    let files = get_files_recursively(
        &[temp_path.to_path_buf()],
        &[],
        &["*".to_string()], // Include all files
        false,
        false,
        10,
    )
    .await
    .unwrap();

    // Should only include the text file
    assert_eq!(files.len(), 1);
    assert!(files.iter().any(|f| f.file_name().unwrap() == "text.txt"));
    assert!(!files.iter().any(|f| f.file_name().unwrap() == "binary.bin"));
}

#[tokio::test]
async fn test_empty_file_handling() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();

    // Create empty file
    fs::write(temp_path.join("empty.rs"), "").unwrap();
    // Create non-empty file
    fs::write(temp_path.join("nonempty.rs"), "fn main() {}").unwrap();

    let files = get_files_recursively(&[temp_path.to_path_buf()], &[], &[], false, false, 10)
        .await
        .unwrap();

    // Should only include non-empty file
    assert_eq!(files.len(), 1);
    assert!(
        files
            .iter()
            .any(|f| f.file_name().unwrap() == "nonempty.rs")
    );
    assert!(!files.iter().any(|f| f.file_name().unwrap() == "empty.rs"));
}

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[test]
    fn test_cli_help() {
        let output = Command::new("cargo")
            .args(["run", "--", "--help"])
            .output()
            .expect("Failed to execute command");

        assert!(output.status.success());
        let stdout = String::from_utf8(output.stdout).unwrap();
        assert!(stdout.contains("Concatenate files with directory structure"));
    }

    #[test]
    fn test_cli_no_args() {
        let output = Command::new("cargo")
            .args(["run"])
            .output()
            .expect("Failed to execute command");

        assert!(!output.status.success());
    }

    #[test]
    fn test_cli_with_file() {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path();
        let test_file = temp_path.join("test.rs");
        fs::write(&test_file, "fn main() {}").unwrap();

        let output = Command::new("cargo")
            .args(["run", "--", test_file.to_str().unwrap(), "--no-copy"])
            .output()
            .expect("Failed to execute command");

        assert!(output.status.success());
    }
}

#[cfg(test)]
mod error_handling_tests {
    use super::*;

    #[tokio::test]
    async fn test_nonexistent_path() {
        let nonexistent = PathBuf::from("/nonexistent/path");

        let files = get_files_recursively(&[nonexistent], &[], &[], false, false, 10)
            .await
            .unwrap();

        assert!(files.is_empty());
    }

    #[tokio::test]
    async fn test_permission_denied() {
        // This test may not work on all systems, so we'll make it conditional
        if cfg!(unix) {
            let temp_dir = TempDir::new().unwrap();
            let temp_path = temp_dir.path();
            let restricted_dir = temp_path.join("restricted");

            fs::create_dir(&restricted_dir).unwrap();
            fs::write(restricted_dir.join("file.rs"), "fn main() {}").unwrap();

            // Try to remove read permissions (may not work in all test environments)
            let _ = Command::new("chmod")
                .args(["000", restricted_dir.to_str().unwrap()])
                .output();

            let files =
                get_files_recursively(&[temp_path.to_path_buf()], &[], &[], false, false, 10)
                    .await
                    .unwrap();

            // Should handle permission errors gracefully
            assert!(files.len() <= 1); // May or may not include the restricted file
        }
    }
}
