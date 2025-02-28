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
    if args.verbose {
        let filter = EnvFilter::from_default_env();
        let directive = "summarize=debug"
            .parse()
            .context("could not parse env filter")?;
        let filter = filter.add_directive(directive);
        tracing_subscriber::fmt()
            .with_env_filter(filter)
            .init();
    } else {
        tracing_subscriber::fmt().init();
    }
    Ok(())
}
