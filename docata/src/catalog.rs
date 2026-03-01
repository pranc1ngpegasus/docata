use crate::scan::Entry;
use serde::Deserialize;
use std::path::{Component, Path};

#[derive(Debug, Deserialize)]
pub struct Catalog {
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
}

#[derive(Debug, Deserialize)]
pub struct Node {
    pub id: String,
    pub path: String,
    #[serde(default, rename = "type")]
    pub kind: Option<String>,
    #[serde(default)]
    pub domain: Option<String>,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub source_of_truth: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd)]
pub struct Edge {
    pub from: String,
    pub to: String,
}

impl Catalog {
    #[must_use]
    pub fn from_entries(entries: &[Entry]) -> Self {
        let mut nodes = entries
            .iter()
            .map(|entry| Node {
                id: entry.id.clone(),
                path: normalize_path_string(&entry.path),
                kind: entry.node_type.clone(),
                domain: entry.domain.clone(),
                status: entry.status.clone(),
                source_of_truth: entry.source_of_truth.clone(),
            })
            .collect::<Vec<_>>();
        nodes.sort_by(|left, right| {
            left.id
                .cmp(&right.id)
                .then(left.path.cmp(&right.path))
                .then(left.kind.cmp(&right.kind))
                .then(left.domain.cmp(&right.domain))
                .then(left.status.cmp(&right.status))
                .then(left.source_of_truth.cmp(&right.source_of_truth))
        });

        let mut edges = Vec::new();
        for entry in entries {
            for dep in &entry.deps {
                edges.push(Edge {
                    from: entry.id.clone(),
                    to: dep.clone(),
                });
            }
        }
        edges.sort();
        edges.dedup();

        Catalog { nodes, edges }
    }
}

fn normalize_path_string(path: &Path) -> String {
    let mut prefix = None::<String>;
    let mut has_root = false;
    let mut parts: Vec<String> = Vec::new();

    for component in path.components() {
        match component {
            Component::Prefix(prefix_component) => {
                prefix = Some(prefix_component.as_os_str().to_string_lossy().to_string());
            },
            Component::RootDir => {
                has_root = true;
                parts.clear();
            },
            Component::CurDir => {},
            Component::ParentDir => {
                if has_root {
                    if !parts.is_empty() {
                        parts.pop();
                    }
                } else if parts.last().is_some_and(|part| part != "..") {
                    parts.pop();
                } else {
                    parts.push("..".to_owned());
                }
            },
            Component::Normal(component) => {
                parts.push(component.to_string_lossy().to_string());
            },
        }
    }

    let mut normalized = String::new();

    if let Some(prefix) = prefix {
        normalized.push_str(&prefix);
    }

    if has_root {
        normalized.push('/');
    }

    normalized.push_str(&parts.join("/"));

    if normalized.is_empty() {
        ".".to_owned()
    } else {
        normalized
    }
}

#[cfg(test)]
mod tests {
    use super::{Catalog, Edge};
    use crate::scan::Entry;
    use std::path::PathBuf;

    fn entry(
        id: &str,
        deps: &[&str],
        path: &str,
    ) -> Entry {
        Entry {
            id: id.to_owned(),
            deps: deps.iter().map(ToString::to_string).collect(),
            path: PathBuf::from(path),
            node_type: Some("note".to_owned()),
            domain: Some("engineering".to_owned()),
            status: Some("published".to_owned()),
            source_of_truth: Some("docs".to_owned()),
        }
    }

    #[test]
    fn normalizes_paths_and_sorts_output() {
        let entries = vec![
            entry("zeta", &["alpha", "alpha"], "./docs/./zeta.md"),
            entry("alpha", &["zeta"], "docs/alpha.md"),
        ];

        let catalog = Catalog::from_entries(&entries);

        assert_eq!(catalog.nodes.len(), 2);
        assert_eq!(catalog.nodes[0].id, "alpha");
        assert_eq!(catalog.nodes[0].path, "docs/alpha.md");
        assert_eq!(catalog.nodes[1].id, "zeta");
        assert_eq!(catalog.nodes[1].path, "docs/zeta.md");

        assert_eq!(
            catalog.edges,
            vec![
                Edge {
                    from: "alpha".to_owned(),
                    to: "zeta".to_owned(),
                },
                Edge {
                    from: "zeta".to_owned(),
                    to: "alpha".to_owned(),
                },
            ]
        );
    }

    #[test]
    fn includes_node_metadata_fields() {
        let entries = vec![entry("alpha", &[], "docs/alpha.md")];

        let catalog = Catalog::from_entries(&entries);
        assert_eq!(catalog.nodes[0].kind.as_deref(), Some("note"));
        assert_eq!(catalog.nodes[0].domain.as_deref(), Some("engineering"));
        assert_eq!(catalog.nodes[0].status.as_deref(), Some("published"));
        assert_eq!(catalog.nodes[0].source_of_truth.as_deref(), Some("docs"));
    }
}
