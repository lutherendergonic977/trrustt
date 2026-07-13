# Component Architecture — Rust-Native v2

## 1. Crate Dependency Graph

```
                         ┌─────────────────────┐
                         │  intellidashboard    │  ← Binary crate (entry point)
                         │  (main binary)       │
                         └──────────┬──────────┘
                                    │ uses all crates below
            ┌───────────────────────┼───────────────────────┐
            │                       │                       │
    ┌───────▼───────┐     ┌────────▼────────┐     ┌────────▼────────┐
    │ xmla-client   │     │ dashboard-gen   │     │   mcp-hub       │
    │               │     │                 │     │                 │
    │ • DISCOVER    │     │ • Layout AI     │     │ • MCP Server    │
    │ • EXECUTE     │     │ • Visual TMSL   │     │ • MCP Client    │
    │ • TMSL        │     │ • Theme Engine  │     │ • Tool Registry │
    │ • Subprocess  │     │ • Grid Engine   │     │ • Protocol      │
    └───────┬───────┘     └────────┬────────┘     └────────┬────────┘
            │                      │                        │
    ┌───────▼───────┐     ┌────────▼────────┐              │
    │  dax-engine   │     │   ai-engine     │              │
    │               │     │                 │              │
    │ • PEG Parser  │     │ • LLM Router    │──────────────┘
    │ • Validator   │     │ • RAG Pipeline  │
    │ • Generator   │     │ • Vision Engine │
    │ • Corrector   │     │ • Prompt Mgr    │
    │ • Explainer   │     │ • Chains        │
    └───────┬───────┘     └────────┬────────┘
            │                      │
    ┌───────▼──────────────────────▼────────┐
    │            config-engine              │
    │                                       │
    │ • Layered Resolution (figment)        │
    │ • Registry + Validation               │
    │ • Encryption (ring)                   │
    │ • Admin Policy Enforcement            │
    └───────────────────┬───────────────────┘
                        │
    ┌───────────────────▼───────────────────┐
    │              shared                   │
    │                                       │
    │ • Domain Types (serde)                │
    │ • Error Types (thiserror)             │
    │ • Telemetry (tracing + opentelemetry) │
    │ • Constants + Utilities               │
    └───────────────────────────────────────┘
```

## 2. Crate Specifications

### 2.1 `xmla-client` — Power BI SSAS Bridge

```
crates/xmla-client/
├── Cargo.toml          (deps: reqwest, serde, serde_json, tokio, thiserror)
└── src/
    ├── lib.rs           # Public API re-exports
    ├── client.rs        # XmlaClient struct, connection pool, auth
    ├── discover.rs      # DISCOVER_XMLA operations (schema metadata)
    ├── execute.rs       # EXECUTE (DAX EVALUATE queries)
    ├── tmsl.rs          # TMSL command builder (createOrReplace, delete, refresh)
    ├── schema.rs        # SchemaMetadata, Table, Column, Measure, Relationship types
    ├── subprocess.rs    # Tabular Editor CLI bridge (offline .pbix mode)
    └── error.rs         # XmlaError enum
```

