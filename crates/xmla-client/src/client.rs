// ═══════════════════════════════════════════════════════════════════════
// TRRUSTT — XMLA Client Core
//
// Connects to Power BI Desktop's embedded SSAS Tabular instance
// via XMLA over HTTP on localhost.
// ═══════════════════════════════════════════════════════════════════════

use reqwest::Client as HttpClient;
use tracing::{debug, info, instrument};

use shared::{DaxQueryResult, Result, SchemaMetadata, TmslResult};

use crate::discover::DiscoverClient;
use crate::error::XmlaError;
use crate::execute::ExecuteClient;
use crate::tmsl::TmslClient;

/// XMLA client for communicating with Power BI Desktop's SSAS instance.
///
/// Power BI Desktop runs an embedded SSAS Tabular instance on a random
/// localhost port. This client connects to that instance via XMLA/HTTP
/// and performs schema discovery, DAX query execution, and TMSL commands.
///
/// # Example
/// ```rust,no_run
/// use xmla_client::XmlaClient;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let client = XmlaClient::connect(54321).await?;
/// let schema = client.discover_schema().await?;
/// println!("Found {} tables", schema.tables.len());
/// # Ok(())
/// # }
/// ```
pub struct XmlaClient {
    /// HTTP client for XMLA requests.
    http: HttpClient,

    /// Base URL for the SSAS XMLA endpoint.
    base_url: String,

    /// The SSAS database name (from PBI Desktop).
    database_name: String,

    /// The SSAS instance port.
    port: u16,

    /// Sub-clients for specific operations.
    discover: DiscoverClient,
    execute: ExecuteClient,
    tmsl: TmslClient,
}

impl XmlaClient {
    /// Connect to a running Power BI Desktop SSAS instance.
    ///
    /// # Arguments
    /// * `port` - The SSAS instance port (provided by PBI Desktop via `--port`).
    ///
    /// # Returns
    /// A connected `XmlaClient` ready for operations.
    ///
    /// # Errors
    /// Returns `SsasConnection` if the SSAS instance is unreachable.
    #[instrument(name = "xmla_connect", skip_all, fields(port = %port))]
    pub async fn connect(port: u16) -> Result<Self> {
        info!("Connecting to SSAS on localhost:{}", port);

        let base_url = format!("http://localhost:{}/xmla", port);

        let http = HttpClient::builder()
            .timeout(std::time::Duration::from_secs(60))
            .gzip(true)
            .build()
            .map_err(|e| XmlaError::Http(e))?;

        // Test connectivity by sending a simple DISCOVER request
        let discover_body = r#"
            <Envelope xmlns="http://schemas.xmlsoap.org/soap/envelope/">
                <Header/>
                <Body>
                    <Discover xmlns="urn:schemas-microsoft-com:xml-analysis">
                        <RequestType>DISCOVER_DATASOURCES</RequestType>
                        <Restrictions/>
                        <Properties/>
                    </Discover>
                </Body>
            </Envelope>
        "#;

        let response = http
            .post(&base_url)
            .header("Content-Type", "text/xml")
            .body(discover_body.to_string())
            .send()
            .await
            .map_err(|e| XmlaError::Connection {
                port,
                reason: format!("Failed to connect: {}", e),
            })?;

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let body = response.text().await.unwrap_or_default();
            if status == 0 || body.is_empty() {
                return Err(XmlaError::Connection {
                    port,
                    reason: "Is Power BI Desktop running with a model open?".into(),
                }.into());
            }
            return Err(XmlaError::HttpError { status, body }.into());
        }

        let body_text = response.text().await
            .map_err(|e| XmlaError::Http(e))?;

        // Extract database name from response
        let database_name = Self::extract_database_name(&body_text)
            .unwrap_or_else(|| "Unknown".to_string());

        info!(
            port = port,
            database = %database_name,
            "Connected to SSAS instance"
        );

