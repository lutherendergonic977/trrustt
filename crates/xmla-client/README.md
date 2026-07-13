# xmla-client — Power BI SSAS Bridge

XMLA/HTTP client for Power BI Desktop's embedded SSAS Tabular instance.

## Operations

- **DISCOVER_XMLA** — Schema metadata (tables, columns, measures, relationships)
- **EXECUTE** — DAX query execution (EVALUATE statements)
- **TMSL** — Tabular Model Scripting Language (createOrReplace, delete, refresh)

## Usage

```rust
use xmla_client::XmlaClient;

// Connect to PBI Desktop's SSAS (port provided via --port)
let client = XmlaClient::connect(54321).await?;

// Discover schema
let schema = client.discover_schema().await?;

// Execute DAX
let result = client.execute_dax("EVALUATE 'Sales'").await?;

// Create a measure
client.create_measure("Sales", "Total Amount", "SUM('Sales'[Amount])").await?;
```
