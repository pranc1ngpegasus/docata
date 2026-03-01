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
mod validate;

pub use error::Error;
pub use format::OutputFormat;
pub use relation::RelationKind;
use std::io::Write;
use std::path::Path;

#[derive(Clone, Copy, Debug, Default)]
pub struct BuildOptions {
    pub include_node_metadata: bool,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct QueryOptions {
    pub strict: bool,
}

/// Build catalog from documents under `root` and write it to `out`.
///
/// # Errors
///
/// Returns `Error` when scanning fails or serialization fails.
pub fn build_catalog<W: Write>(
    root: &Path,
    out: &mut W,
) -> Result<(), Error> {
    build_catalog_with_options(root, out, BuildOptions::default())
}

/// Build catalog from documents under `root` with options and write it to `out`.
///
/// # Errors
///
/// Returns `Error` when scanning fails or serialization fails.
pub fn build_catalog_with_options<W: Write>(
    root: &Path,
    out: &mut W,
    options: BuildOptions,
) -> Result<(), Error> {
    build::run(root, out, options)
}

/// Check document graph structure under `root`.
///
/// # Errors
///
/// Returns `Error` when scanning fails or validation checks fail.
pub fn check_catalog_structure(root: &Path) -> Result<(), Error> {
    let _entries = scan_and_validate(root)?;
    Ok(())
}

/// Check catalog consistency by validating docs and ensuring regenerated output
/// matches `catalog_path`.
///
/// # Errors
///
/// Returns `Error` when scanning fails, validation checks fail, or catalog
/// differs from regenerated output.
pub fn check_catalog(
    root: &Path,
    catalog_path: &Path,
    options: BuildOptions,
) -> Result<(), Error> {
    let entries = scan_and_validate(root)?;
    let catalog = catalog::Catalog::from_entries(&entries);

    let mut regenerated = Vec::new();
    catalog_presentation::write_catalog(&catalog, &mut regenerated, options.include_node_metadata)?;
    let current = std::fs::read(catalog_path)?;

    if current != regenerated {
        return Err(Error::CatalogDiff {
            catalog_path: catalog_path.to_string_lossy().to_string(),
        });
    }

    Ok(())
}

fn scan_and_validate(root: &Path) -> Result<Vec<scan::Entry>, Error> {
    let entries = scan::scan(root)?;
    validate::validate_entries(&entries)?;
    Ok(entries)
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
    query_catalog_relation_with_options(
        query_id,
        catalog_path,
        relation_kind,
        format,
        QueryOptions::default(),
        out,
    )
}

/// Query catalog relations and write output to `out` with options.
///
/// # Errors
///
/// Returns `Error` when reading catalog files or writing output fails.
pub fn query_catalog_relation_with_options<W: Write>(
    query_id: &str,
    catalog_path: &Path,
    relation_kind: RelationKind,
    format: OutputFormat,
    options: QueryOptions,
    out: &mut W,
) -> Result<(), Error> {
    let (catalog, graph) = load_index(catalog_path)?;
    relation::run(
        query_id,
        &catalog,
        &graph,
        relation_kind,
        options.strict,
        format,
        out,
    )
}

#[cfg(test)]
mod tests {
    use super::{
        BuildOptions, Error, OutputFormat, QueryOptions, RelationKind, build_catalog,
        check_catalog, query_catalog_relation_with_options,
    };
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::time::{SystemTime, UNIX_EPOCH};

    struct TestWorkspace {
        root: PathBuf,
    }

    impl TestWorkspace {
        fn new() -> Self {
            let mut root = std::env::temp_dir();
            let timestamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("system time is after epoch")
                .as_nanos();
            root.push(format!("docata-tests-{timestamp}"));
            fs::create_dir_all(&root).expect("create workspace");
            Self { root }
        }

        fn path(&self) -> &Path {
            &self.root
        }
    }

    impl Drop for TestWorkspace {
        fn drop(&mut self) {
            let _result = fs::remove_dir_all(&self.root);
        }
    }

    fn write_markdown(
        root: &Path,
        relative_path: &str,
        id: &str,
        deps: &[&str],
    ) {
        let path = root.join(relative_path);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("create parent directory");
        }

        let mut contents = String::new();
        contents.push_str("---\n");
        contents.push_str("id: ");
        contents.push_str(id);
        contents.push('\n');
        if !deps.is_empty() {
            contents.push_str("deps:\n");
            for dep in deps {
                contents.push_str("  - ");
                contents.push_str(dep);
                contents.push('\n');
            }
        }
        contents.push_str("---\n");

        fs::write(path, contents).expect("write markdown");
    }

    #[test]
    fn strict_query_fails_for_unknown_id() {
        let workspace = TestWorkspace::new();
        let docs = workspace.path().join("docs");
        fs::create_dir_all(&docs).expect("create docs directory");
        write_markdown(&docs, "foo.md", "foo", &[]);

        let catalog_path = workspace.path().join("catalog.json");
        let mut catalog_output = Vec::new();
        build_catalog(&docs, &mut catalog_output).expect("build catalog");
        fs::write(&catalog_path, catalog_output).expect("write catalog");

        let mut output = Vec::new();
        let strict_result = query_catalog_relation_with_options(
            "missing",
            &catalog_path,
            RelationKind::Deps,
            OutputFormat::Json,
            QueryOptions { strict: true },
            &mut output,
        );
        assert!(matches!(
            strict_result,
            Err(Error::QueryIdNotFound { query_id }) if query_id == "missing"
        ));

        let non_strict_result = query_catalog_relation_with_options(
            "missing",
            &catalog_path,
            RelationKind::Deps,
            OutputFormat::Json,
            QueryOptions { strict: false },
            &mut output,
        );
        assert!(non_strict_result.is_ok());
    }

    #[test]
    fn check_catalog_requires_no_regeneration_diff() {
        let workspace = TestWorkspace::new();
        let docs = workspace.path().join("docs");
        fs::create_dir_all(&docs).expect("create docs directory");
        write_markdown(&docs, "foo.md", "foo", &[]);
        write_markdown(&docs, "bar.md", "bar", &["foo"]);

        let catalog_path = workspace.path().join("catalog.json");
        let mut catalog_output = Vec::new();
        build_catalog(&docs, &mut catalog_output).expect("build catalog");
        fs::write(&catalog_path, &catalog_output).expect("write catalog");

        check_catalog(&docs, &catalog_path, BuildOptions::default())
            .expect("check should pass for up-to-date catalog");

        fs::write(&catalog_path, "{}").expect("break catalog content");
        let result = check_catalog(&docs, &catalog_path, BuildOptions::default());
        assert!(matches!(result, Err(Error::CatalogDiff { .. })));
    }
}
