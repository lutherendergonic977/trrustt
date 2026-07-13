# data-ingestion — Data Ingestion Engine

Mode B: Build Power BI data models from raw data files.

## Supported Sources

- **CSV** — Comma/tab delimited files
- **SQL** — Database connections (SQLite, PostgreSQL, etc.)
- **Parquet** — Columnar storage format
- **JSON** — Nested and flat JSON documents

## Pipeline

1. **Ingest** — Read data from source into Polars DataFrames
2. **Profile** — Auto-detect types, distributions, nulls, outliers
3. **Infer Model** — Generate a Tabular model schema
4. **Build TMSL** — Create Power BI model via TMSL commands

## Usage

```rust
use data_ingestion::DataIngestionEngine;

let engine = DataIngestionEngine::new();
// Full pipeline coming in V1
```
