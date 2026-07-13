// ═══════════════════════════════════════════════════════════════════════
// TRRUSTT — XMLA Client: Public API
//
// HTTP client for Power BI Desktop's embedded SSAS Tabular instance.
// Connects to localhost:XXXXX and performs:
// - DISCOVER_XMLA: schema metadata discovery
// - EXECUTE: DAX query execution
// - TMSL: Tabular Model Scripting Language commands
// ═══════════════════════════════════════════════════════════════════════

#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![doc = include_str!("../README.md")]

pub mod client;
pub mod discover;
pub mod error;
pub mod execute;
pub mod tmsl;

// ── Re-exports ────────────────────────────────────────────────────────

pub use client::XmlaClient;
pub use error::XmlaError;
