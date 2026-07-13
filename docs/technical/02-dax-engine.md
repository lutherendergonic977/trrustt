# DAX Engine — Rust Implementation v2

## 1. Overview

The DAX Engine is a pure-Rust crate responsible for:
1. **Parsing** DAX into AST (PEG parser via `pom`)
2. **Validating** generated DAX (7-step pipeline)
3. **Generating** DAX from natural language (via AI engine)
4. **Self-correcting** incorrect DAX (AI + validation loop)
5. **Explaining** DAX in natural language (via AI engine)
6. **Formatting** DAX (pretty-printer)

## 2. PEG Parser

### 2.1 Grammar (pom — Parsing Expression Grammar)

```rust
// crates/dax-engine/src/parser/mod.rs

use pom::parser::*;
use pom::Parser;

/// Top-level DAX expression parser
pub fn dax_expression<'a>() -> Parser<'a, u8, DaxExpression> {
    choice(vec![
        var_return().map(DaxExpression::VarReturn),
        function_call().map(DaxExpression::FunctionCall),
        column_reference().map(DaxExpression::ColumnRef),
        measure_reference().map(DaxExpression::MeasureRef),
        table_reference().map(DaxExpression::TableRef),
        constant().map(DaxExpression::Constant),
        parenthesized(),
        binary_operation(),
    ])
}

/// VAR declarations + RETURN
fn var_return<'a>() -> Parser<'a, u8, VarReturn> {
    let var_decl = (sym(b'V') * sym(b'A') * sym(b'R') - space())
        - identifier() - space()
        - sym(b'=') - space()
        - call(dax_expression);
    
    let return_stmt = sym(b'R') * sym(b'E') * sym(b'T') * sym(b'U') * sym(b'R') * sym(b'N')
        - space() - call(dax_expression);
    
    (list(var_decl, space()) - return_stmt)
        .map(|(vars, ret)| VarReturn { variables: vars, return_expression: Box::new(ret) })
}

/// Function call: FUNC(arg1, arg2, ...)
fn function_call<'a>() -> Parser<'a, u8, FunctionCall> {
    (identifier() - sym(b'(') - space()
        - list(call(dax_expression), sym(b',') - space().opt())
        - space() - sym(b')'))
        .map(|(name, args)| FunctionCall { name, arguments: args })
}

/// Column reference: 'Table'[Column] or [Column]
fn column_reference<'a>() -> Parser<'a, u8, ColumnRef> {
    let qualified = (table_name() - sym(b'[') - identifier() - sym(b']'))
        .map(|(t, c)| ColumnRef { table: Some(t), column: c });
    let unqualified = (sym(b'[') - identifier() - sym(b']'))
        .map(|c| ColumnRef { table: None, column: c });
    qualified | unqualified
}

/// Binary operations with correct precedence
fn binary_operation<'a>() -> Parser<'a, u8, DaxExpression> {
    // Handles: + - * / ^  && ||  = <> < > <= >=  IN
    // With proper operator precedence via Pratt parsing
    // ...
}

/// Parse a DAX string into AST
pub fn parse_dax(input: &str) -> Result<DaxExpression, ParseError> {
    let bytes = input.as_bytes();
    dax_expression()
        .parse(bytes)
        .map_err(|e| ParseError {
            message: format!("Parse error at position {}: {:?}", e.position, e),
            position: e.position,
        })
}
```

### 2.2 AST Types

```rust
// crates/dax-engine/src/ast.rs

use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DaxExpression {
    FunctionCall(FunctionCall),
    ColumnRef(ColumnRef),
    MeasureRef(MeasureRef),
    TableRef(String),
    Constant(ConstantValue),
    VarReturn(VarReturn),
    BinaryOp {
        left: Box<DaxExpression>,
        op: DaxOperator,
        right: Box<DaxExpression>,
    },
    UnaryOp {
        op: UnaryOperator,
        expr: Box<DaxExpression>,
    },
    Parenthesized(Box<DaxExpression>),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FunctionCall {
    pub name: String,
    pub arguments: Vec<DaxExpression>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ColumnRef {
    pub table: Option<String>,
    pub column: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MeasureRef {
    pub table: Option<String>,
    pub measure: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VarReturn {
    pub variables: Vec<VarDeclaration>,
    pub return_expression: Box<DaxExpression>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VarDeclaration {
    pub name: String,
    pub expression: Box<DaxExpression>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ConstantValue {
    Integer(i64),
    Decimal(f64),
    String(String),
    Boolean(bool),
    Blank,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DaxOperator {
    Add, Sub, Mul, Div, Pow,
    Eq, Neq, Lt, Gt, Lte, Gte,
    And, Or,
    In, NotIn,
    Concat,
}
```

