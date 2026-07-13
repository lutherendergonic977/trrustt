// ═══════════════════════════════════════════════════════════════════════
// TRRUSTT — Shared Error Types
//
// Centralized error handling via `thiserror`. Every crate wraps its
// specific errors into the `AppError` variants defined here.
//
// Rule: NEVER use unwrap() or expect(). Always return Result<T, AppError>.
// ═══════════════════════════════════════════════════════════════════════

use std::path::PathBuf;
use thiserror::Error;

/// The central application error type.
///
/// All errors in TRRUSTT flow through this enum. Each variant
/// represents a domain-specific error category. Crates define their
/// own error types and convert into appropriate `AppError` variants
/// via `From` implementations or `.map_err()`.
#[derive(Debug, Error)]
pub enum AppError {
    // ── Configuration Errors ─────────────────────────────────────────

    /// A configuration key was not found.
    #[error("Configuration key not found: {0}")]
    ConfigNotFound(String),

    /// A configuration value failed to deserialize.
    #[error("Configuration resolution failed for '{key}': {reason}")]
    ConfigResolution {
        /// The config path that failed.
        key: String,
        /// Human-readable reason.
        reason: String,
    },

    /// A configuration validation failed.
    #[error("Configuration validation failed for '{key}': {reason}")]
    ConfigValidation {
        /// The config path.
        key: String,
        /// Validation error details.
        reason: String,
    },

    /// Attempted to set a non-user-overridable config key.
    #[error("Configuration key '{0}' is not user-overridable")]
    NotUserOverridable(String),

    /// A config key is enforced by an admin policy.
    #[error("Configuration key '{0}' is enforced by an admin policy")]
    AdminEnforced(String),

    /// A configuration key is not in the registry.
    #[error("Unknown configuration key: {0}")]
    UnknownConfigKey(String),

    /// Failed to encrypt/decrypt a config value.
    #[error("Encryption error for config key '{key}': {reason}")]
    ConfigEncryption {
        /// The config key.
        key: String,
        /// Reason for failure.
        reason: String,
    },

    // ── XMLA / SSAS Errors ───────────────────────────────────────────

    /// Failed to connect to the SSAS instance.
    #[error("Failed to connect to SSAS on port {port}: {reason}")]
    SsasConnection {
        /// The port attempted.
        port: u16,
        /// Reason for failure.
        reason: String,
    },

    /// SSAS returned an error for a DISCOVER request.
    #[error("XMLA DISCOVER failed: {0}")]
    XmlaDiscover(String),

    /// SSAS returned an error for an EXECUTE request.
    #[error("XMLA EXECUTE failed: {0}")]
    XmlaExecute(String),

    /// SSAS returned an error for a TMSL command.
    #[error("TMSL command failed: {0}")]
    TmslCommand(String),

    /// The SSAS instance returned an unexpected response.
    #[error("Unexpected SSAS response: {0}")]
    SsasUnexpectedResponse(String),

    /// Schema discovery returned no tables.
    #[error("Schema discovery returned no tables. Is a PBI model loaded?")]
    EmptySchema,

    /// The target table was not found in the model.
    #[error("Table '{0}' not found in the model")]
    TableNotFound(String),

    /// The target measure was not found in the model.
    #[error("Measure '{0}' not found in the model")]
    MeasureNotFound(String),

    /// The target column was not found in the model.
    #[error("Column '{0}' not found in the model")]
    ColumnNotFound(String),

    // ── DAX Engine Errors ────────────────────────────────────────────

    /// Failed to parse a DAX expression.
    #[error("DAX parse error at position {position}: {message}")]
    DaxParse {
        /// Human-readable error message.
        message: String,
        /// Byte position of the error.
        position: usize,
    },

    /// DAX validation failed with one or more errors.
    #[error("DAX validation failed: {0:?}")]
    DaxValidation(Vec<String>),

    /// DAX generation failed (AI returned unusable output).
    #[error("DAX generation failed: {0}")]
    DaxGeneration(String),

    /// DAX self-correction exceeded max cycles.
    #[error("DAX self-correction exceeded {max_cycles} cycles without success: {last_error}")]
    DaxCorrectionExhausted {
        /// Maximum cycles attempted.
        max_cycles: usize,
        /// Last validation error.
        last_error: String,
    },

    /// The requested DAX complexity level is not allowed for this role.
    #[error("DAX complexity level '{0}' is not allowed for your role")]
    DaxComplexityNotAllowed(String),

    /// A DAX function is disallowed by policy.
    #[error("DAX function '{function}' is disallowed by policy: {reason}")]
    DaxFunctionDisallowed {
        /// The disallowed function name.
        function: String,
        /// The policy reason.
        reason: String,
    },

