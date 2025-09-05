use crate::core::structure_generator::generate_directory_structure;
use crate::utils::language_detection::get_language_from_extension;
use crate::utils::text_processing::remove_comments_and_docstrings;
use anyhow::Result;
use std::path::PathBuf;
use tokio::fs;
use tracing::{debug, instrument, warn};

#[instrument(skip(files))]
pub async fn concatenate_files(
    files: &[PathBuf],
    output_file: Option<&str>,
    ignore_comments: bool,
    ignore_docstrings: bool,
) -> Result<String> {
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
        println!("Output written to: {}", output_path);
    }

    Ok(result)
}
