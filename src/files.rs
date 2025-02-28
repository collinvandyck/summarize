use anyhow::{Context, Result, anyhow, bail};
use futures_util::Stream;
use glob::Pattern;
use rand::seq::IndexedRandom;
use std::{
    fs::{DirEntry, ReadDir},
    path::{Path, PathBuf},
};
use tokio::task::spawn_blocking;
use tokio_stream::wrappers::ReceiverStream;
use tracing::debug;

pub struct Glob {
    glob: Pattern,
    negate: bool,
}

impl Glob {
    pub fn parse(mut s: &str) -> Result<Self> {
        let mut negate = false;
        if s.starts_with("!") {
            negate = true;
            if s.len() <= 1 {
                bail!("invalid glob: {s}");
            }
            s = &s[1..];
        }
        let glob = Pattern::new(s).with_context(|| format!("could not parse glob '{s}'"))?;
        Ok(Self { glob, negate })
    }
    fn matches(&self, s: &str) -> bool {
        let m = self.glob.matches(s);
        if self.negate { !m } else { m }
    }
}

pub struct FindOpts {
    pub dir: PathBuf,
    pub file_types: Vec<String>,
    pub globs: Vec<Glob>,
}

pub fn stream(opts: FindOpts) -> impl Stream<Item = Result<FileInfo>> {
    let (tx, rx) = tokio::sync::mpsc::channel(100);
    spawn_blocking(move || {
        let iter = find(opts);
        for file in iter {
            if tx.blocking_send(file).is_err() {
                return;
            }
        }
    });
    ReceiverStream::new(rx)
}

pub fn find(opts: FindOpts) -> impl Iterator<Item = Result<FileInfo>> {
    let walk = ignore::Walk::new(&opts.dir);
    IntoIter {
        walk,
        fts: opts.file_types,
        globs: opts.globs,
    }
}

pub struct FileInfo {
    pub path: PathBuf,
    pub bs: Vec<u8>,
}

pub struct IntoIter {
    walk: ignore::Walk,
    fts: Vec<String>,
    globs: Vec<Glob>,
}

impl Iterator for IntoIter {
    type Item = Result<FileInfo>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let res = self.walk.next()?;
            let entry = match res {
                Ok(entry) => entry,
                Err(err) => return Some(Err(err.into())),
            };
            let path = entry.path();
            if path.is_dir() {
                continue;
            }
            if !fts_match(path, &self.fts) {
                continue;
            }
            if !globs_match(path, &self.globs) {
                continue;
            }
            return Some(
                std::fs::read(path)
                    .context("could not read file")
                    .map(|bs| {
                        FileInfo {
                            path: path.to_path_buf(),
                            bs,
                        }
                    }),
            );
        }
    }
}

fn globs_match(path: &Path, globs: &[Glob]) -> bool {
    if globs.is_empty() {
        return true;
    }
    globs.iter().any(|pat| {
        path.to_str()
            .map(|path| pat.matches(path))
            .unwrap_or_default()
    })
}

fn fts_match(path: &Path, fts: &[String]) -> bool {
    if fts.is_empty() {
        return true;
    }
    fts.iter().any(|ft| {
        path.extension()
            .and_then(|e| e.to_str())
            .map(|ext| ext == ft)
            .unwrap_or_default()
    })
}
