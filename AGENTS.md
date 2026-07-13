# trRUSTt — AI Agent Instructions

> **⚠️ READ THIS FIRST before generating any code for this project.**
> These instructions are binding. Violating them will cause rework.

---

## 1. WHAT WE ARE BUILDING

**trRUSTt** — An AI-driven Power BI External Tool that automates dashboard creation.

- **Architecture:** Rust-native single binary → Power BI External Tool
- **UI:** Tauri + Svelte 5 + shadcn-svelte + Tailwind CSS 4 (primary GUI — includes interactive visual canvas with drag-drop, grid editor, visual palette, real-time preview)
- **CLI:** clap (automation) + ratatui (TUI admin)
- **Database:** SQLite via `sqlx` (internal DB — projects, measures, dashboards, users, orgs, workspaces, RBAC, audit, AI usage, AI cache) + LanceDB (vector embeddings for RAG). Full schema in `docs/technical/04-internal-database.md`.
- **Policy Engine:** Policy-driven architecture. Backup policies, data retention, security, AI usage limits, DAX governance, access control, notifications, compliance — all as configurable policies. See `docs/technical/05-policy-backup-seeding.md`.
- **Backup Strategy:** Scheduled + pre-migration + pre-destructive + manual backups. Zstd-compressed archives, SHA-256 verified, encrypted at rest, 30-day rotation.
- **Seeding:** Default themes (6), prompt library, system config, default roles/permissions, optional onboarding demo data.
- **Auth:** PBI Desktop SSO bridge (auto-detect Entra ID from PBI Desktop session). Optional enterprise SSO: Azure AD, Google Workspace, generic OIDC.
- **AI Providers:** OpenAI, Azure OpenAI, Anthropic, Google Gemini, DeepSeek, Ollama (multi-provider routing, all configurable).
- **Config Prefix:** All env vars use `TRRUSTT_` prefix. Config dir: `~/.trrustt/`. Binary: `TRRUSTT.exe`.
- **No ORM:** `sqlx` gives compile-time checked SQL with zero runtime overhead. Better than Prisma/Diesel/SeaORM for our use case.

### 2.6 ANTI-HARDCODING ENFORCEMENT — NON-NEGOTIABLE 🚨

```
🚨 RULE #6: ZERO HARD-CODED VALUES. THIS IS THE MOST IMPORTANT RULE.

EVERY SINGLE VALUE that could ever change under any circumstance
MUST come from the config engine. NO EXCEPTIONS.

❌ RECENT VIOLATIONS FOUND (DO NOT REPEAT):
   crates/ai-engine/src/chains.rs:
     model: Some("gpt-4o".into())       → SHOULD BE: CONFIG.get("ai.default_model")
     temperature: Some(0.3)              → SHOULD BE: CONFIG.get("ai.temperature")
     max_tokens: Some(2048)              → SHOULD BE: CONFIG.get("ai.max_tokens")
     system_prompt: Some("You are...")   → SHOULD BE: prompt engine via config key

   categories of values that MUST be config:
   • AI: model name, temperature, max_tokens, top_p, seed, all provider settings
   • AI chain-specific: each chain step can have its own temp/tokens/model
   • DAX: complexity level, naming convention, allowed functions, validation rules
   • Dashboard: grid density, page size, default colors, visual padding
   • UI: theme, language, font sizes, layout preferences
   • Paths: config dir, log dir, data dir, prompt dir, theme dir
   • Limits: rate limits, timeout values, retry counts, cache sizes
   • License: grace period, phone-home interval, feature flags
   • Security: session timeout, password rules, encryption settings
   • ALL numeric constants, ALL string identifiers, ALL thresholds

PRE-FLIGHT CHECK (FIRST THING EVERY CHAT MUST DO):
   1. Scan the codebase for any hard-coded values:
      - grep for quoted strings that look like config values ("gpt-4o", "0.3", etc.)
      - grep for numeric literals used as thresholds/limits
      - grep for hard-coded paths or URLs
   2. For EVERY found value: create the config entry if it doesn't exist
   3. Replace the hard-coded value with CONFIG.get("path.to.config")
   4. Add the config entry to the config registry with description + validation
   5. Document in the appropriate config TOML file

   Example fix:
   ❌ let model = "gpt-4o";
   ✅ let model: String = CONFIG.get("ai.default_model")?;
   
   ❌ if retries > 3 { ... }
   ✅ let max_retries: u32 = CONFIG.get("ai.max_retries")?;
      if retries > max_retries { ... }

   ❌ let path = "~/.trrustt/prompts/";
   ✅ let path: String = CONFIG.get("paths.prompts_dir")?;

VIOLATION CONSEQUENCES:
   Any code containing hard-coded values is considered BROKEN.
   Do not proceed. Do not move to the next crate. Fix it immediately.
   This is the #1 quality gate. Non-negotiable.
```
- **Data Ingestion:** Ingest CSV, SQL, Parquet, JSON → auto-profile via Polars → infer model → build Tabular model via TMSL. Complete data→dashboard pipeline.
- **MCP Servers (Microsoft, pre-configured):**
  - **Power BI Modeling MCP Server (local):** Microsoft's official MCP server. Connects to PBI Desktop / Fabric / PBIP files. 20+ tool categories for semantic model manipulation. Installed alongside TRRUSTT as the primary PBI bridge.
  - **Power BI MCP Server (remote):** Microsoft's MCP server for Power BI Service. Enterprise auth (Entra ID, service principals), workspace/dataset/report operations.
  - Our `mcp-hub` crate auto-connects to both as defaults. Our value-add is the AI layer, GUI, config, data ingestion, visual canvas, and enterprise features on top of Microsoft's infrastructure.
