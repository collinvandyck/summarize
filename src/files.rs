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

#[derive(Debug)]
pub struct Glob {
    original: String,
    glob: Pattern,
    negate: bool,
}

impl Glob {
    pub fn parse(mut s: &str) -> Result<Self> {
        let mut negate = false;
        if s.starts_with("!") {
            negate = true;
            s = &s[1.min(s.len() - 1)..];
        }
        let glob = Pattern::new(s).with_context(|| format!("could not parse glob '{s}'"))?;
        let original = s.to_string();
        Ok(Self { original, glob, negate })
    }
    pub fn matches(&self, p: &Path) -> bool {
        println!("matches {self:#?}");
        let path_only = self.original.contains("**");
        let mut res = self.glob.matches_path(p);
        if !res && !path_only {
            res = p
                .file_name()
                .and_then(|oss| oss.to_str())
                .map(|p| self.glob.matches(p))
                .unwrap_or_default()
        }
        if self.negate { !res } else { res }
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

/// if there are any globs, all of them must match
fn globs_match(path: &Path, globs: &[Glob]) -> bool {
    globs.is_empty() || globs.iter().all(|g| g.matches(path))
}

/// any ft matching is ok
fn fts_match(path: &Path, fts: &[String]) -> bool {
    fts.is_empty()
        || fts.iter().any(|ft| {
            path.extension()
                .and_then(|e| e.to_str())
                .map(|ext| ext == ft)
                .unwrap_or_default()
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    mod globs {
        use super::*;

        #[test]
        fn test_filenames() {
            let path = PathBuf::from("README.md");
            for glob in ["README.md", "*"] {
                let gp = Glob::parse(glob).unwrap();
                assert!(gp.matches(&path), "expected '{glob}' to match {path:?}");
            }
            for glob in ["!*"] {
                let gp = Glob::parse(glob).unwrap();
                assert!(!gp.matches(&path), "did n ot e '{glob}' to match {path:?}");
            }
        }

        #[test]
        fn test_full_paths() {
            let path = PathBuf::from("/foo/bar/baz/README.md");
            for glob in [
                "README.md",
                "README*",
                "*.md",
                "**/README.md",
                "**/bar/**/*.md",
                "*baz/README*",
                "**/foo/**",
                "!**/foobar/**",
            ] {
                let gp = Glob::parse(glob).unwrap();
                assert!(gp.matches(&path), "expected '{glob}' to match {path:?}");
            }
            for glob in ["bar", "baz/README.md", "bar/**"] {
                let gp = Glob::parse(glob).unwrap();
                assert!(!gp.matches(&path), "did not expect '{glob}' to match {path:?}");
            }
        }
    }
}
