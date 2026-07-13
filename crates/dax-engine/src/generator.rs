// TRRUSTT — DAX Generator. AI-powered DAX measure generation.
use crate::complexity::ComplexityLevel;
use crate::validator::SchemaContext;
use shared::Result;

/// A generated DAX measure with metadata.
#[derive(Debug, Clone)]
pub struct GeneratedMeasure {
    pub name: String,
    pub expression: String,
    pub table_name: String,
    pub description: String,
    pub complexity: ComplexityLevel,
}

/// Generate a DAX measure from a natural language description.
/// Uses the schema context to determine available tables/columns.
pub fn generate_measure(
    description: &str,
    schema: &SchemaContext,
    level: ComplexityLevel,
) -> Result<GeneratedMeasure> {
    // Analyze the description for key business concepts
    let name = infer_measure_name(description);
    let table = infer_table(description, schema);
    let expression = build_expression(description, schema, level);

    Ok(GeneratedMeasure {
        name,
        expression,
        table_name: table,
        description: description.to_string(),
        complexity: level,
    })
}

/// Infer a measure name from the description.
fn infer_measure_name(description: &str) -> String {
    let desc_lower = description.to_lowercase();

    if desc_lower.contains("yoy") || desc_lower.contains("year over year") {
        return format!("YoY {}", extract_keyword(description));
    }
    if desc_lower.contains("mom") || desc_lower.contains("month over month") {
        return format!("MoM {}", extract_keyword(description));
    }
    if desc_lower.contains("running total") || desc_lower.contains("cumulative") {
        return format!("{} RT", extract_keyword(description));
    }
    if desc_lower.contains("%") || desc_lower.contains("percent") || desc_lower.contains("share") {
        return format!("{} %", extract_keyword(description));
    }
    if desc_lower.contains("rank") || desc_lower.contains("top") {
        return format!("{} Rank", extract_keyword(description));
    }

    // Default: PascalCase the key metric
    extract_keyword(description)
}

/// Extract a key keyword from the description for naming.
fn extract_keyword(desc: &str) -> String {
    let keywords = ["sales", "revenue", "profit", "cost", "margin", "quantity", "growth", "count"];
    for kw in &keywords {
        if desc.to_lowercase().contains(kw) {
            let mut c = kw.chars();
            match c.next() {
                None => return "Measure".to_string(),
                Some(f) => return format!("{}{}", f.to_uppercase(), c.collect::<String>()),
            }
        }
    }
    "Measure".to_string()
}

/// Infer the table name from the description and schema.
fn infer_table(description: &str, schema: &SchemaContext) -> String {
    let desc_lower = description.to_lowercase();

    // Try to match table names from schema
    for table in &schema.tables {
        if desc_lower.contains(&table.to_lowercase()) {
            return table.clone();
        }
    }

    // If mentions a column, find its table
    for (table, col) in &schema.columns {
        if desc_lower.contains(&col.to_lowercase()) {
            return table.clone();
        }
    }

    // Default to first table
    schema.tables.first().cloned().unwrap_or_else(|| "Sales".to_string())
}

/// Build a basic DAX expression from the description.
fn build_expression(description: &str, schema: &SchemaContext, level: ComplexityLevel) -> String {
    let desc_lower = description.to_lowercase();
    let table = infer_table(description, schema);

    // Find a numeric column to aggregate
    let numeric_col = schema.columns.iter()
        .find(|(t, c)| t == &table && (c.to_lowercase().contains("amount") || c.to_lowercase().contains("value") || c.to_lowercase().contains("price") || c.to_lowercase().contains("quantity")))
        .map(|(_, c)| c.clone())
        .unwrap_or_else(|| "Amount".to_string());

    if level <= ComplexityLevel::Beginner {
        if desc_lower.contains("count") || desc_lower.contains("number of") {
            return format!("COUNTROWS('{}')", table);
        }
        if desc_lower.contains("distinct") || desc_lower.contains("unique") {
            return format!("DISTINCTCOUNT('{}'[{}])", table, find_dimension_col(schema, &table));
        }
        return format!("SUM('{}'[{}])", table, numeric_col);
    }

    if level <= ComplexityLevel::Intermediate {
        if desc_lower.contains("yoy") || desc_lower.contains("year over year") {
            return format!(
                "VAR __Current = SUM('{}'[{}])\nVAR __Previous = CALCULATE(SUM('{}'[{}]), SAMEPERIODLASTYEAR('Date'[Date]))\nRETURN\n    DIVIDE(__Current - __Previous, __Previous)",
                table, numeric_col, table, numeric_col
            );
        }
        if desc_lower.contains("%") || desc_lower.contains("share") {
            return format!(
                "DIVIDE(\n    SUM('{}'[{}]),\n    CALCULATE(SUM('{}'[{}]), ALL('{}'))\n)",
                table, numeric_col, table, numeric_col, table
            );
        }
        return format!(
            "CALCULATE(\n    SUM('{}'[{}]),\n    ALL('{}')\n)",
            table, numeric_col, table
        );
    }

    // Advanced/Expert
    format!(
        "VAR __Total = SUM('{}'[{}])\nVAR __Result =\n    CALCULATE(\n        __Total,\n        FILTER(ALL('{}'), '{}'[{}] > 0)\n    )\nRETURN\n    __Result",
        table, numeric_col, table, table, numeric_col
    )
}

fn find_dimension_col(schema: &SchemaContext, table: &str) -> String {
    schema.columns.iter()
        .find(|(t, c)| t == table && !c.to_lowercase().contains("amount") && !c.to_lowercase().contains("value"))
        .map(|(_, c)| c.clone())
        .unwrap_or_else(|| "ID".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_basic_sum() {
        let schema = SchemaContext {
            tables: vec!["Sales".into()],
            columns: vec![("Sales".into(), "Amount".into()), ("Sales".into(), "ProductID".into())],
            measures: vec![],
            disallowed_functions: vec![],
        };
        let result = generate_measure("Total sales amount", &schema, ComplexityLevel::Beginner).unwrap();
        assert!(result.expression.contains("SUM"));
        assert_eq!(result.table_name, "Sales");
    }

    #[test]
    fn test_infer_measure_name() {
        assert_eq!(infer_measure_name("YoY revenue growth"), "YoY Revenue");
        assert_eq!(infer_measure_name("Customer count"), "Count");
    }
}
}
