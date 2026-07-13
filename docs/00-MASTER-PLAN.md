# 🔷 trRUSTt — Master Product Plan v2

> **Code Name:** *Athena*  
> **Tagline:** *"trRUSTt your data. One binary. Infinite dashboards."*  
> **Product Name:** trRUSTt (stylized) / TRRUSTT (code/config)  
> **Architecture:** Rust-Native · Power BI External Tool · Single Binary  
> **Version:** 2.0.0-ALPHA  
> **Date:** 2026-07-12  
> **Status:** PLANNING → ARCHITECTURE → BUILD

---

## Table of Contents

1. [Executive Summary](#1-executive-summary)
2. [Product Vision & Scope](#2-product-vision--scope)
3. [Core Capabilities](#3-core-capabilities)
4. [Architecture Decision: Why Rust, Why External Tool](#4-architecture-decision-why-rust-why-external-tool)
5. [System Architecture](#5-system-architecture)
6. [Configuration & Settings Engine](#6-configuration--settings-engine)
7. [AI Engine Design](#7-ai-engine-design)
8. [DAX Engine](#8-dax-engine)
9. [Dashboard Generation Pipeline](#9-dashboard-generation-pipeline)
10. [Image-to-Dashboard Pipeline](#10-image-to-dashboard-pipeline)
11. [MCP Server Integration](#11-mcp-server-integration)
12. [Authentication & Authorization](#12-authentication--authorization)
13. [Role-Based Access Control (RBAC)](#13-role-based-access-control-rbac)
14. [Whitelisting System](#14-whitelisting-system)
15. [Branding & White-Label System](#15-branding--white-label-system)
16. [Monetization Strategy](#16-monetization-strategy)
17. [Marketplace & Distribution](#17-marketplace--distribution)
18. [Testing Strategy](#18-testing-strategy)
19. [Packaging & Deployment](#19-packaging--deployment)
20. [CI/CD & Git Strategy](#20-cicd--git-strategy)
21. [Logging, Monitoring & Observability](#21-logging-monitoring--observability)
22. [Security Architecture](#22-security-architecture)
23. [Internationalization (i18n) & Localization (l10n)](#23-internationalization-i18n--localization-l10n)
24. [Roadmap & Milestones](#24-roadmap--milestones)
25. [Open Questions & Decisions](#25-open-questions--decisions)

---

## 1. Executive Summary

**trRUSTt (Athena)** is an AI-driven Power BI External Tool that automates the end-to-end creation of dashboards. It runs as a **single Rust binary** — no runtime dependencies, no installers, no .NET, no Python, no Node.js. It connects to Power BI Desktop's embedded SSAS instance via XMLA, learns the data model, and generates production-ready dashboards with auto-generated DAX measures, themed visuals, and exploratory analytics.

The product is designed as a **sellable, brandable, enterprise-grade software product** with multiple monetization routes, full CI/CD, marketplace distribution, and white-label capabilities.

### Why This Approach Wins

| Attribute | Rust + External Tool | Prior Approaches |
|---|---|---|
| **Binary Size** | 15–25 MB | 250–600 MB |
| **Runtime Dependencies** | Zero (static binary) | .NET Runtime + Node.js/Bun + Python |
| **Startup Time** | Instant (native) | 3–8 seconds (JIT + JS parse + Python import) |
| **Memory Usage** | 20–50 MB idle | 300–600 MB idle |
| **Power BI Interop** | Native XMLA (HTTP) | TOM via Named Pipes + multi-process IPC |
| **Auth & Publishing** | PBI Desktop handles it | Must build SSO, OAuth, REST API |
| **UI** | PBI Desktop IS the UI | Full Electron app to build + maintain |
| **AI Performance** | Single Rust async runtime | Bridged across 3–4 runtimes |
| **Deployment** | Copy one file | Installer with bundled runtimes |
| **Codebase** | One language, one toolchain | 4 languages, 3+ build systems |
| **Safety** | Compile-time guarantees | Runtime error handling |

### Key Differentiators

| Differentiator | Description |
|---|---|
| 🔧 **100% Configurable** | Zero hard-coded values. Every behavior, prompt, theme, and setting lives in config files. |
| 🤖 **AI-Native** | Built from the ground up around LLMs. Prompt chains, RAG over schemas, self-correcting DAX. |
| 🖼️ **Image→Dashboard** | Upload a whiteboard sketch, infographic, or design mockup — AI generates a matching dashboard. |
| 🔌 **MCP-Native** | First-class MCP server/client. Expose as MCP tools. Connect any MCP-compatible server. |
| 🚀 **Single Binary** | One `.exe` file. Copy to External Tools folder. Done. No installer needed. |
| 💰 **Multi-Route Monetization** | Marketplace sales, subscription tiers, enterprise licensing, usage-based API, white-label OEM. |
| 🏷️ **White-Label Ready** | Every brand element is configurable. OEMs can rebrand and resell. |
| 🌍 **Global-Ready** | Full i18n/l10n, regional number/date formatting, RTL support, multi-currency. |
| 🎨 **Beautiful GUI** | Tauri + Svelte + shadcn/ui. Polished dashboard creation experience. Native OS webview, not Electron. |

---

## 2. Product Vision & Scope

### Vision Statement

> *To become the definitive AI-powered dashboard creation tool for Power BI — a single binary that any analyst can drop into their External Tools folder and immediately multiply their productivity 10x. Built by the community, for the community. trRUSTt your data.*

### Scope

#### IN Scope (V1) — LOCKED 🔒

**Zero external dependencies beyond a running Power BI Desktop instance. Period.**

**Mode A — Live PBI Model (External Tool):**
- Register as a Power BI External Tool (one-click from PBI Desktop ribbon)
- Connect to running PBI Desktop SSAS instance via XMLA (localhost:port)
- **Full metadata editing** — push measures, columns, tables, relationships, hierarchies, calculation groups back to the live model (identical to Tabular Editor's Ctrl+S behavior)
- AI-powered schema discovery and profiling (via Polars)
- Auto-generate DAX measures (beginner → expert complexity levels)
- Apply DAX measures directly to the live model (via TMSL/XMLA)
- AI-assisted dashboard layout generation and visual creation
- Theme engine with custom theming (JSON theme files)
- Configuration system (layered config files, admin-enforceable)
- Role-based access (Admin, Designer, Viewer) — config file driven
- CLI mode (for scripting & automation — connects via --port)
- MCP server exposure (stdio transport — expose as MCP tool for AI assistants)
- MCP client connectivity (connect to external MCP servers)
- **Microsoft MCP Servers (pre-configured defaults):**
  - **Power BI Modeling MCP Server (local)** — Microsoft's official MCP server for Power BI semantic models. 20+ tool categories. Installed alongside trRUSTt as the primary bridge to PBI Desktop, Fabric, and PBIP files.
  - **Power BI MCP Server (remote)** — Microsoft's MCP server for Power BI Service. Enterprise auth, workspace/dataset/report operations.
- Image-to-dashboard (upload image → AI extracts layout + colors → generates dashboard)
- DAX self-correction loop (generate → validate → fix → re-validate)
- Prompt library (externalized, versioned, upgradeable via git)
- Git integration for config/prompt/theme versioning
- Full observability (OpenTelemetry tracing, structured logging)
- **Pure Rust JWT license system** (Ed25519 signatures, AES-256-GCM encrypted cache, 30-day offline grace, periodic phone-home)
- **Open-source core** (MIT/Apache 2.0) with paid Pro/Enterprise features

**Mode B — Data Ingestion (Build model from raw data):**
- Ingest CSV, SQL databases, Parquet, JSON/NDJSON, REST APIs
- Auto-profile data via Polars (types, distributions, nulls, cardinality)
- Infer relationships (foreign key detection, column similarity)
- Generate Tabular model structure (TMSL: tables, columns, relationships, hierarchies)
- Load data into SSAS instance
- Then proceed with dashboard generation (same pipeline as Mode A)
- *This makes trRUSTt a complete data→dashboard solution*

**Interactive Visual Canvas (Tauri GUI):**
- WYSIWYG drag-and-drop visual editor
- Resize, reposition, reorder visuals on a grid canvas
- Real-time preview of generated dashboard before applying
- Visual palette: Bar, Line, Pie, Area, KPI Card, Map, Table, Matrix, Gauge, Slicer, Decomposition Tree
- Grid snap, alignment guides, responsive breakpoints
- Property panel: configure axes, legends, colors, filters per visual
- Map canvas state → TMSL commands for PBI Desktop

#### V1 Architecture: What We Do NOT Include (Deferred to V2+)

| Deferred Item | Reason | V2 Plan |
|---|---|---|
| **Tabular Editor CLI dependency** | Not needed — V1 only supports live PBI Desktop mode via XMLA | Add offline .pbix mode via TE2 subprocess in V2 |
| **Offline/headless .pbix mode** | Requires file-level .pbix manipulation | V2 adds Tabular Editor 2 CLI subprocess |
| **Bundled AI models** | Keeps binary small (<35MB). Users bring API keys | Optional local LLM via Ollama/candle in V2 |
| **Custom Visual development** | Different product category | Explore in V3 |
| **Multi-user collaboration** | Complex, requires server infra | V3+ |
| **Direct Power BI Service REST API** | User publishes natively from PBI Desktop | Explore for automated publishing in V2 |

---

## 3. Core Capabilities

| # | Capability | Description | Configurable? |
|---|---|---|---|
| C01 | **Schema Discovery** | Auto-detect tables, columns, relationships, data types via XMLA DISCOVER | ✅ |
| C02 | **Data Profiling** | Distribution analysis, null detection, outlier detection via Polars | ✅ |
| C03 | **DAX Auto-Generation** | Generate measures, calculated columns, calculated tables from natural language | ✅ |
| C04 | **DAX Complexity Levels** | Beginner / Intermediate / Advanced / Expert — configurable per project | ✅ |
| C05 | **DAX Validation** | PEG parser, semantic validation, performance estimation | ✅ |
| C06 | **DAX Self-Correction** | AI loop: validate → identify errors → regenerate → re-validate (max 3 cycles) | ✅ |
| C07 | **DAX Explanation** | Natural language explanation of any generated measure | ✅ |
| C08 | **Dashboard Layout Generation** | AI-suggested layouts based on data profile, user intent, and complexity | ✅ |
| C09 | **Visual Binding** | Auto-bind visuals to measures and fields via TMSL | ✅ |
| C10 | **Theme Engine** | Custom themes, color palettes, font stacks. Import/export JSON themes. | ✅ |
| C11 | **Image-to-Dashboard** | Upload sketch/infographic → AI (GPT-4V) extracts layout/colors → generates matching dashboard | ✅ |
| C12 | **Exploratory Analysis Mode** | Interactive data exploration with AI-guided insights | ✅ |
| C13 | **On-Demand Measure Creation** | Create measures mid-session, validate, apply to model instantly | ✅ |
| C14 | **Self-Correction** | AI detects and fixes broken/inefficient DAX, then reapplies | ✅ |
| C15 | **MCP Server/Client** | Expose as MCP server (stdio). Connect to external MCP servers. | ✅ |
| C16 | **Prompt Management** | All AI prompts externalized as text files, versioned via git, upgradeable | ✅ |
| C17 | **Headless/CLI Mode** | Run full pipeline from terminal. Works with PBI Desktop closed. | ✅ |
| C18 | **White-Label** | Rebrand entire product: name, logos, colors. Config-driven. | ✅ |
| C19 | **Git Integration** | Config-as-code. Full git workflow for settings, themes, prompts. | ✅ |
| C20 | **Data Ingestion** | Ingest CSV, SQL, Parquet, JSON. Auto-profile, infer model. Build Tabular model from scratch. | ✅ |
| C21 | **Visual Canvas** | WYSIWYG drag-drop editor. Grid canvas, visual palette, property panel, real-time preview. | ✅ |

---

## 4. Architecture Decision: Why Rust, Why External Tool

### 4.1 Why Rust

```
╔══════════════════════════════════════════════════════════════════╗
║                 RUST: THE OPTIMAL CHOICE                         ║
╠══════════════════════════════════════════════════════════════════╣
║  PERFORMANCE           SAFETY              DEPLOYMENT            ║
║  ┌────────────────┐   ┌────────────────┐   ┌────────────────┐   ║
║  │ • Zero-cost    │   │ • No null      │   │ • Single        │   ║
║  │   abstractions │   │   pointers     │   │   static binary │   ║
║  │ • No GC pauses │   │ • No data      │   │ • Copy & run    │   ║
║  │ • Native code  │   │   races        │   │ • No runtime    │   ║
║  │ • SIMD for     │   │ • No segfaults │   │   dependencies  │   ║
║  │   data work    │   │ • Compile-time │   │ • 15-25MB       │   ║
║  │ • Async I/O    │   │   guarantees   │   │   total         │   ║
║  └────────────────┘   └────────────────┘   └────────────────┘   ║
║  ECOSYSTEM            AI CAPABILITY         DATA STACK           ║
║  ┌────────────────┐   ┌────────────────┐   ┌────────────────┐   ║
║  │ • reqwest       │   │ • async-openai │   │ • polars       │   ║
║  │   (HTTP/XMLA)   │   │ • candle       │   │ • datafusion   │   ║
║  │ • serde         │   │   (local LLM)  │   │ • arrow-rs     │   ║
║  │   (JSON/TMSL)   │   │ • fastembed-rs │   │ • lancedb      │   ║
║  │ • tokio         │   │   (embeddings) │   │   (vectors)    │   ║
║  │   (async)       │   │ • image-rs     │   │ • parquet      │   ║
║  │ • clap (CLI)    │   │                │   │                │   ║
║  │ • ratatui (TUI) │   │                │   │                │   ║
║  └────────────────┘   └────────────────┘   └────────────────┘   ║
╚══════════════════════════════════════════════════════════════════╝
```

### 4.2 Why External Tool (Not Standalone App)

The Power BI External Tool mechanism is the **native, first-class plugin architecture** for Power BI Desktop — the same mechanism used by Tabular Editor, DAX Studio, ALM Toolkit, Bravo, and every other professional Power BI tool.

**Registration (one JSON file):**
```json
// %APPDATA%\Microsoft\Power BI Desktop\External Tools\TRRUSTT.json
{
  "version": "1.0",
  "name": "🔷 TRRUSTT",
  "description": "AI-powered dashboard builder",
  "path": "C:\\Program Files\\TRRUSTT\\TRRUSTT.exe",
  "arguments": "--port %port% --pbix \"%pbix%\""
}
```

| Benefit | Why It Matters |
|---|---|
| **Zero auth code** | PBI Desktop authenticates the user. We just talk to localhost. |
| **Zero publish code** | User clicks "Publish" in PBI Desktop natively. |
| **Live model access** | Changes appear instantly in PBI Desktop. No file sync needed. |
| **PBI Desktop IS the UI** | Users work in their familiar environment. |
| **No .pbix reverse-engineering** | We query the live SSAS instance via XMLA. |
| **Works with any .pbix** | Any file opened in PBI Desktop is accessible. |
| **Organization SSO** | Works across every org's Entra ID / AAD setup automatically. |

### 4.3 The XMLA Bridge

Power BI Desktop runs an embedded SSAS Tabular instance on a random localhost port:

```
IntelliDashboard (Rust) ──XMLA/HTTP──▶ SSAS Tabular Instance
                                        localhost:XXXXX
                                        (PBI Desktop child process)

XMLA Operations:
  DISCOVER_XMLA           → Schema metadata (tables, columns, measures, relationships)
  EXECUTE (DAX: EVALUATE) → Query data samples, run profiling queries
  TMSL createOrReplace    → Create/update measures, tables, visuals
  TMSL delete             → Remove objects
  TMSL refresh            → Trigger model refresh
```

**For offline .pbix access** (PBI Desktop not running), we spawn Tabular Editor CLI as a subprocess.

### 4.4 Rust Crate Map

| Category | Crate | Purpose |
|---|---|---|
| Async Runtime | `tokio` | Core async engine |
| HTTP Client | `reqwest` | XMLA calls to SSAS, LLM API calls |
| JSON | `serde_json` | TMSL, config, API responses |
| Serialization | `serde` | Derive macros |
| CLI | `clap` | Argument parsing |
| TUI | `ratatui` | Terminal admin UI (optional) |
| Config | `figment` | Layered config system |
| LLM Provider | `async-openai` | OpenAI, Azure OpenAI, Anthropic, Google Gemini, DeepSeek, Ollama |
| Local LLM | `candle` | Optional embedded local inference |
| Embeddings | `fastembed-rs` | Local text embeddings |
| Vector DB | `lancedb` (Rust) | Schema vector store |
| DataFrames | `polars` | Data profiling & analysis |
| Arrow | `arrow-rs` | Columnar data format |
| SQL Engine | `datafusion` | Complex analytical queries |
| PEG Parser | `pom` | DAX expression parser |
| Image | `image` | Image loading & processing |
| Logging | `tracing` | Structured logging |
| Observability | `opentelemetry` | Traces & metrics |
| Encryption | `ring` + `aes-gcm` | Config secrets |
| JWT | `jsonwebtoken` | License validation |
| Git | `git2` (libgit2) | Config versioning |
| Testing | `proptest` | Property-based testing |
| Benchmarking | `criterion` | Performance benchmarks |

---

## 5. System Architecture

### 5.1 One Process. One Binary.

```
┌──────────────────────────────────────────────────────────────────────┐
│                   INTELLIDASHBOARD (Single Rust Binary)                │
│                                                                       │
│  ENTRY POINTS                                                         │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────────────────┐ │
│  │ CLI Mode │  │ TUI Mode │  │ MCP Mode │  │ External Tool Mode   │ │
│  │ (clap)   │  │(ratatui) │  │ (stdio)  │  │ (launched by PBI)    │ │
│  └────┬─────┘  └────┬─────┘  └────┬─────┘  └──────────┬───────────┘ │
│       └─────────────┴─────────────┴───────────────────┘              │
│                            │                                         │
│  ┌─────────────────────────▼──────────────────────────────────────┐  │
│  │  CORE ENGINE                                                     │  │
│  │  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌────────────────┐  │  │
│  │  │XMLA      │  │DAX       │  │Schema    │  │Dashboard       │  │  │
│  │  │Client    │  │Engine    │  │Engine    │  │Generator       │  │  │
│  │  │• DISCOVER│  │• PEG     │  │• Polars  │  │• Layout AI     │  │  │
│  │  │• EXECUTE │  │  Parser  │  │• Profiles│  │• Visual TMSL   │  │  │
│  │  │• TMSL    │  │• Validate│  │• Arrow   │  │• Theme Apply   │  │  │
│  │  │• Subproc │  │• Correct │  │• Quality │  │• Bind Fields   │  │  │
│  │  └──────────┘  └──────────┘  └──────────┘  └────────────────┘  │  │
│  └─────────────────────────────────────────────────────────────────┘  │
│                                                                       │
│  ┌─────────────────────────────────────────────────────────────────┐  │
│  │  AI ENGINE                                                        │  │
│  │  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌────────────────┐  │  │
│  │  │LLM       │  │RAG       │  │Vision    │  │Prompt          │  │  │
│  │  │Router    │  │Pipeline  │  │Engine    │  │Manager         │  │  │
│  │  └──────────┘  └──────────┘  └──────────┘  └────────────────┘  │  │
│  └─────────────────────────────────────────────────────────────────┘  │
│                                                                       │
│  ┌─────────────────────────────────────────────────────────────────┐  │
│  │  INFRASTRUCTURE                                                   │  │
│  │  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌────────────────┐  │  │
│  │  │Config    │  │License   │  │MCP Hub   │  │Observability   │  │  │
│  │  │Engine    │  │Manager   │  │          │  │(OpenTelemetry) │  │  │
│  │  └──────────┘  └──────────┘  └──────────┘  └────────────────┘  │  │
│  └─────────────────────────────────────────────────────────────────┘  │
│                                                                       │
└──────────────────────────────────────────────────────────────────────┘
```

### 5.2 Operating Modes

| Mode | Trigger | UI | Use Case |
|---|---|---|---|
| **GUI (Primary)** | Double-click `TRRUSTT.exe` or PBI Desktop ribbon | Tauri window — dashboard wizard, DAX playground, theme designer, settings | Primary: beautiful polished GUI for all user interactions |
| **External Tool** | PBI Desktop ribbon click | Auto-opens GUI pre-connected to SSAS port | Seamless: click in PBI Desktop → GUI opens ready to go |
| **CLI** | `trrustt [command] --port 54321` | Terminal output | Automation, scripting, CI/CD pipelines |
| **MCP Server** | `trrustt mcp serve` | stdio (JSON-RPC) | Integration with Claude Desktop, Cursor, Continue |
| **TUI Admin** | `trrustt admin` | Terminal UI (ratatui) | Quick config changes, license activation from terminal |

### 5.3 Directory Structure

```
trrustt/
├── Cargo.toml                          # Workspace manifest
├── Cargo.lock
├── rust-toolchain.toml                 # Pin Rust version
├── .github/workflows/                  # CI/CD
├── crates/
│   ├── trrustt/                        # Main binary + Tauri shell
│   ├── xmla-client/                    # XMLA/TMSL client library
│   ├── dax-engine/                     # DAX parsing, validation, generation
│   ├── ai-engine/                      # LLM orchestration, RAG, vision
│   ├── config-engine/                  # Configuration management
│   ├── dashboard-generator/            # Dashboard layout & visual generation
│   ├── mcp-hub/                        # MCP server + client
│   ├── data-store/                     # SQLite + LanceDB internal database
│   ├── data-ingestion/                 # CSV, SQL, Parquet, JSON → Tabular model
│   ├── license-manager/               # License validation & management
│   └── shared/                         # Shared types, utilities, constants
├── prompts/                            # Externalized AI prompts
├── themes/                             # Default theme files
├── config/                             # Default config
├── tests/                              # Integration tests
├── benches/                            # Benchmarks
├── docs/                               # Documentation
└── README.md
```

### 5.4 Communication (Internal)

All internal communication is **direct function calls** within the same process. No IPC, no serialization overhead, no network boundaries between components. The XMLA client talks HTTP to localhost SSAS; the AI engine talks HTTPS to LLM providers. Everything else is pure Rust function calls.

### 5.5 Internal Database (SQLite + LanceDB)

**Two embedded databases, zero servers, zero configuration:**

| Database | Engine | Crate | Purpose |
|---|---|---|---|
| **SQLite** | SQLite | `sqlx` | Projects, measures, dashboards, AI usage tracking, audit log, AI response cache |
| **LanceDB** | LanceDB | `lancedb` (Rust) | Schema embeddings for RAG (vector search over data model) |

**Why this stack:**
- **SQLite:** Public domain, single file (`~/.trrustt/data/trrustt.db`), ACID compliant, full SQL. Access via `sqlx` — compile-time checked queries, no ORM overhead.
- **LanceDB:** Rust-native embedded vector DB. Stores schema metadata as embeddings for AI-powered semantic search.
- **No PostgreSQL, no MySQL, no external servers.** Everything embedded. Backup = copy one folder.

Full database schema in [architecture/02-data-flow.md](architecture/02-data-flow.md#31-internal-database-sqlite--sqlx).

---

## 6. Configuration & Settings Engine

### 6.1 Philosophy

> **"If it exists in the product, it MUST be configurable. Nothing is hard-coded. Ever."**

### 6.2 Resolution Order (highest priority first)

1. Environment variables (`TRRUSTT_AI_PROVIDER=openai`)
2. CLI arguments (`--ai-provider openai`)
3. User config (`~/.intellidashboard/config/user.toml`)
4. Project config (`.intellidashboard/config/project.toml`)
5. Workspace config (`~/.intellidashboard/config/workspace.toml`)
6. Admin-enforced policies
7. System defaults (embedded in binary at compile time)

### 6.3 Rust Implementation

```rust
// crates/config-engine/src/engine.rs
use figment::{Figment, providers::{Toml, Env, Format}};
use serde::{Serialize, Deserialize};

pub struct ConfigEngine {
    figment: Figment,
    registry: ConfigRegistry,
}

impl ConfigEngine {
    pub fn get<T: DeserializeOwned>(&self, path: &str) -> Result<T, ConfigError> {
        self.figment.extract_inner(path)
    }
    
    pub fn set<T: Serialize>(&self, path: &str, value: T, scope: ConfigScope) -> Result<()> {
        self.registry.validate(path, &value)?;
        // Check admin policy override
        // Write to appropriate file
        Ok(())
    }
}
```

### 6.4 Config Format (TOML)

```toml
# ~/.intellidashboard/config/user.toml
[ai]
provider = "openai"
default_model = "gpt-4o"
temperature = 0.3

[dax]
complexity_level = "advanced"
naming_convention = "Pascal Case"
comment_style = "brief"

[dashboard]
default_page_size = "16:9"
grid_density = "normal"

[theme]
active = "modern-dark"

[observability]
log_level = "info"
log_format = "json"
```

---

## 7. AI Engine Design

### 7.1 Multi-Provider Architecture

```rust
// crates/ai-engine/src/router.rs
#[async_trait]
pub trait AiProvider: Send + Sync {
    async fn chat(&self, request: ChatRequest) -> Result<ChatResponse>;
    fn supports_vision(&self) -> bool;
}

// Providers: OpenAiProvider, AzureOpenAiProvider, AnthropicProvider, 
//            OllamaProvider, CandleProvider (local)

pub struct LlmRouter {
    providers: HashMap<String, Box<dyn AiProvider>>,
    cost_tracker: CostTracker,
    rate_limiter: RateLimiter,
}
```

### 7.2 Prompt Chain Architecture

Prompt chains are externalized text files loaded at runtime. Each chain is a sequence of steps: render template → call LLM → parse output → validate → next step.

```
chains/
├── dashboard-generation/
│   ├── 01-classify-intent.txt
│   ├── 02-plan-layout.txt
│   ├── 03-generate-visuals.txt
│   └── 04-validate-layout.txt
├── dax-generation/
│   ├── 01-classify-request.txt
│   ├── 02-generate-measure.txt
│   ├── 03-validate-dax.txt
│   └── 04-explain-measure.txt
└── image-to-dashboard/
    ├── 01-analyze-image.txt
    ├── 02-extract-layout.txt
    └── 03-match-to-schema.txt
```

### 7.3 RAG Pipeline

```rust
// crates/ai-engine/src/rag/pipeline.rs
pub struct RagPipeline {
    embedder: Box<dyn Embedder>,         // fastembed-rs or OpenAI
    vector_store: LanceDbStore,           // Embedded LanceDB
    bm25_index: Bm25Index,               // Keyword search
}

impl RagPipeline {
    pub async fn index_schema(&self, schema: &SchemaMetadata) -> Result<()>;
    pub async fn search(&self, query: &str, top_k: usize) -> Result<Vec<SearchResult>>;
    pub fn build_context(&self, results: &[SearchResult]) -> String;
}
```

---

## 8. DAX Engine

### 8.1 PEG Parser (pom)

```rust
// crates/dax-engine/src/parser/mod.rs
use pom::parser::*;

fn dax_expression<'a>() -> Parser<'a, u8, DaxExpression> {
    choice(vec![
        function_call().map(DaxExpression::FunctionCall),
        column_reference().map(DaxExpression::ColumnRef),
        constant().map(DaxExpression::Constant),
        var_return().map(DaxExpression::VarReturn),
        binary_operation(),
        parenthesized(),
    ])
}

pub fn parse_dax(input: &str) -> Result<DaxAst, ParseError> {
    dax_expression().parse(input.as_bytes())
}
```

### 8.2 Validation Pipeline

Built-in validation steps (each independently configurable):
- **SyntaxValidator** — Parse errors
- **ReferenceValidator** — All references exist in schema
- **TypeValidator** — Type compatibility
- **FunctionValidator** — Function signatures
- **PerformanceValidator** — Flags expensive patterns
- **StyleValidator** — Naming, formatting, comments
- **SecurityValidator** — No disallowed functions per whitelist

### 8.3 Self-Correction Loop

```
Generate DAX → Validate → Valid? → YES → Apply to model
                           ↓ NO
                      AI Fix (low temp)
                           ↓
                      Re-validate
                      (max 3 attempts)
```

### 8.4 Complexity Levels

| Level | Allowed Patterns |
|---|---|
| **Beginner** | SUM, AVERAGE, COUNT, MIN, MAX, DISTINCTCOUNT, CALCULATE (simple filters), DIVIDE |
| **Intermediate** | + VAR/RETURN, time intelligence, basic iterators (SUMX, AVERAGEX), FILTER, ALL, VALUES |
| **Advanced** | + Complex iterators, ADDCOLUMNS, SUMMARIZE, TREATAS, INTERSECT, calculation groups awareness |
| **Expert** | + GENERATE, CROSSJOIN, evaluation context manipulation, SUMMARIZECOLUMNS optimization, debugging patterns |

---

## 9. Dashboard Generation Pipeline

```
STEP 1: Schema Discovery (XMLA DISCOVER)
STEP 2: Data Profiling (Polars via DAX EVALUATE queries)
STEP 3: User Intent (natural language + optional image)
STEP 4: AI Analysis & Planning (LLM + RAG context)
STEP 5: DAX Generation (AI → Validate → Self-Correct → Apply via TMSL)
STEP 6: Layout Generation (AI → Grid calc → Visual TMSL → Apply)
STEP 7: User Review (Live in PBI Desktop — refresh to see changes)
STEP 8: Export (User clicks Publish in PBI Desktop natively)
```

---

## 10. Image-to-Dashboard Pipeline

```
IMAGE INPUT (sketch, infographic, mockup, whiteboard photo)
        │
        ▼
VISION ANALYSIS (GPT-4V / Claude Vision)
  • Detect layout structure (grid, regions)
  • Identify visual elements (charts, cards, KPIs)
  • Extract color palette
  • Detect text labels & annotations
        │
        ▼
LAYOUT MAPPING
  • Map detected regions to Power BI visuals
  • Determine grid coordinates
  • Match color palette to theme
        │
        ▼
DATA MATCHING
  • Match detected labels to schema fields
  • Generate appropriate DAX measures
  • Map data to visual slots
        │
        ▼
DASHBOARD GENERATION
  • Create visuals matching the image layout
  • Apply extracted theme
  • Bind data
        │
        ▼
AI CORRECTION LOOP (max 5 cycles)
  • Compare generated dashboard to original image
  • Score similarity
  • Iteratively refine
```

---

## 11. MCP Server Integration

### 11.1 MCP Server (stdio transport)

Exposes IntelliDashboard as an MCP server — usable from Claude Desktop, Continue, Cursor, and any MCP-compatible AI tool.

**Built-in MCP Tools:**
```
schema_discover         — Discover tables, columns, relationships
schema_profile          — Statistical profile of data
dax_generate            — Generate DAX measure from description
dax_validate            — Validate DAX expression
dax_explain             — Natural language explanation of DAX
dax_optimize            — Suggest performance improvements
dashboard_create        — Create dashboard page with visuals
dashboard_add_visual    — Add a visual to existing page
dashboard_apply_theme   — Apply theme to dashboard
data_query              — Run DAX query against model
data_insights           — AI-generated insights from data
image_analyze_dashboard — Analyze uploaded dashboard image
config_get              — Read configuration value
config_set              — Write configuration value
prompt_list             — List available prompts
prompt_upgrade          — Upgrade prompts from repository
```

### 11.2 MCP Client

Connect to external MCP servers for extended capabilities. Configuration:

```toml
[[mcp.servers]]
id = "sql-explorer"
name = "SQL Explorer"
transport = "stdio"
command = "sql-explorer-mcp"
args = ["--db", "production"]
allowed_tools = ["query", "explain"]
timeout_ms = 30000
```

---

## 12. Authentication & Authorization

### 🔑 PBI Desktop Handles This

Power BI Desktop already authenticates the user against Azure AD / Entra ID. Our tool talks to `localhost:XXXXX` — no authentication required for local XMLA. The SSAS instance is already secured by the OS.

For MCP server and CLI headless modes, optional API key validation is available:

```rust
pub fn validate_api_key(key: &str) -> Result<(), AuthError> {
    let stored_hash = config().get::<String>("security.api_key_hash")?;
    if bcrypt::verify(key, &stored_hash)? { Ok(()) }
    else { Err(AuthError::InvalidKey) }
}
```

---

## 13. Role-Based Access Control (RBAC)

Roles defined in `config/roles.toml`:

```
SUPER_ADMIN ── All permissions
    └── ADMIN ── dashboard:*, dax:*, schema:*, config:*, user:manage
          └── DESIGNER ── dashboard:*, dax:*, schema:read
                └── ANALYST ── dashboard:view, dax:create, schema:read
                      └── VIEWER ── dashboard:view, schema:read
```

---

## 14. Whitelisting System

Configurable whitelist/blocklist for:
- DAX functions (per complexity level)
- Visual types
- Data sources
- Export formats
- AI models
- MCP servers
- User IPs, domains, emails, countries

---

## 15. Branding & White-Label System

Every brand element is configurable: app name, logos, colors, typography, URLs, legal text. OEM partners receive a branding config template and a custom build pipeline.

---

## 16. Monetization Strategy

| Route | Model | Target |
|---|---|---|
| **Direct Sales** | Free tier → Pro ($149/yr) → Pro+ ($249/yr) | B2C/B2B |
| **Enterprise** | $49/user/month, volume discounts | B2B |
| **White-Label/OEM** | $25K–$100K/yr + 5-15% revenue share | B2B |
| **API/PAYG** | Per-request pricing | B2B |
| **Marketplace Add-ons** | Themes, DAX recipes, templates ($10–$200) | B2C |
| **Managed Services** | Consulting, training, custom dev | B2B |

Year 1 Target ARR: ~$167K | Year 3 Target ARR: ~$1.9M

---

## 17. Marketplace & Distribution

| Channel | Revenue Share | Notes |
|---|---|---|
| **Microsoft AppSource** | 15% | Built-in Power BI audience |
| **GitHub Releases** | 0% | Developer community |
| **Official Website** | ~5% (Paddle) | Full control |
| **Azure Marketplace** | 15% | Enterprise procurement |
| **WinGet / Chocolatey / Homebrew** | 0% | Package managers |

---

## 18. Testing Strategy

| Layer | Tool | Target |
|---|---|---|
| Unit Tests | `cargo test` | 90%+ coverage |
| Doc Tests | `cargo test --doc` | Examples verified |
| Integration | `cargo test --test '*'` | Cross-crate |
| Property Tests | `proptest` | Fuzz/generative |
| Benchmarks | `criterion` | Regression detection |
| Linting | `clippy` (strict, `-D warnings`) | Code quality |
| Formatting | `rustfmt` | Consistent style |
| Safety | `#![forbid(unsafe_code)]` + `cargo-geiger` | Zero unsafe |
| Audit | `cargo-audit` + `cargo-deny` | Security + licenses |
| Coverage | `cargo-llvm-cov` | 80%+ target |
| Miri | `cargo miri test` | UB detection |
| DAX Correctness | Custom test harness | 100% of templates |

---

## 19. Packaging & Deployment

```
Release build:
  RUSTFLAGS="-C target-cpu=native -C lto=fat -C codegen-units=1"
  cargo build --release

Result: target/release/IntelliDashboard.exe (~15-25 MB, stripped)
Dependencies: NONE (statically linked)

Deploy:
  1. Copy .exe anywhere
  2. Register in External Tools folder
  3. Done.

Package managers:
  winget install IntelliDashboard.Builder
  scoop install intellidashboard
  choco install intellidashboard
  brew install intellidashboard/builder
```

---

## 20. CI/CD & Git Strategy

```
PR → clippy → fmt → test (win+mac+linux) → audit → coverage
                                                      │
                                                      ▼
merge to main → build release → sign (EV cert) → GitHub Release
                                                    │
                                                    ▼
                                            winget / scoop / choco
                                            auto-update manifest
```

Git-based config versioning for users:
```bash
intellidashboard git init
intellidashboard git push
intellidashboard git pull   # Team config sync
intellidashboard git revert <commit>  # Rollback
```

---

## 21. Logging, Monitoring & Observability

OpenTelemetry-native via `tracing` + `opentelemetry` crates:
- Structured JSON logs (rotating files)
- Distributed tracing (every AI request, DAX generation, TMSL operation traced)
- Custom metrics (dashboard gen time, token usage, error rates)
- Health endpoint
- Configurable export to OTLP collector

---

## 22. Security Architecture

| Area | Measure |
|---|---|
| Memory Safety | `#![forbid(unsafe_code)]` |
| Config Secrets | AES-256-GCM via `ring`, OS keychain storage |
| XMLA Transport | Localhost-only (loopback enforced) |
| Dependency Audit | `cargo-audit` + `cargo-deny` in CI |
| Supply Chain | Committed `Cargo.lock`, `cargo-vet`, SBOM generation |
| Code Signing | EV Code Signing Certificate |
| No Network Exposure | Only outbound to localhost + configured LLM APIs |

---

## 23. Internationalization (i18n) & Localization (l10n)

- Fluent-based localization via `fluent-rs`
- Translation files in `i18n/{locale}/`
- Regional number/date/currency formatting
- RTL support (Arabic, Hebrew, Farsi)
- Configurable per user

---

## 24. Roadmap & Milestones

**Phase 0: Foundation (Months 1-2)**
- [ ] Cargo workspace, crate structure
- [ ] XMLA client (DISCOVER, EXECUTE, TMSL)
- [ ] DAX PEG parser
- [ ] Config engine (figment)
- [ ] CLI framework (clap)
- [ ] CI/CD pipeline

**Phase 1: Core Intelligence (Months 3-4)**
- [ ] AI engine (multi-provider)
- [ ] RAG pipeline (embeddings + LanceDB)
- [ ] DAX validator
- [ ] DAX self-correction
- [ ] Prompt chain engine
- [ ] Dashboard layout generator

**Phase 2: Full Pipeline (Months 5-6)**
- [ ] All 4 DAX complexity levels
- [ ] TMSL visual creation
- [ ] Theme engine
- [ ] Image-to-dashboard
- [ ] MCP server + client
- [ ] CLI headless mode

**Phase 3: Enterprise & Monetization (Months 7-8)**
- [ ] License manager
- [ ] RBAC + whitelisting
- [ ] White-label system
- [ ] Payment integration
- [ ] Auto-update
- [ ] Package manager distribution

**Phase 4: Polish & GA (Months 9-10)**
- [ ] Security audit
- [ ] Performance optimization
- [ ] Documentation
- [ ] GA release v1.0.0

---

## 25. Open Questions & Decisions

| # | Question | Answer / Decision | Status |
|---|---|---|---|
| Q1 | Open-source core with paid features, or fully proprietary? | **Open-source core with paid features.** Build it intelligently — free tier is open-source (MIT/Apache 2.0), Pro/Enterprise features are paid. Transparency builds trust. The core DAX engine, XMLA client, and config engine can be open-source. Premium features (advanced AI chains, image-to-dashboard, enterprise RBAC, white-label) are paid modules. | ✅ DECIDED |
| Q2 | Ship AI models bundled (Ollama/local) or cloud-only? | **Cloud-only.** Do not ship AI models. Keeps binary size small (<25MB). Users bring their own API keys (OpenAI, Azure, Anthropic). Optional: connect to user's local Ollama instance if they already have one running. No model weight files in the binary. | ✅ DECIDED |
| Q3 | Tabular Editor 2 (free) vs Tabular Editor 3 (commercial) dependency? | **NOT NEEDED for V1.** TRRUSTT V1 only supports live PBI Desktop mode via direct XMLA. No Tabular Editor dependency at all — zero external dependencies. Offline .pbix mode (which would need TE2 CLI) is deferred to V2. | ✅ DECIDED — LOCKED 🔒 |
| Q4 | Own license server or third-party (Keygen, Cryptlex)? | **Pure Rust JWT-based license for V1. LOCKED.** Ed25519 signatures, AES-256-GCM encrypted cache, 30-day offline grace, periodic phone-home. Zero infrastructure cost, zero monthly fees. Upgrade to Keygen.sh (~$99/mo) in V2 if automated purchase→license flow is needed. | ✅ DECIDED — LOCKED 🔒 |
| Q5 | Microsoft AppSource approval requirements? | **Research needed during Phase 2.** Likely requirements: Microsoft Partner Network membership ($0 for developer), app certification process, security & compliance review. 4-8 week approval timeline. Not blocking V1 — direct sales via website first. | 🔴 RESEARCH |
| Q6 | Which LLM for vision/image-to-dashboard? | **Separate config for analytics AI vs vision AI.** Analytics (DAX generation, dashboard planning, data insights) can use one provider/model (e.g., GPT-4o via Azure). Vision (image-to-dashboard, layout extraction, color analysis) can use a different provider/model (e.g., GPT-4V or Claude Vision). All independently configurable in `ai.analytics.*` and `ai.vision.*` config sections. | ✅ DECIDED |
| Q7 | Pricing finalization? | **Defer to Phase 3.** Market research needed. Starting assumptions: Free tier (open-source, limited), Pro $149/yr, Team $49/user/mo. Validate with beta users before locking in. | 🟡 DEFERRED |
| Q8 | Company formation / legal entity? | **Individual developer (sole proprietor) for now.** Not forming a company yet. Will operate as an individual. Revisit incorporation when revenue justifies it (likely at $50K+ ARR). | ✅ DECIDED |
| Q9 | Patent search for AI dashboard generation? | **Defer to later.** Not critical for V1. Do prior art search before significant investment in unique algorithms. Likely low risk — AI dashboard generation is a relatively new field with limited patents. | 🟡 DEFERRED |
| Q10 | Product name and branding? | **Product name: "TRRUSTT".** Need a logo design. The name conveys trust + data. All-caps styling: TRRUSTT. Color: blue/indigo primary. Trademark registration deferred until logo is finalized and product has market presence. | 🟡 LOGO NEEDED |
| Q11 | `#![forbid(unsafe_code)]` or allow minimal unsafe for FFI? | **`#![forbid(unsafe_code)]`** — no unsafe Rust anywhere. No FFI needed since we use XMLA (HTTP) instead of COM/.NET interop. The only potential exception is GPU acceleration for local LLM inference (candle), which would be behind a feature flag and opt-in. | ✅ DECIDED |
| Q12 | Bundle Tabular Editor CLI or require separate install? | **NOT APPLICABLE for V1.** No Tabular Editor dependency whatsoever. TRRUSTT V1 connects only to running PBI Desktop via XMLA. When V2 adds offline .pbix mode, TE2 CLI will be an optional, documented dependency (separate install). | ✅ DECIDED — LOCKED 🔒 |
| Q13 | Tauri webview for rich settings UI: V1 or defer? | **Defer to V2.** V1 uses CLI (clap) + optional TUI (ratatui) for settings management. Config files are the source of truth. Rich GUI settings editor comes in V2 when we have more resources. | ✅ DECIDED |
| Q14 | MCP SDK — build from spec or use community crate? | **Monitor Rust MCP ecosystem.** Build from spec if no mature crate exists by implementation time. Protocol is simple JSON-RPC 2.0 over stdio — not complex to implement from scratch if needed. Check crates.io for `mcp-*` crates before building. | 🔴 MONITOR |

### Q3 Detailed: Tabular Editor 2 vs Tabular Editor 3

**Tabular Editor 2 (Free, Open-Source):**
- Full CLI available for scripting and automation (`tabulareditor.exe`)
- Can read/write .pbix files directly (offline mode — no PBI Desktop needed)
- Can connect to running SSAS instance for live model manipulation
- TMSL scripting support — pass JSON commands, get results
- Schema export as JSON (`--export-schema -`)
- Battle-tested, used by thousands of Power BI professionals daily
- MIT licensed — can be bundled/distributed if we wanted to

**Tabular Editor 3 (Commercial, ~$99/year):**
- Everything in TE2, plus:
- Diagram view for model visualization (not needed by TRRUSTT)
- Advanced DAX debugging and querying (not needed — we have our own DAX engine)
- C# scripting for custom automation (not needed)
- Better UI/UX (not needed — TRRUSTT is CLI-first)

**TRRUSTT's Dependency on Tabular Editor:**
```
99% of use cases:  PBI Desktop running → TRRUSTT uses direct XMLA to SSAS
                   NO Tabular Editor needed at all.
                   ┌──────────────────────────────────────────┐
                   │ Read schema?     → XMLA DISCOVER         │
                   │ Create measure?  → TMSL createOrReplace  │
                   │ Modify metadata? → TMSL createOrReplace  │
                   │ Create visual?   → TMSL createOrReplace  │
                   │ Query data?      → XMLA EXECUTE (DAX)    │
                   │ Everything is standard HTTP to localhost │
                   └──────────────────────────────────────────┘

1% of use cases:   PBI Desktop NOT running → TRRUSTT spawns
                   Tabular Editor 2 CLI as subprocess:
                   ┌──────────────────────────────────────────┐
                   │ tabulareditor.exe --file report.pbix     │
                   │   --script deploy-measures.tmsl          │
                   │                                          │
                   │ tabulareditor.exe --file report.pbix     │
                   │   --export-schema - > schema.json        │
                   └──────────────────────────────────────────┘
                   This is an OPTIONAL dependency.
```

**Decision:** Dependency on Tabular Editor 2 CLI only (free, MIT licensed). It is an **optional** dependency — only needed when the user wants to work with .pbix files without PBI Desktop running (headless/offline/CI-CD mode). The core workflow (PBI Desktop open → External Tool ribbon → live model manipulation) uses zero third-party dependencies beyond standard XMLA/HTTP.

### Q4 Detailed: License Server Options

**Option A: Simple JWT-Based Validation (✅ Recommended for V1)**
```
How it works:
  1. User purchases license on website (Paddle/Stripe)
  2. A simple script/function generates a signed JWT containing:
     - License tier (free / pro / team / enterprise)
     - Feature flags (what's enabled)
     - Expiry date (1 year from purchase)
     - Customer email hash
     - Customer ID (for telemetry)
  3. JWT is emailed to user or shown on purchase confirmation page
  4. User activates: trrustt license activate <jwt-token>
  5. TRRUSTT verifies JWT signature locally (Ed25519 or RS256)
     → No server call needed at validation time
  6. License cached encrypted on disk (AES-256-GCM)
  7. Offline: valid for 30 days (configurable) without phoning home
  8. Online: periodic check (every 7 days) to license server to:
     - Verify license hasn't been revoked
     - Get updated feature flags
     - Report usage telemetry

Pros: Zero infrastructure cost, offline-capable, simple, no monthly fees
Cons: Cannot instantly revoke licenses (user has 7-30 day grace period)
      Manual license generation (automate later with Paddle webhooks)
      No customer self-service portal (yet)
Cost: $0/month (just a static endpoint to check revocation — can be a simple JSON file on GitHub)
```

**Option B: Keygen.sh (SaaS License Management)**
```
How it works:
  - REST API for license creation, validation, revocation
  - Webhooks for purchase events (integrate with Paddle/Stripe)
  - Customer portal (self-service: view licenses, download invoices)
  - Machine locking / node-locking (prevent sharing)
  - Analytics dashboard (activations, churn, usage)

Pros: Full-featured, Paddle integration, low maintenance, customer portal
Cons: Monthly cost, dependency on third-party service
Cost: ~$99/month (Indie plan, up to 500 licenses)
```

**Option C: Cryptlex (License Management Platform)**
```
How it works:
  - On-premise or cloud deployment
  - Node-locked and floating licenses
  - Trial management (time-limited, feature-limited)
  - Offline activation with deactivation support
  - Analytics and audit logs

Pros: Enterprise-grade, flexible deployment options
Cons: More complex setup, higher cost
Cost: ~$49-199/month depending on license count
```

**Decision for TRRUSTT:**
- **V1 (Now):** Option A — Simple JWT validation. Zero cost, zero infrastructure, zero monthly fees. Perfect for an individual developer launching a product. The "risk" of delayed revocation is acceptable for a downloadable tool.
- **V2 (When revenue > $2K MRR):** Migrate to Option B (Keygen.sh). Provides automated purchase→license flow via Paddle webhooks, customer self-service portal, and better analytics.
- **Enterprise deals (custom):** Manual license generation + direct invoice. Handle via email until automated.

---

### A. Glossary

| Term | Definition |
|---|---|
| **SSAS** | SQL Server Analysis Services — the engine inside Power BI Desktop |
| **XMLA** | XML for Analysis — protocol for OLAP server communication |
| **TMSL** | Tabular Model Scripting Language — JSON commands for model manipulation |
| **TOM** | Tabular Object Model — .NET API (we avoid via XMLA) |
| **MCP** | Model Context Protocol — open AI-tool integration protocol |
| **RAG** | Retrieval-Augmented Generation — grounding LLM in data |
| **PEG** | Parsing Expression Grammar — parser formalism |

### B. Reference Documents

- [architecture/01-component-architecture.md](architecture/01-component-architecture.md)
- [architecture/02-data-flow.md](architecture/02-data-flow.md)
- [technical/01-configuration-engine.md](technical/01-configuration-engine.md)
- [technical/02-dax-engine.md](technical/02-dax-engine.md)
- [technical/03-testing-packaging-deployment.md](technical/03-testing-packaging-deployment.md)
- [technical/04-internal-database.md](technical/04-internal-database.md)
- [technical/05-policy-backup-seeding.md](technical/05-policy-backup-seeding.md)
- [business/01-monetization-model.md](business/01-monetization-model.md)
- [business/02-go-to-market.md](business/02-go-to-market.md)

### C. Competitive Landscape

| Competitor | What They Do | trRUSTt Advantage |
|---|---|---|
| **PBIFORGE** | Prompt→.pbix file. Ingests CSV/SQL. Template-driven visuals. Free. | trRUSTt: Live PBI model editing, interactive visual canvas, enterprise RBAC/SSO/audit, 100% configurable, GUI + CLI + MCP, image-to-dashboard, white-label, 6 monetization routes, policy engine, full internal DB |
| **Power BI Copilot** | Basic AI features in PBI Service. Limited DAX. | trRUSTt: Deeper DAX (4 levels + validation + self-correction), image-to-dashboard, MCP integration, works offline, configurable |
| **Tabular Editor** | DAX/TOM scripting. No AI. | trRUSTt: AI-powered generation, full dashboard pipeline, visual canvas, data ingestion |
| **ChatGPT/Claude** | General AI. No PBI context. | trRUSTt: Schema-aware RAG, validates output, applies directly to model, purpose-built for Power BI |
| **Zebra BI** | Custom visuals only. | trRUSTt: Full pipeline — data → model → measures → dashboard, not just visuals |

---

> **Document Version:** 2.0  
> **Product Name:** trRUSTt (branding) / TRRUSTT (code, config, binary)  
> **Architecture:** Rust-Native · Power BI External Tool · Single Binary  
> **Classification:** Internal — Confidential

---

*"trRUSTt your data. One binary. Infinite dashboards."*
