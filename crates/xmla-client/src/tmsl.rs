// ═══════════════════════════════════════════════════════════════════════
// TRRUSTT — TMSL Operations
//
// Tabular Model Scripting Language (TMSL) command builder and executor.
// Creates JSON commands for creating/altering/deleting model objects
// (measures, columns, tables, relationships, etc.) and sends them
// via XMLA EXECUTE to the SSAS Tabular instance.
// ═══════════════════════════════════════════════════════════════════════

use std::time::Instant;

use reqwest::Client as HttpClient;
use serde_json::{json, Value};
use tracing::{debug, info, instrument};

use shared::{Result, TmslResult};
use crate::error::XmlaError;

/// Client for TMSL operations.
pub(crate) struct TmslClient;

impl TmslClient {
    /// Execute a raw TMSL JSON command against the SSAS instance.
    ///
    /// # Arguments
    /// * `http` - The HTTP client.
    /// * `base_url` - The XMLA endpoint URL.
    /// * `database_name` - The SSAS database name.
    /// * `tmsl_command` - The TMSL command as a JSON value.
    ///
    /// # Returns
    /// A `TmslResult` with success status, affected objects, and warnings.
    #[instrument(skip(http, tmsl_command))]
    pub async fn execute_tmsl(
        &self,
        http: &HttpClient,
        base_url: &str,
        database_name: &str,
        tmsl_command: &Value,
    ) -> Result<TmslResult> {
        let start = Instant::now();

        let xml_body = Self::build_tmsl_envelope(database_name, tmsl_command);

        let response = http
            .post(base_url)
            .header("Content-Type", "text/xml; charset=utf-8")
            .header("SOAPAction", "urn:schemas-microsoft-com:xml-analysis:Execute")
            .body(xml_body)
            .send()
            .await
            .map_err(|e| XmlaError::Http(e))?;

        let status = response.status();
        let body = response.text().await.map_err(|e| XmlaError::Http(e))?;

        let duration_ms = start.elapsed().as_millis() as u64;

        if !status.is_success() {
            let error_msg = Self::extract_error(&body);
            return Ok(TmslResult {
                success: false,
                affected_objects: vec![],
                warnings: vec![error_msg],
                duration_ms,
            });
        }

        // Extract affected objects and warnings from the response
        let affected_objects = Self::extract_affected(&body);
        let warnings = Self::extract_warnings(&body);

        info!(
            affected = affected_objects.len(),
            warnings = warnings.len(),
            duration_ms = duration_ms,
            "TMSL command executed"
        );

        Ok(TmslResult {
            success: true,
            affected_objects,
            warnings,
            duration_ms,
        })
    }

    /// Create or replace a measure in the model.
    #[instrument(skip(http))]
    pub async fn create_measure(
        &self,
        http: &HttpClient,
        base_url: &str,
        database_name: &str,
        table_name: &str,
        measure_name: &str,
        expression: &str,
        format_string: Option<&str>,
        description: Option<&str>,
        display_folder: Option<&str>,
    ) -> Result<TmslResult> {
        let mut measure_props = json!({
            "name": measure_name,
            "expression": expression,
        });

        if let Some(fs) = format_string {
            measure_props["formatString"] = json!(fs);
        }
        if let Some(desc) = description {
            measure_props["description"] = json!(desc);
        }
        if let Some(folder) = display_folder {
            measure_props["displayFolder"] = json!(folder);
        }

        let command = json!({
            "createOrReplace": {
                "object": {
                    "database": database_name,
                    "table": table_name,
                    "measure": measure_name
                },
                "measure": measure_props
            }
        });

        self.execute_tmsl(http, base_url, database_name, &command).await
    }

    /// Delete a measure from the model.
    #[instrument(skip(http))]
    pub async fn delete_measure(
        &self,
        http: &HttpClient,
        base_url: &str,
        database_name: &str,
        table_name: &str,
        measure_name: &str,
    ) -> Result<TmslResult> {
        let command = json!({
            "delete": {
                "object": {
                    "database": database_name,
                    "table": table_name,
                    "measure": measure_name
                }
            }
        });

        self.execute_tmsl(http, base_url, database_name, &command).await
    }

    /// Refresh a table (process data).
    #[instrument(skip(http))]
    pub async fn refresh_table(
        &self,
        http: &HttpClient,
        base_url: &str,
        database_name: &str,
        table_name: &str,
    ) -> Result<TmslResult> {
        let command = json!({
            "refresh": {
                "type": "full",
                "objects": [{
                    "database": database_name,
                    "table": table_name
                }]
            }
        });

        self.execute_tmsl(http, base_url, database_name, &command).await
    }

