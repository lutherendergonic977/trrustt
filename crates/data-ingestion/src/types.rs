// ═══════════════════════════════════════════════════════════════════════
// TRRUSTT — Data Ingestion: Domain Types
//
// Core types for the data ingestion pipeline: source definitions,
// profiling statistics, table/model representations, and TMSL-ready
// schema definitions.
// ═══════════════════════════════════════════════════════════════════════

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Supported data source formats.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SourceFormat {
    /// Comma-separated values.
    Csv,
    /// SQL database connection.
    Sql,
    /// Apache Parquet columnar format.
    Parquet,
    /// JSON (array of objects or newline-delimited).
    Json,
    /// Microsoft Excel (.xlsx, .xls).
    Excel,
}

impl SourceFormat {
    /// Detect the format from a file extension.
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_lowercase().as_str() {
            "csv" | "tsv" | "txt" => Some(Self::Csv),
            "parquet" | "pq" => Some(Self::Parquet),
            "json" | "ndjson" | "jsonl" => Some(Self::Json),
            "xlsx" | "xls" | "xlsm" => Some(Self::Excel),
            _ => None,
        }
    }

    /// Detect the format from a MIME type.
    pub fn from_mime(mime: &str) -> Option<Self> {
        match mime {
            "text/csv" | "text/tab-separated-values" => Some(Self::Csv),
            "application/parquet" | "application/vnd.apache.parquet" => Some(Self::Parquet),
            "application/json" => Some(Self::Json),
            "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet"
            | "application/vnd.ms-excel" => Some(Self::Excel),
            _ => None,
        }
    }

    /// Human-readable name of the format.
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Csv => "CSV",
            Self::Sql => "SQL",
            Self::Parquet => "Parquet",
            Self::Json => "JSON",
            Self::Excel => "Excel",
        }
    }
}

/// A data source to ingest from.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataSource {
    /// Display name for this source.
    pub name: String,
    /// The format of the source data.
    pub format: SourceFormat,
    /// File path (for file-based sources).
    pub path: Option<String>,
    /// Raw URL for remote sources.
    pub url: Option<String>,
    /// SQL connection string (for database sources).
    pub connection_string: Option<String>,
    /// SQL query (for database sources).
    pub query: Option<String>,
    /// Inline data (for small datasets passed directly).
    pub inline_data: Option<String>,
    /// Source-specific options.
    pub options: HashMap<String, String>,
}

/// Data profiling statistics for a single column.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnProfile {
    /// Column name.
    pub name: String,
    /// Inferred Power BI data type (e.g., "Int64", "Text", "Decimal", "DateTime").
    pub data_type: String,
    /// Total row count.
    pub count: i64,
    /// Number of null/empty values.
    pub null_count: i64,
    /// Null percentage (0.0 - 100.0).
    pub null_percentage: f64,
    /// Number of distinct values.
    pub distinct_count: i64,
    /// Distinctness ratio (distinct / count, 0.0 - 1.0).
    pub distinctness_ratio: f64,
    /// Minimum value (for numeric/date types).
    pub min_value: Option<serde_json::Value>,
    /// Maximum value (for numeric/date types).
    pub max_value: Option<serde_json::Value>,
    /// Mean (for numeric types).
    pub mean: Option<f64>,
    /// Median (for numeric types).
    pub median: Option<f64>,
    /// Standard deviation (for numeric types).
    pub std_dev: Option<f64>,
    /// Skewness (for numeric types).
    pub skewness: Option<f64>,
    /// Most frequent value.
    pub mode: Option<serde_json::Value>,
    /// Whether the column is likely a primary key.
    pub is_likely_key: bool,
    /// Whether the column is likely a measure (numeric, non-key).
    pub is_likely_measure: bool,
    /// Whether the column is likely a date dimension.
    pub is_likely_date: bool,
    /// Sample values (up to 5).
    pub sample_values: Vec<serde_json::Value>,
    /// Detected format for string columns (e.g., "email", "url", "uuid", "iso-date").
    pub detected_format: Option<String>,
}