**Public API:**
```rust
// crates/xmla-client/src/lib.rs

pub struct XmlaClient {
    http: reqwest::Client,
    base_url: String,        // http://localhost:XXXXX/xmla
    database_name: String,
}

impl XmlaClient {
    /// Connect to a running PBI Desktop SSAS instance
    pub async fn connect(port: u16) -> Result<Self>;
    
    /// Discover schema metadata (tables, columns, measures, relationships)
    pub async fn discover_schema(&self) -> Result<SchemaMetadata>;
    
    /// Execute a DAX query and return results
    pub async fn execute_dax(&self, dax: &str) -> Result<DaxQueryResult>;
    
    /// Apply a TMSL command (create/alter/delete model objects)
    pub async fn execute_tmsl(&self, tmsl: &TmslCommand) -> Result<TmslResult>;
    
    /// Create or replace a measure in the model
    pub async fn create_measure(&self, table: &str, measure: &Measure) -> Result<()>;
    
    /// Delete a measure from the model
    pub async fn delete_measure(&self, table: &str, measure_name: &str) -> Result<()>;
    
    /// Refresh a table
    pub async fn refresh_table(&self, table: &str) -> Result<()>;
    
    /// Get the raw SSAS connection string
    pub fn connection_string(&self) -> String;
}

/// Offline mode: read/write .pbix via Tabular Editor CLI
pub struct OfflinePbix {
    path: PathBuf,
}

impl OfflinePbix {
    pub fn open(path: impl Into<PathBuf>) -> Self;
    pub async fn discover_schema(&self) -> Result<SchemaMetadata>;
    pub async fn apply_tmsl(&self, tmsl: &str) -> Result<()>;
    pub async fn export_model_json(&self) -> Result<String>;
}
```

### 2.2 `dax-engine` — DAX Parsing, Validation, Generation

```
crates/dax-engine/
├── Cargo.toml          (deps: pom, serde, tokio, thiserror, regex)
└── src/
    ├── lib.rs           # Public API
    ├── parser/
    │   ├── mod.rs       # PEG grammar definitions
    │   ├── expression.rs  # Expression parser
    │   ├── function.rs    # Function call parser
    │   ├── reference.rs   # Table/column/measure references
    │   └── constant.rs    # Literal value parser
    ├── ast.rs           # DaxAst, DaxExpression, all node types
    ├── validator/
    │   ├── mod.rs       # DaxValidator, ValidationPipeline
    │   ├── syntax.rs    # Syntax checks
    │   ├── semantic.rs  # Reference & type checks
    │   ├── performance.rs # Cost estimation
    │   ├── style.rs     # Naming & formatting
    │   └── security.rs  # Disallowed functions check
    ├── generator/
    │   ├── mod.rs       # AI-powered DAX generation orchestrator
    │   └── prompts.rs   # Prompt template management
    ├── corrector.rs     # Self-correction loop (validate → fix → re-validate)
    ├── explainer.rs     # Natural language explanation of DAX
    ├── complexity.rs    # ComplexityLevel enum + allowed function maps
    ├── formatter.rs     # DAX pretty-printer
    └── rules.rs         # DaxValidationRules config
```

**Public API:**
```rust
// crates/dax-engine/src/lib.rs

pub struct DaxEngine {
    validator: DaxValidator,
    generator: DaxGenerator,
    corrector: DaxSelfCorrector,
    explainer: DaxExplainer,
}

impl DaxEngine {
    /// Parse a DAX expression into an AST
    pub fn parse(&self, expression: &str) -> Result<DaxAst, ParseError>;
    
    /// Validate a DAX expression against a schema
    pub async fn validate(
        &self, 
        expression: &str, 
        schema: &SchemaContext
    ) -> Result<DaxValidationResult>;
    
    /// Generate DAX measure(s) from natural language
    pub async fn generate(
        &self,
        description: &str,
        schema: &SchemaContext,
        complexity: ComplexityLevel,
        existing_measures: &[ExistingMeasure],
    ) -> Result<Vec<GeneratedMeasure>>;
    
    /// Self-correct an invalid DAX expression
    pub async fn correct(
        &self,
        expression: &str,
        schema: &SchemaContext,
        max_attempts: usize,
    ) -> Result<CorrectionResult>;
    
    /// Explain a DAX expression in natural language
    pub async fn explain(
        &self,
        expression: &str,
        schema: &SchemaContext,
    ) -> Result<DaxExplanation>;
    
    /// Format a DAX expression (pretty-print)
    pub fn format(&self, expression: &str) -> Result<String>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ComplexityLevel {
    Beginner,
    Intermediate,
    Advanced,
    Expert,
}
```

### 2.3 `ai-engine` — LLM Orchestration, RAG, Vision

