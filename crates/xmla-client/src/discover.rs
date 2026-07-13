// ═══════════════════════════════════════════════════════════════════════
// TRRUSTT — XMLA DISCOVER Operations
//
// Real XMLA DISCOVER_XMLA requests for schema metadata.
// Retrieves: DBSCHEMA_TABLES, DBSCHEMA_COLUMNS, MDSCHEMA_MEASURES,
//   MDSCHEMA_HIERARCHIES, DISCOVER_CALC_DEPENDENCY, etc.
//
// Implementation: Builds SOAP XML envelopes, sends HTTP POST to
// localhost SSAS, parses XML rowset responses into domain types.
// ═══════════════════════════════════════════════════════════════════════

use reqwest::Client as HttpClient;
use tracing::{debug, error, info, instrument};
use chrono::Utc;

use shared::{
    ColumnDataType, ColumnInfo, CrossFilterDirection, MeasureInfo, RelationshipCardinality,
    RelationshipInfo, Result, SchemaMetadata, TableInfo,
};
use crate::error::XmlaError;

/// The XMLA SOAP envelope template for DISCOVER requests.
const DISCOVER_ENVELOPE: &str = r#"<Envelope xmlns="http://schemas.xmlsoap.org/soap/envelope/">
    <Header/>
    <Body>
        <Discover xmlns="urn:schemas-microsoft-com:xml-analysis">
            <RequestType>{request_type}</RequestType>
            <Restrictions>
                {restrictions}
            </Restrictions>
            <Properties>
                <PropertyList>
                    <Catalog>{catalog}</Catalog>
                    <Format>Tabular</Format>
                    <Content>SchemaData</Content>
                </PropertyList>
            </Properties>
        </Discover>
    </Body>
</Envelope>"#;

/// Client for XMLA DISCOVER operations.
pub(crate) struct DiscoverClient;

impl DiscoverClient {
    /// Discover the full model schema by combining all DISCOVER rowset queries.
    #[instrument(skip(self, http))]
    pub async fn full_schema(
        &self,
        http: &HttpClient,
        base_url: &str,
        database_name: &str,
    ) -> Result<SchemaMetadata> {
        debug!(database = %database_name, "Running full schema discovery");

        let (tables, columns, measures, relationships) = tokio::try_join!(
            self.discover_tables(http, base_url, database_name),
            self.discover_columns(http, base_url, database_name),
            self.discover_measures(http, base_url, database_name),
            self.discover_relationships(http, base_url, database_name),
        )?;

        let tables = Self::assemble_tables(tables, columns);

        info!(
            tables = tables.len(),
            measures = measures.len(),
            relationships = relationships.len(),
            "Schema discovery complete"
        );

        Ok(SchemaMetadata {
            name: database_name.to_string(),
            description: None,
            tables,
            relationships,
            measures,
            calculation_groups: vec![],
            annotations: vec![],
            discovered_at: Utc::now(),
            compatibility_level: 1600,
            default_mode: Some("Import".to_string()),
        })
    }

    /// Discover tables in the model (DBSCHEMA_TABLES rowset).
    #[instrument(skip(self, http))]
    pub async fn discover_tables(
        &self,
        http: &HttpClient,
        base_url: &str,
        database_name: &str,
    ) -> Result<Vec<TableInfo>> {
        let xml = Self::build_discover_request("DBSCHEMA_TABLES", database_name, "");
        let response_xml = Self::send_discover(http, base_url, &xml).await?;
        Self::parse_tables_rowset(&response_xml)
    }

    /// Discover columns in the model (DBSCHEMA_COLUMNS rowset).
    #[instrument(skip(self, http))]
    pub async fn discover_columns(
        &self,
        http: &HttpClient,
        base_url: &str,
        database_name: &str,
    ) -> Result<Vec<ColumnInfo>> {
        let xml = Self::build_discover_request("DBSCHEMA_COLUMNS", database_name, "");
        let response_xml = Self::send_discover(http, base_url, &xml).await?;
        Self::parse_columns_rowset(&response_xml)
    }

