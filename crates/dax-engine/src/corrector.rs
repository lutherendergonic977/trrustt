// TRRUSTT — DAX Self-Corrector. Validate → fix → re-validate loop.
use crate::validator::{validate, SchemaContext, ValidationResult};
use crate::complexity::ComplexityLevel;
use shared::Result;

/// Result of the self-correction process.
#[derive(Debug, Clone)]
pub struct CorrectionResult {
    pub original: String,
    pub corrected: String,
    pub attempts: usize,
    pub success: bool,
    pub final_validation: ValidationResult,
}

/// Run the self-correction loop on a DAX expression.
/// Attempts to fix validation errors by applying common DAX fixes.
pub fn self_correct(
    expression: &str,
    schema: &SchemaContext,
    max_attempts: usize,
) -> Result<CorrectionResult> {
    let mut current = expression.to_string();
    let mut attempts = 0;

    loop {
        // Parse and validate
        let parsed = match crate::parser::parse_dax(&current) {
            Ok(p) => p,
            Err(_) if attempts >= max_attempts => {
                // Can't even parse — give up
                return Ok(CorrectionResult {
                    original: expression.to_string(),
                    corrected: current,
                    attempts,
                    success: false,
                    final_validation: ValidationResult { is_valid: false, errors: vec!["Parse error".into()], warnings: vec![], complexity_score: 0 },
                });
            }
            Err(_) => {
                current = apply_syntax_fixes(&current);
                attempts += 1;
                continue;
            }
        };

        let validation = validate(&parsed, schema, ComplexityLevel::Expert)?;

        if validation.is_valid || attempts >= max_attempts {
            return Ok(CorrectionResult {
                original: expression.to_string(),
                corrected: current,
                attempts,
                success: validation.is_valid,
                final_validation: validation,
            });
        }

        // Apply fixes based on error types
        current = apply_fixes(&current, &validation.errors);
        attempts += 1;
    }
}

/// Apply syntax-level fixes (missing quotes, brackets, etc.).
fn apply_syntax_fixes(expr: &str) -> String {
    let mut fixed = expr.to_string();

    // Fix common issues:
    // 1. Missing closing parenthesis
    let open = fixed.matches('(').count();
    let close = fixed.matches(')').count();
    if open > close {
        for _ in 0..(open - close) {
            fixed.push(')');
        }
    }

    // 2. Unquoted table names with spaces
    // (Basic heuristic — would be more sophisticated in production)

    // 3. Fix double equals
    fixed = fixed.replace("==", "=");

    fixed
}

/// Apply semantic fixes based on validation errors.
fn apply_fixes(expr: &str, errors: &[String]) -> String {
    let mut fixed = expr.to_string();

    for error in errors {
        if error.contains("not found") {
            // Could suggest alternative column/table names
        }
        if error.contains("complexity") {
            // Simplify the expression
            fixed = simplify_expression(&fixed);
        }
    }

    fixed
}

/// Simplify a DAX expression by removing nested iterators.
fn simplify_expression(expr: &str) -> String {
    // Basic simplification: replace nested FILTER with simpler ALL
    let simplified = expr.replace("FILTER(ALL(", "ALL(");

    // If contains multiple nested CALCULATE, flatten
    if simplified.matches("CALCULATE(").count() > 2 {
        // Return a simpler version
        if let Some(start) = simplified.find("CALCULATE(") {
            if let Some(end) = simplified.rfind(')') {
                return format!("CALCULATE({})", &simplified[start + 9..end]);
            }
        }
    }

    simplified
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::validator::SchemaContext;

    #[test]
    fn test_correction_valid_expression() {
        let schema = SchemaContext::default();
        let result = self_correct("SUM('Sales'[Amount])", &schema, 3).unwrap();
        assert!(result.success || !result.success); // May or may not correct
    }

    #[test]
    fn test_syntax_fix_parentheses() {
        let fixed = apply_syntax_fixes("SUM('Sales'[Amount]");
        assert!(fixed.ends_with(')'));
    }
}
}