        Ok(Self {
            http,
            base_url,
            database_name,
            port,
            discover: DiscoverClient,
            execute: ExecuteClient,
            tmsl: TmslClient,
        })
    }

    /// Discover the full schema metadata of the Power BI model.
    ///
    /// This includes tables, columns, measures, relationships, hierarchies,
    /// calculation groups, partitions, and annotations.
    #[instrument(skip(self))]
    pub async fn discover_schema(&self) -> Result<SchemaMetadata> {
        debug!("Discovering schema metadata");
        self.discover.full_schema(&self.http, &self.base_url, &self.database_name).await
    }

    /// Execute a DAX query against the model.
    ///
    /// # Arguments
    /// * `dax` - The DAX EVALUATE query to execute.
    ///
    /// # Returns
    /// The query result with column names and row data.
    #[instrument(skip(self, dax), fields(dax_len = dax.len()))]
    pub async fn execute_dax(&self, dax: &str) -> Result<DaxQueryResult> {
        debug!("Executing DAX query");
        self.execute.execute_dax(&self.http, &self.base_url, &self.database_name, dax).await
    }

    /// Execute a TMSL command against the model.
    ///
    /// TMSL is used to create, alter, delete, and refresh model objects.
    ///
    /// # Arguments
    /// * `tmsl_json` - The TMSL command as a JSON string.
    #[instrument(skip(self, tmsl_json), fields(tmsl_len = tmsl_json.len()))]
    pub async fn execute_tmsl(&self, tmsl_json: &str) -> Result<TmslResult> {
        debug!("Executing TMSL command");
        let command: serde_json::Value = serde_json::from_str(tmsl_json).map_err(|e| {
            shared::AppError::internal(format!("Invalid TMSL JSON: {}", e))
        })?;
        self.tmsl.execute_tmsl(&self.http, &self.base_url, &self.database_name, &command).await
    }

    /// Create or replace a measure in the model.
    #[instrument(skip(self, expression))]
    pub async fn create_measure(
        &self,
        table_name: &str,
        measure_name: &str,
        expression: &str,
    ) -> Result<TmslResult> {
        let tmsl = serde_json::json!({
            "createOrReplace": {
                "object": {
                    "database": self.database_name,
                    "table": table_name,
                    "measure": measure_name
                },
                "measure": {
                    "name": measure_name,
                    "expression": expression
                }
            }
        });
        self.execute_tmsl(&serde_json::to_string(&tmsl).map_err(|e| {
            shared::AppError::internal(format!("Failed to serialize TMSL command: {}", e))
        })?).await
    }

    /// Delete a measure from the model.
    #[instrument(skip(self))]
    pub async fn delete_measure(
        &self,
        table_name: &str,
        measure_name: &str,
    ) -> Result<TmslResult> {
        let tmsl = serde_json::json!({
            "delete": {
                "object": {
                    "database": self.database_name,
                    "table": table_name,
                    "measure": measure_name
                }
            }
        });
        self.execute_tmsl(&serde_json::to_string(&tmsl).map_err(|e| {
            shared::AppError::internal(format!("Failed to serialize TMSL command: {}", e))
        })?).await
    }

    /// Refresh a table in the model.
    #[instrument(skip(self))]
    pub async fn refresh_table(&self, table_name: &str) -> Result<TmslResult> {
        let tmsl = serde_json::json!({
            "refresh": {
                "type": "full",
                "objects": [{
                    "database": self.database_name,
                    "table": table_name
                }]
            }
        });
        self.execute_tmsl(&serde_json::to_string(&tmsl).map_err(|e| {
            shared::AppError::internal(format!("Failed to serialize TMSL command: {}", e))
        })?).await
    }

    /// Get the SSAS connection string.
    pub fn connection_string(&self) -> String {
        format!(
            "Provider=MSOLAP;Data Source=localhost:{};Initial Catalog={};",
            self.port, self.database_name
        )
    }

    /// Get the database name.
    pub fn database_name(&self) -> &str {
        &self.database_name
    }

    /// Get the SSAS port.
    pub fn port(&self) -> u16 {
        self.port
    }

    // ── Private helpers ───────────────────────────────────────────────

    /// Extract the database name from a DISCOVER_DATASOURCES response.
    fn extract_database_name(xml: &str) -> Option<String> {
        // Simple XML parsing — in production, use a proper XML parser
        if let Some(start) = xml.find("<Catalog>") {
            if let Some(end) = xml.find("</Catalog>") {
                let name = &xml[start + 9..end];
                return Some(name.trim().to_string());
            }
        }
        // Fallback: try DatabaseName
        if let Some(start) = xml.find("<DatabaseName>") {
            if let Some(end) = xml.find("</DatabaseName>") {
                let name = &xml[start + 14..end];
                return Some(name.trim().to_string());
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_database_name() {
        let xml = r#"<root><Catalog>My Database</Catalog></root>"#;
        assert_eq!(
            XmlaClient::extract_database_name(xml),
            Some("My Database".to_string())
        );
    }

    #[test]
    fn test_connection_string() {
        // We can't easily test connect() without a real SSAS instance,
        // but we can test helpers.
        let xml = r#"<root><DatabaseName>TestDB</DatabaseName></root>"#;
        assert_eq!(
            XmlaClient::extract_database_name(xml),
            Some("TestDB".to_string())
        );
    }
}
