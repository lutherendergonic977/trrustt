// TRRUSTT — Dashboard Generator
// AI-driven layout planning, visual creation, theme engine.
#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod planner;
pub mod layout;
pub mod visuals;
pub mod binding;
pub mod theme;
pub mod tmsl_builder;
pub mod types;

use shared::Result;

/// The dashboard generator.
pub struct DashboardGenerator;

impl DashboardGenerator {
    /// Create a new dashboard generator.
    pub fn new() -> Self {
        Self
    }
}

impl Default for DashboardGenerator {
    fn default() -> Self {
        Self::new()
    }
}
