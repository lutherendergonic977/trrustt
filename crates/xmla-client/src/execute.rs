// ═══════════════════════════════════════════════════════════════════════
// TRRUSTT — XMLA EXECUTE Operations
//
// EXECUTE requests for running DAX EVALUATE queries against the SSAS
// Tabular instance. Parses XMLA rowset responses into DaxQueryResult.
// ═══════════════════════════════════════════════════════════════════════

use std::time::Instant;

use reqwest::Client as HttpClient;
use serde_json::Value;
use tracing::{debug, instrument};

use shared::{DaxQueryResult, Result};
use crate::error::XmlaError;

/// The XMLA SOAP envelope template for EXECUTE requests.
const EXECUTE_ENVELOPE: &str = r#"<Envelope xmlns="http://schemas.xmlsoap.org/soap/envelope/">
    <Header/>
    <Body>
        <Execute xmlns="urn:schemas-microsoft-com:xml-analysis">
            <Command>
                <Statement>{dax}</Statement>
            </Command>
            <Properties>
                <PropertyList>
                    <Catalog>{catalog}</Catalog>
                    <Format>Tabular</Format>
                </PropertyList>
            </Properties>
        </Execute>
    </Body>
</Envelope>"#;

/// Client for XMLA EXECUTE operations (DAX queries).
pub(crate) struct ExecuteClient;

impl ExecuteClient {
    /// Execute a DAX query against the SSAS instance.
    ///
    /// Sends an EXECUTE XMLA command with the provided DAX statement,
    /// parses the XMLA rowset response, and returns structured results.
    ///
    /// # Arguments
    /// * `http` - The HTTP client.
    /// * `base_url` - The XMLA endpoint URL.
    /// * `database_name` - The SSAS database (catalog) name.
    /// * `dax` - The DAX EVALUATE query to execute.
    ///
    /// # Returns
    /// A `DaxQueryResult` with columns, rows, row count, and duration.
    #[instrument(skip(http, dax))]
    pub async fn execute_dax(
        &self,
        http: &HttpClient,
        base_url: &str,
        database_name: &str,
        dax: &str,
    ) -> Result<DaxQueryResult> {
        let start = Instant::now();

        let xml_body = EXECUTE_ENVELOPE
            .replace("{dax}", &Self::escape_xml(dax))
            .replace("{catalog}", database_name);

        let response = http
            .post(base_url)
            .header("Content-Type", "text/xml; charset=utf-8")
            .header("SOAPAction", "urn:schemas-microsoft-com:xml-analysis:Execute")
            .body(xml_body)
            .send()
            .await
            .map_err(|e| XmlaError::Http(e))?;

        let status = response.status();
        let body = response
            .text()
            .await
            .map_err(|e| XmlaError::Http(e))?;

        if !status.is_success() {
            let error_msg = body
                .find("<faultstring>")
                .and_then(|i| {
                    let rest = &body[i + "<faultstring>".len()..];
                    rest.find("</faultstring>")
                        .map(|j| rest[..j].to_string())
                })
                .unwrap_or_else(|| format!("HTTP {}: {}", status.as_u16(), &body[..body.len().min(500)]));
            return Err(XmlaError::Execute(error_msg).into());
        }

        let duration_ms = start.elapsed().as_millis() as u64;

        // Parse the XMLA rowset
        let (columns, rows, row_count) = Self::parse_execute_result(&body)?;

        debug!(
            columns = columns.len(),
            rows = row_count,
            duration_ms = duration_ms,
            "DAX query executed successfully"
        );

        Ok(DaxQueryResult {
            columns,
            rows,
            row_count,
            duration_ms,
        })
    }

    /// Parse an XMLA EXECUTE response into column names and row data.
    fn parse_execute_result(xml: &str) -> Result<(Vec<String>, Vec<Vec<Value>>, usize)> {
        // Find the rowset section
        let rowset_start = xml.find("<rowset")
            .or_else(|| xml.find("<rowset "))
            .ok_or_else(|| XmlaError::Execute("No rowset in EXECUTE response".into()))?;

        let rest = &xml[rowset_start..];

        // Extract column names from the first <row>
        let columns = Self::extract_column_names(rest);

        // Extract all <row> elements
        let rows = Self::extract_rows(rest, &columns)?;
        let row_count = rows.len();

        Ok((columns, rows, row_count))
    }

