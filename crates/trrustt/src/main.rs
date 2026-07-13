// ═══════════════════════════════════════════════════════════════════════
// TRRUSTT — Main Entry Point
//
// Mode dispatch: CLI → TUI → MCP → GUI (default)
//
// Usage:
//   TRRUSTT.exe                          → GUI mode (default)
//   TRRUSTT.exe schema discover --port X → CLI: discover schema
//   TRRUSTT.exe dax generate "YoY"       → CLI: generate DAX
//   TRRUSTT.exe admin                    → TUI admin mode
//   TRRUSTT.exe mcp serve                → MCP server mode
// ═══════════════════════════════════════════════════════════════════════

#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![doc = include_str!("../README.md")]

mod app;
mod cli;
mod mcp;
mod tui;

use std::collections::HashMap;
use std::process;

use shared::{init_telemetry, PRODUCT_NAME_CODE};
use tracing::{error, info};

/// Application entry point.
///
/// Dispatches to the appropriate mode based on CLI arguments.
#[tokio::main]
async fn main() {
    // Parse CLI arguments first to get log level for telemetry init
    let args = cli::parse_args();

    // Initialize telemetry
    init_telemetry(
        &args.log_level,
        &args.log_format,
        args.log_file.as_deref(),
        args.otlp_endpoint.as_deref(),
        PRODUCT_NAME_CODE,
    );

    info!(
        version = env!("CARGO_PKG_VERSION"),
        mode = ?args.command,
        "{} starting", PRODUCT_NAME_CODE
    );

    // Execute the appropriate mode
    let result = match &args.command {
        cli::Command::Gui => run_gui(args).await,
        cli::Command::Schema { action } => cli::handle_schema(action, &args).await,
        cli::Command::Dax { action } => cli::handle_dax(action, &args).await,
        cli::Command::Dashboard { action } => cli::handle_dashboard(action, &args).await,
        cli::Command::Admin => run_tui(args).await,
        cli::Command::Mcp { action } => mcp::handle_mcp(action, &args).await,
        cli::Command::Config { action } => cli::handle_config(action, &args).await,
        cli::Command::License { action } => cli::handle_license(action, &args).await,
    };

    match result {
        Ok(()) => {
            info!("{} shutting down normally", PRODUCT_NAME_CODE);
        }
        Err(e) => {
            error!(error = %e, "{} encountered a fatal error", PRODUCT_NAME_CODE);
            eprintln!("Error: {}", e);
            process::exit(1);
        }
    }
}

/// Run in GUI mode (Tauri window).
#[cfg(feature = "gui")]
async fn run_gui(args: cli::Args) -> shared::Result<()> {
    info!("Starting GUI mode");

    let app_state = app::AppState::initialize(&args).await?;

    tauri::Builder::default()
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            app::commands::greet,
            app::commands::get_config,
        ])
        .run(tauri::generate_context!())
        .map_err(|e| shared::AppError::internal(format!("Tauri error: {}", e)))?;

    Ok(())
}

/// Run in GUI mode (stub when gui feature disabled).
#[cfg(not(feature = "gui"))]
async fn run_gui(args: cli::Args) -> shared::Result<()> {
    let _ = args;
    Err(shared::AppError::not_implemented(
        "GUI mode requires the 'gui' feature. Rebuild with --features gui"
    ))
}

/// Run in TUI admin mode.
async fn run_tui(args: cli::Args) -> shared::Result<()> {
    info!("Starting TUI admin mode");
    let app_state = app::AppState::initialize(&args).await?;
    tui::run(app_state).await
}
