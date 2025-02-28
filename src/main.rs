use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;
use summarize::Args;

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    summarize::run(args.into()).await
}
