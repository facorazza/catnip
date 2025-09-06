use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "catnip")]
#[command(about = "Concatenate and patch codebases")]
pub struct Args {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Concatenate files content with directory structure
    Cat {
        /// Paths to process
        paths: Vec<PathBuf>,

        /// Output file name (optional)
        #[arg(short = 'o', long)]
        output: Option<String>,

        /// Don't copy to clipboard
        #[arg(long)]
        no_copy: bool,

        /// Additional patterns to exclude
        #[arg(short = 'e', long)]
        exclude: Vec<String>,

        /// Additional patterns to include
        #[arg(short = 'i', long)]
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
        #[arg(short = 'b', long)]
        backup: bool,
    },
}
