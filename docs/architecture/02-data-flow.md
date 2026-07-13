# Data Flow Architecture — Rust-Native v2

## 1. Primary Data Flows

### 1.1 Schema Discovery Flow

```
┌──────────────────────────────────────────────────────────────────┐
│                      SCHEMA DISCOVERY                             │
│                                                                   │
│  1. PBI Desktop launches SSAS on localhost:XXXXX                  │
│  2. IntelliDashboard.exe receives --port XXXXX                    │
│  3. XmlaClient::connect(XXXXX)                                    │
│  4. XmlaClient::discover_schema()                                 │
│                                                                   │
│  ┌─────────────┐    XMLA DISCOVER     ┌──────────────────┐       │
│  │XmlaClient   │─────────────────────▶│  SSAS Instance    │       │
│  │(reqwest)    │◀─────────────────────│  localhost:XXXXX  │       │
│  └──────┬──────┘    Schema JSON       └──────────────────┘       │
│         │                                                         │
│         │ SchemaMetadata { tables, columns, measures, rels }     │
│         ▼                                                         │
│  ┌─────────────┐    Embed each chunk  ┌──────────────────┐       │
│  │AiEngine     │─────────────────────▶│  LanceDB         │       │
│  │(RAG index)  │◀─────────────────────│  (vector store)  │       │
│  └─────────────┘    Store vectors     └──────────────────┘       │
│         │                                                         │
│         ▼                                                         │
│  Schema ready for AI queries                                      │
└──────────────────────────────────────────────────────────────────┘
```

### 1.2 Dashboard Generation Flow

```
User (PBI Desktop)          IntelliDashboard              SSAS (localhost)
      │                           │                            │
      │  1. Click ribbon button   │                            │
      │──────────────────────────▶│                            │
      │                           │  2. DISCOVER schema        │
      │                           │───────────────────────────▶│
      │                           │◀───────────────────────────│
      │                           │                            │
      │                           │  3. Build RAG index        │
      │                           │  (LanceDB embeddings)      │
      │                           │                            │
      │  4. "Create exec dashboard│                            │
      │     with KPIs and trends" │                            │
      │──────────────────────────▶│                            │
      │                           │                            │
      │                           │  5. RAG search             │
      │                           │  "KPIs, trends, executive" │
      │                           │  ──▶ Relevant tables found │
      │                           │                            │
      │                           │  6. AI plans dashboard     │
      │                           │  LLM generates layout plan │
      │                           │                            │
      │                           │  7. AI generates DAX       │
      │                           │  (per complexity config)   │
      │                           │                            │
      │                           │  8. PEG parse + validate   │
      │                           │  All references check out  │
      │                           │                            │
      │                           │  9. TMSL createOrReplace   │
      │                           │───────────────────────────▶│
      │                           │◀────── success ───────────│
      │                           │                            │
      │                           │  10. TMSL create visuals   │
      │                           │───────────────────────────▶│
      │                           │◀────── success ───────────│
      │                           │                            │
      │  11. "Done! 6 measures,   │                            │
      │       8 visuals created"  │                            │
      │◀──────────────────────────│                            │
      │                           │                            │
      │  12. Refresh PBI Desktop  │                            │
      │  Sees all changes live    │                            │
      │                           │                            │
      │  13. File → Publish       │                            │
      │  (PBI Desktop native)     │                            │
      │───────────────────────────────────────────────────────▶│
      │                           │           Power BI Service │
```

### 1.3 Image-to-Dashboard Flow

```
User                IntelliDashboard          AI Provider (GPT-4V)
 │                        │                          │
 │  1. Upload sketch.png  │                          │
 │───────────────────────▶│                          │
 │                        │  2. Read image bytes     │
 │                        │  3. Base64 encode        │
 │                        │                          │
 │                        │  4. Send to vision model │
 │                        │─────────────────────────▶│
 │                        │                          │
 │                        │  5. Extract:             │
 │                        │  • Layout grid           │
 │                        │  • Visual types          │
 │                        │  • Color palette         │
 │                        │  • Text labels           │
 │                        │◀─────────────────────────│
 │                        │                          │
 │  6. Show extracted     │                          │
 │     layout preview     │                          │
 │◀───────────────────────│                          │
 │                        │                          │
 │  7. Confirm / adjust   │                          │
 │───────────────────────▶│                          │
 │                        │                          │
 │                        │  8. Match labels to      │
 │                        │     schema fields        │
 │                        │     (RAG search)         │
 │                        │                          │
 │                        │  9. Generate DAX for     │
 │                        │     each visual slot     │
 │                        │                          │
 │                        │  10. Apply visuals       │
 │                        │      to SSAS via TMSL    │
 │                        │                          │
 │  11. Dashboard created │                          │
 │      matching sketch   │                          │
 │◀───────────────────────│                          │
```

