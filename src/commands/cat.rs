use anyhow::Result;
use std::path::PathBuf;
use tracing::{error, info};

use crate::clipboard::copy_to_clipboard;
use crate::file_processor;
use crate::prompt;

#[allow(clippy::too_many_arguments)]
pub async fn execute_cat(
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
        result = format!(
            "{}
{}",
            result,
            prompt::PROMPT
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
