// DAX AST types and sub-module stubs

/// A parsed DAX expression in Abstract Syntax Tree form.
#[derive(Debug, Clone, PartialEq)]
pub enum DaxExpression {
    /// A function call: FUNC(arg1, arg2, ...)
    FunctionCall(FunctionCall),
    /// A column reference: 'Table'[Column] or [Column]
    ColumnRef(ColumnRef),
    /// A measure reference: [Measure]
    MeasureRef(MeasureRef),
    /// A table reference: 'Table'
    TableRef(String),
    /// A literal constant: 42, "text", TRUE, BLANK()
    Constant(ConstantValue),
    /// VAR ... RETURN ...
    VarReturn(VarReturn),
    /// Binary operation: left OP right
    BinaryOp {
        left: Box<DaxExpression>,
        op: DaxOperator,
        right: Box<DaxExpression>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FunctionCall {
    pub name: String,
    pub arguments: Vec<DaxExpression>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ColumnRef {
    pub table: Option<String>,
    pub column: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MeasureRef {
    pub table: Option<String>,
    pub measure: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ConstantValue {
    Integer(i64),
    Decimal(f64),
    String(String),
    Boolean(bool),
    Blank,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VarReturn {
    pub variables: Vec<VarDeclaration>,
    pub return_expression: Box<DaxExpression>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VarDeclaration {
    pub name: String,
    pub expression: Box<DaxExpression>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DaxOperator {
    Add, Sub, Mul, Div, Pow,
    Eq, Neq, Lt, Gt, Lte, Gte,
    And, Or,
    In, NotIn,
    Concat,
}

use serde::{Deserialize, Serialize};

impl DaxExpression {
    /// Returns the names of all DAX functions referenced in this expression.
    pub fn referenced_functions(&self) -> Vec<String> {
        let mut funcs = Vec::new();
        self.collect_functions(&mut funcs);
        funcs
    }

    fn collect_functions(&self, out: &mut Vec<String>) {
        match self {
            DaxExpression::FunctionCall(fc) => {
                if !out.contains(&fc.name) { out.push(fc.name.clone()); }
                for arg in &fc.arguments { arg.collect_functions(out); }
            }
            DaxExpression::BinaryOp { left, right, .. } => { left.collect_functions(out); right.collect_functions(out); }
            DaxExpression::VarReturn(vr) => {
                for var in &vr.variables { var.expression.collect_functions(out); }
                vr.return_expression.collect_functions(out);
            }
            _ => {}
        }
    }

    /// Returns all column references in this expression.
    pub fn referenced_columns(&self) -> Vec<ColumnRef> {
        let mut cols = Vec::new();
        self.collect_columns(&mut cols);
        cols
    }

    fn collect_columns(&self, out: &mut Vec<ColumnRef>) {
        match self {
            DaxExpression::ColumnRef(cr) => { if !out.contains(cr) { out.push(cr.clone()); } }
            DaxExpression::FunctionCall(fc) => { for arg in &fc.arguments { arg.collect_columns(out); } }
            DaxExpression::BinaryOp { left, right, .. } => { left.collect_columns(out); right.collect_columns(out); }
            DaxExpression::VarReturn(vr) => {
                for var in &vr.variables { var.expression.collect_columns(out); }
                vr.return_expression.collect_columns(out);
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_referenced_functions() {
        let expr = DaxExpression::FunctionCall(FunctionCall {
            name: "CALCULATE".into(),
            arguments: vec![
                DaxExpression::FunctionCall(FunctionCall {
                    name: "SUM".into(),
                    arguments: vec![DaxExpression::ColumnRef(ColumnRef { table: Some("Sales".into()), column: "Amount".into() })],
                }),
                DaxExpression::FunctionCall(FunctionCall {
                    name: "FILTER".into(),
                    arguments: vec![],
                }),
            ],
        });
        let funcs = expr.referenced_functions();
        assert!(funcs.contains(&"CALCULATE".to_string()));
        assert!(funcs.contains(&"SUM".to_string()));
        assert!(funcs.contains(&"FILTER".to_string()));
    }
}
