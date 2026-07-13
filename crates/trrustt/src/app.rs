// ═══════════════════════════════════════════════════════════════════════
// TRRUSTT — Application State
//
// Central dependency injection point. Wires all crates together.
// This is where the config engine, database, and all services are
// initialized and made available to CLI, TUI, MCP, and GUI modes.
// ═══════════════════════════════════════════════════════════════════════

use std::collections::HashMap;
use std::sync::Arc;

use config_engine::ConfigEngine;
use data_store::DataStore;
use shared::Result;
use tracing::info;

use crate::cli::Args;

/// The central application state shared across all modes.
///
/// This is the dependency injection container. Every service
/// and resource is initialized here and made available to
/// all components.
pub struct AppState {
    /// Configuration engine (resolved from all 7 layers).
    pub config: Arc<ConfigEngine>,

    /// Internal database (SQLite + LanceDB).
    pub db: Arc<DataStore>,

    /// CLI arguments (used across modes).
    pub args: Args,
}

impl AppState {
    /// Initialize the application state.
    ///
    /// This is the bootstrap sequence:
    /// 1. Load configuration from all sources
    /// 2. Open/create the internal database
    /// 3. Run pending migrations
    /// 4. Seed default data if needed
    /// 5. Initialize telemetry and tracing
    ///
    /// # Arguments
    /// * `args` - Parsed CLI arguments.
    #[tracing::instrument(name = "app_init", skip(args))]
    pub async fn initialize(args: &Args) -> Result<Self> {
        // ── Step 1: Load configuration ───────────────────────────────
        let cli_overrides = HashMap::new();
        let config = ConfigEngine::new(cli_overrides)?;
        info!("Configuration engine initialized");

        // ── Step 2: Open database ─────────────────────────────────────
        let db_path: String = config.get("paths.database_path")
            .unwrap_or_else(|_| "~/.trrustt/data/trrustt.db".to_string());

        // Expand tilde in path
        let db_path = if db_path.starts_with('~') {
            let home = dirs::home_dir().unwrap_or_else(|| std::path::PathBuf::from("."));
            let stripped = db_path.strip_prefix("~/").unwrap_or(&db_path);
            home.join(stripped)
        } else {
            std::path::PathBuf::from(&db_path)
        };

        let db = DataStore::open(&db_path).await?;
        info!(path = %db_path.display(), "Database opened");

        // ── Step 3: Run migrations ────────────────────────────────────
        db.migrate().await?;
        info!("Database migrations complete");

        // ── Step 4: Seed default data ─────────────────────────────────
        data_store::seed::seed_defaults(&db).await?;
        info!("Default data seeded");

        Ok(Self {
            config: Arc::new(config),
            db: Arc::new(db),
            args: args.clone(),
        })
    }
}

// ── Tauri command handlers (only compiled with gui feature) ──────────

#[cfg(feature = "gui")]
pub mod commands {
    use tauri::State;
    use super::AppState;

    /// Example Tauri command: greet the user.
    #[tauri::command]
    pub fn greet(name: &str) -> String {
        format!("Hello, {}! Welcome to trRUSTt.", name)
    }

    /// Example Tauri command: get a config value.
    #[tauri::command]
    pub fn get_config(state: State<AppState>, key: String) -> Result<String, String> {
        state.config.get::<serde_json::Value>(&key)
            .map(|v| v.to_string())
            .map_err(|e| e.to_string())
    }
}

impl Clone for Args {
    fn clone(&self) -> Self {
        Self {
            command: self.command.clone(),
            port: self.port,
            pbix: self.pbix.clone(),
            config_dir: self.config_dir.clone(),
            log_level: self.log_level.clone(),
            log_format: self.log_format.clone(),
            log_file: self.log_file.clone(),
            otlp_endpoint: self.otlp_endpoint.clone(),
        }
    }
}
