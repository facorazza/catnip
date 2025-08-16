use std::fs;
use std::path::PathBuf;
use std::process::Command;
use tempfile::TempDir;

fn get_test_binary() -> PathBuf {
    // Get the path to the compiled test binary
    let mut path = std::env::current_exe().unwrap();
    path.pop(); // Remove test executable name
    path.pop(); // Remove 'deps' directory
    path.push("catnip");

    // Add .exe extension on Windows
    if cfg!(target_os = "windows") {
        path.set_extension("exe");
    }

    path
}

#[test]
fn test_cli_help() {
    let binary = get_test_binary();

    let output = Command::new(&binary)
        .arg("--help")
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Concatenate and patch codebases"));
    assert!(stdout.contains("cat"));
    assert!(stdout.contains("patch"));
}

#[test]
fn test_cli_version() {
    let binary = get_test_binary();

    let output = Command::new(&binary)
        .arg("--version")
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
}

#[test]
fn test_cli_no_args() {
    let binary = get_test_binary();

    let output = Command::new(&binary)
        .output()
        .expect("Failed to execute command");

    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("error") || stderr.contains("required"));
}

#[test]
fn test_cat_command_with_file() {
    let binary = get_test_binary();
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();
    let test_file = temp_path.join("test.rs");

    fs::write(
        &test_file,
        "fn main() {\n    println!(\"Hello, world!\");\n}",
    )
    .unwrap();

    let output = Command::new(&binary)
        .args(["cat", test_file.to_str().unwrap(), "--no-copy"])
        .output()
        .expect("Failed to execute command");

    assert!(
        output.status.success(),
        "Command failed with stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("# Project Structure"));
    assert!(stdout.contains("# File Contents"));
    assert!(stdout.contains("test.rs"));
    assert!(stdout.contains("fn main()"));
    assert!(stdout.contains("println!"));
}

#[test]
fn test_cat_command_with_directory() {
    let binary = get_test_binary();
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();

    // Create directory structure
    fs::create_dir_all(temp_path.join("src")).unwrap();
    fs::write(temp_path.join("src").join("main.rs"), "fn main() {}").unwrap();
    fs::write(temp_path.join("src").join("lib.rs"), "pub fn hello() {}").unwrap();
    fs::write(temp_path.join("Cargo.toml"), "[package]\nname = \"test\"").unwrap();

    let output = Command::new(&binary)
        .args(["cat", temp_path.to_str().unwrap(), "--no-copy"])
        .output()
        .expect("Failed to execute command");

    assert!(
        output.status.success(),
        "Command failed with stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("# Project Structure"));
    assert!(stdout.contains("src"));
    assert!(stdout.contains("main.rs"));
    assert!(stdout.contains("lib.rs"));
    assert!(stdout.contains("Cargo.toml"));
    assert!(stdout.contains("fn main()"));
    assert!(stdout.contains("pub fn hello()"));
}

#[test]
fn test_cat_command_with_output_file() {
    let binary = get_test_binary();
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();
    let test_file = temp_path.join("test.rs");
    let output_file = temp_path.join("output.md");

    fs::write(&test_file, "fn main() {}").unwrap();

    let output = Command::new(&binary)
        .args([
            "cat",
            test_file.to_str().unwrap(),
            "--output",
            output_file.to_str().unwrap(),
        ])
        .output()
        .expect("Failed to execute command");

    assert!(
        output.status.success(),
        "Command failed with stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output_file.exists());

    let file_content = fs::read_to_string(&output_file).unwrap();
    assert!(file_content.contains("# Project Structure"));
    assert!(file_content.contains("test.rs"));
    assert!(file_content.contains("fn main()"));
}

#[test]
fn test_cat_command_with_exclusions() {
    let binary = get_test_binary();
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();

    // Create files
    fs::write(temp_path.join("main.rs"), "fn main() {}").unwrap();
    fs::write(temp_path.join("test.log"), "log data").unwrap();
    fs::write(temp_path.join("data.json"), "{}").unwrap();

    let output = Command::new(&binary)
        .args([
            "cat",
            temp_path.to_str().unwrap(),
            "--exclude",
            "*.log",
            "--exclude",
            "*.json",
            "--no-copy",
        ])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("main.rs"));
    assert!(!stdout.contains("test.log"));
    assert!(!stdout.contains("data.json"));
}

#[test]
fn test_cat_command_with_inclusions() {
    let binary = get_test_binary();
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();

    // Create files
    fs::write(temp_path.join("main.rs"), "fn main() {}").unwrap();
    fs::write(temp_path.join("script.py"), "print('hello')").unwrap();
    fs::write(temp_path.join("data.txt"), "text data").unwrap();

    let output = Command::new(&binary)
        .args([
            "cat",
            temp_path.to_str().unwrap(),
            "--include",
            "*.rs",
            "--no-copy",
        ])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("main.rs"));
    assert!(!stdout.contains("script.py"));
    assert!(!stdout.contains("data.txt"));
}

#[test]
fn test_cat_command_with_size_limit() {
    let binary = get_test_binary();
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();

    // Create small and large files
    fs::write(temp_path.join("small.rs"), "fn main() {}").unwrap();
    fs::write(temp_path.join("large.rs"), "x".repeat(2 * 1024 * 1024)).unwrap(); // 2MB

    let output = Command::new(&binary)
        .args([
            "cat",
            temp_path.to_str().unwrap(),
            "--max-size-mb",
            "1", // 1MB limit
            "--no-copy",
        ])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("small.rs"));
    assert!(!stdout.contains("large.rs")); // Should be filtered out
}