    /// Create or replace a TMSL sequence (batch of commands).
    #[instrument(skip(http))]
    pub async fn execute_sequence(
        &self,
        http: &HttpClient,
        base_url: &str,
        database_name: &str,
        commands: &[Value],
    ) -> Result<TmslResult> {
        let sequence = json!({
            "sequence": {
                "operations": commands
            }
        });

        self.execute_tmsl(http, base_url, database_name, &sequence).await
    }

    // ── Private helpers ───────────────────────────────────────────────

    /// Build the XMLA SOAP envelope wrapping a TMSL command.
    fn build_tmsl_envelope(database_name: &str, tmsl_command: &Value) -> String {
        let tmsl_json = serde_json::to_string(tmsl_command)
            .unwrap_or_else(|_| "{}".to_string());

        format!(
            r#"<Envelope xmlns="http://schemas.xmlsoap.org/soap/envelope/">
    <Header/>
    <Body>
        <Execute xmlns="urn:schemas-microsoft-com:xml-analysis">
            <Command>
                <Statement><![CDATA[{tmsl}]]></Statement>
            </Command>
            <Properties>
                <PropertyList>
                    <Catalog>{catalog}</Catalog>
                    <Format>Tabular</Format>
                </PropertyList>
            </Properties>
        </Execute>
    </Body>
</Envelope>"#,
            tmsl = tmsl_json,
            catalog = database_name,
        )
    }

    /// Extract error message from a TMSL response.
    fn extract_error(body: &str) -> String {
        if let Some(start) = body.find("<faultstring>") {
            let rest = &body[start + "<faultstring>".len()..];
            if let Some(end) = rest.find("</faultstring>") {
                return rest[..end].to_string();
            }
        }
        format!("TMSL command failed: {}", &body[..body.len().min(500)])
    }

    /// Extract affected object names from a TMSL response.
    fn extract_affected(body: &str) -> Vec<String> {
        let mut affected = Vec::new();
        let mut pos = 0;
        while let Some(start) = body[pos..].find("\"name\":\"") {
            let abs_start = pos + start + "\"name\":\"".len();
            if let Some(end) = body[abs_start..].find('"') {
                let name = &body[abs_start..abs_start + end];
                affected.push(name.to_string());
                pos = abs_start + end + 1;
            } else {
                break;
            }
        }
        affected
    }

    /// Extract warning messages from a TMSL response.
    fn extract_warnings(body: &str) -> Vec<String> {
        let mut warnings = Vec::new();
        let mut pos = 0;
        let pattern = "<Warning";
        while let Some(start) = body[pos..].find(pattern) {
            let abs_start = pos + start;
            if let Some(end) = body[abs_start..].find("/>") {
                let warn_xml = &body[abs_start..abs_start + end + 2];
                if let Some(code_start) = warn_xml.find("Code=\"") {
                    let code = &warn_xml[code_start + "Code=\"".len()..]
                        .split('"')
                        .next()
                        .unwrap_or("?");
                    if let Some(desc_start) = warn_xml.find("Description=\"") {
                        let desc = &warn_xml[desc_start + "Description=\"".len()..]
                            .split('"')
                            .next()
                            .unwrap_or("?");
                        warnings.push(format!("[{}] {}", code, desc));
                    }
                }
                pos = abs_start + end + 2;
            } else {
                break;
            }
        }
        warnings
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_tmsl_envelope() {
        let cmd = json!({"createOrReplace": {"object": {"database": "TestDB"}}});
        let envelope = TmslClient::build_tmsl_envelope("TestDB", &cmd);
        assert!(envelope.contains("<Envelope"));
        assert!(envelope.contains("createOrReplace"));
        assert!(envelope.contains("TestDB"));
    }

    #[test]
    fn test_extract_error() {
        let body = r#"<soap:Body><soap:Fault><faultstring>Invalid object reference</faultstring></soap:Fault></soap:Body>"#;
        let err = TmslClient::extract_error(body);
        assert!(err.contains("Invalid object reference"));
    }

    #[test]
    fn test_extract_affected() {
        let body = r#"{"results":[{"name":"NewMeasure","status":"created"}]}"#;
        let affected = TmslClient::extract_affected(body);
        assert!(affected.contains(&"NewMeasure".to_string()));
    }
}
