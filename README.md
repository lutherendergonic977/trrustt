# <img src="branding/icon.svg" width="32" height="32" alt="trRUSTt logo"> trRUSTt [V1 — In Development]

> **"trRUSTt your data. One binary. Infinite dashboards."**

**AI-powered Power BI External Tool.** Beautiful GUI. Single 30MB binary. Zero dependencies. Drop it in your External Tools folder. 10× faster dashboards.

---

## 🎯 The Problem

Most BI work is **mechanical configuration, not insight.** Dragging visuals, formatting axes, writing boilerplate DAX, recreating the same KPI cards — this burns 2–4 hours per dashboard before analysis even starts. DAX is hard. Consistency is harder. Analysts spend more time configuring than thinking.

**trRUSTt handles the repetitive layer so analysts stay in the judgment seat — not the configuration one.**

---

## 💡 The Solution

| Before trRUSTt | With trRUSTt |
|---|---|
| 2–4 hours per dashboard | 3–5 minutes |
| Manual DAX writing + debugging | AI-generated, validated, self-correcting |
| Inconsistent formatting | Config-driven themes, applied automatically |
| BI expertise required | Describe what you want in plain English |
| No audit trail, no RBAC | Full enterprise: users, orgs, workspaces, SSO |
| Single tool, locked in | MCP-native — connect to any AI tool |

**10× faster. AI-powered. 100% configurable. Enterprise-ready.**

---

## 🌍 Who Is This For?

**trRUSTt is for anyone who uses Microsoft Power BI.** Any industry. Any country. Any role.

| Persona | Why trRUSTt |
|---|---|
| **BI Analysts** | Generate first-draft dashboards in seconds. Spend time improving the story, not configuring visuals. |
| **BI Consultants** | Deliver consistent, professional dashboards across multiple clients. 10× faster = more projects. |
| **Data Team Leads** | Standardize quality. Config-driven DAX governance. AI upskills junior analysts. |
| **Enterprise IT / Admins** | Full RBAC, SSO, audit logging, whitelisting. Policy-driven control. Deploy via SCCM/Intune. |
| **Business Teams** | Describe the dashboard you need. No DAX syntax. No visual configuration. Just business questions. |
| **OEM / SaaS Companies** | White-label trRUSTt. Embed AI-powered analytics in your product. Rebrand and resell. |

---

## 🚀 Quick Start

```bash
# 1. Download TRRUSTT.exe from GitHub Releases (~30 MB)

# 2. Copy to External Tools folder
copy TRRUSTT.exe "%APPDATA%\Microsoft\Power BI Desktop\External Tools\"

# 3. Register (create one JSON file)
#    %APPDATA%\Microsoft\Power BI Desktop\External Tools\TRRUSTT.json
{
  "name": "🔷 trRUSTt",
  "path": "TRRUSTT.exe",
  "arguments": "--port %port% --pbix \"%pbix%\""
}

# 4. Open Power BI Desktop → External Tools → 🔷 trRUSTt
#    Beautiful GUI opens pre-connected to your PBI model
```

---

## 🗺️ Roadmap

### ✅ V1 (In Development — 2026)

| Category | Capabilities |
|---|---|
| **PBI Integration** | External Tool, live XMLA to PBI Desktop SSAS, full metadata editing (measures, columns, tables, relationships, hierarchies, calc groups) |
| **GUI** | Tauri + Svelte 5 + shadcn/ui, Dashboard Wizard, Interactive Visual Canvas (drag-drop), Theme Designer, Settings Panel, Schema Explorer, DAX Playground |
| **DAX Engine** | 4 complexity levels (Beginner → Expert), PEG parser, 7-step validation, self-correction, natural language explanation |
| **AI** | 6 providers (OpenAI, Azure, Anthropic, Gemini, DeepSeek, Ollama), RAG (LanceDB), Vision (image-to-dashboard), prompt chains |
| **Data** | Data ingestion (CSV, SQL, Parquet, JSON → Polars profiler → TMSL model), schema profiling |
| **MCP** | Microsoft Power BI Modeling MCP Server (local) + Power BI MCP Server (remote) pre-configured, trRUSTt exposed as MCP server |
| **Enterprise** | RBAC (5 roles), orgs + workspaces, SQLite internal DB (15 tables), audit logging, AI cost tracking, AI cache |
| **Config** | 100% configurable, 7-layer resolution (TRRUSTT_ env → CLI → user → project → workspace → admin → defaults), 8 policy categories |
| **License** | Pure Rust JWT (Ed25519), AES-256-GCM encrypted cache, 30-day offline grace, periodic phone-home |
| **Branding** | trRUSTt (only RUST in caps for display), TRRUSTT (all caps for code/config), white-label ready |
| **Platform** | Windows (Power BI Desktop is Windows-only) |

