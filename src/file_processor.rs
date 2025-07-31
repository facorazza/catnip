use crate::exclusion_patterns::{DEFAULT_EXCLUDE_PATTERNS, DEFAULT_INCLUDE_PATTERNS};
use crate::pattern_matcher::matches_any_pattern;
use crate::structure_generator::generate_directory_structure;
use anyhow::{Context, Result};
use regex::Regex;
use std::fs;
use std::path::{Path, PathBuf};
use tracing::{Level, debug, error, info, instrument, span, warn};
use walkdir::WalkDir;

#[instrument(skip(exclude_patterns, include_patterns))]
pub async fn get_files_recursively(
    paths: &[PathBuf],
    exclude_patterns: &[String],
    include_patterns: &[String],
    ignore_comments: bool,
    ignore_docstrings: bool,
) -> Result<Vec<PathBuf>> {
    let span = span!(
        Level::INFO,
        "get_files_recursively",
        paths_count = paths.len(),
        exclude_count = exclude_patterns.len(),
        include_count = include_patterns.len()
    );
    let _enter = span.enter();

    debug!("Building pattern lists");
    let mut all_exclude = DEFAULT_EXCLUDE_PATTERNS
        .iter()
        .map(|s| s.to_string())
        .collect::<Vec<_>>();
    all_exclude.extend_from_slice(exclude_patterns);

    let mut all_include = DEFAULT_INCLUDE_PATTERNS
        .iter()
        .map(|s| s.to_string())
        .collect::<Vec<_>>();
    all_include.extend_from_slice(include_patterns);

    debug!(
        "Total exclude patterns: {}, include patterns: {}",
        all_exclude.len(),
        all_include.len()
    );

    let mut found_files = Vec::new();

    for (i, path) in paths.iter().enumerate() {
        debug!(
            "Processing path {}/{}: {}",
            i + 1,
            paths.len(),
            path.display()
        );

        if path.is_file() {
            if is_valid_file(
                path,
                &all_exclude,
                &all_include,
                ignore_comments,
                ignore_docstrings,
            )
            .await?
            {
                debug!("Added file: {}", path.display());
                found_files.push(path.clone());
            }
        } else if path.is_dir() {
            let files_before = found_files.len();
            collect_files_recursive(
                path,
                &all_exclude,
                &all_include,
                ignore_comments,
                ignore_docstrings,
                &mut found_files,
            )
            .await?;
            let files_added = found_files.len() - files_before;
            info!("Directory {} yielded {} files", path.display(), files_added);
        } else {
            warn!(
                "Path does not exist or is not accessible: {}",
                path.display()
            );
        }
    }

    info!("Total files found: {}", found_files.len());
    Ok(found_files)
}

#[instrument(skip(exclude_patterns, include_patterns, found_files))]
async fn collect_files_recursive(
    dir: &Path,
    exclude_patterns: &[String],
    include_patterns: &[String],
    ignore_comments: bool,
    ignore_docstrings: bool,
    found_files: &mut Vec<PathBuf>,
) -> Result<()> {
    debug!("Traversing directory: {}", dir.display());

    let mut file_count = 0;
    let mut dir_count = 0;
    let mut excluded_count = 0;

    for entry in WalkDir::new(dir).into_iter() {
        let entry = entry.context("Failed to read directory entry")?;
        let path = entry.path();

        if path.is_dir() {
            dir_count += 1;
            // Skip directories that match exclude patterns
            if matches_any_pattern(path, exclude_patterns) {
                debug!("Excluding directory: {}", path.display());
                excluded_count += 1;
                continue;
            }
        }

        if path.is_file() {
            file_count += 1;
            if is_valid_file(
                path,
                exclude_patterns,
                include_patterns,
                ignore_comments,
                ignore_docstrings,
            )
            .await?
            {
                debug!("Adding file: {}", path.display());
                found_files.push(path.to_path_buf());
            } else {
                excluded_count += 1;
            }
        }
    }

    info!(
        "Directory scan complete - Files: {}, Dirs: {}, Excluded: {}",
        file_count, dir_count, excluded_count
    );

    Ok(())
}

