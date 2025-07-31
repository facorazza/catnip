use crate::exclusion_patterns::{DEFAULT_EXCLUDE_PATTERNS, DEFAULT_INCLUDE_PATTERNS};
use crate::pattern_matcher::matches_any_pattern;
use crate::structure_generator::generate_directory_structure;
use anyhow::{Context, Result};
use regex::Regex;
use std::fs;
use std::path::{Path, PathBuf};
use tracing::{Level, debug, error, info, instrument, span, warn};
use walkdir::WalkDir;

const BYTES_PER_MB: u64 = 1024 * 1024;

#[instrument(skip(exclude_patterns, include_patterns))]
pub async fn get_files_recursively(
    paths: &[PathBuf],
    exclude_patterns: &[String],
    include_patterns: &[String],
    ignore_comments: bool,
    ignore_docstrings: bool,
    max_size_mb: u64,
) -> Result<Vec<PathBuf>> {
    let span = span!(
        Level::INFO,
        "get_files_recursively",
        paths_count = paths.len(),
        exclude_count = exclude_patterns.len(),
        include_count = include_patterns.len()
    );
    let _enter = span.enter();

    let all_exclude = build_pattern_list(DEFAULT_EXCLUDE_PATTERNS, exclude_patterns);
    let all_include = build_pattern_list(DEFAULT_INCLUDE_PATTERNS, include_patterns);
    let max_size_bytes = max_size_mb * BYTES_PER_MB;

    debug!(
        "Exclude patterns: {}, Include patterns: {}, Max size: {}MB",
        all_exclude.len(),
        all_include.len(),
        max_size_mb
    );

    let mut found_files = Vec::new();

    for path in paths {
        if path.is_file() {
            if is_valid_file(
                path,
                &all_exclude,
                &all_include,
                ignore_comments,
                ignore_docstrings,
                max_size_bytes,
            )
            .await?
            {
                found_files.push(path.clone());
            }
        } else if path.is_dir() {
            collect_files_recursive(
                path,
                &all_exclude,
                &all_include,
                ignore_comments,
                ignore_docstrings,
                max_size_bytes,
                &mut found_files,
            )
            .await?;
        } else {
            warn!("Path not found: {}", path.display());
        }
    }

    info!("Total files found: {}", found_files.len());
    Ok(found_files)
}

fn build_pattern_list(defaults: &[&str], additional: &[String]) -> Vec<String> {
    let mut patterns = defaults.iter().map(|s| s.to_string()).collect::<Vec<_>>();
    patterns.extend_from_slice(additional);
    patterns
}

#[instrument(skip(exclude_patterns, include_patterns, found_files))]
async fn collect_files_recursive(
    dir: &Path,
    exclude_patterns: &[String],
    include_patterns: &[String],
    ignore_comments: bool,
    ignore_docstrings: bool,
    max_size_bytes: u64,
    found_files: &mut Vec<PathBuf>,
) -> Result<()> {
    for entry in WalkDir::new(dir)
        .into_iter()
        .filter_entry(|e| !matches_any_pattern(e.path(), exclude_patterns))
    {
        let entry = entry.context("Failed to read directory entry")?;
        let path = entry.path();

        if path.is_file()
            && is_valid_file(
                path,
                exclude_patterns,
                include_patterns,
                ignore_comments,
                ignore_docstrings,
                max_size_bytes,
            )
            .await?
        {
            found_files.push(path.to_path_buf());
        }
    }

    Ok(())
}

#[instrument(skip(exclude_patterns, include_patterns))]
async fn is_valid_file(
    file_path: &Path,
    exclude_patterns: &[String],
    include_patterns: &[String],
    ignore_comments: bool,
    ignore_docstrings: bool,
    max_size_bytes: u64,
) -> Result<bool> {
    // Check file size first (quick check)
    let metadata = fs::metadata(file_path).context("Failed to read file metadata")?;
    if metadata.len() == 0 || metadata.len() > max_size_bytes {
        return Ok(false);
    }

    // Check patterns
    if !matches_any_pattern(file_path, include_patterns)
        || matches_any_pattern(file_path, exclude_patterns)
    {
        return Ok(false);
    }

    // Check if file is likely binary
    if is_likely_binary(file_path).await? {
        return Ok(false);
    }

    // Check content if needed
    if ignore_comments || ignore_docstrings {
        let content = fs::read_to_string(file_path).context("Failed to read file content")?;
        let processed_content =
            process_content(file_path, &content, ignore_comments, ignore_docstrings).await?;

        if processed_content.trim().is_empty() {
            return Ok(false);
        }
    }

    Ok(true)
}

