// TRRUSTT — DAX PEG Parser.
// Parses DAX expressions into AST using pom PEG combinators.
use pom::parser::*;
use pom::Parser;
use crate::ast::*;
use shared::Result;

fn space<'a>() -> Parser<'a, u8, ()> { one_of(b" \t\r\n").repeat(0..).discard() }
fn identifier<'a>() -> Parser<'a, u8, String> {
    (one_of(b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz_") -
     one_of(b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz_0123456789").repeat(0..))
    .map(|(c, rest)| { let mut s = String::new(); s.push(c as char); s.extend(rest.iter().map(|&b| b as char)); s })
}
fn integer<'a>() -> Parser<'a, u8, ConstantValue> {
    one_of(b"0123456789").repeat(1..).map(|d| ConstantValue::Integer(
        String::from_utf8_lossy(&d).parse().unwrap_or(0)))
}
fn string_lit<'a>() -> Parser<'a, u8, ConstantValue> {
    (sym(b'"') - none_of(b"\"").repeat(0..) - sym(b'"'))
        .map(|(_, chars, _)| ConstantValue::String(String::from_utf8_lossy(&chars).to_string()))
}
fn boolean<'a>() -> Parser<'a, u8, ConstantValue> {
    seq(b"TRUE").map(|_| ConstantValue::Boolean(true)) |
    seq(b"FALSE").map(|_| ConstantValue::Boolean(false))
}
fn constant<'a>() -> Parser<'a, u8, ConstantValue> {
    string_lit | integer | boolean
}
fn column_ref<'a>() -> Parser<'a, u8, ColumnRef> {
    (opt(sym(b'\'') - identifier() - sym(b'\'')) - sym(b'[') - identifier() - sym(b']'))
        .map(|(table_opt, (_, col))| ColumnRef { table: table_opt.map(|(_, t, _)| t), column: col })
}
fn args<'a>() -> Parser<'a, u8, Vec<DaxExpression>> {
    list(call(dax_expr), sym(b',') - space())
}
fn function_call<'a>() -> Parser<'a, u8, DaxExpression> {
    (identifier() - sym(b'(') - space() - args() - space() - sym(b')'))
        .map(|((name, _), a)| DaxExpression::FunctionCall(FunctionCall { name, arguments: a }))
}
fn table_ref<'a>() -> Parser<'a, u8, DaxExpression> {
    (sym(b'\'') - identifier() - sym(b'\''))
        .map(|(_, name)| DaxExpression::TableRef(name))
}
fn parenthesized<'a>() -> Parser<'a, u8, DaxExpression> {
    (sym(b'(') - space() - call(dax_expr) - space() - sym(b')'))
        .map(|(_, e)| DaxExpression::Parenthesized(Box::new(e)))
}
fn atom<'a>() -> Parser<'a, u8, DaxExpression> {
    parenthesized | function_call | column_ref().map(DaxExpression::ColumnRef) |
    table_ref | constant().map(DaxExpression::Constant)
}
fn binary_op<'a>() -> Parser<'a, u8, DaxExpression> {
    let ops: [(&[u8], DaxOperator); 15] = [
        (b"&&", DaxOperator::And), (b"||", DaxOperator::Or),
        (b"<>", DaxOperator::Neq), (b"<=", DaxOperator::Lte), (b">=", DaxOperator::Gte),
        (b"=", DaxOperator::Eq), (b"<", DaxOperator::Lt), (b">", DaxOperator::Gt),
        (b"+", DaxOperator::Add), (b"-", DaxOperator::Sub),
        (b"*", DaxOperator::Mul), (b"/", DaxOperator::Div),
        (b"^", DaxOperator::Pow), (b"&", DaxOperator::Concat),
        (b"IN", DaxOperator::In),
    ];
    // Simple left-associative binary operations for practical DAX parsing
    atom.clone() - space() - one_of(ops.iter().map(|(s, _)| *s).collect::<Vec<_>>().as_slice())
        .map(|b| {
            let op = ops.iter().find(|(s, _)| s.len() == 1 && s[0] == b).map(|(_, o)| *o)
                .or_else(|| ops.iter().find(|(s, _)| s.len() == 2 && s[0] == b).map(|(_, o)| *o))
                .unwrap_or(DaxOperator::Add);
            op
        }) - space() - call(atom.clone())
        .map(|((left, op), right)| DaxExpression::BinaryOp {
            left: Box::new(left), op, right: Box::new(right),
        })
}
fn var_return<'a>() -> Parser<'a, u8, DaxExpression> {
    let var_decl = (seq(b"VAR") - space() - identifier() - space() - sym(b'=') - space() - call(dax_expr))
        .map(|((((name, _), _), e), _)| VarDeclaration { name, expression: Box::new(e) });
    let ret = (seq(b"RETURN") - space() - call(dax_expr))
        .map(|(_, e)| e);
    (var_decl.repeat(0..) - space() - ret)
        .map(|(vars, ret_expr)| DaxExpression::VarReturn(VarReturn { variables: vars, return_expression: Box::new(ret_expr) }))
}
fn dax_expr<'a>() -> Parser<'a, u8, DaxExpression> {
    var_return | binary_op() | atom()
}

/// Parse a DAX string into an AST.
pub fn parse_dax(input: &str) -> Result<DaxExpression> {
    let bytes = input.trim().as_bytes();
    if bytes.is_empty() {
        return Ok(DaxExpression::Constant(ConstantValue::Blank));
    }
    dax_expr().parse(bytes).map_err(|e| {
        shared::AppError::DaxParse {
            message: format!("Parse error at position {}: {:?}", e.position, e),
            position: e.position,
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_simple_function() {
        let r = parse_dax("SUM('Sales'[Amount])");
        assert!(r.is_ok());
    }
    #[test]
    fn test_integer() {
        let r = parse_dax("42");
        assert!(r.is_ok());
    }
    #[test]
    fn test_var_return() {
        let r = parse_dax("VAR x = 5 RETURN x + 1");
        assert!(r.is_ok());
    }
}
}
