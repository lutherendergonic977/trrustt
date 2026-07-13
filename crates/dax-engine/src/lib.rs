// TRRUSTT — DAX Engine
// PEG parser, 7-step validation, AI-powered generation, self-correction.
#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod ast;
pub mod parser;
pub mod validator;
pub mod generator;
pub mod corrector;
pub mod explainer;
pub mod formatter;
pub mod complexity;

use shared::Result;

/// The DAX engine — parse, validate, generate, correct, explain, format.
pub struct DaxEngine {
    complexity: complexity::ComplexityLevel,
}

impl DaxEngine {
    /// Create a new DAX engine with the default complexity level.
    pub fn new() -> Self {
        Self { complexity: complexity::ComplexityLevel::Intermediate }
    }

    /// Create a new DAX engine with a specific complexity level.
    pub fn with_complexity(level: complexity::ComplexityLevel) -> Self {
        Self { complexity: level }
    }

    /// Parse a DAX expression into an AST.
    /// Uses a PEG (Parsing Expression Grammar) parser.
    pub fn parse(&self, expression: &str) -> Result<ast::DaxExpression> {
        parser::parse_dax(expression)
    }

    /// Validate a DAX expression against a schema context.
    /// Runs the full 7-step validation pipeline:
    /// 1. Syntax  2. Semantic  3. Reference  4. Performance
    /// 5. Security  6. Style  7. Dependencies
    pub fn validate(&self, expression: &str, schema: &validator::SchemaContext) -> Result<validator::ValidationResult> {
        let parsed = self.parse(expression)?;
        validator::validate(&parsed, schema, self.complexity)
    }

    /// Generate DAX measures from natural language description.
    /// This delegates to the AI-powered generator.
    pub fn generate(
        &self,
        description: &str,
        schema: &validator::SchemaContext,
        level: complexity::ComplexityLevel,
    ) -> Result<generator::GeneratedMeasure> {
        generator::generate_measure(description, schema, level)
    }

    /// Self-correct an invalid DAX expression.
    /// Runs the validate→fix→re-validate loop up to max_attempts times.
    pub fn correct(
        &self,
        expression: &str,
        schema: &validator::SchemaContext,
        max_attempts: usize,
    ) -> Result<corrector::CorrectionResult> {
        corrector::self_correct(expression, schema, max_attempts)
    }

    /// Explain a DAX expression in natural language.
    pub fn explain(
        &self,
        expression: &str,
        schema: &validator::SchemaContext,
    ) -> Result<explainer::DaxExplanation> {
        explainer::explain_dax(expression, schema)
    }

    /// Format a DAX expression (pretty-print).
    pub fn format(&self, expression: &str) -> Result<String> {
        let parsed = self.parse(expression)?;
        Ok(formatter::format_ast(&parsed, 0))
    }

    /// Get the current complexity level.
    pub fn complexity_level(&self) -> complexity::ComplexityLevel {
        self.complexity
    }
}

impl Default for DaxEngine {
    fn default() -> Self { Self::new() }
}