## 3. Validation Pipeline

### 3.1 Pipeline Architecture

```rust
// crates/dax-engine/src/validator/mod.rs

use async_trait::async_trait;
use crate::ast::DaxExpression;
use crate::rules::DaxValidationRules;

/// Each validation step is independently configurable and testable
#[async_trait]
pub trait ValidationStep: Send + Sync {
    /// Unique name for this step
    fn name(&self) -> &str;
    
    /// Error severity level
    fn severity(&self) -> ValidationSeverity;
    
    /// Is this step enabled? (configurable)
    fn enabled(&self, rules: &DaxValidationRules) -> bool;
    
    /// Run the validation
    async fn validate(
        &self,
        ast: &DaxExpression,
        schema: &SchemaContext,
        rules: &DaxValidationRules,
    ) -> Result<ValidationStepResult>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationSeverity {
    /// Informational — does not block
    Info,
    /// Warning — best practice violation
    Warning,
    /// Error — blocks the measure from being applied
    Error,
    /// Critical — security concern, immediately abort
    Critical,
}

pub struct DaxValidator {
    steps: Vec<Box<dyn ValidationStep>>,
}

impl DaxValidator {
    pub fn new(rules: DaxValidationRules) -> Self {
        // Steps are added in order. Each can be disabled via config.
        Self {
            steps: vec![
                Box::new(SyntaxValidator),        // Step 1
                Box::new(ReferenceValidator),      // Step 2
                Box::new(TypeValidator),           // Step 3
                Box::new(FunctionValidator),       // Step 4
                Box::new(PerformanceValidator),    // Step 5
                Box::new(StyleValidator),          // Step 6
                Box::new(SecurityValidator),       // Step 7
                Box::new(CompletenessValidator),   // Step 8
            ],
        }
    }
    
    pub async fn validate(
        &self,
        expression: &str,
        schema: &SchemaContext,
        rules: &DaxValidationRules,
    ) -> Result<DaxValidationResult> {
        // Step 0: Parse
        let ast = parse_dax(expression)?;
        
        let mut result = DaxValidationResult {
            expression: expression.to_string(),
            ast: Some(ast.clone()),
            is_valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
            infos: Vec::new(),
        };
        
        for step in &self.steps {
            if !step.enabled(rules) {
                continue;
            }
            
            let step_result = step.validate(&ast, schema, rules).await?;
            
            match step.severity() {
                ValidationSeverity::Info => result.infos.extend(step_result.messages),
                ValidationSeverity::Warning => result.warnings.extend(step_result.messages),
                ValidationSeverity::Error => {
                    result.errors.extend(step_result.messages);
                    result.is_valid = false;
                }
                ValidationSeverity::Critical => {
                    result.errors.extend(step_result.messages);
                    result.is_valid = false;
                    break; // Stop on critical
                }
            }
        }
        
        Ok(result)
    }
}
```

### 3.2 Individual Validators