### 🔷 Phase 2 (2027)

| Category | Planned Features |
|---|---|
| **Cross-Platform** | macOS + Linux (CLI + MCP server + Docker) |
| **Offline Mode** | .pbix file manipulation via Tabular Editor 2 CLI (works without PBI Desktop running) |
| **PBI Service** | Direct Power BI Service publishing, Fabric integration, automated deployment pipelines |
| **Collaboration** | Multi-user workspaces, shared prompts/themes, team config sync via git |
| **Marketplace** | Built-in theme marketplace, DAX recipe packs, industry templates (Finance, Healthcare, Retail) |
| **Advanced AI** | Local LLM inference (candle + CUDA), fine-tuned DAX models, automated insight generation |
| **Mobile Companion** | iOS/Android app (view dashboards, receive alerts, approve changes) |
| **Enterprise Console** | Web-based admin dashboard for fleet management across org |

---

## 🎯 Core Capabilities (V1)

| # | Capability |
|---|---|
| 🎨 | **Beautiful GUI** — Dashboard Wizard, Drag-Drop Visual Canvas, Theme Designer, DAX Playground, Settings Panel, Schema Explorer, Prompt Editor |
| 🤖 | **AI-Powered DAX** — 4 complexity levels (Beginner/Intermediate/Advanced/Expert), 7-step validation, self-correction, natural language explanation |
| 📊 | **Dashboard Auto-Generation** — AI plans layout, selects visuals, binds measures, applies theme — all pushed live to PBI Desktop |
| 🖼️ | **Image-to-Dashboard** — Upload sketch/infographic → AI extracts layout + colors → generates matching dashboard |
| 🔌 | **MCP Server/Client** — Microsoft MCP servers pre-configured as defaults. Expose trRUSTt tools to any AI assistant. Connect external MCP servers. |
| 🔧 | **100% Configurable** — Nothing hard-coded. 7-layer config resolution. 8 policy categories (backup, retention, security, AI usage, DAX, access, notification, compliance). Admin-enforceable. |
| 📐 | **Full Metadata Editing** — Push measures, columns, tables, relationships, hierarchies, calculation groups, display folders, format strings |
| 📥 | **Data Ingestion** — CSV, SQL, Parquet, JSON → auto-profile via Polars → infer relationships → build Tabular model → generate dashboard |
| 🏷️ | **White-Label Ready** — Rebrand everything: name, logo, colors, URLs. OEM partners can resell as their own product. |
| 💰 | **6 Monetization Routes** — Free tier, Pro ($149/yr), Team ($49/user/mo), Enterprise, OEM white-label, API/PAYG |

---

## 🛠️ CLI Usage

```bash
# GUI mode (double-click TRRUSTT.exe — primary mode)
TRRUSTT.exe

# Schema discovery (CLI)
TRRUSTT.exe schema discover --port 54321

# Generate DAX (CLI)
TRRUSTT.exe dax generate --description "YoY growth" --complexity advanced --port 54321

# Create dashboard (CLI)
TRRUSTT.exe dashboard create --intent "Executive KPI dashboard" --port 54321

# MCP server mode (for Claude Desktop, Cursor, Continue)
TRRUSTT.exe mcp serve

# TUI admin (quick terminal config)
TRRUSTT.exe admin
```

---

## 🏗️ Architecture

```
         TRRUSTT (Single Binary ~30 MB, zero runtime deps)
                              │
        ┌─────────────────────┼─────────────────────┐
        │                     │                     │
   🎨 Tauri Shell        🦀 Rust Core          🌐 AI Engine
   (Svelte 5 +           (XMLA, DAX,           (async-openai,
    shadcn/ui)            config, license)       RAG, vision)
                              │
                    Power BI Desktop
                    SSAS Instance (localhost:XXXXX)
```

