use crate::scan::Entry;
use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Serialize)]
pub struct Catalog {
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
}

#[derive(Debug, Serialize)]
pub struct Node {
    pub id: String,
    pub path: String,
}

#[derive(Debug, Serialize)]
pub struct Edge {
    pub from: String,
    pub to: String,
}

#[derive(Debug, Error)]
pub enum CatalogError {
    #[error("file error: {0}")]
    File(#[from] std::io::Error),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
}

impl Catalog {
    #[must_use]
    pub fn from_entries(entries: &[Entry]) -> Self {
        let nodes = entries
            .iter()
            .map(|entry| Node {
                id: entry.id.clone(),
                path: entry.path.to_string_lossy().to_string(),
            })
            .collect();

        let mut edges = Vec::new();
        for entry in entries {
            for dep in &entry.deps {
                edges.push(Edge {
                    from: entry.id.clone(),
                    to: dep.clone(),
                });
            }
        }

        Catalog { nodes, edges }
    }

    /// Writes the catalog to `path` as pretty-printed JSON.
    ///
    /// # Errors
    ///
    /// Returns an error when file creation/writing fails or JSON serialization fails.
    pub fn write(
        &self,
        path: &std::path::Path,
    ) -> Result<(), CatalogError> {
        let file = std::fs::File::create(path)?;

        serde_json::to_writer_pretty(file, self)?;

        Ok(())
    }
}
