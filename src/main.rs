use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use tracing::{debug, error, info, warn};

mod exclusion_patterns;
mod file_processor;
mod pattern_matcher;
mod prompt;
mod structure_generator;

#[derive(Parser)]
#[command(name = "catnip")]
#[command(about = "Concatenate and patch codebases")]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Concatenate files with directory structure and content
    Cat {
        /// Paths to process
        paths: Vec<PathBuf>,

        /// Output file name (optional)
        #[arg(short, long)]
        output: Option<String>,

        /// Don't copy to clipboard (clipboard is default behavior)
        #[arg(long)]
        no_copy: bool,

        /// Additional patterns to exclude
        #[arg(long)]
        exclude: Vec<String>,

        /// Additional patterns to include
        #[arg(long)]
        include: Vec<String>,

        /// Ignore code comments
        #[arg(long)]
        ignore_comments: bool,

        /// Ignore docstrings
        #[arg(long)]
        ignore_docstrings: bool,

        /// Maximum file size in MB (default: 10MB)
        #[arg(long, default_value = "10")]
        max_size_mb: u64,
        /// Include prompt instructions from codebase_prompt.md
        #[arg(long)]
        include_prompt: bool,
    },
    /// Apply JSON-formatted code updates to files
    Patch {
        /// JSON file containing updates, or '-' to read from stdin
        json_file: String,

        /// Dry run - show what would be changed without applying updates
        #[arg(long)]
        dry_run: bool,

        /// Create backup files before updating
        #[arg(long)]
        backup: bool,
    },
}

#[derive(Debug, Deserialize, Serialize)]
struct UpdateRequest {
    analysis: String,
    files: Vec<FileUpdate>,
}

