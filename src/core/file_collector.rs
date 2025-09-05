use crate::config::patterns::{DEFAULT_EXCLUDE_PATTERNS, DEFAULT_INCLUDE_PATTERNS};
use crate::core::pattern_matcher::PatternMatcher;
use anyhow::Result;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use tokio::fs;
use tracing::{debug, info, instrument};
use walkdir::{DirEntry, WalkDir};

pub fn is_binary_file(content: &[u8]) -> bool {
    let check_len = content.len().min(1024);
    content[..check_len].contains(&0)
}

fn should_skip_directory(entry: &DirEntry, exclude_matcher: &PatternMatcher) -> bool {
    let path = entry.path();

    // Quick checks for common directories to skip
    if let Some(
        ".git" | ".svn" | ".hg" | ".bzr" | "node_modules" | "__pycache__" | ".mypy_cache"
        | ".pytest_cache" | ".vscode" | ".idea" | "target" | "build" | "dist" | "out",
    ) = path.file_name().and_then(|n| n.to_str())
    {
        return true;
    }

    exclude_matcher.matches_path(path)
}

fn should_include_file(
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
        metadata.len() <= max_size_bytes && metadata.len() > 0
    } else {
        false
    }
}

async fn is_text_file(path: &Path) -> bool {
    match fs::read(path).await {
        Ok(content) => !is_binary_file(&content),
        Err(_) => false,
    }
}

#[instrument(skip(additional_excludes, additional_includes))]
pub async fn collect_files(
    paths: &[PathBuf],
    additional_excludes: &[String],
    additional_includes: &[String],
    max_size_mb: u64,
) -> Result<Vec<PathBuf>> {
    let max_size_bytes = max_size_mb * 1024 * 1024;

    // Build pattern matchers
    let mut exclude_patterns = DEFAULT_EXCLUDE_PATTERNS.to_vec();
    exclude_patterns.extend(additional_excludes.iter().map(|s| s.as_str()));
    let exclude_patterns: Vec<String> = exclude_patterns.iter().map(|s| s.to_string()).collect();

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
            if should_include_file(path, &exclude_matcher, &include_matcher, max_size_bytes)
                && is_text_file(path).await
            {
                all_files.push(path.clone());
            }
        } else if path.is_dir() {
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
                    && should_include_file(
                        entry_path,
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

    if !all_files.is_empty() {
        println!("\n📁 Files to be included:");
        print_file_tree(&all_files);
        println!();
    }

    Ok(all_files)
}

fn print_file_tree(files: &[PathBuf]) {
    let current_dir = std::env::current_dir().unwrap_or_default();
    let mut tree = BTreeMap::new();

    // Build tree structure
    for file in files {
        let relative_path = file.strip_prefix(&current_dir).unwrap_or(file);
        add_file_to_tree(&mut tree, relative_path);
    }

    // Print tree
    print_tree_recursive(&tree, "", true);
}

fn add_file_to_tree(tree: &mut BTreeMap<String, TreeNode>, path: &Path) {
    let components: Vec<_> = path.components().collect();
    if components.is_empty() {
        return;
    }

    let mut current = tree;

    for (i, component) in components.iter().enumerate() {
        let name = component.as_os_str().to_string_lossy().to_string();
        let is_file = i == components.len() - 1;

        if is_file {
            current.insert(name, TreeNode::File);
            break;
        }

        let entry = current
            .entry(name)
            .or_insert_with(|| TreeNode::Directory(BTreeMap::new()));

        match entry {
            TreeNode::Directory(subtree) => {
                current = subtree;
            }
            TreeNode::File => break,
        }
    }
}

#[derive(Debug)]
enum TreeNode {
    File,
    Directory(BTreeMap<String, TreeNode>),
}

fn print_tree_recursive(tree: &BTreeMap<String, TreeNode>, prefix: &str, is_root: bool) {
    let items: Vec<_> = tree.iter().collect();

    for (i, (name, node)) in items.iter().enumerate() {
        let is_last = i == items.len() - 1;
        let connector = if is_root {
            if is_last { "└── " } else { "├── " }
        } else if is_last {
            "└── "
        } else {
            "├── "
        };

        match node {
            TreeNode::File => {
                println!("{}{}📄 {}", prefix, connector, name);
            }
            TreeNode::Directory(subtree) => {
                println!("{}{}📁 {}/", prefix, connector, name);
                let new_prefix = format!("{}{}", prefix, if is_last { "    " } else { "│   " });
                print_tree_recursive(subtree, &new_prefix, false);
            }
        }
    }
}
