use catnip::file_processor::*;
use std::path::Path;
use tempfile::TempDir;
use tokio::fs;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_language_from_extension() {
        assert_eq!(get_language_from_extension(Path::new("main.rs")), "rust");
        assert_eq!(
            get_language_from_extension(Path::new("script.py")),
            "python"
        );
        assert_eq!(
            get_language_from_extension(Path::new("app.js")),
            "javascript"
        );
        assert_eq!(
            get_language_from_extension(Path::new("component.tsx")),
            "jsx"
        );
        assert_eq!(get_language_from_extension(Path::new("style.css")), "css");
        assert_eq!(
            get_language_from_extension(Path::new("config.yaml")),
            "yaml"
        );
        assert_eq!(
            get_language_from_extension(Path::new("Dockerfile")),
            "dockerfile"
        );
        assert_eq!(
            get_language_from_extension(Path::new("Makefile")),
            "makefile"
        );
        assert_eq!(
            get_language_from_extension(Path::new("unknown.xyz")),
            "text"
        );
    }

    #[test]
    fn test_is_binary_file() {
        // Text content
        let text_content = b"Hello, world!\nThis is text.";
        assert!(!is_binary_file(text_content));

        // Binary content with null bytes
        let binary_content = b"Hello\x00\x01\x02World";
        assert!(is_binary_file(binary_content));

        // Empty content
        let empty_content = b"";
        assert!(!is_binary_file(empty_content));

        // Content with high bytes but no nulls (could be UTF-8)
        let utf8_content = "Hello 世界".as_bytes();
        assert!(!is_binary_file(utf8_content));
    }

    #[test]
    fn test_remove_comments_rust() {
        let rust_code = r#"
// This is a line comment
fn main() {
    /* This is a block comment */
    println!("Hello, world!");
    // Another comment
}"#;

        let result = remove_comments_and_docstrings(rust_code, "rust", true, false);

        assert!(!result.contains("// This is a line comment"));
        assert!(!result.contains("/* This is a block comment */"));
        assert!(result.contains("fn main()"));
        assert!(result.contains("println!"));
    }

    #[test]
    fn test_remove_comments_python() {
        let python_code = r#"
# This is a comment
def hello():
    """This is a docstring"""
    print("Hello")  # Inline comment
    '''Another docstring'''
    return True
"#;

        let result_comments = remove_comments_and_docstrings(python_code, "python", true, false);
        assert!(!result_comments.contains("# This is a comment"));
        assert!(!result_comments.contains("# Inline comment"));
        assert!(result_comments.contains("def hello()"));

        let result_docstrings = remove_comments_and_docstrings(python_code, "python", false, true);
        assert!(!result_docstrings.contains("\"\"\"This is a docstring\"\"\""));
        assert!(!result_docstrings.contains("'''Another docstring'''"));
        assert!(result_docstrings.contains("def hello()"));

        let result_both = remove_comments_and_docstrings(python_code, "python", true, true);
        assert!(!result_both.contains("# This is a comment"));
        assert!(!result_both.contains("\"\"\"This is a docstring\"\"\""));
        assert!(result_both.contains("def hello()"));
    }

    #[test]
    fn test_remove_comments_disabled() {
        let code = r#"
// This comment should remain
fn test() {
    /* Block comment */
    println!("test");
}
"#;

        let result = remove_comments_and_docstrings(code, "rust", false, false);
        assert_eq!(result, code); // Should be unchanged
    }

    #[tokio::test]
    async fn test_get_files_recursively_single_file() {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path();

        let test_file = temp_path.join("test.rs");
        fs::write(&test_file, "fn main() {}").await.unwrap();

        let files = get_files_recursively(&[test_file.clone()], &[], &[], false, false, 10)
            .await
            .unwrap();

        assert_eq!(files.len(), 1);
        assert_eq!(files[0], test_file);
    }

    #[tokio::test]
    async fn test_get_files_recursively_directory() {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path();

        // Create test files
        fs::create_dir_all(temp_path.join("src")).await.unwrap();
        fs::write(temp_path.join("src/main.rs"), "fn main() {}")
            .await
            .unwrap();
        fs::write(temp_path.join("src/lib.rs"), "pub fn test() {}")
            .await
            .unwrap();
        fs::write(temp_path.join("README.md"), "# Test")
            .await
            .unwrap();
        fs::write(temp_path.join("Cargo.toml"), "[package]")
            .await
            .unwrap();

        // Create excluded directory
        fs::create_dir_all(temp_path.join("target")).await.unwrap();
        fs::write(temp_path.join("target/debug"), "binary")
            .await
            .unwrap();

        let files = get_files_recursively(&[temp_path.to_path_buf()], &[], &[], false, false, 10)
            .await
            .unwrap();

        // Should find Rust, Markdown, and TOML files but exclude target
        let file_names: Vec<String> = files
            .iter()
            .map(|p| p.file_name().unwrap().to_string_lossy().to_string())
            .collect();

        assert!(file_names.contains(&"main.rs".to_string()));
        assert!(file_names.contains(&"lib.rs".to_string()));
        assert!(file_names.contains(&"README.md".to_string()));
        assert!(file_names.contains(&"Cargo.toml".to_string()));
        assert!(!file_names.contains(&"debug".to_string())); // Should be excluded
    }

    #[tokio::test]
    async fn test_get_files_recursively_with_exclusions() {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path();

        fs::write(temp_path.join("main.rs"), "fn main() {}")
            .await
            .unwrap();
        fs::write(temp_path.join("test.log"), "log data")
            .await
            .unwrap();
        fs::write(temp_path.join("data.json"), "{}").await.unwrap();

        let files = get_files_recursively(
            &[temp_path.to_path_buf()],
            &["*.log".to_string(), "*.json".to_string()],
            &[],
            false,
            false,
            10,
        )
        .await
        .unwrap();

        let file_names: Vec<String> = files
            .iter()
            .map(|p| p.file_name().unwrap().to_string_lossy().to_string())
            .collect();

        assert!(file_names.contains(&"main.rs".to_string()));
        assert!(!file_names.contains(&"test.log".to_string()));
        assert!(!file_names.contains(&"data.json".to_string()));
    }

    #[tokio::test]
    async fn test_get_files_recursively_with_inclusions() {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path();

        fs::write(temp_path.join("main.rs"), "fn main() {}")
            .await
            .unwrap();
        fs::write(temp_path.join("script.py"), "print('hello')")
            .await
            .unwrap();
        fs::write(temp_path.join("data.txt"), "text data")
            .await
            .unwrap();

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

        let file_names: Vec<String> = files
            .iter()
            .map(|p| p.file_name().unwrap().to_string_lossy().to_string())
            .collect();

        assert!(file_names.contains(&"main.rs".to_string()));
        assert!(!file_names.contains(&"script.py".to_string()));
        assert!(!file_names.contains(&"data.txt".to_string()));
    }

    #[tokio::test]
    async fn test_get_files_recursively_size_limit() {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path();

        // Small file
        fs::write(temp_path.join("small.rs"), "fn main() {}")
            .await
            .unwrap();

        // Large file (2MB)
        let large_content = "x".repeat(2 * 1024 * 1024);
        fs::write(temp_path.join("large.rs"), &large_content)
            .await
            .unwrap();

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

        let file_names: Vec<String> = files
            .iter()
            .map(|p| p.file_name().unwrap().to_string_lossy().to_string())
            .collect();

        assert!(file_names.contains(&"small.rs".to_string()));
        assert!(!file_names.contains(&"large.rs".to_string()));
    }

    #[tokio::test]
    async fn test_get_files_recursively_binary_exclusion() {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path();

        // Text file
        fs::write(temp_path.join("text.rs"), "fn main() {}")
            .await
            .unwrap();

        // Binary file (with null bytes)
        fs::write(temp_path.join("binary.dat"), &[0u8, 1u8, 2u8, 0u8])
            .await
            .unwrap();

        let files = get_files_recursively(
            &[temp_path.to_path_buf()],
            &[],
            &["*.rs".to_string(), "*.dat".to_string()], // Include both extensions
            false,
            false,
            10,
        )
        .await
        .unwrap();

        let file_names: Vec<String> = files
            .iter()
            .map(|p| p.file_name().unwrap().to_string_lossy().to_string())
            .collect();

        assert!(file_names.contains(&"text.rs".to_string()));
        assert!(!file_names.contains(&"binary.dat".to_string())); // Should be excluded as binary
    }

    #[tokio::test]
    async fn test_concatenate_files() {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path();

        let file1 = temp_path.join("main.rs");
        let file2 = temp_path.join("lib.rs");

        fs::write(&file1, "fn main() {\n    println!(\"Hello\");\n}")
            .await
            .unwrap();
        fs::write(&file2, "pub fn helper() {\n    // Helper function\n}")
            .await
            .unwrap();

        let files = vec![file1, file2];
        let result = concatenate_files(&files, None, false, false).await.unwrap();

        // Should contain project structure
        assert!(result.contains("# Project Structure"));
        assert!(result.contains("main.rs"));
        assert!(result.contains("lib.rs"));

        // Should contain file contents
        assert!(result.contains("# File Contents"));
        assert!(result.contains("fn main()"));
        assert!(result.contains("pub fn helper()"));
        assert!(result.contains("println!"));

        // Should have proper code formatting
        assert!(result.contains("```rust"));
    }

    #[tokio::test]
    async fn test_concatenate_files_with_output() {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path();

        let source_file = temp_path.join("main.rs");
        let output_file = temp_path.join("output.md");

        fs::write(&source_file, "fn main() {}").await.unwrap();

        let files = vec![source_file];
        concatenate_files(&files, Some(output_file.to_str().unwrap()), false, false)
            .await
            .unwrap();

        // Output file should exist and contain the content
        assert!(output_file.exists());
        let content = fs::read_to_string(&output_file).await.unwrap();
        assert!(content.contains("# Project Structure"));
        assert!(content.contains("fn main()"));
    }

    #[tokio::test]
    async fn test_concatenate_files_with_comment_removal() {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path();

        let rust_file = temp_path.join("main.rs");
        fs::write(
            &rust_file,
            "// Comment\nfn main() {\n    /* Block */\n    println!(\"test\");\n}",
        )
        .await
        .unwrap();

        let files = vec![rust_file];
        let result = concatenate_files(&files, None, true, false).await.unwrap();

        assert!(!result.contains("// Comment"));
        assert!(!result.contains("/* Block */"));
        assert!(result.contains("fn main()"));
        assert!(result.contains("println!"));
    }

    #[tokio::test]
    async fn test_concatenate_files_unreadable_file() {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path();

        let good_file = temp_path.join("good.rs");
        let bad_file = temp_path.join("nonexistent.rs");

        fs::write(&good_file, "fn main() {}").await.unwrap();
        // bad_file doesn't exist

        let files = vec![good_file, bad_file];
        let result = concatenate_files(&files, None, false, false).await.unwrap();

        // Should contain good file
        assert!(result.contains("fn main()"));

        // Should contain error message for bad file
        assert!(result.contains("Error reading file"));
    }
}
