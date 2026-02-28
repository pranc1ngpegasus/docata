use crate::catalog::Catalog;
use serde::Serialize;
use std::io::{Read, Write};
use thiserror::Error;

#[derive(Debug, Serialize)]
struct CatalogNode<'a> {
    id: &'a str,
    path: &'a str,
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

impl<'a> From<&'a Catalog> for CatalogView<'a> {
    fn from(catalog: &'a Catalog) -> Self {
        let nodes = catalog
            .nodes
            .iter()
            .map(|node| CatalogNode {
                id: node.id.as_str(),
                path: node.path.as_str(),
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
) -> Result<(), CatalogPresentationError> {
    let view = CatalogView::from(catalog);

    serde_json::to_writer_pretty(out, &view)?;
    Ok(())
}