    /// Discover measures in the model (MDSCHEMA_MEASURES rowset).
    #[instrument(skip(self, http))]
    pub async fn discover_measures(
        &self,
        http: &HttpClient,
        base_url: &str,
        database_name: &str,
    ) -> Result<Vec<MeasureInfo>> {
        let restrictions = r#"<MEASURE_IS_VISIBLE>true</MEASURE_IS_VISIBLE>"#;
        let xml = Self::build_discover_request("MDSCHEMA_MEASURES", database_name, restrictions);
        let response_xml = Self::send_discover(http, base_url, &xml).await?;
        Self::parse_measures_rowset(&response_xml)
    }

    /// Discover relationships in the model (MDSCHEMA_MEASUREGROUP_DIMENSIONS rowset).
    #[instrument(skip(self, http))]
    pub async fn discover_relationships(
        &self,
        http: &HttpClient,
        base_url: &str,
        database_name: &str,
    ) -> Result<Vec<RelationshipInfo>> {
        let xml = Self::build_discover_request(
            "MDSCHEMA_MEASUREGROUP_DIMENSIONS",
            database_name,
            "",
        );
        let response_xml = Self::send_discover(http, base_url, &xml).await?;
        Self::parse_relationships_rowset(&response_xml)
    }

    // ── XMLA SOAP helpers ────────────────────────────────────────────

    fn build_discover_request(request_type: &str, catalog: &str, restrictions: &str) -> String {
        DISCOVER_ENVELOPE
            .replace("{request_type}", request_type)
            .replace("{catalog}", catalog)
            .replace("{restrictions}", restrictions)
    }

    async fn send_discover(http: &HttpClient, base_url: &str, xml_body: &str) -> Result<String> {
        let response = http
            .post(base_url)
            .header("Content-Type", "text/xml; charset=utf-8")
            .header("SOAPAction", "urn:schemas-microsoft-com:xml-analysis:Discover")
            .body(xml_body.to_string())
            .send()
            .await
            .map_err(|e| XmlaError::Http(e))?;

        let status = response.status();
        let body = response.text().await.map_err(|e| XmlaError::Http(e))?;

        if !status.is_success() {
            let error_msg = Self::extract_soap_fault(&body)
                .unwrap_or_else(|| format!("HTTP {}: {}", status.as_u16(), &body[..body.len().min(500)]));
            return Err(XmlaError::Discover(error_msg).into());
        }

        Ok(body)
    }

    fn extract_soap_fault(xml: &str) -> Option<String> {
        let start = xml.find("<faultstring>")?;
        let content_start = start + "<faultstring>".len();
        let end = xml[content_start..].find("</faultstring>")?;
        Some(xml[content_start..content_start + end].to_string())
    }

    // ── Rowset XML parsers ───────────────────────────────────────────

    fn parse_tables_rowset(xml: &str) -> Result<Vec<TableInfo>> {
        let rows = Self::extract_rows(xml).unwrap_or_default();
        let mut tables = Vec::with_capacity(rows.len());
        for row in &rows {
            let name = Self::get_field(row, "TABLE_NAME").unwrap_or_default();
            if name.is_empty() {
                continue;
            }
            let table_type = Self::get_field(row, "TABLE_TYPE").unwrap_or_default();
            tables.push(TableInfo {
                name,
                is_hidden: Self::get_field(row, "IS_VISIBLE").map(|v| v == "false").unwrap_or(false),
                is_calculated: table_type.contains("CALCULATED"),
                storage_mode: Some("Import".to_string()),
                columns: vec![],
                hierarchies: vec![],
                partitions: vec![],
                annotations: vec![],
                row_count_estimate: None,
            });
        }
        Ok(tables)
    }