```rust
// Step 2: Reference Validator
pub struct ReferenceValidator;

#[async_trait]
impl ValidationStep for ReferenceValidator {
    fn name(&self) -> &str { "reference-check" }
    fn severity(&self) -> ValidationSeverity { ValidationSeverity::Error }
    fn enabled(&self, rules: &DaxValidationRules) -> bool { 
        rules.reference_check_enabled 
    }
    
    async fn validate(
        &self,
        ast: &DaxExpression,
        schema: &SchemaContext,
        rules: &DaxValidationRules,
    ) -> Result<ValidationStepResult> {
        let mut messages = Vec::new();
        let references = ast.extract_references(); // Walk AST collect all refs
        
        for reference in &references {
            match reference {
                Ref::Column { table, column } => {
                    if let Some(table_name) = table {
                        if !schema.has_table(table_name) {
                            messages.push(ValidationMessage::error(
                                format!("Table '{}' not found in schema", table_name)
                            ));
                        } else if !schema.has_column(table_name, column) {
                            messages.push(ValidationMessage::error(
                                format!("Column '{}'[{}] not found", table_name, column)
                            ));
                        }
                    } else if rules.require_table_prefix {
                        messages.push(ValidationMessage::warning(
                            format!("Column reference [{}] should include table prefix", column)
                        ));
                    }
                }
                Ref::Measure { table, measure } => {
                    // Check measure exists
                }
            }
        }
        
        Ok(ValidationStepResult { messages })
    }
}

// Step 5: Performance Validator
pub struct PerformanceValidator;

#[async_trait]
impl ValidationStep for PerformanceValidator {
    fn name(&self) -> &str { "performance-check" }
    fn severity(&self) -> ValidationSeverity { ValidationSeverity::Warning }
    fn enabled(&self, rules: &DaxValidationRules) -> bool { 
        rules.performance_check_enabled 
    }
    
    async fn validate(
        &self,
        ast: &DaxExpression,
        schema: &SchemaContext,
        rules: &DaxValidationRules,
    ) -> Result<ValidationStepResult> {
        let mut messages = Vec::new();
        
        // Check for expensive patterns
        if ast.contains_function("FILTER") && ast.contains_function("VALUES") {
            // Detect FILTER(VALUES(...)) which can be optimized to just VALUES
            messages.push(ValidationMessage::info(
                "Consider simplifying FILTER(VALUES(...)) to just VALUES(...)"
            ));
        }
        
        // Check for iterative functions over large tables
        if let Some(iterator) = ast.find_iterator_over_large_table(schema) {
            messages.push(ValidationMessage::warning(
                format!("Iterator '{}' over table '{}' ({} rows) may be slow. Consider using SUMMARIZECOLUMNS instead.",
                    iterator.function, iterator.table, iterator.row_count)
            ));
        }
        
        // Detect potential cartesian product
        if ast.contains_crossjoin_pattern() {
            messages.push(ValidationMessage::warning(
                "Potential cartesian product detected. This may cause performance issues."
            ));
        }
        
        Ok(ValidationStepResult { messages })
    }
}
```

## 4. DAX Generator (AI-Powered)

```rust
// crates/dax-engine/src/generator/mod.rs

use ai_engine::AiEngine;
use ai_engine::chains::dax::DaxGenerationChain;

pub struct DaxGenerator {
    ai: Arc<AiEngine>,
    chain: DaxGenerationChain,
}

impl DaxGenerator {
    pub async fn generate(
        &self,
        description: &str,
        schema: &SchemaContext,
        complexity: ComplexityLevel,
        existing_measures: &[ExistingMeasure],
    ) -> Result<Vec<GeneratedMeasure>, DaxError> {
        // 1. Get RAG context
        let rag_context = self.ai.schema_context(description)?;
        
        // 2. Build chain context
        let mut ctx = ChainContext::new();
        ctx.insert("description", description);
        ctx.insert("schema", &schema.to_prompt_string());
        ctx.insert("rag_context", &rag_context);
        ctx.insert("complexity", &complexity.to_string());
        ctx.insert("existing_measures", &serde_json::to_string(existing_measures)?);
        ctx.insert("naming_convention", &self.config.dax_naming_convention());
        ctx.insert("comment_style", &self.config.dax_comment_style());
        
        // 3. Execute generation chain
        let chain_result = self.chain.execute(&self.ai, &mut ctx).await?;
        
        // 4. Parse chain output
        let measures: Vec<GeneratedMeasure> = serde_json::from_str(
            &chain_result.final_output
        )?;
        
        Ok(measures)
    }
}
```

## 5. Self-Correction Loop