```
crates/ai-engine/
├── Cargo.toml          (deps: async-openai, reqwest, serde, serde_json,
│                         tokio, thiserror, base64, image, fastembed,
│                         lancedb, tera, uuid)
└── src/
    ├── lib.rs
    ├── router.rs        # LlmRouter: multi-provider dispatch
    ├── providers/
    │   ├── mod.rs       # AiProvider trait
    │   ├── openai.rs    # OpenAI / Azure OpenAI
    │   ├── anthropic.rs # Anthropic Claude
    │   ├── ollama.rs    # Local Ollama
    │   └── candle.rs    # Embedded candle (local inference)
    ├── prompt.rs        # Prompt loading, templating (tera), versioning
    ├── chains/
    │   ├── mod.rs       # PromptChain, ChainStep, ChainContext
    │   ├── dashboard.rs # Dashboard generation chain
    │   ├── dax.rs       # DAX generation chain
    │   └── insights.rs  # Data insights chain
    ├── rag/
    │   ├── mod.rs       # RagPipeline
    │   ├── embedder.rs  # Embedding generation (OpenAI or fastembed-rs)
    │   ├── store.rs     # LanceDB vector store
    │   ├── search.rs    # Hybrid search (vector + BM25)
    │   └── context.rs   # Context assembly for LLM prompts
    ├── vision/
    │   ├── mod.rs       # VisionEngine
    │   ├── analyzer.rs  # GPT-4V / Claude Vision image analysis
    │   ├── layout.rs    # Layout extraction from images
    │   └── colors.rs    # Color palette extraction
    ├── cost.rs          # Token cost tracking
    └── rate_limit.rs    # Rate limiting per provider
```

**Public API:**
```rust
// crates/ai-engine/src/lib.rs

pub struct AiEngine {
    router: LlmRouter,
    rag: RagPipeline,
    vision: VisionEngine,
    prompt_manager: PromptManager,
}

impl AiEngine {
    // ── LLM Routing ──────────────────────────
    pub async fn chat(&self, request: ChatRequest) -> Result<ChatResponse>;
    pub async fn chat_with_schema(
        &self, 
        message: &str, 
        schema: &SchemaContext
    ) -> Result<ChatResponse>;
    
    // ── RAG ──────────────────────────────────
    pub async fn index_schema(&self, schema: &SchemaMetadata) -> Result<()>;
    pub async fn search_schema(&self, query: &str, k: usize) -> Result<Vec<SearchResult>>;
    pub fn schema_context(&self, query: &str) -> Result<String>;
    
    // ── Vision ───────────────────────────────
    pub async fn analyze_dashboard_image(&self, image: &[u8]) -> Result<ImageAnalysis>;
    pub async fn extract_layout(&self, image: &[u8]) -> Result<LayoutExtraction>;
    pub async fn extract_colors(&self, image: &[u8]) -> Result<ColorPalette>;
    
    // ── Prompt Management ────────────────────
    pub fn load_prompt(&self, name: &str) -> Result<String>;
    pub fn render_prompt(&self, template: &str, ctx: &tera::Context) -> Result<String>;
    pub async fn upgrade_prompts(&self) -> Result<Vec<String>>;
    pub fn list_prompts(&self) -> Vec<PromptInfo>;
    
    // ── Cost & Usage ─────────────────────────
    pub fn cost_summary(&self) -> CostSummary;
    pub fn token_usage(&self) -> TokenUsage;
}
```

### 2.4 `config-engine` — Configuration Management

```
crates/config-engine/
├── Cargo.toml          (deps: figment, serde, serde_json, toml,
│                         thiserror, ring, aes-gcm, dirs)
└── src/
    ├── lib.rs
    ├── engine.rs        # ConfigEngine: layered resolution
    ├── registry.rs      # Config entry registry with validation
    ├── layers.rs        # ConfigScope enum, file paths per scope
    ├── validation.rs    # Schema validation for config values
    ├── encryption.rs    # AES-256-GCM encrypt/decrypt sensitive values
    ├── admin.rs         # Admin policy loading & enforcement
    └── hot_reload.rs    # File watcher for live config changes
```