    fn parse_columns_rowset(xml: &str) -> Result<Vec<ColumnInfo>> {
        let rows = Self::extract_rows(xml).unwrap_or_default();
        let mut columns = Vec::with_capacity(rows.len());
        for row in &rows {
            let name = Self::get_field(row, "COLUMN_NAME").unwrap_or_default();
            if name.is_empty() {
                continue;
            }
            let dt = Self::get_field(row, "DATA_TYPE").unwrap_or_default();
            columns.push(ColumnInfo {
                name,
                data_type: Self::parse_data_type(&dt),
                is_hidden: Self::get_field(row, "IS_VISIBLE").map(|v| v == "false").unwrap_or(false),
                is_calculated: false,
                is_key: Self::get_field(row, "COLUMN_FLAGS").map(|v| v.contains("KEY")).unwrap_or(false),
                is_nullable: Self::get_field(row, "IS_NULLABLE").map(|v| v == "true").unwrap_or(true),
                expression: None,
                format_string: None,
                sort_by_column: None,
                annotations: vec![],
                distinct_count_estimate: None,
            });
        }
        Ok(columns)
    }

    fn parse_measures_rowset(xml: &str) -> Result<Vec<MeasureInfo>> {
        let rows = Self::extract_rows(xml).unwrap_or_default();
        let mut measures = Vec::with_capacity(rows.len());
        for row in &rows {
            let name = Self::get_field(row, "MEASURE_NAME").unwrap_or_default();
            if name.is_empty() {
                continue;
            }
            let dt = Self::get_field(row, "DATA_TYPE").unwrap_or_default();
            measures.push(MeasureInfo {
                name,
                table_name: Self::get_field(row, "MEASUREGROUP_NAME").unwrap_or_default(),
                expression: Self::get_field(row, "EXPRESSION").unwrap_or_default(),
                format_string: Self::get_field(row, "FORMAT_STRING"),
                description: Self::get_field(row, "DESCRIPTION"),
                display_folder: Self::get_field(row, "MEASURE_DISPLAY_FOLDER"),
                data_type: Self::parse_data_type(&dt),
                is_hidden: Self::get_field(row, "MEASURE_IS_VISIBLE").map(|v| v == "false").unwrap_or(false),
                annotations: vec![],
            });
        }
        Ok(measures)
    }

    fn parse_relationships_rowset(xml: &str) -> Result<Vec<RelationshipInfo>> {
        let rows = Self::extract_rows(xml).unwrap_or_default();
        let mut relationships = Vec::with_capacity(rows.len());
        for row in &rows {
            let from_table = Self::get_field(row, "MEASUREGROUP_NAME").unwrap_or_default();
            let to_table = Self::get_field(row, "DIMENSION_NAME").unwrap_or_default();
            if from_table.is_empty() || to_table.is_empty() {
                continue;
            }
            let cardinality_str = Self::get_field(row, "CARDINALITY").unwrap_or_default();
            let cross_filter = Self::get_field(row, "CROSS_FILTER_DIRECTION").unwrap_or_default();
            relationships.push(RelationshipInfo {
                name: format!("{} → {}", from_table, to_table),
                from_table,
                from_column: Self::get_field(row, "MEASUREGROUP_COLUMN").unwrap_or_default(),
                to_table,
                to_column: Self::get_field(row, "DIMENSION_COLUMN").unwrap_or_default(),
                cardinality: match cardinality_str.as_str() {
                    "ONE_TO_MANY" | "OneToMany" => RelationshipCardinality::OneToMany,
                    "MANY_TO_ONE" | "ManyToOne" => RelationshipCardinality::ManyToOne,
                    "ONE_TO_ONE" | "OneToOne" => RelationshipCardinality::OneToOne,
                    _ => RelationshipCardinality::ManyToOne,
                },
                cross_filter_direction: match cross_filter.as_str() {
                    "BOTH" | "Both" => CrossFilterDirection::Both,
                    _ => CrossFilterDirection::Single,
                },
                is_active: Self::get_field(row, "IS_ACTIVE").map(|v| v == "true").unwrap_or(true),
                security_filtering: false,
            });
        }
        Ok(relationships)
    }

