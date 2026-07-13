// ═══════════════════════════════════════════════════════════════════════
// TRRUSTT — XMLA Client Error Types
// ═══════════════════════════════════════════════════════════════════════

use thiserror::Error;

/// XMLA client specific errors.
#[derive(Debug, Error)]
pub enum XmlaError {
    /// Failed to connect to the SSAS instance.
    #[error("Failed to connect to SSAS on port {port}: {reason}")]
    Connection {
        /// Port attempted.
        port: u16,
        /// Reason.
        reason: String,
    },

    /// SSAS rejected the request (HTTP error).
    #[error("SSAS returned HTTP {status}: {body}")]
    HttpError {
        /// HTTP status code.
        status: u16,
        /// Response body.
        body: String,
    },

    /// XMLA DISCOVER request failed.
    #[error("DISCOVER_XMLA failed: {0}")]
    Discover(String),

    /// XMLA EXECUTE request failed.
    #[error("EXECUTE failed: {0}")]
    Execute(String),

    /// TMSL command failed.
    #[error("TMSL command '{command}' failed: {reason}")]
    Tmsl {
        /// The TMSL command name.
        command: String,
        /// Failure reason.
        reason: String,
    },

    /// Failed to parse XMLA response.
    #[error("Failed to parse XMLA response: {0}")]
    Parse(String),

    /// Schema is empty (no tables found).
    #[error("Schema is empty — no tables discovered")]
    EmptySchema,

    /// Table not found in the model.
    #[error("Table '{0}' not found")]
    TableNotFound(String),

    /// Measure not found.
    #[error("Measure '{0}' not found")]
    MeasureNotFound(String),

    /// Column not found.
    #[error("Column '{0}' not found")]
    ColumnNotFound(String),

    /// Request timed out.
    #[error("Request timed out after {0}ms")]
    Timeout(u64),

    /// I/O error.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// HTTP client error.
    #[error("HTTP client error: {0}")]
    Http(#[from] reqwest::Error),
}

impl From<XmlaError> for shared::AppError {
    fn from(e: XmlaError) -> Self {
        match e {
            XmlaError::Connection { port, reason } => {
                shared::AppError::SsasConnection { port, reason }
            }
            XmlaError::HttpError { status, body } => {
                shared::AppError::SsasUnexpectedResponse(format!("HTTP {}: {}", status, body))
            }
            XmlaError::Discover(msg) => shared::AppError::XmlaDiscover(msg),
            XmlaError::Execute(msg) => shared::AppError::XmlaExecute(msg),
            XmlaError::Tmsl { command, reason } => {
                shared::AppError::TmslCommand(format!("{}: {}", command, reason))
            }
            XmlaError::Parse(msg) => shared::AppError::Parse {
                path: std::path::PathBuf::from("<xmla>"),
                reason: msg,
            },
            XmlaError::EmptySchema => shared::AppError::EmptySchema,
            XmlaError::TableNotFound(t) => shared::AppError::TableNotFound(t),
            XmlaError::MeasureNotFound(m) => shared::AppError::MeasureNotFound(m),
            XmlaError::ColumnNotFound(c) => shared::AppError::ColumnNotFound(c),
            XmlaError::Timeout(ms) => shared::AppError::timeout("XMLA request", ms),
            XmlaError::Io(e) => shared::AppError::from(e),
            XmlaError::Http(e) => shared::AppError::Internal(format!("HTTP client: {}", e)),
        }
    }
}