```rust
// crates/dax-engine/src/corrector.rs

pub struct DaxSelfCorrector {
    validator: DaxValidator,
    ai: Arc<AiEngine>,
}

impl DaxSelfCorrector {
    pub async fn correct(
        &self,
        expression: &str,
        schema: &SchemaContext,
        max_attempts: usize,
    ) -> Result<CorrectionResult, DaxError> {
        let mut current = expression.to_string();
        let mut corrections = Vec::new();
        let mut attempts = 0;
        
        loop {
            attempts += 1;
            
            // Validate current expression
            let validation = self.validator.validate(
                &current, schema, &self.config.validation_rules()
            ).await?;
            
            if validation.is_valid {
                return Ok(CorrectionResult {
                    success: true,
                    final_expression: current,
                    original_expression: expression.to_string(),
                    attempts,
                    corrections,
                });
            }
            
            if attempts >= max_attempts {
                return Ok(CorrectionResult {
                    success: false,
                    final_expression: current,
                    original_expression: expression.to_string(),
                    attempts,
                    corrections,
                });
            }
            
            // Build correction prompt
            let errors_text = validation.errors.iter()
                .map(|e| format!("- {}: {}", e.severity, e.message))
                .collect::<Vec<_>>()
                .join("\n");
            
            let correction_prompt = format!(
                "The following DAX expression has validation errors:\n\n```dax\n{}\n```\n\nErrors:\n{}\n\nPlease fix the expression. Return ONLY the corrected DAX.",
                current, errors_text
            );
            
            // Get AI correction (low temperature for precision)
            let response = self.ai.chat(ChatRequest {
                system_prompt: DAX_CORRECTION_PROMPT.to_string(),
                user_message: correction_prompt,
                temperature: 0.1,
                max_tokens: 2048,
                ..Default::default()
            }).await?;
            
            let corrected = extract_dax_from_response(&response.content)?;
            
            corrections.push(Correction {
                attempt: attempts,
                original: current.clone(),
                corrected: corrected.clone(),
                errors_fixed: validation.errors.len(),
            });
            
            current = corrected;
        }
    }
}
```

## 6. Complexity Levels

```rust
// crates/dax-engine/src/complexity.rs

impl ComplexityLevel {
    /// Get the whitelist of allowed DAX functions for this level
    pub fn allowed_functions(&self) -> &'static [&'static str] {
        match self {
            Self::Beginner => &[
                "SUM", "AVERAGE", "COUNT", "COUNTA", "COUNTROWS",
                "MIN", "MAX", "DISTINCTCOUNT",
                "CALCULATE", "DIVIDE", "FORMAT",
                "IF", "SWITCH", "BLANK",
                "RELATED", "RELATEDTABLE",
            ],
            Self::Intermediate => &[
                // All Beginner functions, plus:
                "SUMX", "AVERAGEX", "COUNTX", "MINX", "MAXX",
                "FILTER", "ALL", "ALLEXCEPT", "ALLSELECTED",
                "VALUES", "DISTINCT", "CALCULATETABLE",
                "SAMEPERIODLASTYEAR", "DATEADD", "DATESYTD",
                "DATESQTD", "DATESMTD", "PARALLELPERIOD",
                "USERELATIONSHIP", "CROSSFILTER",
                "HASONEVALUE", "SELECTEDVALUE", "ISFILTERED",
                "RANKX", "TOPN",
                "VAR", "RETURN",
            ],
            Self::Advanced => &[
                // All Intermediate functions, plus:
                "ADDCOLUMNS", "SUMMARIZE", "SUMMARIZECOLUMNS",
                "GENERATE", "GENERATEALL",
                "TREATAS", "INTERSECT", "EXCEPT", "UNION",
                "ROLLUP", "ROLLUPGROUP", "ROLLUPISSUBTOTAL",
                "GROUPBY", "CURRENTGROUP",
                "NATURALINNERJOIN", "NATURALLEFTOUTERJOIN",
                "SUBSTITUTEWITHINDEX",
                "KEEPFILTERS", "REMOVEFILTERS",
                "ISINSCOPE", "ISSELECTEDMEASURE",
            ],
            Self::Expert => &[
                // All functions — no restrictions
                // But performance/style checks are stricter
            ],
        }
    }
    
    pub fn max_nesting_depth(&self) -> usize {
        match self {
            Self::Beginner => 3,
            Self::Intermediate => 5,
            Self::Advanced => 8,
            Self::Expert => 12,
        }
    }
    
    pub fn allow_variables(&self) -> bool {
        matches!(self, Self::Intermediate | Self::Advanced | Self::Expert)
    }
    
    pub fn allow_iterators(&self) -> bool {
        matches!(self, Self::Intermediate | Self::Advanced | Self::Expert)
    }
    
    pub fn allow_time_intelligence(&self) -> bool {
        matches!(self, Self::Intermediate | Self::Advanced | Self::Expert)
    }
}
```

