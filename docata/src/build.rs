use crate::{BuildOptions, catalog::Catalog, catalog_presentation, error::Error, scan::scan};
use std::io::Write;
use std::path::Path;

/// Build catalog from documents under `root` and write it to `out`.
///
/// # Errors
///
/// Returns `Error` when scanning fails or JSON serialization fails.
pub fn run<W: Write>(
    root: &Path,
    out: &mut W,
    options: BuildOptions,
) -> Result<(), Error> {
    let entries = scan(root)?;
    let catalog = Catalog::from_entries(&entries);

    catalog_presentation::write_catalog(&catalog, out, options.include_node_metadata)?;
    Ok(())
}