**Public API:**
```rust
// crates/config-engine/src/lib.rs

pub struct ConfigEngine {
    figment: Figment,
    registry: ConfigRegistry,
    admin_policies: AdminPolicies,
}

impl ConfigEngine {
    /// Initialize with all config layers
    pub fn new(cli_overrides: HashMap<String, String>) -> Result<Self>;
    
    /// Get resolved value
    pub fn get<T: DeserializeOwned>(&self, path: &str) -> Result<T>;
    
    /// Set value at specific scope
    pub async fn set<T: Serialize>(&self, path: &str, value: T, scope: ConfigScope) -> Result<()>;
    
    /// Reset to default
    pub async fn reset(&self, path: &str, scope: ConfigScope) -> Result<()>;
    
    /// Export all resolved config
    pub fn export(&self) -> Result<serde_json::Value>;
    
    /// Import config from JSON
    pub async fn import(&self, data: serde_json::Value, scope: ConfigScope) -> Result<()>;
    
    /// Load admin policies
    pub async fn load_policies(&mut self, url: &str) -> Result<()>;
    
    /// Start hot-reload watcher
    pub fn watch(&self) -> Result<()>;
}
```

### 2.5 `dashboard-generator` — Dashboard Layout & Visuals

```
crates/dashboard-generator/
├── Cargo.toml          (deps: serde, serde_json, tokio, thiserror)
└── src/
    ├── lib.rs
    ├── planner.rs       # AI-driven layout planning
    ├── layout.rs        # Grid layout engine
    ├── visuals.rs       # Visual type selection & configuration
    ├── binding.rs       # Measure-to-visual field binding
    ├── theme.rs         # JSON theme engine
    ├── tmsl_builder.rs  # Build TMSL commands for visual creation
    └── types.rs         # Dashboard, Page, Visual, GridPosition types
```

### 2.6 `mcp-hub` — MCP Server & Client

```
crates/mcp-hub/
├── Cargo.toml          (deps: serde, serde_json, tokio, thiserror)
└── src/
    ├── lib.rs
    ├── protocol.rs      # JSON-RPC 2.0 message types
    ├── server.rs        # MCP server (stdio transport)
    ├── client.rs        # MCP client (stdio + SSE + HTTP transports)
    ├── tools.rs         # Tool definitions & dynamic registry
    ├── resources.rs     # Resource definitions
    └── transport.rs     # Transport abstraction (Stdio, Sse, Http)
```

### 2.7 `data-ingestion` — Data Import & Profiling

```
crates/data-ingestion/
├── Cargo.toml          (deps: polars, sqlx, serde, serde_json, tokio, thiserror)
└── src/
    ├── lib.rs
    ├── sources/
    │   ├── csv.rs        # CSV/TSV ingestion via Polars
    │   ├── sql.rs        # SQL database connector (any SQL via sqlx)
    │   ├── parquet.rs    # Parquet file reader
    │   ├── json.rs       # JSON/NDJSON ingestion
    │   └── api.rs        # REST API data source
    ├── profiler.rs       # Polars-based data profiling
    ├── inferrer.rs       # Type + relationship inference
    └── model_builder.rs  # Generate TMSL model from profiled data
```

**Public API:**
```rust
pub struct DataIngestionEngine {
    profiler: DataProfiler,
}

impl DataIngestionEngine {
    pub async fn ingest_csv(&self, path: &Path) -> Result<DataFrame>;
    pub async fn ingest_sql(&self, conn_str: &str, query: &str) -> Result<DataFrame>;
    pub async fn ingest_parquet(&self, path: &Path) -> Result<DataFrame>;
    pub async fn profile(&self, df: &DataFrame) -> Result<DataProfile>;
    pub async fn infer_model(&self, profile: &DataProfile) -> Result<ModelDefinition>;
    pub async fn build_tmsl(&self, model: &ModelDefinition) -> Result<TmslScript>;
    pub async fn load_into_ssas(&self, tmsl: &TmslScript, client: &XmlaClient) -> Result<()>;
}
```

