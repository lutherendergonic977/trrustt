// TRRUSTT — Data Ingestion Engine
// Mode B: Build Power BI data models from raw data files.
// Ingest CSV, SQL, Parquet, JSON → profile via Polars → infer model → build TMSL.
#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod error;
pub mod ingestor;
pub mod profiler;
pub mod model_builder;
pub mod sources;
pub mod types;

use shared::Result;
use crate::ingestor::DataIngestor;
use crate::profiler::DataProfiler;
use crate::model_builder::ModelBuilder;

/// The data ingestion engine.
/// Reads data from various sources, profiles it with Polars,
/// infers a tabular model schema, and builds TMSL commands.
pub struct DataIngestionEngine {
    ingestor: DataIngestor,
    profiler: DataProfiler,
    model_builder: ModelBuilder,
}

impl DataIngestionEngine {
    pub fn new() -> Self {
        Self {
            ingestor: DataIngestor::new(),
            profiler: DataProfiler::new(),
            model_builder: ModelBuilder::new(),
        }
    }

    /// Ingest data from a file or source and return a tabular model definition.
    pub async fn ingest_to_model(
        &self,
        source: &sources::DataSource,
        model_name: &str,
    ) -> Result<ingestor::IngestedModel> {
        let raw_data = self.ingestor.ingest(source).await?;
        let profile = self.profiler.profile(&raw_data)?;
        let model = self.model_builder.build(model_name, &profile)?;
        Ok(model)
    }

    pub fn ingestor(&self) -> &DataIngestor { &self.ingestor }
    pub fn profiler(&self) -> &DataProfiler { &self.profiler }
    pub fn model_builder(&self) -> &ModelBuilder { &self.model_builder }
}

impl Default for DataIngestionEngine {
    fn default() -> Self { Self::new() }
}
