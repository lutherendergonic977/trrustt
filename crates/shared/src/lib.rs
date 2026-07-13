// ═══════════════════════════════════════════════════════════════════════
// TRRUSTT — Shared Crate: Public API
//
// Re-exports all public types from sub-modules.
// This is the ONLY file other crates should import from.
// ═══════════════════════════════════════════════════════════════════════

#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![doc = include_str!("../README.md")]

pub mod constants;
pub mod errors;
pub mod telemetry;
pub mod types;

// ── Convenience re-exports ────────────────────────────────────────────

pub use constants::*;
pub use errors::AppError;
pub use telemetry::init_telemetry;
pub use types::*;