### 1.4 MCP Server Flow

```
Claude Desktop / Cursor        IntelliDashboard (MCP Server)
        │                              │
        │  1. Launch MCP server        │
        │  IntelliDashboard.exe        │
        │  mcp serve                   │
        │─────────────────────────────▶│
        │                              │
        │  2. initialize               │
        │─────────────────────────────▶│
        │◀────── tools list ──────────│
        │  schema_discover             │
        │  dax_generate                │
        │  dashboard_create            │
        │  data_query                  │
        │  ... (16 tools)              │
        │                              │
        │  3. tools/call               │
        │  "dax_generate"              │
        │  { description: "YoY growth" │
        │    complexity: "advanced" }  │
        │─────────────────────────────▶│
        │                              │
        │                              │  4. Connect to SSAS
        │                              │     Get schema
        │                              │     Generate DAX via AI
        │                              │     Validate
        │                              │     Apply via TMSL
        │                              │
        │◀────── result ──────────────│
        │  { measure_name: "YoY ...",  │
        │    expression: "VAR ...",    │
        │    validation: "valid" }     │
        │                              │
```

### 1.5 License Activation Flow

```
User                IntelliDashboard         License Server
 │                        │                       │
 │  1. Enter license key  │                       │
 │───────────────────────▶│                       │
 │                        │  2. Validate JWT      │
 │                        │──────────────────────▶│
 │                        │                       │
 │                        │  3. Return:           │
 │                        │  • Tier (Pro/Team/Ent)│
 │                        │  • Feature flags      │
 │                        │  • Expiry date        │
 │                        │  • Seat count         │
 │                        │◀──────────────────────│
 │                        │                       │
 │                        │  4. Encrypt + cache   │
 │                        │  (AES-256-GCM)        │
 │                        │                       │
 │  5. License activated  │                       │
 │◀───────────────────────│                       │
 │                        │                       │
 │  ── Periodic check ──  │                       │
 │                        │  6. Phone home        │
 │                        │  (configurable        │
 │                        │   interval)           │
 │                        │──────────────────────▶│
 │                        │◀── still valid ───────│
 │                        │                       │
 │  ── Offline mode ──    │                       │
 │                        │  7. Use cached token  │
 │                        │  (30-day grace period)│
```

## 2. XMLA Protocol Details

### 2.1 Connection Discovery

```rust
// PBI Desktop starts SSAS on a random port.
// We discover it via the --port argument.

// IntelliDashboard.exe --port 54321

let client = XmlaClient::connect(54321).await?;
// Internally: http://localhost:54321/xmla
```

### 2.2 DISCOVER Request (Schema Metadata)

```xml
<!-- POST http://localhost:54321/xmla -->
<Envelope xmlns="http://schemas.xmlsoap.org/soap/envelope/">
  <Body>
    <Discover xmlns="urn:schemas-microsoft-com:xml-analysis">
      <RequestType>DISCOVER_XMLA</RequestType>
      <Restrictions>
        <RestrictionList>
          <DATABASE_NAME>AdventureWorks</DATABASE_NAME>
        </RestrictionList>
      </Restrictions>
      <Properties>
        <PropertyList>
          <Content>SchemaData</Content>
        </PropertyList>
      </Properties>
    </Discover>
  </Body>
</Envelope>
```

### 2.3 EXECUTE Request (DAX Query)

```json
// POST http://localhost:54321/xmla — JSON format
{
  "execute": {
    "command": "EVALUATE SUMMARIZECOLUMNS('Sales'[ProductKey], \"Total\", SUM('Sales'[SalesAmount]))",
    "properties": {}
  }
}
```

### 2.4 TMSL createOrReplace (Create Measure)

```json
{
  "createOrReplace": {
    "object": {
      "database": "AdventureWorks",
      "table": "Sales",
      "measure": "YoY Growth %"
    },
    "measure": {
      "name": "YoY Growth %",
      "expression": "VAR Current = [Total Sales]\nVAR Previous = CALCULATE([Total Sales], SAMEPERIODLASTYEAR('Calendar'[Date]))\nRETURN DIVIDE(Current - Previous, Previous)",
      "formatString": "0.00%",
      "description": "Year-over-Year sales growth percentage"
    }
  }
}
```