#[instrument(skip(exclude_patterns, include_patterns))]
async fn is_valid_file(
    file_path: &Path,
    exclude_patterns: &[String],
    include_patterns: &[String],
    ignore_comments: bool,
    ignore_docstrings: bool,
) -> Result<bool> {
    // Check if file is empty
    let metadata = fs::metadata(file_path).context("Failed to read file metadata")?;
    if metadata.len() == 0 {
        debug!("Skipping empty file: {}", file_path.display());
        return Ok(false);
    }

    // Check include patterns first
    if !matches_any_pattern(file_path, include_patterns) {
        debug!(
            "File doesn't match include patterns: {}",
            file_path.display()
        );
        return Ok(false);
    }

    // Check exclude patterns
    if matches_any_pattern(file_path, exclude_patterns) {
        debug!("File matches exclude patterns: {}", file_path.display());
        return Ok(false);
    }

    // Check content if needed
    if ignore_comments || ignore_docstrings {
        debug!(
            "Processing content for comments/docstrings: {}",
            file_path.display()
        );
        let content = fs::read_to_string(file_path).context("Failed to read file content")?;
        let processed_content =
            process_content(file_path, &content, ignore_comments, ignore_docstrings).await?;

        if processed_content.trim().is_empty() {
            debug!(
                "File became empty after processing: {}",
                file_path.display()
            );
            return Ok(false);
        }
    }

    Ok(true)
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
        debug!("Removing comments from: {}", file_path.display());
        processed = remove_comments(file_path, &processed).await?;
    }

    if ignore_docstrings {
        debug!("Removing docstrings from: {}", file_path.display());
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

    debug!("Removing comments for file type: {}", ext);

    match ext.as_str() {
        "py" | "pyw" => {
            let re = Regex::new(r#"(#.*?$)|('''.*?'''|""".*?""")"#)?;
            Ok(re.replace_all(content, "").to_string())
        }
        "js" | "java" | "c" | "cpp" | "cs" | "swift" | "rs" => {
            let re = Regex::new(r"(//.*?$)|(/\*.*?\*/)")?;
            Ok(re.replace_all(content, "").to_string())
        }
        "html" | "xml" => {
            let re = Regex::new(r"<!--.*?-->")?;
            Ok(re.replace_all(content, "").to_string())
        }
        _ => {
            debug!("No comment removal pattern for extension: {}", ext);
            Ok(content.to_string())
        }
    }
}

#[instrument]
async fn remove_docstrings(content: &str) -> Result<String> {
    debug!("Removing docstrings from content");
    let re = Regex::new(r#"('''.*?'''|""".*?""")"#)?;
    Ok(re.replace_all(content, "").to_string())
}

#[instrument(skip(files))]
pub async fn concatenate_files(files: &[PathBuf], output_file: Option<&str>) -> Result<String> {
    info!("Concatenating {} files", files.len());

    let mut output = String::new();

    // Add project structure
    debug!("Generating project structure");
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

    debug!("Processing file contents");
    for (i, file) in sorted_files.iter().enumerate() {
        debug!(
            "Processing file {}/{}: {}",
            i + 1,
            sorted_files.len(),
            file.display()
        );

        let current_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        let relative_path = file.strip_prefix(&current_dir).unwrap_or(file);

        let file_type = get_file_type(file);

        output.push_str(&format!("## {}\n\n", relative_path.display()));
        output.push_str(&format!("```{}\n", file_type));

        match fs::read_to_string(file) {
            Ok(content) => {
                output.push_str(&content.trim_end_matches('\n'));
            }
            Err(e) => {
                error!("Error reading file {}: {}", file.display(), e);
            }
        }

        output.push_str("\n```\n\n");
    }

    // Write to output file if specified
    if let Some(output_path) = output_file {
        debug!("Writing output to file: {}", output_path);
        fs::write(output_path, &output).context("Failed to write output file")?;
        info!("Successfully created output file: {}", output_path);
        println!("Successfully created output file: {}", output_path);
    }

    info!(
        "Concatenation completed - total output size: {} characters",
        output.len()
    );
    Ok(output)
}

fn get_file_type(file: &Path) -> &'static str {
    let ext = file
        .extension()
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