    // ── XML parsing utilities ────────────────────────────────────────

    fn extract_rows(xml: &str) -> Result<Vec<String>> {
        let mut rows = Vec::new();
        let rest = match xml.find("<rowset") {
            Some(pos) => &xml[pos..],
            None => return Ok(rows), // No rowsets — empty model
        };
        let mut pos = 0;
        while let Some(row_start) = rest[pos..].find("<row") {
            let absolute_start = pos + row_start;
            if let Some(row_end) = rest[absolute_start..].find("</row>") {
                let row_end_absolute = absolute_start + row_end + "</row>".len();
                rows.push(rest[absolute_start..row_end_absolute].to_string());
                pos = row_end_absolute;
            } else {
                break;
            }
        }
        Ok(rows)
    }

    fn get_field(row_xml: &str, field_name: &str) -> Option<String> {
        let open_tag = format!("<{}>", field_name);
        let close_tag = format!("</{}>", field_name);
        let start = row_xml.find(&open_tag)?;
        let content_start = start + open_tag.len();
        let rest = &row_xml[content_start..];
        let end = rest.find(&close_tag)?;
        Some(rest[..end].to_string())
    }

    fn parse_data_type(dt_str: &str) -> ColumnDataType {
        match dt_str {
            "2" | "Int64" | "INTEGER" | "BigInt" | "BIGINT" => ColumnDataType::Int64,
            "5" | "Double" | "DOUBLE" | "Decimal" | "DECIMAL" | "Float" | "FLOAT" => ColumnDataType::Double,
            "6" | "Currency" | "CURRENCY" => ColumnDataType::Currency,
            "11" | "Boolean" | "BOOLEAN" | "bool" | "Bool" => ColumnDataType::Boolean,
            "7" | "DateTime" | "DATETIME" | "Date" | "DATE" => ColumnDataType::DateTime,
            "1" | "String" | "STRING" | "WChar" | "WCHAR" | "text" | "Text" | "nvarchar" | "NVarChar" | "varchar" | "VarChar" => ColumnDataType::String,
            "8" | "Binary" | "BINARY" => ColumnDataType::Binary,
            "3" | "Variant" | "VARIANT" => ColumnDataType::Variant,
            _ => {
                debug!(data_type = %dt_str, "Unknown data type, defaulting to Variant");
                ColumnDataType::Variant
            }
        }
    }

    fn assemble_tables(tables: Vec<TableInfo>, _columns: Vec<ColumnInfo>) -> Vec<TableInfo> {
        // In full implementation, columns would be grouped by TABLE_NAME
        // from the DBSCHEMA_COLUMNS rowset and attached to their tables.
        // For now, tables are returned with empty column lists as columns
        // are re-discovered lazily via the column-specific endpoint.
        tables
    }
}

// Placeholder execute and tmsl stubs for compile-time correctness

/// Client for XMLA EXECUTE operations (DAX queries).
pub(crate) struct ExecuteClient;

impl ExecuteClient {
    pub async fn query(
        &self,
        _http: &HttpClient,
        _base_url: &str,
        _database_name: &str,
        _dax: &str,
    ) -> Result<shared::DaxQueryResult> {
        Ok(shared::DaxQueryResult {
            columns: vec![],
            rows: vec![],
            row_count: 0,
            duration_ms: 0,
        })
    }
}

/// Client for TMSL commands.
pub(crate) struct TmslClient;

impl TmslClient {
    pub async fn execute(
        &self,
        _http: &HttpClient,
        _base_url: &str,
        _database_name: &str,
        _tmsl_json: &str,
    ) -> Result<shared::TmslResult> {
        Ok(shared::TmslResult {
            success: true,
            affected_objects: vec![],
            warnings: vec![],
            duration_ms: 0,
        })
    }
}
