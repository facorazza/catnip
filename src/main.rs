use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;
use tracing::{error, info};

use catnip::clipboard::copy_to_clipboard;
use catnip::file_processor;
use catnip::patch::execute_patch;
use catnip::prompt;

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
