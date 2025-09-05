use anyhow::Result;
use catnip::cli::commands::{cat, patch};
use catnip::cli::{Args, Commands, Parser};

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
            cat::execute(
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
            patch::execute(json_file, dry_run, backup).await?;
        }
    }

    Ok(())
}