- **Product name:** trRUSTt (branding/display — only RUST in caps) / TRRUSTT (code, config, env vars, binary)
- **Code name:** Athena
- **Tagline:** "trRUSTt your data. One binary. Infinite dashboards."
- **Developer:** Individual (sole proprietor), no company entity yet
- **License model:** Open-source core (MIT/Apache 2.0) + paid Pro/Enterprise features
- **Code name:** Athena
- **Tagline:** "TRRUSTT your data. One binary. Infinite dashboards."
- **Developer:** Individual (sole proprietor), no company entity yet
- **License model:** Open-source core (MIT/Apache 2.0) + paid Pro/Enterprise features

---

## 2. ABSOLUTE RULES — NEVER VIOLATE THESE

### 2.1 Configuration-First Mandate
```
🚨 RULE #1: NOTHING IS EVER HARD-CODED. EVER.

❌ FORBIDDEN:
   let model = "gpt-4o";                    // Hard-coded string
   let port = 54321;                         // Hard-coded port
   const MAX_RETRIES = 3;                    // Hard-coded constant
   let path = "~/.trrustt/config";           // Hard-coded path

✅ REQUIRED:
   let model = CONFIG.get::<String>("ai.default_model")?;
   let port = CONFIG.get::<u16>("xmla.port")?;
   let retries = CONFIG.get::<u32>("ai.max_retries")?;
   let path = CONFIG.get::<String>("paths.config_dir")?;

Every value goes through the config engine (crates/config-engine).
Config is resolved via 7-layer priority system.
Defaults are embedded at compile time (build.rs).
Users override via TOML files, env vars, or CLI args.
```

### 2.2 Crate Boundaries
```
🚨 RULE #2: Every Rust crate is an independent, self-contained unit.

Each crate MUST have:
  - Its own Cargo.toml (independent version, deps)
  - Its own src/lib.rs (public API)
  - Its own tests/ (independent test suite)
  - Its own README.md (what it does, how to use)
  - Zero implicit dependencies on sibling crates

Crates communicate ONLY through public traits and types defined in `shared`.

❌ FORBIDDEN:
   // In dax-engine, directly importing from ai-engine's internal module
   use ai_engine::router::internal::Something;  // NO!

✅ REQUIRED:
   // Each crate exposes a clean public API in lib.rs
   // Other crates use only the public re-exports
   use ai_engine::AiEngine;          // ✅ Public type
   use ai_engine::ChatRequest;       // ✅ Public type
```

### 2.3 Safety
```
🚨 RULE #3: #![forbid(unsafe_code)] in every crate.

No exceptions. No unsafe blocks. No FFI.
If you think you need unsafe, you're wrong — find another way.
The only exception: candle (local LLM) behind a feature flag, opt-in only.
```

### 2.4 Error Handling
```
🚨 RULE #4: Always use Result<T, AppError>. Never panic. Never unwrap().

❌ FORBIDDEN:
   let x = something().unwrap();      // Will crash
   something().expect("oh no");        // Will crash
   panic!("something went wrong");    // Will crash

✅ REQUIRED:
   let x = something()?;                          // Propagates error
   let x = something().context("while doing X")?; // Add context
   // Or handle explicitly:
   match something() {
       Ok(x) => { /* use x */ },
       Err(e) => {
           tracing::error!(error = %e, "Failed to do something");
           return Err(AppError::from(e));
       }
   }

All errors are logged with tracing spans and correlation IDs.
Error messages are i18n-ready (use fluent-rs keys where possible).
```

