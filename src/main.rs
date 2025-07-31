mod exclusion_patterns;
mod file_processor;
mod pattern_matcher;
mod structure_generator;

use anyhow::Result;
use clap::Parser;
use copypasta::{ClipboardContext, ClipboardProvider};
use std::path::PathBuf;

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

fn main() -> Result<()> {
    env_logger::init();

    let args = Args::parse();

    if args.paths.is_empty() {
        eprintln!("No paths provided");
        std::process::exit(1);
    }

    let files = file_processor::get_files_recursively(
        &args.paths,
        &args.exclude,
        &args.include,
        args.ignore_comments,
        args.ignore_docstrings,
    )?;

    let result = file_processor::concatenate_files(&files, args.output.as_deref())?;

    if args.copy {
        copy_to_clipboard(&result)?;
    } else if args.output.is_none() {
        println!("Copy to clipboard? (y/N): ");
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        if input.trim().to_lowercase() == "y" {
            copy_to_clipboard(&result)?;
        }
    }

    Ok(())
}

fn copy_to_clipboard(content: &str) -> Result<()> {
    let mut ctx = ClipboardContext::new()?;
    ctx.set_contents(content.to_owned())?;
    println!("Content copied to clipboard");
    Ok(())
}