11 Rust crates. Tauri + Svelte + shadcn/ui GUI. SQLite (sqlx) + LanceDB internal database. 6 AI providers. 15 database tables. 8 policy categories. All in one binary.

---

## 🏗️ Development

### Prerequisites

- **Rust 1.80+** — Install via [rustup.rs](https://rustup.rs)
- **Node.js 22+** + **pnpm** — for Tauri frontend
- **Windows** — Power BI Desktop is Windows-only (cross-platform CLI in Phase 2)

> 💡 This project uses `rust-lld` (Rust's built-in linker) — no MSVC Build Tools needed. See `.cargo/config.toml`.

### Build Commands

```bash
git clone https://github.com/oshjain/trrustt.git && cd trrustt

# Frontend dependencies
cd crates/trrustt/frontend && pnpm install && cd ../../..

# Build
cargo build
cargo build --release              # Production build (~30 MB)

# Dev mode (frontend hot reload)
cargo tauri dev

# Test & Lint
cargo test --workspace
cargo clippy --workspace -- -D warnings
cargo fmt --all -- --check
cargo audit
```

---

## 📦 Crate Structure

```
crates/
├── trrustt/              # 🔷 Binary + Tauri shell + Svelte frontend
├── xmla-client/           # 📡 XMLA/TMSL client (reqwest)
├── dax-engine/            # 📐 DAX parser (pom), validator, generator
├── ai-engine/             # 🤖 LLM router, RAG (LanceDB), vision
├── config-engine/         # ⚙️ 7-layer config (figment), encryption (ring)
├── dashboard-generator/   # 📊 Layout AI, visual TMSL, themes
├── mcp-hub/               # 🔌 MCP server (stdio) + client (Microsoft defaults)
├── data-store/            # 🗄️ SQLite (sqlx) + LanceDB — 15 tables, RBAC, audit
├── data-ingestion/        # 📥 CSV, SQL, Parquet, JSON → Polars → TMSL model
├── license-manager/       # 🔑 Pure Rust JWT (Ed25519), AES-256-GCM
└── shared/                # 📦 Types, errors (thiserror), telemetry
```

---

## 📋 Documentation

| Document | Description |
|---|---|
| [📘 Master Product Plan](docs/00-MASTER-PLAN.md) | Full PRD — 25 sections, competitive analysis |
| [🏗️ Component Architecture](docs/architecture/01-component-architecture.md) | 11 crates, dependency graph, public APIs |
| [📊 Data Flow Architecture](docs/architecture/02-data-flow.md) | XMLA flows, storage layout, SQLite schema |
| [⚙️ Configuration Engine](docs/technical/01-configuration-engine.md) | 7-layer config, figment, encryption, registry |
| [📐 DAX Engine](docs/technical/02-dax-engine.md) | PEG parser, 7-step validation, self-correction |
| [🧪 Testing & Deployment](docs/technical/03-testing-packaging-deployment.md) | CI/CD, cross-compile, single-binary deploy |
| [🗄️ Internal Database](docs/technical/04-internal-database.md) | 15 tables, RBAC, orgs, workspaces, SSO bridge |
| [📋 Policies & Backups](docs/technical/05-policy-backup-seeding.md) | 8 policy categories, backup strategy, seeding |
| [💰 Monetization Model](docs/business/01-monetization-model.md) | 6 routes, pricing tiers, projections |
| [🚀 Go-To-Market](docs/business/02-go-to-market.md) | Positioning, launch plan, partnerships |

---

## 🔒 Safety

- `#![forbid(unsafe_code)]` — zero unsafe Rust
- `cargo-audit` + `cargo-deny` in CI
- AES-256-GCM for sensitive config
- EV Code Signing for Windows binaries

## 📄 License

Open-source core (MIT/Apache 2.0). Pro/Enterprise features are paid.

---

> **V1 In Development | trRUSTt (branding) / TRRUSTT (code/config) | Phase 2: 2027**

*"trRUSTt your data. One binary. Infinite dashboards."*