#[instrument]
async fn is_likely_binary(file_path: &Path) -> Result<bool> {
    // Read first 1KB to check for binary content
    let content = match fs::read(file_path) {
        Ok(data) => data,
        Err(_) => return Ok(true), // Assume binary if can't read
    };

    let sample_size = std::cmp::min(1024, content.len());
    let sample = &content[..sample_size];

    // Check for null bytes (common in binary files)
    let null_bytes = sample.iter().filter(|&&b| b == 0).count();
    let binary_ratio = null_bytes as f64 / sample_size as f64;

    Ok(binary_ratio > 0.01) // More than 1% null bytes suggests binary
}

#[instrument]
async fn process_content(
    file_path: &Path,
    content: &str,
    ignore_comments: bool,
    ignore_docstrings: bool,
) -> Result<String> {
    let mut processed = content.to_string();

    if ignore_comments {
        processed = remove_comments(file_path, &processed).await?;
    }

    if ignore_docstrings {
        processed = remove_docstrings(&processed).await?;
    }

    Ok(processed)
}

#[instrument]
async fn remove_comments(file_path: &Path, content: &str) -> Result<String> {
    let ext = file_path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    let pattern = match ext.as_str() {
        "py" | "pyw" => r#"(#.*?$)|('''[\s\S]*?'''|"""[\s\S]*?""")"#,
        "js" | "ts" | "java" | "c" | "cpp" | "cs" | "swift" | "rs" | "go" => {
            r"(//.*?$)|(/\*[\s\S]*?\*/)"
        }
        "html" | "xml" => r"<!--[\s\S]*?-->",
        _ => return Ok(content.to_string()),
    };

    let re = Regex::new(pattern)?;
    Ok(re.replace_all(content, "").to_string())
}

#[instrument]
async fn remove_docstrings(content: &str) -> Result<String> {
    let re = Regex::new(r#"('''[\s\S]*?'''|"""[\s\S]*?""")"#)?;
    Ok(re.replace_all(content, "").to_string())
}

#[instrument(skip(files))]
pub async fn concatenate_files(files: &[PathBuf], output_file: Option<&str>) -> Result<String> {
    info!("Concatenating {} files", files.len());

    let mut output = String::new();
    let current_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));

    // Add project structure
    output.push_str("# Project Structure\n\n├── ./\n");
    let directory_structure = generate_directory_structure(files);
    for line in directory_structure {
        output.push_str(&line);
        output.push('\n');
    }
    output.push_str("\n# File Contents\n\n");

    // Add file contents
    let mut sorted_files = files.to_vec();
    sorted_files.sort();

    for file in &sorted_files {
        let relative_path = file.strip_prefix(&current_dir).unwrap_or(file);
        let file_type = get_file_type(file);

        output.push_str(&format!(
            "## {}\n\n```{}\n",
            relative_path.display(),
            file_type
        ));

        match fs::read_to_string(file) {
            Ok(content) => output.push_str(content.trim_end_matches('\n')),
            Err(e) => error!("Error reading {}: {}", file.display(), e),
        }

        output.push_str("\n```\n\n");
    }

    if let Some(output_path) = output_file {
        fs::write(output_path, &output).context("Failed to write output file")?;
        info!("Output written to: {}", output_path);
    }

    Ok(output)
}

fn get_file_type(file: &Path) -> &'static str {
    let ext = file
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    match ext.as_str() {
        "py" | "pyw" => "python",
        "js" | "mjs" => "javascript",
        "ts" => "typescript",
        "jsx" => "jsx",
        "tsx" => "tsx",
        "rs" => "rust",
        "go" => "go",
        "java" => "java",
        "kt" => "kotlin",
        "c" => "c",
        "cpp" | "cc" | "cxx" => "cpp",
        "cs" => "csharp",
        "php" => "php",
        "rb" => "ruby",
        "swift" => "swift",
        "html" | "htm" => "html",
        "css" => "css",
        "scss" => "scss",
        "sass" => "sass",
        "json" | "jsonc" => "json",
        "yaml" | "yml" => "yaml",
        "toml" => "toml",
        "xml" => "xml",
        "md" | "markdown" => "markdown",
        "sh" | "bash" => "bash",
        "ps1" => "powershell",
        "bat" | "cmd" => "batch",
        "sql" => "sql",
        "dockerfile" => "dockerfile",
        _ => "",
    }
}