    // ── AI Engine Errors ─────────────────────────────────────────────

    /// An AI provider returned an error.
    #[error("AI provider '{provider}' error: {message}")]
    AiProvider {
        /// Which provider failed.
        provider: String,
        /// Error message from the provider.
        message: String,
    },

    /// All AI providers failed (exhausted fallback chain).
    #[error("All AI providers exhausted: {0:?}")]
    AiAllProvidersExhausted(Vec<String>),

    /// The configured AI provider is unknown or not configured.
    #[error("Unknown AI provider: {0}")]
    AiUnknownProvider(String),

    /// The AI provider is not configured (missing API key).
    #[error("AI provider '{0}' is not configured. Set TRRUSTT_AI_PROVIDERS_{1}_API_KEY.")]
    AiProviderNotConfigured(String, String),

    /// AI rate limit exceeded.
    #[error("AI rate limit exceeded for provider '{provider}': {message}")]
    AiRateLimit {
        /// The provider that is rate-limited.
        provider: String,
        /// Details.
        message: String,
    },

    /// AI cost limit would be exceeded.
    #[error("AI cost limit exceeded: {reason}")]
    AiCostLimitExceeded {
        /// Reason/details.
        reason: String,
    },

    /// AI response was empty or malformed.
    #[error("AI returned an empty or malformed response from {provider}")]
    AiEmptyResponse {
        /// The provider.
        provider: String,
    },

    /// Failed to parse AI JSON response.
    #[error("Failed to parse AI JSON response: {0}")]
    AiJsonParse(String),

    /// RAG embedding generation failed.
    #[error("RAG embedding failed: {0}")]
    RagEmbedding(String),

    /// RAG search failed.
    #[error("RAG search failed: {0}")]
    RagSearch(String),

    /// Vision analysis failed.
    #[error("Vision analysis failed: {0}")]
    VisionAnalysis(String),

    /// Prompt template rendering failed.
    #[error("Prompt rendering failed for '{template}': {reason}")]
    PromptRender {
        /// Template name.
        template: String,
        /// Rendering error.
        reason: String,
    },

    // ── Database Errors ──────────────────────────────────────────────

    /// A database query failed.
    #[error("Database error: {0}")]
    Database(String),

    /// A database migration failed.
    #[error("Database migration failed (v{version}): {reason}")]
    DatabaseMigration {
        /// Migration version that failed.
        version: i32,
        /// Reason.
        reason: String,
    },

    /// A database backup or restore failed.
    #[error("Database backup/restore failed: {0}")]
    DatabaseBackup(String),

    /// Entity not found in database.
    #[error("{entity} not found (id: {id})")]
    EntityNotFound {
        /// Entity type name.
        entity: String,
        /// Entity ID.
        id: String,
    },

    /// A database constraint was violated (e.g., unique, foreign key).
    #[error("Database constraint violation on {entity}: {reason}")]
    DatabaseConstraint {
        /// Entity type.
        entity: String,
        /// Reason.
        reason: String,
    },

    // ── License Errors ───────────────────────────────────────────────

    /// License is invalid or expired.
    #[error("License error: {0}")]
    License(String),

    /// License has expired and grace period is over.
    #[error("License expired. Grace period exhausted. Please renew.")]
    LicenseExpired,

    /// License feature not available in current tier.
    #[error("Feature '{feature}' requires {required_tier} tier (current: {current_tier})")]
    LicenseFeatureNotAvailable {
        /// The feature name.
        feature: String,
        /// Required tier.
        required_tier: String,
        /// Current tier.
        current_tier: String,
    },

    /// License phone-home failed.
    #[error("License validation check failed: {0}")]
    LicenseCheckFailed(String),

    // ── MCP Errors ───────────────────────────────────────────────────

    /// MCP protocol error.
    #[error("MCP protocol error: {0}")]
    McpProtocol(String),

    /// MCP tool execution failed.
    #[error("MCP tool '{tool}' failed: {reason}")]
    McpToolError {
        /// Tool name.
        tool: String,
        /// Error reason.
        reason: String,
    },

    /// MCP connection error.
    #[error("MCP connection error: {0}")]
    McpConnection(String),

    // ── I/O Errors ───────────────────────────────────────────────────

    /// File not found.
    #[error("File not found: {path}")]
    FileNotFound {
        /// The file path.
        path: PathBuf,
    },

    /// Generic I/O error.
    #[error("I/O error on '{path}': {source}")]
    Io {
        /// The file path.
        path: PathBuf,
        /// Underlying I/O error.
        #[source]
        source: std::io::Error,
    },

