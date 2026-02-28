use crate::{catalog::Catalog, error::Error, format::OutputFormat, graph::Graph};
use std::io::Write;

pub use crate::domain::RelationKind;

/// Run relation command and write formatted output to the provided writer.
///
/// # Errors
///
/// Returns `Error` when response construction or writing fails.
pub fn run<W: Write>(
    query_id: &str,
    catalog: &Catalog,
    graph: &Graph,
    relation_kind: RelationKind,
    format: OutputFormat,
    out: &mut W,
) -> Result<(), Error> {
    let response = crate::domain::build_relation(query_id, catalog, graph, relation_kind);

    crate::relation_presentation::write(&response, format, out)?;

    Ok(())
}
