use crate::scan::Entry;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Catalog {
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
}

#[derive(Debug, Deserialize)]
pub struct Node {
    pub id: String,
    pub path: String,
}

#[derive(Debug, Deserialize)]
pub struct Edge {
    pub from: String,
    pub to: String,
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
}