    /// Failed to parse a file (TOML, JSON, etc.).
    #[error("Failed to parse '{path}': {reason}")]
    Parse {
        /// The file path.
        path: PathBuf,
        /// Parse error details.
        reason: String,
    },

    // ── Auth Errors ──────────────────────────────────────────────────

    /// Authentication failed.
    #[error("Authentication failed: {0}")]
    Auth(String),

    /// User is not authorized for this action.
    #[error("Not authorized: {0}")]
    Forbidden(String),

    /// Session expired.
    #[error("Session expired. Please log in again.")]
    SessionExpired,

    // ── Policy Errors ────────────────────────────────────────────────

    /// A policy was violated.
    #[error("Policy violation(s): {0:?}")]
    PolicyViolation(Vec<String>),

    /// A required policy is not configured.
    #[error("Required policy '{0}' is not configured")]
    PolicyNotConfigured(String),

    // ── General / Internal Errors ────────────────────────────────────

    /// An internal error occurred (unexpected, should not happen).
    #[error("Internal error: {0}. Please report this bug.")]
    Internal(String),

    /// A feature is not yet implemented.
    #[error("Not implemented: {0}")]
    NotImplemented(String),

    /// Invalid input from user.
    #[error("Invalid input: {0}")]
    InvalidInput(String),

    /// Operation timed out.
    #[error("Operation timed out after {duration_ms}ms: {operation}")]
    Timeout {
        /// What timed out.
        operation: String,
        /// Duration in milliseconds.
        duration_ms: u64,
    },
}

// ── Convenience type alias ────────────────────────────────────────────

/// The standard Result type for all TRRUSTT operations.
pub type Result<T> = std::result::Result<T, AppError>;

// ── From implementations for common external error types ─────────────

impl From<std::io::Error> for AppError {
    fn from(source: std::io::Error) -> Self {
        AppError::Io {
            path: PathBuf::from("<unknown>"),
            source,
        }
    }
}

impl From<serde_json::Error> for AppError {
    fn from(e: serde_json::Error) -> Self {
        AppError::Parse {
            path: PathBuf::from("<json>"),
            reason: e.to_string(),
        }
    }
}

impl From<uuid::Error> for AppError {
    fn from(e: uuid::Error) -> Self {
        AppError::InvalidInput(format!("Invalid UUID: {}", e))
    }
}

impl From<chrono::ParseError> for AppError {
    fn from(e: chrono::ParseError) -> Self {
        AppError::InvalidInput(format!("Invalid date/time: {}", e))
    }
}

// ── Helper constructors ───────────────────────────────────────────────

impl AppError {
    /// Create an I/O error with a known path.
    pub fn io(path: impl Into<PathBuf>, source: std::io::Error) -> Self {
        AppError::Io {
            path: path.into(),
            source,
        }
    }

    /// Create a file-not-found error.
    pub fn file_not_found(path: impl Into<PathBuf>) -> Self {
        AppError::FileNotFound { path: path.into() }
    }

    /// Create an entity-not-found error.
    pub fn entity_not_found(entity: impl Into<String>, id: impl Into<String>) -> Self {
        AppError::EntityNotFound {
            entity: entity.into(),
            id: id.into(),
        }
    }

    /// Create an internal error.
    pub fn internal(msg: impl Into<String>) -> Self {
        AppError::Internal(msg.into())
    }

    /// Create a not-implemented error.
    pub fn not_implemented(feature: impl Into<String>) -> Self {
        AppError::NotImplemented(feature.into())
    }

    /// Create an invalid-input error.
    pub fn invalid_input(msg: impl Into<String>) -> Self {
        AppError::InvalidInput(msg.into())
    }

    /// Create a database error from any displayable error.
    pub fn database(e: impl std::fmt::Display) -> Self {
        AppError::Database(e.to_string())
    }

    /// Create a timeout error.
    pub fn timeout(operation: impl Into<String>, duration_ms: u64) -> Self {
        AppError::Timeout {
            operation: operation.into(),
            duration_ms,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_error_display() {
        let err = AppError::ConfigNotFound("ai.provider".into());
        assert!(err.to_string().contains("ai.provider"));
    }

    #[test]
    fn test_io_error_conversion() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file missing");
        let app_err: AppError = io_err.into();
        assert!(matches!(app_err, AppError::Io { .. }));
    }

    #[test]
    fn test_helper_constructors() {
        let err = AppError::file_not_found("/tmp/test.toml");
        assert!(matches!(err, AppError::FileNotFound { .. }));

        let err = AppError::entity_not_found("User", "abc-123");
        assert!(err.to_string().contains("User"));
        assert!(err.to_string().contains("abc-123"));
    }
}
