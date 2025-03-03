use anyhow::{Context, bail};
use files::{FileInfo, FindOpts};
use futures_util::StreamExt;
use genai::{
    Client,
    chat::{ChatMessage, ChatRequest, ChatResponse, ContentPart, MessageContent},
};
use glob::Pattern;
use itertools::Itertools;
use rand::seq::IndexedRandom;
use std::{
    env,
    path::{Path, PathBuf},
};
use strum::Display;
use tracing::debug;

mod files;

#[derive(clap::Parser, Debug)]
pub struct Args {
    /// The directory to walk. Defaults to the current dir.
    #[arg(long, short)]
    pub dir: Option<PathBuf>,

    /// The file types to inclue (e.g. 'kt', 'rs')
    #[arg(long, short)]
    pub file_types: Vec<String>,

    /// Globs to include.
    #[arg(long, short)]
    pub globs: Vec<String>,

    /// The language model to use
    #[arg(long, short, default_value = "gpt-4o-mini")]
    pub model: String,

    /// Do not make requests to the LLM
    #[arg(long)]
    pub dry_run: bool,

    /// Print extra debugging information
    #[arg(long, short)]
    pub verbose: bool,

    /// The optional prompt
    prompt: Option<String>,
}

trait HumanBytes {
    fn to_human(&self) -> String;
}

impl HumanBytes for usize {
    fn to_human(&self) -> String {
        match *self {
            0..1024 => format!("{self}b"),
            1024..1048576 => format!("{:.2}Kb", ((*self) as f64 / 1024.0)),
            _ => format!("{:.2}Mb", ((*self) as f64 / 1024.0 / 1024.0)),
        }
    }
}

static PROMPT: &str = include_str!("../prompt.md");

#[derive(clap::ValueEnum)]
#[derive(Default, Debug, Clone, Display)]
pub enum ModelKind {
    #[default]
    #[clap(name = "gpt-4o-mini")]
    Gpt4oMini,
}

fn file_block(path: &Path, content: &[u8]) -> String {
    let mut buf = String::new();
    let content = String::from_utf8_lossy(content);
    buf.push_str(&format!("<file path={path:?}>\n"));
    buf.push_str(&format!("{content}\n"));
    buf.push_str("</file>\n");
    buf
}

async fn project_path() -> anyhow::Result<PathBuf> {
    let proj = home::home_dir()
        .context("no home dir found for user")?
        .join(".config")
        .join("summarize");
    tokio::fs::create_dir_all(&proj)
        .await
        .context("ensure project dir")?;
    Ok(proj)
}

pub async fn run(args: Args) -> anyhow::Result<()> {
    let proj = project_path()
        .await
        .context("could not get project path dir")?;
    debug!("Starting run...");
    debug!("Proj dir: {proj:?}");
    debug!("Model: {}", args.model);
    let dir = match args.dir.clone() {
        Some(dir) => dir,
        None => std::env::current_dir().context("could not get current dir")?,
    };
    let globs = args
        .globs
        .iter()
        .map(|glob| files::Glob::parse(glob))
        .collect::<Result<Vec<_>, _>>()?;
    let mut stream = files::stream(FindOpts {
        dir: dir.clone(),
        file_types: args.file_types.clone(),
        globs,
    });
    let mut buf = String::new();
    while let Some(res) = stream.next().await {
        let FileInfo { path, bs } = res?;
        tracing::debug!("Including {path:?} [{}]", bs.len().to_human());
        buf.push_str(&file_block(&path, &bs));
        buf.push_str("\n\n");
    }
    let prompt = PROMPT.replace("FILES_CONTENT", &buf);
    let prompt = prompt.replace("USER_PROMPT", &args.prompt.unwrap_or_default());

    debug!("Created prompt of size {}", prompt.len().to_human());
    //debug!("Prompt:\n{}", prompt);

    tokio::fs::write("request.md", prompt.as_bytes())
        .await
        .context("failed to write generated prompt")?;

    let reqs = ChatRequest::new(vec![ChatMessage::system(&prompt)]);
    let client = Client::default();
    if !args.dry_run {
        let resp: ChatResponse = client
            .exec_chat(&args.model, reqs, None)
            .await
            .context("failed to call model")?;
        let ChatResponse {
            content,
            reasoning_content,
            model_iden,
            usage,
        } = resp;
        let Some(content) = content else {
            bail!("no content received")
        };
        let MessageContent::Text(content) = content else {
            bail!("unexpected response: {content:?}");
        };
        tokio::fs::write("response.md", content.as_bytes())
            .await
            .context("failed to write generated prompt")?;

        println!("{content}");
    }
    Ok(())
}
