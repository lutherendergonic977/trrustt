// ═══════════════════════════════════════════════════════════════════════
// TRRUSTT — Configuration Engine: Public API
//
// Central configuration system. Every configurable value in the entire
// product flows through this engine. Nothing is hard-coded. Ever.
// ═══════════════════════════════════════════════════════════════════════

#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![doc = include_str!("../README.md")]

pub mod engine;
pub mod encryption;
pub mod layers;
pub mod policy;
pub mod registry;
pub mod validation;

// ── Re-exports ────────────────────────────────────────────────────────

pub use engine::ConfigEngine;
pub use encryption::EncryptionEngine;
pub use layers::ConfigScope;
pub use policy::{PolicyCategory, PolicyDefinition, PolicyEngine};
pub use registry::{ConfigCategory, ConfigEntry, ConfigRegistry};
pub use validation::ConfigValidator;
