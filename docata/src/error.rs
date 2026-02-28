use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("clap parse error: {0}")]
    Clap(String),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("scan error: {0}")]
    Scan(#[from] crate::scan::ScanError),
    #[error("catalog presentation error: {0}")]
    CatalogPresentation(#[from] crate::catalog_presentation::CatalogPresentationError),
    #[error("relation presentation error: {0}")]
    RelationPresentation(#[from] crate::relation_presentation::RelationPresentationError),
}
