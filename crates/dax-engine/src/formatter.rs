// TRRUSTT — DAX Formatter. Pretty-printer for DAX expressions.
use crate::ast::*;
use shared::Result;

/// Format a parsed DAX expression into a nicely indented string.
pub fn format_ast(expr: &DaxExpression, indent: usize) -> String {
    let pad = " ".repeat(indent * 4);
    match expr {
        DaxExpression::FunctionCall(fc) => {
            let args: Vec<String> = fc.arguments.iter().map(|a| format_ast(a, indent + 1)).collect();
            if args.iter().all(|a| !a.contains('\n')) && args.join(", ").len() < 60 {
                format!("{}{}({})", pad, fc.name, args.join(", "))
            } else {
                format!("{}{}(\n{}\n{})", pad, fc.name, args.join(",\n"), pad)
            }
        }
        DaxExpression::ColumnRef(cr) => {
            match &cr.table {
                Some(t) => format!("'{}'[{}]", t, cr.column),
                None => format!("[{}]", cr.column),
            }
        }
        DaxExpression::MeasureRef(mr) => {
            match &mr.table {
                Some(t) => format!("'{}'[{}]", t, mr.measure),
                None => format!("[{}]", mr.measure),
            }
        }
        DaxExpression::TableRef(t) => format!("'{}'", t),
        DaxExpression::Constant(c) => match c {
            ConstantValue::Integer(i) => i.to_string(),
            ConstantValue::Decimal(d) => format!("{:.2}", d),
            ConstantValue::String(s) => format!("\"{}\"", s),
            ConstantValue::Boolean(b) => b.to_string().to_uppercase(),
            ConstantValue::Blank => "BLANK()".to_string(),
        },
        DaxExpression::BinaryOp { left, right, op } => {
            format!("{} {} {}", format_ast(left, indent), op_to_str(*op), format_ast(right, indent))
        }
        DaxExpression::VarReturn(vr) => {
            let mut result = String::new();
            for var in &vr.variables {
                result.push_str(&format!("{}VAR {} =\n{}{}\n", pad, var.name, pad, format_ast(&var.expression, indent + 1)));
            }
            result.push_str(&format!("{}RETURN\n{}{}", pad, pad, format_ast(&vr.return_expression, indent + 1)));
            result
        }
        DaxExpression::Parenthesized(inner) => format!("({})", format_ast(inner, indent)),
        _ => "<?>".to_string(),
    }
}

fn op_to_str(op: DaxOperator) -> &'static str {
    match op {
        DaxOperator::Add => "+", DaxOperator::Sub => "-", DaxOperator::Mul => "*", DaxOperator::Div => "/",
        DaxOperator::Pow => "^", DaxOperator::Eq => "=", DaxOperator::Neq => "<>",
        DaxOperator::Lt => "<", DaxOperator::Gt => ">", DaxOperator::Lte => "<=", DaxOperator::Gte => ">=",
        DaxOperator::And => "&&", DaxOperator::Or => "||", DaxOperator::Concat => "&",
        DaxOperator::In => "IN", DaxOperator::NotIn => "NOT IN",
    }
}

/// Format a raw DAX string by parsing and pretty-printing.
pub fn format_dax(input: &str) -> Result<String> {
    let parsed = crate::parser::parse_dax(input)?;
    Ok(format_ast(&parsed, 0))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_simple() {
        let result = format_dax("SUM('Sales'[Amount])").unwrap();
        assert!(result.contains("SUM"));
    }

    #[test]
    fn test_format_constant() {
        let c = DaxExpression::Constant(ConstantValue::Integer(42));
        assert_eq!(format_ast(&c, 0), "42");
    }
}
