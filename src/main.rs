use anyhow::{Context, Result};
use clap::Parser;
use std::path::PathBuf;
use summarize::Args;
use tracing_subscriber::{EnvFilter, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    init_tracing(&args);
    summarize::run(args).await
}

fn init_tracing(args: &Args) -> Result<()> {
    let filter = if args.verbose {
        EnvFilter::from_default_env().add_directive(
            "summarize=debug"
                .parse()
                .context("could not parse env filter")?,
        )
    } else {
        EnvFilter::from_default_env()
    };
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_env_filter(filter)
        .init();
    Ok(())
}
