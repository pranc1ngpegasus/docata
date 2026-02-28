use crate::{catalog::Catalog, graph::Graph};
use std::collections::HashMap;

pub type RelationResolver = fn(&Graph, &str) -> Vec<String>;

#[derive(Clone, Copy, Debug)]
pub enum RelationKind {
    Deps,
    Refs,
}

impl RelationKind {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            RelationKind::Deps => "deps",
            RelationKind::Refs => "refs",
        }
    }

    #[must_use]
    pub const fn resolver(self) -> RelationResolver {
        match self {
            RelationKind::Deps => Graph::deps,
            RelationKind::Refs => Graph::refs,
        }
    }
}

#[derive(Debug)]
pub struct RelationItem {
    pub id: String,
    pub path: Option<String>,
    pub resolved: bool,
}

#[derive(Debug)]
pub struct RelationMeta {
    pub missing_nodes: Vec<String>,
}

#[derive(Debug)]
pub struct RelationResponse {
    pub command: RelationKind,
    pub query_id: String,
    pub count: usize,
    pub items: Vec<RelationItem>,
    pub meta: RelationMeta,
}

/// Build relation output from an already-created catalog.
#[must_use]
pub fn build_relation(
    query_id: &str,
    catalog: &Catalog,
    graph: &Graph,
    relation_kind: RelationKind,
) -> RelationResponse {
    let mut ids = (relation_kind.resolver())(graph, query_id);

    ids.sort();
    ids.dedup();

    let node_paths = catalog
        .nodes
        .iter()
        .map(|node| (node.id.as_str(), node.path.as_str()))
        .collect::<HashMap<_, _>>();

    let mut missing_nodes = Vec::new();
    let mut items = Vec::with_capacity(ids.len());

    for id in ids {
        if let Some(path) = node_paths.get(id.as_str()) {
            items.push(RelationItem {
                id,
                path: Some((*path).to_owned()),
                resolved: true,
            });
        } else {
            missing_nodes.push(id.clone());
            items.push(RelationItem {
                id,
                path: None,
                resolved: false,
            });
        }
    }

    missing_nodes.sort();

    RelationResponse {
        command: relation_kind,
        query_id: query_id.to_owned(),
        count: items.len(),
        items,
        meta: RelationMeta { missing_nodes },
    }
}