#[derive(Debug, Deserialize, Serialize)]
struct FileUpdate {
    path: String,
    updates: Vec<LineUpdate>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct LineUpdate {
    line_start: usize,
    line_end: usize,
    old_content: String,
    new_content: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let args = Args::parse();

    match args.command {
        Commands::Cat {
            paths,
            output,
            no_copy,
            exclude,
            include,
            ignore_comments,
            ignore_docstrings,
            include_prompt,
            max_size_mb,
        } => {
            execute_cat(
                paths,
                output,
                no_copy,
                exclude,
                include,
                ignore_comments,
                ignore_docstrings,
                include_prompt,
                max_size_mb,
            )
            .await?;
        }
        Commands::Patch {
            json_file,
            dry_run,
            backup,
        } => {
            execute_patch(json_file, dry_run, backup).await?;
        }
    }

    Ok(())
}

#[allow(clippy::too_many_arguments)]
async fn execute_cat(
    paths: Vec<PathBuf>,
    output: Option<String>,
    no_copy: bool,
    exclude: Vec<String>,
    include: Vec<String>,
    ignore_comments: bool,
    ignore_docstrings: bool,
    include_prompt: bool,
    max_size_mb: u64,
) -> Result<()> {
    if paths.is_empty() {
        error!("No paths provided");
        std::process::exit(1);
    }

    let files = file_processor::get_files_recursively(
        &paths,
        &exclude,
        &include,
        ignore_comments,
        ignore_docstrings,
        max_size_mb,
    )
    .await?;

    info!("Found {} files to process", files.len());

    let mut result = file_processor::concatenate_files(&files, output.as_deref()).await?;

    // Add prompt instructions if requested
    if include_prompt {
        result = format!("{}\n{}", result, prompt::PROMPT);
        info!("Added prompt instructions from constant");
    }

    // Copy to clipboard by default unless --no-copy is specified or output file is provided
    if !no_copy && output.is_none() {
        copy_to_clipboard(&result).await?;
    }

    info!("Processing completed successfully");
    Ok(())
}

async fn execute_patch(json_file: String, dry_run: bool, backup: bool) -> Result<()> {
    // Read JSON from file or stdin
    let json_content = if json_file == "-" {
        use std::io::{self, BufRead};
        let stdin = io::stdin();
        let lines: Result<Vec<_>, _> = stdin.lock().lines().collect();
        lines.context("Failed to read from stdin")?.join("\n")
    } else {
        fs::read_to_string(&json_file)
            .with_context(|| format!("Failed to read JSON file: {}", json_file))?
    };

    let update_request: UpdateRequest =
        serde_json::from_str(&json_content).context("Failed to parse JSON content")?;

    info!("Analysis: {}", update_request.analysis);
    info!("Processing {} files", update_request.files.len());

    if dry_run {
        info!("DRY RUN MODE - No files will be modified");
    }

    let mut total_updates = 0;
    let mut successful_files = 0;

    for file_update in &update_request.files {
        match process_file_update(file_update, dry_run, backup).await {
            Ok(update_count) => {
                total_updates += update_count;
                successful_files += 1;
                info!("✓ {} - {} updates applied", file_update.path, update_count);
            }
            Err(e) => {
                error!("✗ {} - Error: {}", file_update.path, e);
            }
        }
    }

    info!(
        "Completed: {}/{} files processed successfully, {} total updates",
        successful_files,
        update_request.files.len(),
        total_updates
    );

    if successful_files != update_request.files.len() {
        std::process::exit(1);
    }

    Ok(())
}

async fn process_file_update(
    file_update: &FileUpdate,
    dry_run: bool,
    create_backup: bool,
) -> Result<usize> {
    let file_path = PathBuf::from(&file_update.path);

    debug!("Processing file: {}", file_path.display());

    if !file_path.exists() {
        return Err(anyhow::anyhow!(
            "File does not exist: {}",
            file_path.display()
        ));
    }

    // Read current file content
    let original_content = fs::read_to_string(&file_path)
        .with_context(|| format!("Failed to read file: {}", file_path.display()))?;

    let mut lines: Vec<String> = original_content.lines().map(|s| s.to_string()).collect();

    // Sort updates by line number in reverse order to avoid offset issues
    let mut sorted_updates = file_update.updates.clone();
    sorted_updates.sort_by(|a, b| b.line_start.cmp(&a.line_start));

    let mut applied_updates = 0;

    for update in &sorted_updates {
        debug!(
            "Applying update: lines {}-{}",
            update.line_start, update.line_end
        );

        if let Err(e) = validate_and_apply_update(&mut lines, update) {
            return Err(anyhow::anyhow!(
                "Failed to apply update at lines {}-{}: {}",
                update.line_start,
                update.line_end,
                e
            ));
        }

        applied_updates += 1;
    }

    if dry_run {
        info!(
            "DRY RUN: Would apply {} updates to {}",
            applied_updates,
            file_path.display()
        );

        // Show preview of changes
        for update in &file_update.updates {
            println!("\n--- Lines {}-{} ---", update.line_start, update.line_end);
            println!("- {}", update.old_content.replace('\n', "\n- "));
            println!("+ {}", update.new_content.replace('\n', "\n+ "));
        }

        return Ok(applied_updates);
    }

    // Create backup if requested
    if create_backup {
        let backup_path = format!("{}.backup", file_path.display());
        fs::copy(&file_path, &backup_path)
            .with_context(|| format!("Failed to create backup: {}", backup_path))?;
        debug!("Created backup: {}", backup_path);
    }

    // Write updated content
    let new_content = lines.join("\n");
    fs::write(&file_path, &new_content)
        .with_context(|| format!("Failed to write updated file: {}", file_path.display()))?;

    Ok(applied_updates)
}

fn validate_and_apply_update(lines: &mut Vec<String>, update: &LineUpdate) -> Result<()> {
    // Convert to 0-based indexing
    let start_idx = update.line_start.saturating_sub(1);
    let end_idx = update.line_end.saturating_sub(1);

    if start_idx >= lines.len() {
        return Err(anyhow::anyhow!(
            "Start line {} is beyond file length {}",
            update.line_start,
            lines.len()
        ));
    }

    if end_idx >= lines.len() {
        return Err(anyhow::anyhow!(
            "End line {} is beyond file length {}",
            update.line_end,
            lines.len()
        ));
    }

    if start_idx > end_idx {
        return Err(anyhow::anyhow!(
            "Start line {} is after end line {}",
            update.line_start,
            update.line_end
        ));
    }

    // Extract current content for the specified lines
    let current_content = lines[start_idx..=end_idx].join("\n");

    // Normalize line endings for comparison
    let normalized_current = current_content.replace("\r\n", "\n");
    let normalized_expected = update.old_content.replace("\r\n", "\n");

    if normalized_current != normalized_expected {
        warn!(
            "Content mismatch at lines {}-{}",
            update.line_start, update.line_end
        );
        debug!("Expected:\n{}", normalized_expected);
        debug!("Found:\n{}", normalized_current);

        return Err(anyhow::anyhow!(
            "Content mismatch - the old_content doesn't match the current file content"
        ));
    }

    // Apply the update
    let new_lines: Vec<String> = update.new_content.lines().map(|s| s.to_string()).collect();

    // Replace the range with new content
    lines.splice(start_idx..=end_idx, new_lines);

    debug!(
        "Successfully applied update at lines {}-{}",
        update.line_start, update.line_end
    );
    Ok(())
}

// Clipboard functionality
#[derive(Debug)]
enum ClipboardType {
    Wayland,
    X11,
    MacOS,
    Windows,
    Unsupported,
}

fn detect_clipboard_system() -> ClipboardType {
    if cfg!(target_os = "windows") {
        return ClipboardType::Windows;
    }

    if cfg!(target_os = "macos") {
        return ClipboardType::MacOS;
    }

    // For Linux/Unix systems
    if std::env::var("WAYLAND_DISPLAY").is_ok() && command_exists("wl-copy") {
        return ClipboardType::Wayland;
    }

    if std::env::var("DISPLAY").is_ok() && command_exists("xclip") {
        return ClipboardType::X11;
    }

    ClipboardType::Unsupported
}

fn command_exists(cmd: &str) -> bool {
    Command::new("which")
        .arg(cmd)
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

async fn copy_to_clipboard_native(content: &str) -> Result<()> {
    let clipboard_type = detect_clipboard_system();
    debug!("Detected clipboard system: {:?}", clipboard_type);

    let (cmd, args): (&str, Vec<&str>) = match clipboard_type {
        ClipboardType::Wayland => ("wl-copy", vec![]),
        ClipboardType::X11 => ("xclip", vec!["-selection", "clipboard"]),
        ClipboardType::MacOS => ("pbcopy", vec![]),
        ClipboardType::Windows => ("clip", vec![]),
        ClipboardType::Unsupported => {
            return Err(anyhow::anyhow!(
                "No supported clipboard system found. Install:\n\
                - Wayland: wl-clipboard\n\
                - X11: xclip\n\
                - Or use --output to save to file"
            ));
        }
    };

    let mut child = Command::new(cmd)
        .args(&args)
        .stdin(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| anyhow::anyhow!("Failed to spawn {}: {}", cmd, e))?;

    if let Some(stdin) = child.stdin.as_mut() {
        use std::io::Write;
        stdin
            .write_all(content.as_bytes())
            .map_err(|e| anyhow::anyhow!("Failed to write to {} stdin: {}", cmd, e))?;
    }

    let status = child
        .wait()
        .map_err(|e| anyhow::anyhow!("Failed to wait for {}: {}", cmd, e))?;

    if !status.success() {
        return Err(anyhow::anyhow!("{} failed with status: {}", cmd, status));
    }

    info!("Content copied to clipboard using {}", cmd);
    println!("Content copied to clipboard");
    Ok(())
}

pub async fn copy_to_clipboard(content: &str) -> Result<()> {
    debug!("Copying {} characters to clipboard", content.len());
    copy_to_clipboard_native(content).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[tokio::test]
    async fn test_simple_update() -> Result<()> {
        let mut temp_file = NamedTempFile::new()?;
        writeln!(temp_file, "line 1")?;
        writeln!(temp_file, "line 2")?;
        writeln!(temp_file, "line 3")?;

        let update = FileUpdate {
            path: temp_file.path().to_string_lossy().to_string(),
            updates: vec![LineUpdate {
                line_start: 2,
                line_end: 2,
                old_content: "line 2".to_string(),
                new_content: "updated line 2".to_string(),
            }],
        };

        let result = process_file_update(&update, false, false).await?;
        assert_eq!(result, 1);

        let content = fs::read_to_string(temp_file.path())?;
        assert!(content.contains("updated line 2"));

        Ok(())
    }

    #[tokio::test]
    async fn test_content_mismatch() -> Result<()> {
        let mut temp_file = NamedTempFile::new()?;
        writeln!(temp_file, "line 1")?;
        writeln!(temp_file, "different content")?;
        writeln!(temp_file, "line 3")?;

        let update = FileUpdate {
            path: temp_file.path().to_string_lossy().to_string(),
            updates: vec![LineUpdate {
                line_start: 2,
                line_end: 2,
                old_content: "line 2".to_string(), // This doesn't match
                new_content: "updated line 2".to_string(),
            }],
        };

        let result = process_file_update(&update, false, false).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Content mismatch"));

        Ok(())
    }
}