### 2.8 `data-store` — Internal Database

```
crates/license-manager/
├── Cargo.toml          (deps: jsonwebtoken, serde, reqwest, tokio, thiserror)
└── src/
    ├── lib.rs
    ├── validator.rs     # JWT validation & feature extraction
    ├── features.rs      # Feature flag resolution per tier
    ├── offline.rs       # Offline grace period management
    ├── telemetry.rs     # License usage reporting
    └── cache.rs         # Encrypted local license cache
```

### 2.8 `shared` — Shared Types & Utilities

```
crates/shared/
├── Cargo.toml          (deps: serde, thiserror, tracing, opentelemetry, uuid)
└── src/
    ├── lib.rs
    ├── types.rs         # SchemaMetadata, Measure, Dashboard, etc.
    ├── errors.rs        # AppError enum (thiserror)
    ├── telemetry.rs     # OpenTelemetry initialization + helpers
    ├── constants.rs     # App-wide constants
    └── utils.rs         # Common utilities
```

## 3. Compile-Time Feature Flags

```toml
# crates/intellidashboard/Cargo.toml
[features]
default = ["openai", "tui"]

# AI Providers
openai = ["ai-engine/openai"]
azure-openai = ["ai-engine/azure-openai"]
anthropic = ["ai-engine/anthropic"]
google = ["ai-engine/google"]
deepseek = ["ai-engine/deepseek"]
ollama = ["ai-engine/ollama"]
candle = ["ai-engine/candle"]        # Local LLM (CPU)

# UI Modes
tui = ["ratatui", "crossterm"]       # Terminal UI
webview = ["tauri"]                  # Rich settings UI (V2)

# Platform-specific
win-msvc = []
macos = []
linux-musl = []

# Enterprise
enterprise = ["license-manager/enterprise", "mcp-hub/enterprise"]
```

## 4. Cross-Crate Communication Patterns

```rust
// All cross-crate communication uses Rust traits.
// No HTTP, no serialization, no IPC between crates within the same binary.

// Example: dashboard-generator depends on dax-engine and xmla-client:

use dax_engine::DaxEngine;
use xmla_client::XmlaClient;
use ai_engine::AiEngine;

pub struct DashboardGenerator {
    dax: Arc<DaxEngine>,      // Shared ownership via Arc
    xmla: Arc<XmlaClient>,
    ai: Arc<AiEngine>,
}

impl DashboardGenerator {
    pub async fn generate(&self, intent: &str) -> Result<Dashboard> {
        // Direct function calls — zero overhead
        let schema = self.xmla.discover_schema().await?;
        let layout = self.ai.plan_dashboard(intent, &schema).await?;
        let measures = self.dax.generate(&layout.measure_descriptions, &schema).await?;
        self.xmla.apply_measures(&measures).await?;
        self.apply_layout(&layout, &measures).await?;
        Ok(Dashboard { layout, measures })
    }
}
```

## 5. Entry Point Dispatch

```rust
// crates/intellidashboard/src/main.rs

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    init_observability(&cli)?;
    
    let app = App::new(cli.config_overrides).await?;
    
    match cli.mode {
        Mode::ExternalTool { port, pbix } => {
            app.run_external_tool(port, &pbix).await
        }
        Mode::Cli { command } => {
            app.run_cli(command).await
        }
        Mode::Tui => {
            app.run_tui().await
        }
        Mode::McpServe => {
            app.run_mcp_server().await
        }
        Mode::Webview => {
            app.run_webview().await
        }
    }
}
```

---

> **Document Version:** 2.0  
> **Part of:** IntelliDashboard Builder Architecture Docs (Rust-Native)
