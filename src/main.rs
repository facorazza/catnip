mod exclusion_patterns;
mod file_processor;
mod pattern_matcher;
mod structure_generator;

use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;
use std::process::Command;
use tracing::{debug, error, info, instrument, warn};

#[derive(Parser)]
#[command(about = "Concatenate files with directory structure and content")]
struct Args {
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

    if args.paths.is_empty() {
        error!("No paths provided");
        std::process::exit(1);
    }

    let files = file_processor::get_files_recursively(
        &args.paths,
        &args.exclude,
        &args.include,
        args.ignore_comments,
        args.ignore_docstrings,
        args.max_size_mb,
    )
    .await?;

    info!("Found {} files to process", files.len());

    let result = file_processor::concatenate_files(&files, args.output.as_deref()).await?;

    // Copy to clipboard by default unless --no-copy is specified or output file is provided
    if !args.no_copy && args.output.is_none() {
        copy_to_clipboard(&result).await?;
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

#[instrument]
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

#[instrument]
async fn copy_to_clipboard_fallback(content: &str) -> Result<()> {
    use copypasta::{ClipboardContext, ClipboardProvider};

    debug!("Using copypasta fallback");
    let mut ctx = ClipboardContext::new()
        .map_err(|e| anyhow::anyhow!("Failed to create clipboard context: {}", e))?;

    ctx.set_contents(content.to_owned())
        .map_err(|e| anyhow::anyhow!("Failed to set clipboard contents: {}", e))?;

    info!("Content copied to clipboard using fallback");
    println!("Content copied to clipboard");
    Ok(())
}

#[instrument]
pub async fn copy_to_clipboard(content: &str) -> Result<()> {
    debug!("Copying {} characters to clipboard", content.len());

    if let Err(e) = copy_to_clipboard_native(content).await {
        warn!("Native clipboard failed: {}, trying fallback", e);
        copy_to_clipboard_fallback(content).await?;
    }

    Ok(())
}