### 2.5 Async Runtime
```
🚨 RULE #5: All I/O is async. Use tokio. Never block the runtime.

❌ FORBIDDEN:
   std::fs::read_to_string("file.txt")     // Blocking I/O
   reqwest::blocking::get(url)              // Blocking HTTP
   std::thread::sleep(duration)             // Blocking sleep

✅ REQUIRED:
   tokio::fs::read_to_string("file.txt").await
   reqwest::get(url).await
   tokio::time::sleep(duration).await

CPU-intensive work (Polars, embeddings) goes through tokio::task::spawn_blocking.
```

---

## 3. PROJECT STRUCTURE

```
trrustt/
├── Cargo.toml                    # Workspace root
├── Cargo.lock                    # Committed (binary project)
├── rust-toolchain.toml           # Pin Rust version (stable)
├── .github/workflows/            # CI/CD pipelines
│   ├── ci.yml                    # PR: lint, test, audit, coverage
│   └── release.yml               # Tag: build, sign, publish
├── crates/
│   ├── trrustt/                  # 🔷 Binary crate — entry point
│   │   └── src/
│   │       ├── main.rs           # Mode dispatch (cli/tui/mcp/external-tool)
│   │       ├── cli/              # clap argument definitions
│   │       ├── tui/              # ratatui admin interface
│   │       └── app.rs            # Dependency injection, app state
│   ├── xmla-client/              # 📡 XMLA/TMSL client for SSAS
│   ├── dax-engine/               # 📐 DAX parser, validator, generator, corrector
│   ├── ai-engine/                # 🤖 LLM router, RAG pipeline, vision
│   ├── config-engine/            # ⚙️ 7-layer config, encryption, admin policies
│   ├── dashboard-generator/      # 📊 Layout AI, visual TMSL, themes
│   ├── mcp-hub/                  # 🔌 MCP server (stdio) + client
│   ├── data-store/               # 🗄️ SQLite + LanceDB internal database  │   ├── data-ingestion/            # 📥 CSV, SQL, Parquet → Tabular model│   ├── license-manager/          # 🔑 JWT validation, offline grace
│   └── shared/                   # 📦 Common types, errors, telemetry
├── prompts/                      # Externalized AI prompt library
├── themes/                       # Default theme JSON files
├── config/                       # Default config TOML files
├── tests/                        # Integration tests
├── benches/                      # Criterion benchmarks
└── docs/                         # Comprehensive documentation
```

---

## 4. CODE STYLE & CONVENTIONS

### 4.1 Naming
```rust
// Crates: kebab-case (xmla-client, dax-engine)
// Types: PascalCase (XmlaClient, DaxValidator, SchemaMetadata)
// Functions: snake_case (discover_schema, generate_measures)
// Constants: SCREAMING_SNAKE_CASE (MAX_RETRIES, DEFAULT_PORT)
// Config keys: snake_case with dots (ai.default_model, dax.complexity_level)
// Files: snake_case (xmla_client.rs, dax_validator.rs)
```

### 4.2 Documentation
```rust
/// Every public type and function MUST have a doc comment.
/// 
/// # Arguments
/// * `port` - The SSAS instance port number
/// * `database` - Optional database name (auto-detected if None)
///
/// # Returns
/// Schema metadata including tables, columns, measures, and relationships
///
/// # Errors
/// Returns `XmlaError::ConnectionFailed` if SSAS is unreachable
/// Returns `XmlaError::DiscoveryFailed` if DISCOVER_XMLA fails
///
/// # Example
/// ```rust,no_run
/// let client = XmlaClient::connect(54321).await?;
/// let schema = client.discover_schema().await?;
/// println!("Found {} tables", schema.tables.len());
/// ```
pub async fn discover_schema(&self) -> Result<SchemaMetadata, XmlaError>
```

### 4.3 Imports
```rust
// Order: std → external crates → workspace crates → crate local
use std::path::PathBuf;
use std::collections::HashMap;

use reqwest::Client;
use serde::{Serialize, Deserialize};
use tracing::{info, error, instrument};

use shared::types::SchemaMetadata;
use config_engine::ConfigEngine;