## 3. Storage Layout

```
~/.trrustt/
├── config/
│   ├── system.toml              # System-level defaults
│   ├── user.toml                # User preferences
│   ├── workspace.toml           # Multi-project workspace
│   ├── admin-policies.toml      # Admin-enforced settings
│   └── license.dat             # Encrypted license JWT (AES-256-GCM)
├── data/
│   ├── trrustt.db               # SQLite database (see schema below)
│   └── vectors/                 # LanceDB vector store
│       └── schema-embeddings/   # Schema embeddings for RAG
├── prompts/                     # Prompt library (git-tracked)
│   ├── registry.toml
│   ├── system/
│   ├── chains/
│   ├── few-shot/
│   └── custom/
├── themes/                      # Saved theme files
│   ├── modern-dark.json
│   └── custom-brand.json
├── logs/
│   ├── trrustt.log             # Structured JSON logs (rotating)
│   └── audit.log               # Audit trail (separate, configurable)
└── i18n/
    ├── en-US/
    ├── es-ES/
    └── ja-JP/
```

### 3.1 Internal Database (SQLite + sqlx)

**Engine:** SQLite via `sqlx` — compile-time checked SQL, async, pure Rust.
**Why SQLite:** Public domain, single file, zero configuration, ACID compliant, most deployed database in the world.

**Schema:**

```sql
-- Projects: one per .pbix file the user works with
CREATE TABLE projects (
    id TEXT PRIMARY KEY,                 -- UUID
    name TEXT NOT NULL,                  -- Display name
    pbix_path TEXT,                      -- Path to .pbix (NULL if SSAS connection)
    ssas_port INTEGER,                   -- SSAS port (NULL if offline)
    ssas_database TEXT,                  -- Database name from SSAS
    schema_hash TEXT,                    -- SHA-256 of last known schema
    last_schema_snapshot TEXT,           -- JSON: cached schema metadata
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- DAX Measures: generated, validated, applied
CREATE TABLE measures (
    id TEXT PRIMARY KEY,                 -- UUID
    project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    name TEXT NOT NULL,                  -- Measure name
    table_name TEXT NOT NULL,            -- Which table it belongs to
    expression TEXT NOT NULL,            -- DAX expression
    format_string TEXT,                  -- e.g., "0.00%", "#,##0"
    description TEXT,                    -- What it does
    complexity TEXT NOT NULL,            -- beginner|intermediate|advanced|expert
    is_applied INTEGER DEFAULT 0,        -- 1 if pushed to SSAS model
    is_ai_generated INTEGER DEFAULT 0,   -- 1 if created by AI
    validation_status TEXT,              -- valid|warning|error|corrected
    parent_measure_id TEXT,              -- If this was a correction of another
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Dashboards: generated layouts
CREATE TABLE dashboards (
    id TEXT PRIMARY KEY,                 -- UUID
    project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    name TEXT NOT NULL,                  -- Dashboard name
    pages_json TEXT NOT NULL,            -- JSON: all pages, visuals, bindings
    theme_id TEXT,                       -- Which theme was used
    user_intent TEXT,                    -- The original user request
    image_source_path TEXT,              -- If created from image, the source path
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- AI Usage: cost tracking per request
CREATE TABLE ai_usage (
    id TEXT PRIMARY KEY,                 -- UUID
    project_id TEXT REFERENCES projects(id),
    provider TEXT NOT NULL,              -- openai|azure-openai|anthropic|ollama
    model TEXT NOT NULL,                 -- gpt-4o|claude-3.5-sonnet|etc.
    operation TEXT NOT NULL,             -- dax_generate|dax_validate|dashboard_plan|vision|chat
    prompt_tokens INTEGER NOT NULL,
    completion_tokens INTEGER NOT NULL,
    estimated_cost_usd REAL NOT NULL,
    latency_ms INTEGER NOT NULL,
    correlation_id TEXT NOT NULL,        -- Links to tracing spans
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Audit Log: who did what when
CREATE TABLE audit_log (
    id TEXT PRIMARY KEY,                 -- UUID
    timestamp TEXT NOT NULL DEFAULT (datetime('now')),
    action TEXT NOT NULL,                -- dax.create|dash.create|config.update|license.activate
    resource_type TEXT NOT NULL,         -- measure|dashboard|config|license|theme
    resource_id TEXT,                    -- UUID of affected resource
    details TEXT,                        -- JSON: what changed
    correlation_id TEXT NOT NULL         -- Links to tracing spans
);

-- AI Response Cache: avoid redundant expensive API calls
CREATE TABLE ai_cache (
    cache_key TEXT PRIMARY KEY,          -- SHA-256 of (provider + model + prompt)
    provider TEXT NOT NULL,
    model TEXT NOT NULL,
    prompt_hash TEXT NOT NULL,
    response_json TEXT NOT NULL,         -- Cached LLM response
    token_count INTEGER NOT NULL,
    cost_usd REAL NOT NULL,
    hit_count INTEGER DEFAULT 1,         -- How many times served from cache
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    expires_at TEXT NOT NULL             -- TTL for cache entry
);

-- Indexes for performance
CREATE INDEX idx_measures_project ON measures(project_id);
CREATE INDEX idx_dashboards_project ON dashboards(project_id);
CREATE INDEX idx_ai_usage_project ON ai_usage(project_id);
CREATE INDEX idx_ai_usage_created ON ai_usage(created_at);
CREATE INDEX idx_audit_timestamp ON audit_log(timestamp);
CREATE INDEX idx_ai_cache_lookup ON ai_cache(cache_key, expires_at);
```

