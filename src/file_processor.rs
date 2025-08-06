use crate::exclusion_patterns::{DEFAULT_EXCLUDE_PATTERNS, DEFAULT_INCLUDE_PATTERNS};
use crate::pattern_matcher::matches_any_pattern;
use crate::structure_generator::generate_directory_structure;
use anyhow::Result;
use regex::Regex;
use std::path::{Path, PathBuf};
use tokio::fs;
use tracing::{debug, info, instrument, warn};
use walkdir::WalkDir;

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
            // Check for special filenames
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
    // Check for null bytes in the first 1024 bytes
    let check_len = std::cmp::min(content.len(), 1024);
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
                    // Remove single-line comments
                    let re = Regex::new(r"//.*$").unwrap();
                    result = re.replace_all(&result, "").to_string();

                    // Remove multi-line comments
                    let re = Regex::new(r"/\*.*?\*/").unwrap();
                    result = re.replace_all(&result, "").to_string();
                }
            }
            "python" => {
                if ignore_comments {
                    // Remove single-line comments
                    let re = Regex::new(r"#.*$").unwrap();
                    result = re.replace_all(&result, "").to_string();
                }
                if ignore_docstrings {
                    // Remove triple-quoted docstrings
                    let re = Regex::new(r#"""".*?""""#).unwrap();
                    result = re.replace_all(&result, "").to_string();
                    let re = Regex::new(r"'''.*?'''").unwrap();
                    result = re.replace_all(&result, "").to_string();
                }
            }
            "ruby" => {
                if ignore_comments {
                    let re = Regex::new(r"#.*$").unwrap();
                    result = re.replace_all(&result, "").to_string();
                }
            }
            "bash" | "sh" | "zsh" | "fish" => {
                if ignore_comments {
                    let re = Regex::new(r"#.*$").unwrap();
                    result = re.replace_all(&result, "").to_string();
                }
            }
            _ => {}
        }
    }

    // Clean up extra whitespace
    let lines: Vec<&str> = result.lines().collect();
    let cleaned_lines: Vec<&str> = lines
        .into_iter()
        .filter(|line| !line.trim().is_empty())
        .collect();

    cleaned_lines.join("\n")
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
    let mut all_files = Vec::new();
    let max_size_bytes = max_size_mb * 1024 * 1024;

    // Combine default and additional patterns
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

    debug!("Using {} exclude patterns", exclude_patterns.len());
    debug!("Using {} include patterns", include_patterns.len());

    for path in paths {
        if path.is_file() {
            // Single file
            if should_include_file(path, &exclude_patterns, &include_patterns, max_size_bytes)
                .await?
            {
                all_files.push(path.clone());
            }
        } else if path.is_dir() {
            // Directory - walk recursively
            for entry in WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
                let entry_path = entry.path();

                if entry_path.is_file()
                    && should_include_file(
                        entry_path,
                        &exclude_patterns,
                        &include_patterns,
                        max_size_bytes,
                    )
                    .await?
                {
                    all_files.push(entry_path.to_path_buf());
                }
            }
        }
    }

    info!("Found {} files after filtering", all_files.len());
    Ok(all_files)
}

async fn should_include_file(
    path: &Path,
    exclude_patterns: &[String],
    include_patterns: &[String],
    max_size_bytes: u64,
) -> Result<bool> {
    // Check exclusion patterns first
    if matches_any_pattern(path, exclude_patterns) {
        debug!("File excluded by pattern: {}", path.display());
        return Ok(false);
    }

    // Check inclusion patterns
    if !matches_any_pattern(path, include_patterns) {
        debug!("File not matching include patterns: {}", path.display());
        return Ok(false);
    }

    // Check file size
    match fs::metadata(path).await {
        Ok(metadata) => {
            if metadata.len() > max_size_bytes {
                debug!(
                    "File too large ({}MB): {}",
                    metadata.len() / (1024 * 1024),
                    path.display()
                );
                return Ok(false);
            }

            if metadata.len() == 0 {
                debug!("Empty file skipped: {}", path.display());
                return Ok(false);
            }
        }
        Err(e) => {
            warn!("Could not read metadata for {}: {}", path.display(), e);
            return Ok(false);
        }
    }

    // Check if file is binary
    match fs::read(path).await {
        Ok(content) => {
            if is_binary_file(&content) {
                debug!("Binary file skipped: {}", path.display());
                return Ok(false);
            }
        }
        Err(e) => {
            warn!("Could not read file {}: {}", path.display(), e);
            return Ok(false);
        }
    }

    Ok(true)
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

    for file_path in files {
        let relative_path = if let Ok(current_dir) = std::env::current_dir() {
            file_path.strip_prefix(&current_dir).unwrap_or(file_path)
        } else {
            file_path
        };

        result.push_str(&format!("## {}\n\n", relative_path.display()));

        match fs::read_to_string(file_path).await {
            Ok(content) => {
                let language = get_language_from_extension(file_path);
                let processed_content =
                    remove_comments_and_docstrings(&content, language, false, false);

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

    // Write to output file if specified
    if let Some(output_path) = output_file {
        fs::write(output_path, &result).await?;
        info!("Output written to: {}", output_path);
        println!("Output written to: {}", output_path);
    }

    Ok(result)
}
