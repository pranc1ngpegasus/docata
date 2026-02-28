mod build;
mod catalog;
mod catalog_presentation;
mod domain;
mod error;
mod format;
mod graph;
mod relation;
mod relation_presentation;
mod scan;

pub use error::Error;
pub use format::OutputFormat;
pub use relation::RelationKind;
use std::io::Write;
use std::path::Path;

/// Build catalog from documents under `root` and write it to `out`.
///
/// # Errors
///
/// Returns `Error` when scanning fails or serialization fails.
pub fn build_catalog<W: Write>(
    root: &Path,
    out: &mut W,
) -> Result<(), Error> {
    build::run(root, out)
}

fn load_index(catalog_path: &Path) -> Result<(catalog::Catalog, graph::Graph), Error> {
    let mut file = std::fs::File::open(catalog_path)?;
    let catalog = catalog_presentation::read_catalog(&mut file)?;
    let graph = graph::Graph::from_catalog(&catalog);

    Ok((catalog, graph))
}

/// Query catalog relations and write output to `out`.
///
/// # Errors
///
/// Returns `Error` when reading catalog files or writing output fails.
pub fn query_catalog_relation<W: Write>(
    query_id: &str,
    catalog_path: &Path,
    relation_kind: RelationKind,
    format: OutputFormat,
    out: &mut W,
) -> Result<(), Error> {
    let (catalog, graph) = load_index(catalog_path)?;
    relation::run(query_id, &catalog, &graph, relation_kind, format, out)
}
