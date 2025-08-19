use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tracing::{debug, error, info, warn};

use crate::clipboard::read_from_clipboard;

#[derive(Debug, Deserialize, Serialize)]
pub struct UpdateRequest {
    pub analysis: String,
    pub files: Vec<FileUpdate>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct FileUpdate {
    pub path: String,
    pub updates: Vec<CodeUpdate>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CodeUpdate {
    pub old_content: String,
    pub new_content: String,
    #[serde(default)]
    pub description: Option<String>,
}

pub async fn execute_patch(json_file: Option<String>, dry_run: bool, backup: bool) -> Result<()> {
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

    // Check if this is a file creation operation
    let is_file_creation = file_update.updates.iter().all(|u| u.old_content.is_empty());

    if is_file_creation {
        if file_path.exists() {
            return Err(anyhow::anyhow!(
                "Cannot create file - already exists: {}",
                file_path.display()
            ));
        }

        // Create parent directories if they don't exist
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent).with_context(|| {
                format!(
                    "Failed to create parent directories for: {}",
                    file_path.display()
                )
            })?;
        }

        // For file creation, concatenate all new_content
        let content: String = file_update
            .updates
            .iter()
            .map(|u| u.new_content.as_str())
            .collect::<Vec<_>>()
            .join("");

        if dry_run {
            info!("DRY RUN: Would create new file: {}", file_path.display());
            println!("\n--- New File: {} ---", file_path.display());
            println!("{}", content);
            return Ok(file_update.updates.len());
        }

        fs::write(&file_path, &content)
            .with_context(|| format!("Failed to create file: {}", file_path.display()))?;

        info!("Created new file: {}", file_path.display());
        return Ok(file_update.updates.len());
    }

    // Existing file update logic
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