    /// Extract column names from the rowset XML schema or first data row.
    fn extract_column_names(rowset_xml: &str) -> Vec<String> {
        // Try XSD schema first
        let mut columns = Vec::new();
        let mut pos = 0;
        while let Some(start) = rowset_xml[pos..].find(r#"name=""#) {
            let abs_start = pos + start + r#"name=""#.len();
            if let Some(end) = rowset_xml[abs_start..].find('"') {
                let name = &rowset_xml[abs_start..abs_start + end];
                if !name.is_empty() && !columns.contains(&name.to_string()) {
                    columns.push(name.to_string());
                }
                pos = abs_start + end + 1;
            } else {
                break;
            }
        }

        // Fallback: extract from first <row> element
        if columns.is_empty() {
            if let Some(first_row_start) = rowset_xml.find("<row") {
                let first_row = &rowset_xml[first_row_start..];
                let mut search_pos = 0;
                while let Some(tag_start) = first_row[search_pos..].find('<') {
                    let abs = search_pos + tag_start;
                    if first_row[abs..].starts_with("</") || first_row[abs..].starts_with("<row") {
                        search_pos = abs + 1;
                        continue;
                    }
                    if let Some(tag_end) = first_row[abs..].find('>') {
                        let tag_name = &first_row[abs + 1..abs + tag_end];
                        if !tag_name.is_empty() && !tag_name.contains(' ') && !columns.contains(&tag_name.to_string()) {
                            columns.push(tag_name.to_string());
                        }
                        search_pos = abs + tag_end + 1;
                    } else {
                        break;
                    }
                }
            }
        }

        columns
    }

    /// Extract all <row> elements and their field values.
    fn extract_rows(rowset_xml: &str, columns: &[String]) -> Result<Vec<Vec<Value>>> {
        let mut rows = Vec::new();
        let mut pos = 0;

        while let Some(row_start) = rowset_xml[pos..].find("<row") {
            let absolute_start = pos + row_start;

            // Find the closing </row> (handle nested elements)
            let mut depth = 0;
            let mut row_end = absolute_start;
            let mut search_pos = absolute_start;
            while search_pos < rowset_xml.len() {
                if rowset_xml[search_pos..].starts_with("<row") && !rowset_xml[search_pos..].starts_with("</row>") {
                    depth += 1;
                    search_pos += 4;
                } else if rowset_xml[search_pos..].starts_with("</row>") {
                    depth -= 1;
                    if depth == 0 {
                        row_end = search_pos + "</row>".len();
                        break;
                    }
                    search_pos += 6;
                } else {
                    search_pos += 1;
                }
            }

            let row_xml = &rowset_xml[absolute_start..row_end];
            let mut row_values = Vec::with_capacity(columns.len());

            for col in columns {
                let value = Self::extract_column_value(row_xml, col);
                row_values.push(value);
            }

            rows.push(row_values);
            pos = row_end;
        }

        Ok(rows)
    }

    /// Extract a single column's value from a row XML element.
    fn extract_column_value(row_xml: &str, column_name: &str) -> Value {
        let open_tag = format!("<{}>", column_name);
        let close_tag = format!("</{}>", column_name);

        if let Some(start) = row_xml.find(&open_tag) {
            let content_start = start + open_tag.len();
            let rest = &row_xml[content_start..];
            if let Some(end) = rest.find(&close_tag) {
                let content = &rest[..end];
                // Try parsing as number
                if let Ok(n) = content.parse::<i64>() {
                    return Value::Number(serde_json::Number::from(n));
                }
                if let Ok(n) = content.parse::<f64>() {
                    if let Some(num) = serde_json::Number::from_f64(n) {
                        return Value::Number(num);
                    }
                }
                // Try parsing as boolean
                if content.eq_ignore_ascii_case("true") {
                    return Value::Bool(true);
                }
                if content.eq_ignore_ascii_case("false") {
                    return Value::Bool(false);
                }
                return Value::String(content.to_string());
            }
        }

        Value::Null
    }

    /// Escape special XML characters in DAX expressions.
    fn escape_xml(s: &str) -> String {
        s.replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;")
            .replace('\'', "&apos;")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_escape_xml() {
        let input = "IF(x < 5 && y > 10, \"YES\", \"NO\")";
        let escaped = ExecuteClient::escape_xml(input);
        assert!(!escaped.contains('<'));
        assert!(escaped.contains("&lt;"));
    }

    #[test]
    fn test_extract_column_names() {
        let xml = r#"<xsd:element name="Sales" /><xsd:element name="Amount" />"#;
        let cols = ExecuteClient::extract_column_names(xml);
        assert!(cols.contains(&"Sales".to_string()));
        assert!(cols.contains(&"Amount".to_string()));
    }
}
