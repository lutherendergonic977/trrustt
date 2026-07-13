// TRRUSTT — DAX Explainer. Natural language explanation of DAX.
use crate::ast::*;
use crate::validator::SchemaContext;
use shared::Result;

/// A natural language explanation of a DAX expression.
#[derive(Debug, Clone)]
pub struct DaxExplanation {
    pub summary: String,
    pub steps: Vec<String>,
    pub complexity: String,
}

/// Explain a DAX expression in natural language.
pub fn explain_dax(expression: &str, _schema: &SchemaContext) -> Result<DaxExplanation> {
    let parsed = crate::parser::parse_dax(expression)?;
    let mut steps = Vec::new();

    explain_recursive(&parsed, &mut steps, 0);

    let summary = if steps.is_empty() {
        format!("This measure computes: {}", expression)
    } else {
        format!("This measure {} {}", steps.first().map(|s| s.as_str()).unwrap_or("calculates"), if steps.len() > 1 { format!("in {} steps", steps.len()) } else { String::new() })
    };

    let complexity = match steps.len() {
        0..=1 => "simple",
        2..=4 => "moderate",
        _ => "complex",
    };

    Ok(DaxExplanation { summary, steps, complexity: complexity.to_string() })
}

fn explain_recursive(expr: &DaxExpression, steps: &mut Vec<String>, depth: usize) {
    let indent = "  ".repeat(depth);

    match expr {
        DaxExpression::FunctionCall(fc) => {
            let desc = describe_function(&fc.name, &fc.arguments);
            steps.push(format!("{}{}", indent, desc));
            for arg in &fc.arguments {
                explain_recursive(arg, steps, depth + 1);
            }
        }
        DaxExpression::ColumnRef(cr) => {
            let table = cr.table.as_deref().unwrap_or("<table>");
            steps.push(format!("{}references column '{}'[{}]", indent, table, cr.column));
        }
        DaxExpression::MeasureRef(mr) => {
            let table = mr.table.as_deref().unwrap_or("<table>");
            steps.push(format!("{}references measure '{}'[{}]", indent, table, mr.measure));
        }
        DaxExpression::TableRef(t) => {
            steps.push(format!("{}references table '{}'", indent, t));
        }
        DaxExpression::Constant(c) => {
            steps.push(format!("{}uses constant {}", indent, describe_constant(c)));
        }
        DaxExpression::BinaryOp { left, right, op } => {
            steps.push(format!("{}performs {} operation", indent, describe_op(*op)));
            explain_recursive(left, steps, depth + 1);
            explain_recursive(right, steps, depth + 1);
        }
        DaxExpression::VarReturn(vr) => {
            steps.push(format!("{}defines {} variable(s) and returns a result", indent, vr.variables.len()));
            for var in &vr.variables {
                steps.push(format!("{}  variable '{}' =", indent, var.name));
                explain_recursive(&var.expression, steps, depth + 2);
            }
            steps.push(format!("{}  returns:", indent));
            explain_recursive(&vr.return_expression, steps, depth + 2);
        }
        DaxExpression::Parenthesized(inner) => {
            explain_recursive(inner, steps, depth);
        }
        _ => {}
    }
}

fn describe_function(name: &str, args: &[DaxExpression]) -> String {
    match name {
        "CALCULATE" => format!("evaluates an expression with modified filter context ({} args)", args.len()),
        "FILTER" => "filters a table based on a condition".to_string(),
        "SUM" => "sums up numeric values".to_string(),
        "AVERAGE" | "AVERAGEX" => "calculates the average of values".to_string(),
        "COUNT" | "COUNTROWS" => "counts rows".to_string(),
        "DISTINCTCOUNT" => "counts distinct values".to_string(),
        "MIN" => "finds the minimum value".to_string(),
        "MAX" => "finds the maximum value".to_string(),
        "IF" => "evaluates a conditional expression".to_string(),
        "SWITCH" => "evaluates multiple conditions".to_string(),
        "DIVIDE" => "performs safe division (handles division by zero)".to_string(),
        "ALL" => "removes all filters from a table or column".to_string(),
        "ALLEXCEPT" => "removes all filters except specified columns".to_string(),
        "VALUES" => "returns unique values from a column".to_string(),
        "RELATED" => "fetches a value from a related table".to_string(),
        "SAMEPERIODLASTYEAR" => "shifts dates back by one year".to_string(),
        "DATEADD" => "shifts dates by a specified interval".to_string(),
        "TOTALYTD" | "DATESYTD" => "computes year-to-date values".to_string(),
        "RANKX" => "ranks values in a table".to_string(),
        _ => format!("calls the {} function", name),
    }
}

fn describe_constant(c: &ConstantValue) -> String {
    match c {
        ConstantValue::Integer(i) => i.to_string(),
        ConstantValue::Decimal(d) => format!("{:.2}", d),
        ConstantValue::String(s) => format!("\"{}\"", s),
        ConstantValue::Boolean(b) => b.to_string(),
        ConstantValue::Blank => "BLANK()".to_string(),
    }
}

fn describe_op(op: DaxOperator) -> String {
    match op {
        DaxOperator::Add => "addition (+)",
        DaxOperator::Sub => "subtraction (-)",
        DaxOperator::Mul => "multiplication (*)",
        DaxOperator::Div => "division (/)",
        DaxOperator::Eq => "equality (=)",
        DaxOperator::Neq => "not-equal (<>)",
        DaxOperator::Lt => "less-than (<)",
        DaxOperator::Gt => "greater-than (>)",
        DaxOperator::And => "logical AND (&&)",
        DaxOperator::Or => "logical OR (||)",
        _ => "comparison",
    }
    .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_explain_simple_sum() {
        let explanation = explain_dax("SUM('Sales'[Amount])", &SchemaContext::default()).unwrap();
        assert!(!explanation.steps.is_empty());
        assert_eq!(explanation.complexity, "simple");
    }
}
}
