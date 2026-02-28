use crate::domain::{RelationItem, RelationMeta, RelationResponse};
use crate::format::OutputFormat;
use serde::Serialize;
use std::io::Write;
use thiserror::Error;

#[derive(Debug, Serialize)]
struct RelationItemJson {
    id: String,
    path: Option<String>,
    resolved: bool,
}

impl From<&RelationItem> for RelationItemJson {
    fn from(item: &RelationItem) -> Self {
        Self {
            id: item.id.clone(),
            path: item.path.clone(),
            resolved: item.resolved,
        }
    }
}

#[derive(Debug, Serialize)]
struct RelationMetaJson {
    missing_nodes: Vec<String>,
}

impl From<&RelationMeta> for RelationMetaJson {
    fn from(meta: &RelationMeta) -> Self {
        Self {
            missing_nodes: meta.missing_nodes.clone(),
        }
    }
}

#[derive(Debug, Serialize)]
struct RelationResponseJson {
    command: String,
    query_id: String,
    count: usize,
    items: Vec<RelationItemJson>,
    meta: RelationMetaJson,
}

impl From<&RelationResponse> for RelationResponseJson {
    fn from(response: &RelationResponse) -> Self {
        let items = response.items.iter().map(RelationItemJson::from).collect();

        Self {
            command: response.command.as_str().to_owned(),
            query_id: response.query_id.clone(),
            count: response.count,
            items,
            meta: RelationMetaJson::from(&response.meta),
        }
    }
}

#[derive(Debug, Error)]
pub enum RelationPresentationError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("json encoding error: {0}")]
    Json(#[from] serde_json::Error),
}

/// Write a relation response according to the selected output format.
///
/// # Errors
///
/// Returns `RelationPresentationError` if JSON serialization or writing fails.
pub fn write<W: Write>(
    response: &RelationResponse,
    format: OutputFormat,
    out: &mut W,
) -> Result<(), RelationPresentationError> {
    match format {
        OutputFormat::Text => write_text(response, out),
        OutputFormat::Json => write_json(response, out),
    }
}

/// Write a relation response as JSON to the provided writer.
///
/// # Errors
///
/// Returns `RelationPresentationError` if JSON serialization fails.
pub fn write_json<W: Write>(
    response: &RelationResponse,
    out: &mut W,
) -> Result<(), RelationPresentationError> {
    let response_json = RelationResponseJson::from(response);

    serde_json::to_writer_pretty(&mut *out, &response_json)?;
    writeln!(out)?;
    Ok(())
}

/// Write a relation response as line-delimited text to the provided writer.
///
/// # Errors
///
/// Returns `RelationPresentationError` if writing fails.
pub fn write_text<W: Write>(
    response: &RelationResponse,
    out: &mut W,
) -> Result<(), RelationPresentationError> {
    for item in &response.items {
        writeln!(out, "{}", item.id)?;
    }

    Ok(())
}
