use crate::{catalog::Catalog, catalog_presentation, error::Error, scan::scan};
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
) -> Result<(), Error> {
    let entries = scan(root)?;
    let catalog = Catalog::from_entries(&entries);

    catalog_presentation::write_catalog(&catalog, out)?;
    Ok(())
}