## 7. DAX Explainer

```rust
// crates/dax-engine/src/explainer.rs

impl DaxExplainer {
    pub async fn explain(
        &self,
        expression: &str,
        schema: &SchemaContext,
    ) -> Result<DaxExplanation, DaxError> {
        // 1. Parse to AST
        let ast = parse_dax(expression)?;
        
        // 2. Build component breakdown
        let components = ast.decompose(); // Walk AST, label each part
        
        // 3. Get AI explanation
        let prompt = format!(
            "Explain this DAX measure in simple terms:\n\n```dax\n{}\n```\n\n\
             Context:\n{}\n\n\
             Provide: 1) One-line summary, 2) Step-by-step explanation, \
             3) What each component does, 4) Any assumptions or edge cases.",
            expression,
            schema.to_summary_string()
        );
        
        let response = self.ai.chat(ChatRequest {
            system_prompt: DAX_EXPLAINER_PROMPT.to_string(),
            user_message: prompt,
            temperature: 0.2,
            ..Default::default()
        }).await?;
        
        Ok(DaxExplanation {
            summary: extract_summary(&response.content),
            detailed: extract_detailed(&response.content),
            components,
            dependencies: ast.extract_dependencies(),
            assumptions: extract_assumptions(&response.content),
        })
    }
}
```

## 8. DAX Pretty-Printer

```rust
// crates/dax-engine/src/formatter.rs

impl DaxFormatter {
    pub fn format(&self, expression: &str) -> Result<String, DaxError> {
        let ast = parse_dax(expression)?;
        let config = self.config.format_options();
        
        let mut output = String::new();
        self.write_expression(&ast, &mut output, 0, &config)?;
        Ok(output)
    }
    
    fn write_expression(
        &self,
        expr: &DaxExpression,
        output: &mut String,
        indent: usize,
        config: &FormatOptions,
    ) -> Result<(), DaxError> {
        let indent_str = " ".repeat(indent * config.indent_size);
        
        match expr {
            DaxExpression::FunctionCall(fc) => {
                write!(output, "{}(", fc.name)?;
                if fc.arguments.len() > 1 || fc.name.len() > 15 {
                    output.push('\n');
                    for (i, arg) in fc.arguments.iter().enumerate() {
                        write!(output, "{}    ", indent_str)?;
                        self.write_expression(arg, output, indent + 1, config)?;
                        if i < fc.arguments.len() - 1 {
                            output.push_str(",\n");
                        }
                    }
                    output.push('\n');
                    write!(output, "{})", indent_str)?;
                } else {
                    // Single arg, inline
                    if let Some(arg) = fc.arguments.first() {
                        self.write_expression(arg, output, indent, config)?;
                    }
                    output.push(')');
                }
            }
            DaxExpression::VarReturn(vr) => {
                for var in &vr.variables {
                    writeln!(output, "{}VAR {} = ", indent_str, var.name)?;
                    self.write_expression(&var.expression, output, indent + 1, config)?;
                    output.push('\n');
                }
                writeln!(output, "{}RETURN", indent_str)?;
                self.write_expression(&vr.return_expression, output, indent + 1, config)?;
            }
            // ... handle all other variants
        }
        Ok(())
    }
}
```

---

> **Document Version:** 2.0  
> **Part of:** IntelliDashboard Builder Technical Docs (Rust-Native)
