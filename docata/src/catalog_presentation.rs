use crate::catalog::Catalog;
use serde::Serialize;
use std::io::{Read, Write};
use thiserror::Error;

#[derive(Debug, Serialize)]
struct CatalogNodeBasic<'a> {
    id: &'a str,
    path: &'a str,
}

#[derive(Debug, Serialize)]
struct CatalogNodeWithMetadata<'a> {
    id: &'a str,
    path: &'a str,
    #[serde(rename = "type")]
    kind: Option<&'a str>,
    domain: Option<&'a str>,
    status: Option<&'a str>,
    source_of_truth: Option<&'a str>,
}

#[derive(Debug, Serialize)]
#[serde(untagged)]
enum CatalogNode<'a> {
    Basic(CatalogNodeBasic<'a>),
    WithMetadata(CatalogNodeWithMetadata<'a>),
}

#[derive(Debug, Serialize)]
struct CatalogEdge<'a> {
    from: &'a str,
    to: &'a str,
}

#[derive(Debug, Serialize)]
struct CatalogView<'a> {
    nodes: Vec<CatalogNode<'a>>,
    edges: Vec<CatalogEdge<'a>>,
}

impl<'a> CatalogView<'a> {
    fn from_catalog(
        catalog: &'a Catalog,
        include_node_metadata: bool,
    ) -> Self {
        let nodes = catalog
            .nodes
            .iter()
            .map(|node| {
                if include_node_metadata {
                    CatalogNode::WithMetadata(CatalogNodeWithMetadata {
                        id: node.id.as_str(),
                        path: node.path.as_str(),
                        kind: node.kind.as_deref(),
                        domain: node.domain.as_deref(),
                        status: node.status.as_deref(),
                        source_of_truth: node.source_of_truth.as_deref(),
                    })
                } else {
                    CatalogNode::Basic(CatalogNodeBasic {
                        id: node.id.as_str(),
                        path: node.path.as_str(),
                    })
                }
            })
            .collect();

        let edges = catalog
            .edges
            .iter()
            .map(|edge| CatalogEdge {
                from: edge.from.as_str(),
                to: edge.to.as_str(),
            })
            .collect();

        Self { nodes, edges }
    }
}

#[derive(Debug, Error)]
pub enum CatalogPresentationError {
    #[error("json encoding error: {0}")]
    Json(#[from] serde_json::Error),
}

/// Read catalog JSON from the provided reader.
///
/// # Errors
///
/// Returns `CatalogPresentationError` when deserialization fails.
pub fn read_catalog<R: Read>(input: &mut R) -> Result<Catalog, CatalogPresentationError> {
    let catalog = serde_json::from_reader(input)?;
    Ok(catalog)
}

/// Write catalog JSON to the provided writer.
///
/// # Errors
///
/// Returns `CatalogPresentationError` when serialization or output fails.
pub fn write_catalog<W: Write>(
    catalog: &Catalog,
    out: &mut W,
    include_node_metadata: bool,
) -> Result<(), CatalogPresentationError> {
    let view = CatalogView::from_catalog(catalog, include_node_metadata);

    serde_json::to_writer_pretty(out, &view)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::write_catalog;
    use crate::catalog::{Catalog, Edge, Node};

    fn catalog_fixture() -> Catalog {
        Catalog {
            nodes: vec![Node {
                id: "foo".to_owned(),
                path: "docs/foo.md".to_owned(),
                kind: Some("spec".to_owned()),
                domain: Some("billing".to_owned()),
                status: Some("draft".to_owned()),
                source_of_truth: Some("handbook".to_owned()),
            }],
            edges: vec![Edge {
                from: "foo".to_owned(),
                to: "bar".to_owned(),
            }],
        }
    }

    #[test]
    fn writes_basic_node_without_metadata_fields() {
        let catalog = catalog_fixture();
        let mut output = Vec::new();
        write_catalog(&catalog, &mut output, false).expect("write catalog");

        let json = String::from_utf8(output).expect("valid utf-8");
        assert!(json.contains("\"id\": \"foo\""));
        assert!(json.contains("\"path\": \"docs/foo.md\""));
        assert!(!json.contains("\"type\""));
        assert!(!json.contains("\"domain\""));
        assert!(!json.contains("\"status\""));
        assert!(!json.contains("\"source_of_truth\""));
    }

    #[test]
    fn writes_node_with_metadata_fields_when_enabled() {
        let catalog = catalog_fixture();
        let mut output = Vec::new();
        write_catalog(&catalog, &mut output, true).expect("write catalog");

        let json = String::from_utf8(output).expect("valid utf-8");
        assert!(json.contains("\"type\": \"spec\""));
        assert!(json.contains("\"domain\": \"billing\""));
        assert!(json.contains("\"status\": \"draft\""));
        assert!(json.contains("\"source_of_truth\": \"handbook\""));
    }
}
