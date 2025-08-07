use crate::pattern_matcher::PatternMatcher;
use crate::patterns::{DEFAULT_EXCLUDE_PATTERNS, DEFAULT_INCLUDE_PATTERNS};
use crate::structure_generator::generate_directory_structure;
use anyhow::Result;
use regex::Regex;
use std::path::{Path, PathBuf};
use tokio::fs;
use tracing::{debug, info, instrument, warn};
use walkdir::{DirEntry, WalkDir};

fn get_language_from_extension(path: &Path) -> &'static str {
    match path.extension().and_then(|s| s.to_str()) {
        Some("rs") => "rust",
        Some("py") | Some("pyw") => "python",
        Some("js") | Some("mjs") => "javascript",
        Some("ts") => "typescript",
        Some("tsx") | Some("jsx") => "jsx",
        Some("java") => "java",
        Some("kt") => "kotlin",
        Some("scala") => "scala",
        Some("clj") => "clojure",
        Some("c") => "c",
        Some("cpp") | Some("cc") | Some("cxx") => "cpp",
        Some("h") | Some("hpp") => "c",
        Some("cs") => "csharp",
        Some("fs") => "fsharp",
        Some("vb") => "vbnet",
        Some("php") => "php",
        Some("rb") => "ruby",
        Some("go") => "go",
        Some("swift") => "swift",
        Some("m") | Some("mm") => "objc",
        Some("dart") => "dart",
        Some("lua") => "lua",
        Some("pl") => "perl",
        Some("r") | Some("R") => "r",
        Some("html") | Some("htm") => "html",
        Some("css") => "css",
        Some("scss") => "scss",
        Some("sass") => "sass",
        Some("less") => "less",
        Some("vue") => "vue",
        Some("svelte") => "svelte",
        Some("json") | Some("jsonc") => "json",
        Some("yaml") | Some("yml") => "yaml",
        Some("toml") => "toml",
        Some("xml") => "xml",
        Some("sql") => "sql",
        Some("sh") | Some("bash") => "bash",
        Some("zsh") => "zsh",
        Some("fish") => "fish",
        Some("ps1") => "powershell",
        Some("bat") | Some("cmd") => "batch",
        Some("tf") => "hcl",
        Some("dockerfile") => "dockerfile",
        Some("md") | Some("markdown") => "markdown",
        Some("tex") => "latex",
        Some("cmake") => "cmake",
        _ => {
            if let Some(filename) = path.file_name().and_then(|s| s.to_str()) {
                match filename {
                    "Makefile" | "makefile" => "makefile",
                    "Dockerfile" => "dockerfile",
                    "Jenkinsfile" => "groovy",
                    _ => "text",
                }
            } else {
                "text"
            }
        }
    }
}

fn is_binary_file(content: &[u8]) -> bool {
    let check_len = content.len().min(1024);
    content[..check_len].contains(&0)
}