/// Profiling statistics for a table.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableProfile {
    /// Table name (derived from filename or source).
    pub name: String,
    /// Total row count.
    pub row_count: i64,
    /// Total column count.
    pub column_count: usize,
    /// Per-column profiles.
    pub columns: Vec<ColumnProfile>,
    /// Estimated memory size in bytes.
    pub estimated_size_bytes: u64,
    /// Source format this table came from.
    pub source_format: SourceFormat,
    /// Detected primary key columns.
    pub detected_keys: Vec<String>,
    /// Detected date columns.
    pub detected_dates: Vec<String>,
    /// Detected measure columns.
    pub detected_measures: Vec<String>,
}

/// Complete profiling result for a data ingestion run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileResult {
    /// All profiled tables.
    pub tables: Vec<TableProfile>,
    /// Total rows across all tables.
    pub total_rows: i64,
    /// Total columns across all tables.
    pub total_columns: usize,
    /// Duration of profiling in milliseconds.
    pub duration_ms: u64,
    /// Detected relationships between tables.
    pub detected_relationships: Vec<DetectedRelationship>,
}

/// A detected relationship between two tables.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectedRelationship {
    /// Source table name.
    pub from_table: String,
    /// Source column name.
    pub from_column: String,
    /// Target table name.
    pub to_table: String,
    /// Target column name.
    pub to_column: String,
    /// Cardinality (e.g., "many-to-one").
    pub cardinality: String,
    /// Cross-filtering direction.
    pub cross_filter_direction: String,
}

/// A column mapping for the tabular model.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnMapping {
    /// Column name.
    pub name: String,
    /// Power BI data type.
    pub data_type: String,
    /// Source column name (if different from model name).
    pub source_name: Option<String>,
    /// Whether this column is hidden.
    pub is_hidden: bool,
    /// Sort-by column name.
    pub sort_by_column: Option<String>,
    /// Display folder.
    pub display_folder: Option<String>,
    /// Data category (e.g., "Address", "City", "Country", "WebURL").
    pub data_category: Option<String>,
    /// Format string (e.g., "#,##0.00", "MM/dd/yyyy").
    pub format_string: Option<String>,
    /// Summarize-by mode (e.g., "sum", "count", "none").
    pub summarize_by: Option<String>,
}

/// A table definition for the tabular model.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableDefinition {
    /// Table name.
    pub name: String,
    /// Column definitions.
    pub columns: Vec<ColumnMapping>,
    /// Measures defined on this table.
    pub measures: Vec<MeasureDefinition>,
    /// Hierarchies defined on this table.
    pub hierarchies: Vec<HierarchyDefinition>,
    /// Whether the table is hidden.
    pub is_hidden: bool,
    /// Description.
    pub description: Option<String>,
    /// Source query or source table reference.
    pub source_expression: String,
}

/// A measure definition for the tabular model.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeasureDefinition {
    /// Measure name.
    pub name: String,
    /// DAX expression.
    pub expression: String,
    /// Display folder.
    pub display_folder: Option<String>,
    /// Format string.
    pub format_string: Option<String>,
    /// Description.
    pub description: Option<String>,
    /// Whether auto-generated.
    pub is_auto_generated: bool,
}

/// A hierarchy definition for the tabular model.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HierarchyDefinition {
    /// Hierarchy name.
    pub name: String,
    /// Levels (column names).
    pub levels: Vec<String>,
    /// Description.
    pub description: Option<String>,
    /// Whether hidden.
    pub is_hidden: bool,
}

/// A fully ingested model ready for TMSL deployment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IngestedModel {
    /// Model/database name.
    pub name: String,
    /// Table definitions.
    pub tables: Vec<TableDefinition>,
    /// Relationships between tables.
    pub relationships: Vec<DetectedRelationship>,
    /// Source data rows (for TMSL partition creation).
    pub source_rows: HashMap<String, Vec<HashMap<String, serde_json::Value>>>,
    /// Creation timestamp.
    pub created_at: String,
    /// Source format.
    pub source_format: SourceFormat,
}

