use anyhow::Result;
use std::path::PathBuf;
use tracing::{error, info};

use crate::config::prompt::PROMPT;
use crate::core::content_processor::concatenate_files;
use crate::core::file_collector::collect_files;
use crate::io::clipboard::copy_to_clipboard;

#[allow(clippy::too_many_arguments)]
pub async fn execute(
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

    let files = collect_files(&paths, &exclude, &include, max_size_mb).await?;

    info!("Found {} files to process", files.len());

    let mut result = concatenate_files(
        &files,
        output.as_deref(),
        ignore_comments,
        ignore_docstrings,
    )
    .await?;

    // Add prompt instructions if requested
    if prompt {
        result = format!(
            "{}
{}",
            result, PROMPT
        );
        info!("Added prompt instructions from constant");
    }

    // Copy to clipboard by default unless --no-copy is specified or output file is provided
    if !no_copy && output.is_none() {
        copy_to_clipboard(&result).await?;
    }

    info!("Processing completed successfully");
    Ok(())
}