use crate::client::XmlaClient;
use crate::error::XmlaError;
```

### 4.4 Tracing
```rust
// Every significant function gets an instrument attribute
#[instrument(skip(self, schema), fields(table_count = schema.tables.len()))]
pub async fn generate_measures(
    &self,
    description: &str,
    schema: &SchemaContext,
) -> Result<Vec<GeneratedMeasure>> {
    info!(description = %description, "Generating DAX measures");
    // ...
    info!(count = measures.len(), "DAX generation complete");
    Ok(measures)
}

// Use structured fields, not string formatting
info!(user_id = %id, action = "dashboard_create", duration_ms = %ms);
// NOT: info!("User {} created dashboard in {}ms", id, ms);
```

---

## 5. BUILDING & TESTING

### 5.0 Environment Setup — READ THIS FIRST

```
🚨 WINDOWS LINKER ISSUE — ALREADY SOLVED:

This project uses rust-lld (Rust's built-in LLD linker) instead of the MSVC linker.
Config is in .cargo/config.toml. NO MSVC Build Tools needed. NO admin rights needed.

BEFORE ANY BUILD COMMAND:
  export PATH="$HOME/.cargo/bin:$PATH"

Then proceed normally:
  cargo build
  cargo test
  etc.

The project's .cargo/config.toml already has:
  [target.x86_64-pc-windows-msvc]
  linker = "rust-lld"

DO NOT change this. DO NOT try to install VS Build Tools or switch to GNU.
rust-lld IS the linker. Period.
```

### 5.1 Development Commands
```bash
# Build (debug)
cargo build

# Build (release — what users get)
cargo build --profile dist

# Run tests
cargo test --workspace

# Run specific crate tests
cargo test -p dax-engine

# Run with output
cargo test -- --nocapture

# Lint (strict — warnings are errors in CI)
cargo clippy --workspace --all-features -- -D warnings

# Format
cargo fmt --all
cargo fmt --all -- --check   # CI check

# Security audit
cargo audit
cargo deny check

# Coverage
cargo llvm-cov --workspace --lcov --output-path coverage.lcov

# Benchmarks
cargo bench

# Documentation
cargo doc --workspace --no-deps --open
```

### 5.2 Before Every Commit
```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-features -- -D warnings
cargo test --workspace
cargo audit
```

---

## 6. DEPLOYMENT & PACKAGING — BUILT-IN FROM DAY ONE

### 6.1 Release Profile (Cargo.toml)
```toml
[profile.dist]
inherits = "release"
lto = "fat"              # Maximum link-time optimization
codegen-units = 1        # Single codegen unit = better opts
panic = "abort"          # Smaller binary
strip = true             # Remove debug symbols
opt-level = 3            # Maximum optimization
```

### 6.2 Cross-Compilation Targets
```bash
# Windows (primary)
cargo build --profile dist --target x86_64-pc-windows-msvc

# macOS Intel
cargo build --profile dist --target x86_64-apple-darwin

# macOS Apple Silicon
cargo build --profile dist --target aarch64-apple-darwin

# Linux static MUSL
cargo build --profile dist --target x86_64-unknown-linux-musl
```

### 6.3 Binary Size Budget
```
Target: 25-35 MB (release, stripped, includes Tauri web assets)
Budget:
  trrustt binary (Rust):    15-20 MB
  Frontend (Svelte):          2-5 MB
  ─────────────────────────────
  TOTAL:                     25-35 MB ✅

Monitor with: cargo bloat --release --crates
If > 35 MB, investigate and reduce before merging.
```

### 6.4 Distribution
```
Primary:   GitHub Releases (IntelliDashboard.exe → rename to TRRUSTT.exe)
Package mgrs: winget, scoop, choco, brew, cargo install
Update:    self_update crate (checks GitHub Releases)
Signing:   EV Code Signing certificate (Windows), Apple notarization (macOS)
```

---

## 7. BRANDING

- **Product Name:** trRUSTt (branding/display — only RUST in caps, rest lowercase)
- **Code/Config:** TRRUSTT (all caps for env vars, binary names, config keys)
- **Code Name:** Athena (internal only)
- **Primary Color:** #2563EB (blue-600)
- **Accent Color:** #7C3AED (violet-600)
- **Logo:** TBD (need design — must incorporate the trRUSTt styling)
- **Binary name:** `TRRUSTT.exe` (code) / trRUSTt (display in UI)
- **Config dir:** `~/.trrustt/`
- **GitHub org/repo:** `trrustt/builder` (or personal repo for now)

**Styling rules:**
- In UI, splash screen, about dialog, documentation headers → **trRUSTt**
- In code, env vars, CLI commands, file names → TRRUSTT / TRRUSTT.exe / TRRUSTT_AI_PROVIDER
- Never use "IntelliDashboard" — that was the old name.

---

## 8. WHAT YOU MUST NEVER DO

```
❌ NEVER hard-code any value — use CONFIG.get() always
❌ NEVER use unsafe Rust — #![forbid(unsafe_code)]
❌ NEVER call unwrap() or expect() in production code
❌ NEVER do blocking I/O on the async runtime — use tokio::fs, not std::fs
❌ NEVER import from a sibling crate's internal modules — use public API only
❌ NEVER skip writing tests — every public function needs tests
❌ NEVER skip doc comments — every public type/function needs /// docs
❌ NEVER commit without: fmt, clippy, test, audit
❌ NEVER use the old name "IntelliDashboard" — it's TRRUSTT now
❌ NEVER add a dependency without checking: is it maintained? safe? necessary?
❌ NEVER assume PBI Desktop port — always read from --port CLI arg or config
❌ NEVER assume the user has any specific AI provider — support all configured providers
🚨 NEVER create mock/stub/todo/skeleton implementations — every function must have FULL, REAL, WORKING implementation. No matter how large the task. This is NON-NEGOTIABLE. No MockConnection, no FakeClient, no unimplemented!(), no todo!(), no "placeholder for later". Build it completely or don't build it at all.
```

---

## 9. WHAT YOU MUST ALWAYS DO

```
✅ Always use CONFIG.get() for every configurable value
✅ Always add tracing spans to significant functions
✅ Always return Result<T, AppError> — never panic
✅ Always write doc tests for public API examples
✅ Always use the type system to make invalid states unrepresentable
✅ Always consider: "Can this be configured?" If yes → add config entry
✅ Always consider: "Can this crate be used standalone?" If no → fix coupling
✅ Always add a CHANGELOG entry for user-facing changes
✅ Always think about deployment: "Will this compile on Windows? macOS? Linux?"
✅ Always think about the user: "Is the error message clear? Is the CLI intuitive?"
✅ Always update the memory file (/memories/repo/intellidashboard-plan.md) for major decisions
```

---

## 10. KEY ARCHITECTURAL DECISIONS (Already Made — Do Not Revisit)

1. **Rust-native** — No Python, no .NET, no Node.js, no Electron
2. **Power BI External Tool** — Not a standalone app, not a custom visual
3. **Single binary** — Statically linked, zero runtime dependencies
4. **XMLA bridge** — HTTP to SSAS on localhost, not .NET TOM
5. **V1: Live PBI Desktop only** — No offline .pbix mode, no Tabular Editor
6. **Cloud-only AI** — Users bring API keys, no bundled models
7. **Pure Rust JWT license** — Ed25519, AES-256-GCM, offline-capable
8. **Open-source core** — MIT/Apache 2.0, paid Pro/Enterprise modules
9. **GUI-first** — Tauri + Svelte 5 + shadcn-svelte + Tailwind CSS 4 in V1. Beautiful polished UI from Day 1.
10. **7-layer config** — Figment, TOML files, env vars, admin policies

---

## 11. CURRENT STATE

- [x] Product vision & scope (25-section master plan)
- [x] Architecture (9 crates, XMLA bridge, data flows)
- [x] Configuration engine design (7-layer, figment)
- [x] DAX engine design (PEG parser, 7-step validation)
- [x] AI engine design (multi-provider, RAG, vision)
- [x] Monetization model (6 routes, pricing)
- [x] Go-to-market strategy
- [x] Testing strategy (proptest, criterion, cargo-audit)
- [x] Packaging & deployment design (single binary, cross-compile)
- [x] CI/CD pipeline design (GitHub Actions)
- [x] 14 open questions answered and locked
- [ ] Cargo workspace initialization
- [ ] First crate: `shared` (types, errors, telemetry)
- [ ] Second crate: `config-engine`
- [ ] Third crate: `xmla-client`
- [ ] ... continue through all 9 crates
- [ ] CI/CD pipeline implementation
- [ ] First binary: `trrustt` CLI mode
- [ ] External Tool registration
- [ ] Alpha release

---

> **These instructions are binding. Generated code must comply.**
> **Last updated:** 2026-07-12
> **File location:** `AGENTS.md` in project root
