use rayon::prelude::*;
use serde::Deserialize;
use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::{Path, PathBuf},
};
use thiserror::Error;
use walkdir::WalkDir;

#[derive(Debug)]
pub struct Entry {
    pub id: String,
    pub deps: Vec<String>,
    pub path: PathBuf,
}

#[derive(Debug, Error)]
pub enum ScanError {
    #[error("failed to read directory entries in '{root}': {source}")]
    WalkDir {
        root: PathBuf,
        #[source]
        source: walkdir::Error,
    },
    #[error("failed to open file '{path}': {source}")]
    OpenFile {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("failed to read file '{path}': {source}")]
    ReadLine {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("failed to parse yaml frontmatter in '{path}': {source}")]
    ParseYaml {
        path: PathBuf,
        #[source]
        source: yaml_serde::Error,
    },
    #[error("frontmatter is too large in '{path}'")]
    FrontmatterTooLarge { path: PathBuf },
}

/// Scan markdown documents under `root` and extract frontmatter entries.
///
/// # Errors
///
/// Returns `ScanError` when walking the directory, opening files, reading
/// lines, or parsing frontmatter fails.
pub fn scan(root: &Path) -> Result<Vec<Entry>, ScanError> {
    let paths: Vec<PathBuf> = WalkDir::new(root)
        .into_iter()
        .map(|entry| {
            let entry = entry.map_err(|source| ScanError::WalkDir {
                root: root.to_path_buf(),
                source,
            })?;

            if !entry.file_type().is_file() {
                return Ok(None);
            }

            if entry.path().extension().is_some_and(|ext| ext == "md") {
                Ok(Some(entry.into_path()))
            } else {
                Ok(None)
            }
        })
        .collect::<Result<Vec<_>, ScanError>>()?
        .into_iter()
        .flatten()
        .collect();

    let entries: Vec<Option<Entry>> = paths
        .par_iter()
        .map(|path| parse_frontmatter(path))
        .collect::<Result<_, ScanError>>()?;

    Ok(entries.into_iter().flatten().collect())
}

#[derive(Deserialize)]
struct Frontmatter {
    id: String,
    #[serde(default)]
    deps: Vec<String>,
}

fn parse_frontmatter(path: &Path) -> Result<Option<Entry>, ScanError> {
    let file = File::open(path).map_err(|source| ScanError::OpenFile {
        path: path.to_path_buf(),
        source,
    })?;
    let mut reader = BufReader::new(file);

    let mut first_line = String::new();
    reader
        .read_line(&mut first_line)
        .map_err(|source| ScanError::ReadLine {
            path: path.to_path_buf(),
            source,
        })?;

    if first_line.trim() != "---" {
        return Ok(None);
    }

    let mut yaml_buf = String::with_capacity(512);

    loop {
        let mut line = String::new();
        let bytes = reader
            .read_line(&mut line)
            .map_err(|source| ScanError::ReadLine {
                path: path.to_path_buf(),
                source,
            })?;
        if bytes == 0 {
            break;
        }

        if line.trim() == "---" {
            break;
        }

        yaml_buf.push_str(&line);

        if yaml_buf.len() > 32_000 {
            return Err(ScanError::FrontmatterTooLarge {
                path: path.to_path_buf(),
            });
        }
    }

    let fm: Frontmatter =
        yaml_serde::from_str(&yaml_buf).map_err(|source| ScanError::ParseYaml {
            path: path.to_path_buf(),
            source,
        })?;

    Ok(Some(Entry {
        id: fm.id,
        deps: fm.deps,
        path: path.to_path_buf(),
    }))
}
