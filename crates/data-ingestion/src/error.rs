// Data Ingestion: Error types
use thiserror::Error;

#[derive(Debug, Error)]
pub enum IngestionError {
    #[error("Unsupported file format: {0}")]
    UnsupportedFormat(String),
    #[error("Failed to read source: {0}")]
    ReadError(String),
    #[error("Profiling failed: {0}")]
    ProfilingError(String),
    #[error("Model inference failed: {0}")]
    ModelInferenceError(String),
}

impl From<IngestionError> for shared::AppError {
    fn from(e: IngestionError) -> Self {
        shared::AppError::Database(e.to_string())
    }
}
