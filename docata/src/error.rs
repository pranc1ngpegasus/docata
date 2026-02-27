use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("clap parse error: {0}")]
    Clap(String),
    #[error("scan error: {0}")]
    Scan(#[from] crate::scan::ScanError),
    #[error("catalog error: {0}")]
    Catalog(#[from] crate::catalog::CatalogError),
}
