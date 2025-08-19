use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tracing::{debug, error, info, warn};

use catnip::clipboard::{copy_to_clipboard, read_from_clipboard};

mod file_processor;
mod pattern_matcher;
mod patterns;
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
    /// Concatenate files content with directory structure
    Cat {
        /// Paths to process
        paths: Vec<PathBuf>,

        /// Output file name (optional)
        #[arg(short, long)]
        output: Option<String>,

        /// Don't copy to clipboard
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
        /// Include prompt instructions
        #[arg(short = 'p', long = "prompt")]
        prompt: bool,
    },
    /// Apply JSON-formatted code updates to files
    Patch {
        /// JSON file containing updates, '-' to read from stdin, or omit to read from clipboard
        json_file: Option<String>,

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
    updates: Vec<CodeUpdate>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct CodeUpdate {
    old_content: String,
    new_content: String,
    #[serde(default)]
    description: Option<String>,
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
            prompt,
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
                prompt,
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
    prompt: bool,
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

    let mut result = file_processor::concatenate_files(
        &files,
        output.as_deref(),
        ignore_comments,
        ignore_docstrings,
    )
    .await?;

    // Add prompt instructions if requested
    if prompt {
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

async fn execute_patch(json_file: Option<String>, dry_run: bool, backup: bool) -> Result<()> {
    // Read JSON from file, stdin, or clipboard
    let json_content = match json_file.as_deref() {
        Some("-") => {
            use std::io::{self, BufRead};
            let stdin = io::stdin();
            let lines: Result<Vec<_>, _> = stdin.lock().lines().collect();
            lines.context("Failed to read from stdin")?.join("\n")
        }
        Some(file_path) => fs::read_to_string(file_path)
            .with_context(|| format!("Failed to read JSON file: {}", file_path))?,
        None => read_from_clipboard()
            .await
            .context("Failed to read from clipboard")?,
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

    let mut updated_content = original_content.clone();
    let mut applied_updates = 0;

    // Apply updates in order
    for (i, update) in file_update.updates.iter().enumerate() {
        debug!(
            "Applying update {}/{}: {}",
            i + 1,
            file_update.updates.len(),
            update.description.as_deref().unwrap_or("no description")
        );

        if !updated_content.contains(&update.old_content) {
            return Err(anyhow::anyhow!(
                "Old content not found in file. Expected content:\n{}",
                update.old_content
            ));
        }

        // Count occurrences to ensure we're not making ambiguous replacements
        let occurrences = updated_content.matches(&update.old_content).count();
        if occurrences > 1 {
            warn!(
                "Old content appears {} times in file, replacing all occurrences",
                occurrences
            );
        }

        // Replace the old content with new content
        updated_content = updated_content.replace(&update.old_content, &update.new_content);
        applied_updates += 1;
    }

    if dry_run {
        info!(
            "DRY RUN: Would apply {} updates to {}",
            applied_updates,
            file_path.display()
        );

        // Show preview of changes
        println!("\n--- File: {} ---", file_path.display());
        for (i, update) in file_update.updates.iter().enumerate() {
            println!("\n--- Update {} ---", i + 1);
            if let Some(desc) = &update.description {
                println!("Description: {}", desc);
            }
            println!("- OLD:\n{}", update.old_content);
            println!("+ NEW:\n{}", update.new_content);
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
    fs::write(&file_path, &updated_content)
        .with_context(|| format!("Failed to write updated file: {}", file_path.display()))?;

    Ok(applied_updates)
}
