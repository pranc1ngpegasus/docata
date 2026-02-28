use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("scan error: {0}")]
    Scan(#[from] crate::scan::ScanError),
    #[error("catalog presentation error: {0}")]
    CatalogPresentation(#[from] crate::catalog_presentation::CatalogPresentationError),
    #[error("relation presentation error: {0}")]
    RelationPresentation(#[from] crate::relation_presentation::RelationPresentationError),
    #[error("{0}")]
    Validation(#[from] crate::validate::ValidationError),
    #[error("query id '{query_id}' was not found in catalog (strict mode)")]
    QueryIdNotFound { query_id: String },
    #[error("catalog check failed: regenerated output differs from '{catalog_path}'")]
    CatalogDiff { catalog_path: String },
}
