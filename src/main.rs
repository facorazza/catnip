mod exclusion_patterns;
mod file_processor;
mod pattern_matcher;
mod structure_generator;

use anyhow::Result;
use clap::Parser;
use copypasta::{ClipboardContext, ClipboardProvider};
use std::path::PathBuf;
use tracing::{debug, error, info, instrument};

#[derive(Parser)]
#[command(about = "Concatenate files with directory structure and content")]
struct Args {
    /// Paths to process
    paths: Vec<PathBuf>,

    /// Output file name (optional)
    #[arg(short, long)]
    output: Option<String>,

    /// Copy the output to the clipboard
    #[arg(long)]
    copy: bool,

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
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing subscriber
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let args = Args::parse();

    if args.paths.is_empty() {
        error!("No paths provided");
        std::process::exit(1);
    }

    debug!("Processing paths: {:?}", args.paths);

    let files = file_processor::get_files_recursively(
        &args.paths,
        &args.exclude,
        &args.include,
        args.ignore_comments,
        args.ignore_docstrings,
    )
    .await?;

    info!("Found {} files to process", files.len());

    let result = file_processor::concatenate_files(&files, args.output.as_deref()).await?;

    if args.copy {
        copy_to_clipboard(&result).await?;
    } else if args.output.is_none() {
        println!("Copy to clipboard? (y/N): ");
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        if input.trim().to_lowercase() == "y" {
            copy_to_clipboard(&result).await?;
        }
    }

    info!("Processing completed successfully");
    Ok(())
}

#[instrument]
async fn copy_to_clipboard(content: &str) -> Result<()> {
    debug!(
        "Attempting to copy {} characters to clipboard",
        content.len()
    );

    let mut ctx = ClipboardContext::new()
        .map_err(|e| anyhow::anyhow!("Failed to create clipboard context: {}", e))?;
    ctx.set_contents(content.to_owned())
        .map_err(|e| anyhow::anyhow!("Failed to set clipboard contents: {}", e))?;

    info!("Content successfully copied to clipboard");
    println!("Content copied to clipboard");
    Ok(())
}