fn remove_comments_and_docstrings(
    content: &str,
    language: &str,
    ignore_comments: bool,
    ignore_docstrings: bool,
) -> String {
    if !ignore_comments && !ignore_docstrings {
        return content.to_string();
    }

    let mut result = content.to_string();

    if ignore_comments || ignore_docstrings {
        match language {
            "rust" | "javascript" | "typescript" | "java" | "kotlin" | "scala" | "c" | "cpp"
            | "csharp" | "go" | "swift" | "dart" => {
                if ignore_comments {
                    let re = Regex::new(r"//.*$").unwrap();
                    result = re.replace_all(&result, "").to_string();

                    let re = Regex::new(r"/\*.*?\*/").unwrap();
                    result = re.replace_all(&result, "").to_string();
                }
            }
            "python" => {
                if ignore_comments {
                    let re = Regex::new(r"#.*$").unwrap();
                    result = re.replace_all(&result, "").to_string();
                }
                if ignore_docstrings {
                    let re = Regex::new(r#""""".*?""""#).unwrap();
                    result = re.replace_all(&result, "").to_string();
                    let re = Regex::new(r"'''.*?'''").unwrap();
                    result = re.replace_all(&result, "").to_string();
                }
            }
            "ruby" | "bash" | "sh" | "zsh" | "fish" => {
                if ignore_comments {
                    let re = Regex::new(r"#.*$").unwrap();
                    result = re.replace_all(&result, "").to_string();
                }
            }
            _ => {}
        }
    }

    result
        .lines()
        .filter(|line| !line.trim().is_empty())
        .collect::<Vec<_>>()
        .join("\n")
}

// Optimized directory filter function
fn should_skip_directory(entry: &DirEntry, exclude_matcher: &PatternMatcher) -> bool {
    let path = entry.path();

    // Quick checks for common directories to skip
    if let Some(dir_name) = path.file_name().and_then(|n| n.to_str()) {
        match dir_name {
            ".git" | ".svn" | ".hg" | ".bzr" | "node_modules" | "__pycache__" | ".mypy_cache"
            | ".pytest_cache" | ".vscode" | ".idea" | "target" | "build" | "dist" | "out" => {
                return true;
            }
            _ => {}
        }
    }

    exclude_matcher.matches_path(path)
}

// Optimized file filter function
fn should_include_file_fast(
    entry: &DirEntry,
    exclude_matcher: &PatternMatcher,
    include_matcher: &PatternMatcher,
    max_size_bytes: u64,
) -> bool {
    let path = entry.path();

    // Quick exclusion check
    if exclude_matcher.matches_path(path) {
        return false;
    }

    // Quick inclusion check
    if !include_matcher.matches_path(path) {
        return false;
    }

    // Size and binary checks
    if let Ok(metadata) = entry.metadata() {
        if metadata.len() > max_size_bytes || metadata.len() == 0 {
            return false;
        }
    } else {
        return false;
    }

    true
}

// Function to check single file without using DirEntry::from_path
fn should_include_single_file(
    path: &Path,
    exclude_matcher: &PatternMatcher,
    include_matcher: &PatternMatcher,
    max_size_bytes: u64,
) -> bool {
    // Quick exclusion check
    if exclude_matcher.matches_path(path) {
        return false;
    }

    // Quick inclusion check
    if !include_matcher.matches_path(path) {
        return false;
    }

    // Size and binary checks
    if let Ok(metadata) = std::fs::metadata(path) {
        if metadata.len() > max_size_bytes || metadata.len() == 0 {
            return false;
        }
    } else {
        return false;
    }

    true
}

async fn is_text_file(path: &Path) -> bool {
    match fs::read(path).await {
        Ok(content) => !is_binary_file(&content),
        Err(_) => false,
    }
}

#[instrument(skip(additional_excludes, additional_includes))]
pub async fn get_files_recursively(
    paths: &[PathBuf],
    additional_excludes: &[String],
    additional_includes: &[String],
    ignore_comments: bool,
    ignore_docstrings: bool,
    max_size_mb: u64,
) -> Result<Vec<PathBuf>> {
    let max_size_bytes = max_size_mb * 1024 * 1024;

    // Build optimized pattern matchers
    let mut exclude_patterns: Vec<String> = DEFAULT_EXCLUDE_PATTERNS
        .iter()
        .map(|s| s.to_string())
        .collect();
    exclude_patterns.extend(additional_excludes.iter().cloned());

    let include_patterns: Vec<String> = if additional_includes.is_empty() {
        DEFAULT_INCLUDE_PATTERNS
            .iter()
            .map(|s| s.to_string())
            .collect()
    } else {
        additional_includes.to_vec()
    };

    let exclude_matcher = PatternMatcher::new(&exclude_patterns);
    let include_matcher = PatternMatcher::new(&include_patterns);

    debug!("Using {} exclude patterns", exclude_patterns.len());
    debug!("Using {} include patterns", include_patterns.len());

    let mut all_files = Vec::new();

    for path in paths {
        if path.is_file() {
            if should_include_single_file(path, &exclude_matcher, &include_matcher, max_size_bytes)
                && is_text_file(path).await
            {
                all_files.push(path.clone());
            }
        } else if path.is_dir() {
            // Use optimized directory traversal
            for entry in WalkDir::new(path)
                .into_iter()
                .filter_entry(|e| {
                    if e.path().is_dir() {
                        !should_skip_directory(e, &exclude_matcher)
                    } else {
                        true
                    }
                })
                .filter_map(|e| e.ok())
            {
                let entry_path = entry.path();

                if entry_path.is_file()
                    && should_include_file_fast(
                        &entry,
                        &exclude_matcher,
                        &include_matcher,
                        max_size_bytes,
                    )
                    && is_text_file(entry_path).await
                {
                    all_files.push(entry_path.to_path_buf());
                }
            }
        }
    }

    info!("Found {} files after filtering", all_files.len());
    Ok(all_files)
}

#[instrument(skip(files))]
pub async fn concatenate_files(files: &[PathBuf], output_file: Option<&str>) -> Result<String> {
    let mut result = String::new();

    // Generate directory structure
    result.push_str("# Project Structure\n\n");
    result.push_str("```\n");
    let structure = generate_directory_structure(files);
    for line in structure {
        result.push_str(&line);
        result.push('\n');
    }
    result.push_str("```\n\n");

    // Add file contents
    result.push_str("# File Contents\n\n");

    let current_dir = std::env::current_dir().unwrap_or_default();

    for file_path in files {
        let relative_path = file_path.strip_prefix(&current_dir).unwrap_or(file_path);

        result.push_str(&format!("## {}\n\n", relative_path.display()));

        match fs::read_to_string(file_path).await {
            Ok(content) => {
                let language = get_language_from_extension(file_path);
                let processed_content = remove_comments_and_docstrings(
                    &content,
                    language,
                    ignore_comments,
                    ignore_docstrings,
                );

                result.push_str(&format!("```{}\n", language));
                result.push_str(&processed_content);
                result.push_str("\n```\n\n");

                debug!(
                    "Added file: {} ({} chars)",
                    relative_path.display(),
                    processed_content.len()
                );
            }
            Err(e) => {
                warn!("Could not read file {}: {}", file_path.display(), e);
                result.push_str(&format!("*Error reading file: {}*\n\n", e));
            }
        }
    }

    if let Some(output_path) = output_file {
        fs::write(output_path, &result).await?;
        info!("Output written to: {}", output_path);
        println!("Output written to: {}", output_path);
    }

    Ok(result)
}
