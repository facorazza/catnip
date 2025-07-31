use crate::exclusion_patterns::{DEFAULT_EXCLUDE_PATTERNS, DEFAULT_INCLUDE_PATTERNS};
use crate::pattern_matcher::matches_any_pattern;
use crate::structure_generator::generate_directory_structure;
use anyhow::{Context, Result};
use regex::Regex;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

pub fn get_files_recursively(
    paths: &[PathBuf],
    exclude_patterns: &[String],
    include_patterns: &[String],
    ignore_comments: bool,
    ignore_docstrings: bool,
) -> Result<Vec<PathBuf>> {
    let mut all_exclude = DEFAULT_EXCLUDE_PATTERNS.iter().map(|s| s.to_string()).collect::<Vec<_>>();
    all_exclude.extend_from_slice(exclude_patterns);

    let mut all_include = DEFAULT_INCLUDE_PATTERNS.iter().map(|s| s.to_string()).collect::<Vec<_>>();
    all_include.extend_from_slice(include_patterns);

    let mut found_files = Vec::new();

    for path in paths {
        if path.is_file() {
            if is_valid_file(path, &all_exclude, &all_include, ignore_comments, ignore_docstrings)? {
                found_files.push(path.clone());
            }
        } else if path.is_dir() {
            collect_files_recursive(path, &all_exclude, &all_include, ignore_comments, ignore_docstrings, &mut found_files)?;
        }
    }

    Ok(found_files)
}

fn collect_files_recursive(
    dir: &Path,
    exclude_patterns: &[String],
    include_patterns: &[String],
    ignore_comments: bool,
    ignore_docstrings: bool,
    found_files: &mut Vec<PathBuf>,
) -> Result<()> {
    for entry in WalkDir::new(dir).into_iter() {
        let entry = entry.context("Failed to read directory entry")?;
        let path = entry.path();

        // Skip directories that match exclude patterns
        if path.is_dir() && matches_any_pattern(path, exclude_patterns) {
            continue;
        }

        if path.is_file() && is_valid_file(path, exclude_patterns, include_patterns, ignore_comments, ignore_docstrings)? {
            found_files.push(path.to_path_buf());
        }
    }

    Ok(())
}

fn is_valid_file(
    file_path: &Path,
    exclude_patterns: &[String],
    include_patterns: &[String],
    ignore_comments: bool,
    ignore_docstrings: bool,
) -> Result<bool> {
    // Check if file is empty
    let metadata = fs::metadata(file_path).context("Failed to read file metadata")?;
    if metadata.len() == 0 {
        return Ok(false);
    }

    // Check include patterns first
    if !matches_any_pattern(file_path, include_patterns) {
        return Ok(false);
    }

    // Check exclude patterns
    if matches_any_pattern(file_path, exclude_patterns) {
        return Ok(false);
    }

    // Check content if needed
    if ignore_comments || ignore_docstrings {
        let content = fs::read_to_string(file_path).context("Failed to read file content")?;
        let processed_content = process_content(file_path, &content, ignore_comments, ignore_docstrings)?;

        if processed_content.trim().is_empty() {
            return Ok(false);
        }
    }

    Ok(true)
}

fn process_content(
    file_path: &Path,
    content: &str,
    ignore_comments: bool,
    ignore_docstrings: bool,
) -> Result<String> {
    let mut processed = content.to_string();

    if ignore_comments {
        processed = remove_comments(file_path, &processed)?;
    }

    if ignore_docstrings {
        processed = remove_docstrings(&processed)?;
    }

    Ok(processed)
}

fn remove_comments(file_path: &Path, content: &str) -> Result<String> {
    let ext = file_path.extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    match ext.as_str() {
        "py" | "pyw" => {
            let re = Regex::new(r"(#.*?$)|('''.*?'''|"".*?"")")?;
            Ok(re.replace_all(content, "").to_string())
        },
        "js" | "java" | "c" | "cpp" | "cs" | "swift" | "rs" => {
            let re = Regex::new(r"(//.*?$)|(/\*.*?\*/)")?;
            Ok(re.replace_all(content, "").to_string())
        },
        "html" | "xml" => {
            let re = Regex::new(r"<!--.*?-->")?;
            Ok(re.replace_all(content, "").to_string())
        },
        _ => Ok(content.to_string()),
    }
}

fn remove_docstrings(content: &str) -> Result<String> {
    let re = Regex::new(r#"('''.*?'''|""".*?""")"#)?;
    Ok(re.replace_all(content, "").to_string())
}

pub fn concatenate_files(files: &[PathBuf], output_file: Option<&str>) -> Result<String> {
    let mut output = String::new();

    // Add project structure
    output.push_str("# Project Structure\n\n");
    output.push_str("├── ./\n");

    let directory_structure = generate_directory_structure(files);
    for line in directory_structure {
        output.push_str(&line);
        output.push('\n');
    }

    output.push_str("\n# File Contents\n\n");

    // Add file contents
    let mut sorted_files = files.to_vec();
    sorted_files.sort();

    for file in sorted_files {
        let current_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        let relative_path = file.strip_prefix(&current_dir).unwrap_or(&file);

        let file_type = get_file_type(&file);

        output.push_str(&format!("## {}\n\n", relative_path.display()));
        output.push_str(&format!("```{}\n", file_type));

        match fs::read_to_string(&file) {
            Ok(content) => {
                output.push_str(&content.trim_end_matches('\n'));
            }
            Err(e) => {
                log::error!("Error reading file {}: {}", file.display(), e);
            }
        }

        output.push_str("\n```\n\n");
    }

    // Write to output file if specified
    if let Some(output_path) = output_file {
        fs::write(output_path, &output).context("Failed to write output file")?;
        println!("Successfully created output file: {}", output_path);
    }

    Ok(output)
}

fn get_file_type(file: &Path) -> &'static str {
    let ext = file.extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    match ext.as_str() {
        "py" => "python",
        "js" => "javascript",
        "html" => "html",
        "css" => "css",
        "json" => "json",
        "log" => "",
        "txt" => "",
        "md" => "markdown",
        "xml" => "xml",
        "yaml" | "yml" => "yaml",
        "sh" => "shell",
        "c" => "c",
        "cpp" => "cpp",
        "java" => "java",
        "php" => "php",
        "rb" => "ruby",
        "go" => "go",
        "swift" => "swift",
        "rs" => "rust",
        "pl" => "perl",
        "ps1" => "powershell",
        "bat" => "batch",
        "vbs" => "vbscript",
        "ini" => "ini",
        "toml" => "toml",
        "csv" => "csv",
        "tsv" => "tsv",
        "rst" => "rst",
        "tex" => "tex",
        "org" => "org",
        "jsx" => "jsx",
        "tsx" => "tsx",
        _ => "",
    }
}
