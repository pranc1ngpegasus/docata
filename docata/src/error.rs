use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("clap parse error: {0}")]
    Clap(String),
    #[error("json encoding error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("scan error: {0}")]
    Scan(#[from] crate::scan::ScanError),
    #[error("catalog error: {0}")]
    Catalog(#[from] crate::catalog::CatalogError),
}