### 3.2 Vector Store (LanceDB)

**Engine:** LanceDB — embedded, Rust-native, no server needed.
**Purpose:** Semantic search over schema metadata for RAG (Retrieval-Augmented Generation).
**Data stored:** Schema chunks (table descriptions, column names, measure definitions, relationships) embedded as vectors.

```rust
// Schema for LanceDB
// Collection: schema_chunks
// Each row:
{
    "id": "uuid",
    "project_id": "uuid",
    "chunk_type": "table" | "column" | "measure" | "relationship",
    "content": "Table 'Sales' contains sales transactions with columns: ...",
    "embedding": [0.023, -0.451, ...],  // 1536-dim (text-embedding-3-small) or 384-dim (all-MiniLM)
    "metadata": {
        "table_name": "Sales",
        "column_name": null,
        "measure_name": null,
    }
}
```

### 3.3 Tech Stack Summary

| Data Type | Engine | Crate | Reason |
|---|---|---|---|
| Projects, measures, dashboards, audit | SQLite | `sqlx` | Compile-time SQL checks, async, no ORM overhead |
| AI usage & cost tracking | SQLite | `sqlx` | Just another table, same DB |
| AI response cache | SQLite | `sqlx` | Simple key-value via SQL, TTL via `expires_at` |
| Schema embeddings (RAG) | LanceDB | `lancedb` | Rust-native vector DB, embedded, no server |
| Config | Files (TOML) | `figment` | Better than DB for config — human-editable, git-trackable |
| License | File (encrypted) | `ring` crypto | Single encrypted blob, no DB needed |
| Prompts, themes | Files (text/JSON) | std `fs` + `git2` | Git-tracked, human-editable |

## 4. Error Flow

```
┌──────────────────────────────────────────────────────┐
│                 ERROR HANDLING STRATEGY                │
│                                                       │
│  Any operation returns Result<T, AppError>            │
│                                                       │
│  AppError variants:                                   │
│  ┌─────────────────────────────────────────────────┐ │
│  │ XmlaError        — SSAS connection/query failure │ │
│  │ DaxParseError    — Invalid DAX syntax            │ │
│  │ DaxValidation    — Semantic/schema errors        │ │
│  │ AiError          — LLM provider failures         │ │
│  │ ConfigError      — Invalid/missing config        │ │
│  │ LicenseError     — Invalid/expired license       │ │
│  │ McpError         — MCP protocol errors           │ │
│  │ IoError          — File system errors            │ │
│  │ AuthError        — Authentication failures       │ │
│  └─────────────────────────────────────────────────┘ │
│                                                       │
│  Each error:                                          │
│  • Has a correlation ID (UUID)                        │
│  • Is logged with tracing span context                │
│  • Has a user-friendly message (i18n-ready)           │
│  • Has a detailed debug message (for logs)            │
│  • Never contains secrets or PII                      │
│                                                       │
│  AI errors additionally:                              │
│  • Retry with exponential backoff (configurable)      │
│  • Fall back to alternative provider (configurable)   │
│  • Circuit-break after N failures                     │
└──────────────────────────────────────────────────────┘
```

---

> **Document Version:** 2.0  
> **Part of:** IntelliDashboard Builder Architecture Docs (Rust-Native)