#[test]
fn test_cat_nonexistent_file() {
    let binary = get_test_binary();

    let output = Command::new(&binary)
        .args(["cat", "/nonexistent/file.rs", "--no-copy"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success()); // Should succeed but find no files

    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("# Project Structure")); // Header should still be there
}

#[test]
fn test_patch_command_help() {
    let binary = get_test_binary();

    let output = Command::new(&binary)
        .args(["patch", "--help"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Apply JSON-formatted code updates"));
    assert!(stdout.contains("--dry-run"));
    assert!(stdout.contains("--backup"));
}

#[test]
fn test_patch_command_dry_run() {
    let binary = get_test_binary();
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();

    // Create test file
    let test_file = temp_path.join("test.rs");
    fs::write(&test_file, "fn old_function() {\n    println!(\"old\");\n}").unwrap();

    // Create patch JSON
    let patch_json = serde_json::json!({
        "analysis": "Replace old function with new function",
        "files": [{
            "path": test_file.to_str().unwrap(),
            "updates": [{
                "line_start": 1,
                "line_end": 3,
                "old_content": "fn old_function() {\n    println!(\"old\");\n}",
                "new_content": "fn new_function() {\n    println!(\"new\");\n}"
            }]
        }]
    });

    let patch_file = temp_path.join("patch.json");
    fs::write(&patch_file, patch_json.to_string()).unwrap();

    let output = Command::new(&binary)
        .args(["patch", patch_file.to_str().unwrap(), "--dry-run"])
        .output()
        .expect("Failed to execute command");

    assert!(
        output.status.success(),
        "Command failed with stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // File should not be modified in dry run
    let file_content = fs::read_to_string(&test_file).unwrap();
    assert!(file_content.contains("old_function"));
    assert!(!file_content.contains("new_function"));

    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("DRY RUN"));
}

#[test]
fn test_patch_command_invalid_json() {
    let binary = get_test_binary();
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();

    // Create invalid JSON
    let patch_file = temp_path.join("invalid.json");
    fs::write(&patch_file, "{ invalid json }").unwrap();

    let output = Command::new(&binary)
        .args(["patch", patch_file.to_str().unwrap()])
        .output()
        .expect("Failed to execute command");

    assert!(!output.status.success());

    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("Failed to parse JSON") || stderr.contains("error"));
}

#[test]
fn test_mixed_file_and_directory_processing() {
    let binary = get_test_binary();
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();

    // Create individual file
    let individual_file = temp_path.join("individual.rs");
    fs::write(&individual_file, "fn individual() {}").unwrap();

    // Create directory with files
    fs::create_dir_all(temp_path.join("src")).unwrap();
    fs::write(temp_path.join("src").join("main.rs"), "fn main() {}").unwrap();
    fs::write(temp_path.join("src").join("lib.rs"), "pub fn lib() {}").unwrap();

    let output = Command::new(&binary)
        .args([
            "cat",
            individual_file.to_str().unwrap(),
            temp_path.join("src").to_str().unwrap(),
            "--no-copy",
        ])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("individual.rs"));
    assert!(stdout.contains("main.rs"));
    assert!(stdout.contains("lib.rs"));
    assert!(stdout.contains("fn individual()"));
    assert!(stdout.contains("fn main()"));
    assert!(stdout.contains("pub fn lib()"));
}

#[test]
fn test_error_handling_no_permissions() {
    // This test is conditional and may not work in all environments
    if !cfg!(unix) {
        return; // Skip on non-Unix systems
    }

    let binary = get_test_binary();
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();

    // Create a directory we can't read (this might not work in all test environments)
    let restricted_dir = temp_path.join("restricted");
    fs::create_dir_all(&restricted_dir).unwrap();
    fs::write(restricted_dir.join("file.rs"), "fn test() {}").unwrap();

    // Try to remove read permissions
    let _ = Command::new("chmod")
        .args(["000", restricted_dir.to_str().unwrap()])
        .output();

    let output = Command::new(&binary)
        .args(["cat", temp_path.to_str().unwrap(), "--no-copy"])
        .output()
        .expect("Failed to execute command");

    // Should succeed even if some files can't be read
    assert!(output.status.success());
}

#[test]
fn test_large_number_of_files() {
    let binary = get_test_binary();
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();

    // Create many small files
    for i in 0..50 {
        fs::write(
            temp_path.join(format!("file_{:03}.rs", i)),
            format!("fn function_{}() {{}}", i),
        )
        .unwrap();
    }

    let output = Command::new(&binary)
        .args(["cat", temp_path.to_str().unwrap(), "--no-copy"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).unwrap();

    // Should contain some of the files
    assert!(stdout.contains("file_000.rs"));
    assert!(stdout.contains("file_049.rs"));
    assert!(stdout.contains("function_0"));
    assert!(stdout.contains("function_49"));
}

#[test]
fn test_special_characters_in_filenames() {
    let binary = get_test_binary();
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();

    // Create files with special characters (where supported by filesystem)
    let normal_file = temp_path.join("normal.rs");
    let underscore_file = temp_path.join("with_underscore.rs");
    let dash_file = temp_path.join("with-dash.rs");

    fs::write(&normal_file, "fn normal() {}").unwrap();
    fs::write(&underscore_file, "fn with_underscore() {}").unwrap();
    fs::write(&dash_file, "fn with_dash() {}").unwrap();

    let output = Command::new(&binary)
        .args(["cat", temp_path.to_str().unwrap(), "--no-copy"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("normal.rs"));
    assert!(stdout.contains("with_underscore.rs"));
    assert!(stdout.contains("with-dash.rs"));
}
