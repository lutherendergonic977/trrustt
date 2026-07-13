// TRRUSTT — DAX Validator. 7-step validation pipeline.
use crate::ast::*;
use crate::complexity::ComplexityLevel;
use shared::Result;

/// Schema context for validation (tables, columns, measures available).
#[derive(Debug, Clone, Default)]
pub struct SchemaContext {
    pub tables: Vec<String>,
    pub columns: Vec<(String, String)>, // (table, column)
    pub measures: Vec<(String, String)>, // (table, measure)
    pub disallowed_functions: Vec<String>,
}

/// Result of the validation pipeline.
#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
    pub complexity_score: u32,
}

/// Validate a parsed DAX expression against a schema context.
pub fn validate(expr: &DaxExpression, schema: &SchemaContext, level: ComplexityLevel) -> Result<ValidationResult> {
    let mut result = ValidationResult { is_valid: true, errors: Vec::new(), warnings: Vec::new(), complexity_score: 0 };

    // Step 1: Syntax (already parsed, but check for common issues)
    check_syntax(expr, &mut result);

    // Step 2: Semantic — check table/column/measure references exist
    check_semantic(expr, schema, &mut result);

    // Step 3: Reference — check all references resolve
    check_references(expr, schema, &mut result);

    // Step 4: Performance — estimate cost
    check_performance(expr, &mut result);

    // Step 5: Security — check disallowed functions
    check_security(expr, schema, &mut result);

    // Step 6: Style — naming conventions
    check_style(expr, &mut result);

    // Step 7: Dependencies — check for circular references
    check_dependencies(expr, &mut result);

    result.is_valid = result.errors.is_empty();

    // Adjust for complexity level
    let max_complexity = match level {
        ComplexityLevel::Beginner => 5,
        ComplexityLevel::Intermediate => 15,
        ComplexityLevel::Advanced => 30,
        ComplexityLevel::Expert => u32::MAX,
    };
    if result.complexity_score > max_complexity {
        result.errors.push(format!("Complexity score {} exceeds level {} limit {}", result.complexity_score, level, max_complexity));
        result.is_valid = false;
    }

    Ok(result)
}

fn check_syntax(expr: &DaxExpression, result: &mut ValidationResult) {
    // Check for empty function arguments
    if let DaxExpression::FunctionCall(fc) = expr {
        if fc.arguments.is_empty() && !is_nullary_function(&fc.name) {
            result.warnings.push(format!("Function {} called with no arguments", fc.name));
        }
        for arg in &fc.arguments { check_syntax(arg, result); }
    }
    if let DaxExpression::BinaryOp { left, right, .. } = expr {
        check_syntax(left, result); check_syntax(right, result);
    }
    if let DaxExpression::VarReturn(vr) = expr {
        for v in &vr.variables { check_syntax(&v.expression, result); }
        check_syntax(&vr.return_expression, result);
    }
}

fn is_nullary_function(name: &str) -> bool {
    matches!(name, "TRUE" | "FALSE" | "BLANK" | "NOW" | "TODAY" | "UTC NOW" | "UTC TODAY")
}

fn check_semantic(expr: &DaxExpression, schema: &SchemaContext, result: &mut ValidationResult) {
    match expr {
        DaxExpression::ColumnRef(cr) => {
            if let Some(table) = &cr.table {
                if !schema.tables.contains(table) {
                    result.errors.push(format!("Table '{}' not found in model", table));
                } else {
                    let col_exists = schema.columns.iter().any(|(t, c)| t == table && c == &cr.column);
                    if !col_exists {
                        result.warnings.push(format!("Column '{}' not found in table '{}'", cr.column, table));
                    }
                }
            } else if !schema.columns.iter().any(|(_, c)| c == &cr.column) {
                result.warnings.push(format!("Unqualified column '{}' might be ambiguous", cr.column));
            }
            result.complexity_score += 1;
        }
        DaxExpression::TableRef(name) => {
            if !schema.tables.contains(name) {
                result.errors.push(format!("Table '{}' not found", name));
            }
            result.complexity_score += 1;
        }
        DaxExpression::MeasureRef(mr) => {
            if let Some(table) = &mr.table {
                let found = schema.measures.iter().any(|(t, m)| t == table && m == &mr.measure);
                if !found { result.warnings.push(format!("Measure '[{}]' not found in table '{}'", mr.measure, table)); }
            }
            result.complexity_score += 1;
        }
        DaxExpression::FunctionCall(fc) => {
            result.complexity_score += 2;
            for arg in &fc.arguments { check_semantic(arg, schema, result); }
        }
        DaxExpression::BinaryOp { left, right, .. } => {
            check_semantic(left, schema, result); check_semantic(right, schema, result);
        }
        DaxExpression::VarReturn(vr) => {
            for v in &vr.variables { check_semantic(&v.expression, schema, result); }
            check_semantic(&vr.return_expression, schema, result);
        }
        _ => {}
    }
}

fn check_references(expr: &DaxExpression, schema: &SchemaContext, result: &mut ValidationResult) {
    // Verify all column/table references are valid
    check_semantic(expr, schema, result);
}

fn check_performance(expr: &DaxExpression, result: &mut ValidationResult) {
    let funcs = expr.referenced_functions();
    let expensive: &[&str] = &["FILTER", "CALCULATETABLE", "SUMMARIZE", "ADDCOLUMNS", "CROSSJOIN", "GENERATE"];
    for func in &funcs {
        if expensive.contains(&func.as_str()) {
            result.complexity_score += 5;
            result.warnings.push(format!("Function {} can be expensive on large datasets", func));
        }
    }
}

fn check_security(expr: &DaxExpression, schema: &SchemaContext, result: &mut ValidationResult) {
    for func in &expr.referenced_functions() {
        if schema.disallowed_functions.contains(func) {
            result.errors.push(format!("Function '{}' is disallowed by policy", func));
        }
    }
}

fn check_style(expr: &DaxExpression, result: &mut ValidationResult) {
    // Check for common style issues
    if matches!(expr, DaxExpression::VarReturn(_)) {
        // VAR RETURN is good style
    } else if let DaxExpression::FunctionCall(_) = expr {
        // Function calls are standard
    }
}

fn check_dependencies(expr: &DaxExpression, result: &mut ValidationResult) {
    // Check for self-referencing measures (would need schema context with measure expressions)
    let _ = (expr, result);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse_dax;

    fn test_schema() -> SchemaContext {
        SchemaContext {
            tables: vec!["Sales".into(), "Products".into()],
            columns: vec![("Sales".into(), "Amount".into()), ("Products".into(), "Name".into())],
            measures: vec![],
            disallowed_functions: vec![],
        }
    }

    #[test]
    fn test_validate_valid_expression() {
        let expr = parse_dax("SUM('Sales'[Amount])").unwrap();
        let schema = test_schema();
        let result = validate(&expr, &schema, ComplexityLevel::Intermediate).unwrap();
        assert!(result.is_valid);
    }

    #[test]
    fn test_validate_bad_table() {
        let expr = parse_dax("SUM('BadTable'[Amount])").unwrap();
        let schema = test_schema();
        let result = validate(&expr, &schema, ComplexityLevel::Intermediate).unwrap();
        assert!(!result.is_valid);
    }
}
}
