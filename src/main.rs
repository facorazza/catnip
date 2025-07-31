mod exclusion_patterns;
mod file_processor;
mod pattern_matcher;
mod structure_generator;

use anyhow::Result;
use clap::Parser;
use copypasta::{ClipboardContext, ClipboardProvider};
use std::path::PathBuf;
use std::process::Command;

use tracing::{debug, error,info, instrument, warn};

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


#[derive(Debug)]
enum ClipboardType {
    Wayland,
    X11,
    MacOS,
    Windows,
    Unsupported,
}

fn detect_clipboard_system() -> ClipboardType {
    // Check for Windows
    if cfg!(target_os = "windows") {
        return ClipboardType::Windows;
    }

    // Check for macOS
    if cfg!(target_os = "macos") {
        return ClipboardType::MacOS;
    }

    // For Linux/Unix systems, check environment
    if std::env::var("WAYLAND_DISPLAY").is_ok() {
        // Check if wl-copy is available
        if Command::new("which")
            .arg("wl-copy")
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
        {
            return ClipboardType::Wayland;
        }
    }

    if std::env::var("DISPLAY").is_ok() {
        // Check if xclip is available
        if Command::new("which")
            .arg("xclip")
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
        {
            return ClipboardType::X11;
        }
    }

    ClipboardType::Unsupported
}

#[instrument]
async fn copy_to_clipboard_native(content: &str) -> Result<()> {
    let clipboard_type = detect_clipboard_system();
    debug!("Detected clipboard system: {:?}", clipboard_type);

    match clipboard_type {
        ClipboardType::Wayland => {
            debug!("Using wl-copy for Wayland");
            let mut child = Command::new("wl-copy")
                .stdin(std::process::Stdio::piped())
                .spawn()
                .map_err(|e| anyhow::anyhow!("Failed to spawn wl-copy: {}", e))?;

            if let Some(stdin) = child.stdin.as_mut() {
                use std::io::Write;
                stdin.write_all(content.as_bytes())
                    .map_err(|e| anyhow::anyhow!("Failed to write to wl-copy stdin: {}", e))?;
            }

            let status = child.wait()
                .map_err(|e| anyhow::anyhow!("Failed to wait for wl-copy: {}", e))?;

            if !status.success() {
                return Err(anyhow::anyhow!("wl-copy failed with status: {}", status));
            }
        },

        ClipboardType::X11 => {
            debug!("Using xclip for X11");
            let mut child = Command::new("xclip")
                .args(["-selection", "clipboard"])
                .stdin(std::process::Stdio::piped())
                .spawn()
                .map_err(|e| anyhow::anyhow!("Failed to spawn xclip: {}", e))?;

            if let Some(stdin) = child.stdin.as_mut() {
                use std::io::Write;
                stdin.write_all(content.as_bytes())
                    .map_err(|e| anyhow::anyhow!("Failed to write to xclip stdin: {}", e))?;
            }

            let status = child.wait()
                .map_err(|e| anyhow::anyhow!("Failed to wait for xclip: {}", e))?;

            if !status.success() {
                return Err(anyhow::anyhow!("xclip failed with status: {}", status));
            }
        },

        ClipboardType::MacOS => {
            debug!("Using pbcopy for macOS");
            let mut child = Command::new("pbcopy")
                .stdin(std::process::Stdio::piped())
                .spawn()
                .map_err(|e| anyhow::anyhow!("Failed to spawn pbcopy: {}", e))?;

            if let Some(stdin) = child.stdin.as_mut() {
                use std::io::Write;
                stdin.write_all(content.as_bytes())
                    .map_err(|e| anyhow::anyhow!("Failed to write to pbcopy stdin: {}", e))?;
            }

            let status = child.wait()
                .map_err(|e| anyhow::anyhow!("Failed to wait for pbcopy: {}", e))?;

            if !status.success() {
                return Err(anyhow::anyhow!("pbcopy failed with status: {}", status));
            }
        },

        ClipboardType::Windows => {
            debug!("Using clip for Windows");
            let mut child = Command::new("clip")
                .stdin(std::process::Stdio::piped())
                .spawn()
                .map_err(|e| anyhow::anyhow!("Failed to spawn clip: {}", e))?;

            if let Some(stdin) = child.stdin.as_mut() {
                use std::io::Write;
                stdin.write_all(content.as_bytes())
                    .map_err(|e| anyhow::anyhow!("Failed to write to clip stdin: {}", e))?;
            }

            let status = child.wait()
                .map_err(|e| anyhow::anyhow!("Failed to wait for clip: {}", e))?;

            if !status.success() {
                return Err(anyhow::anyhow!("clip failed with status: {}", status));
            }
        },

        ClipboardType::Unsupported => {
            return Err(anyhow::anyhow!(
                "No supported clipboard system found. Please install:\n\
                - For Wayland: wl-clipboard (wl-copy)\n\
                - For X11: xclip\n\
                - Or use the --output flag to save to a file instead"
            ));
        }
    }

    info!("Content successfully copied to clipboard using native system");
    println!("Content copied to clipboard");
    Ok(())
}

// Fallback function using copypasta crate
#[instrument]
async fn copy_to_clipboard_fallback(content: &str) -> Result<()> {
    use copypasta::{ClipboardContext, ClipboardProvider};

    debug!("Using copypasta fallback");
    let mut ctx = ClipboardContext::new()
        .map_err(|e| anyhow::anyhow!("Failed to create clipboard context: {}", e))?;

    ctx.set_contents(content.to_owned())
        .map_err(|e| anyhow::anyhow!("Failed to set clipboard contents: {}", e))?;

    info!("Content successfully copied to clipboard using fallback");
    println!("Content copied to clipboard");
    Ok(())
}

#[instrument]
pub async fn copy_to_clipboard(content: &str) -> Result<()> {
    debug!("Attempting to copy {} characters to clipboard", content.len());

    // Try native system clipboard first
    if let Err(e) = copy_to_clipboard_native(content).await {
        warn!("Native clipboard failed: {}, trying fallback", e);

        // Fall back to copypasta crate
        copy_to_clipboard_fallback(content).await?;
    }

    Ok(())
}
