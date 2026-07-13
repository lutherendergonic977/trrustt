// ═══════════════════════════════════════════════════════════════════════
// TRRUSTT — CLI Argument Definitions (clap)
//
// All CLI commands and arguments. The CLI is the automation interface.
// Every option can also be set via TRRUSTT_ env vars or config files.
// ═══════════════════════════════════════════════════════════════════════

use clap::{Parser, Subcommand};
use config_engine::ConfigEngine;
use data_store::DataStore;
use std::collections::HashMap;
use std::path::PathBuf;

use shared::Result;

/// TRRUSTT — AI-driven Power BI External Tool.
///
/// One binary. Infinite dashboards.
#[derive(Parser, Debug)]
#[command(
    name = "TRRUSTT",
    version,
    about = "AI-driven Power BI External Tool",
    long_about = "trRUSTt your data. One binary. Infinite dashboards.\n\n\
                  Automates dashboard creation: schema discovery, DAX generation,\n\
                  AI-powered layouts, theme engine, MCP integration."
)]
pub struct Args {
    /// Command to execute (defaults to GUI mode)
    #[command(subcommand)]
    pub command: Command,

    /// SSAS instance port (from PBI Desktop, auto-detected if not provided)
    #[arg(short, long, env = "TRRUSTT_SSAS_PORT", global = true)]
    pub port: Option<u16>,

    /// Path to the .pbix file
    #[arg(long, env = "TRRUSTT_PBIX_PATH", global = true)]
    pub pbix: Option<PathBuf>,

    /// Config directory override
    #[arg(long, env = "TRRUSTT_CONFIG_DIR", global = true)]
    pub config_dir: Option<PathBuf>,

    /// Log level
    #[arg(long, env = "TRRUSTT_LOG_LEVEL", default_value = "info", global = true)]
    pub log_level: String,

    /// Log format (pretty, json)
    #[arg(long, env = "TRRUSTT_LOG_FORMAT", default_value = "pretty", global = true)]
    pub log_format: String,

    /// Log file path (optional)
    #[arg(long, env = "TRRUSTT_LOG_FILE", global = true)]
    pub log_file: Option<String>,

    /// OpenTelemetry OTLP endpoint (optional)
    #[arg(long, env = "TRRUSTT_OTLP_ENDPOINT", global = true)]
    pub otlp_endpoint: Option<String>,
}

/// Top-level commands.
#[derive(Subcommand, Debug, Clone)]
pub enum Command {
    /// Launch the GUI (default mode)
    Gui,

    /// Schema discovery and management
    Schema {
        #[command(subcommand)]
        action: SchemaAction,
    },

    /// DAX generation, validation, and correction
    Dax {
        #[command(subcommand)]
        action: DaxAction,
    },

    /// Dashboard generation and management
    Dashboard {
        #[command(subcommand)]
        action: DashboardAction,
    },

    /// Launch the TUI admin interface
    Admin,

    /// MCP server operations
    Mcp {
        #[command(subcommand)]
        action: McpAction,
    },

    /// Configuration management
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },

    /// License management
    License {
        #[command(subcommand)]
        action: LicenseAction,
    },
}

/// Schema-related commands.
#[derive(Subcommand, Debug, Clone)]
pub enum SchemaAction {
    /// Discover and display the model schema
    Discover,
    /// Profile data distributions, nulls, outliers
    Profile {
        /// Table to profile (omit for all)
        table: Option<String>,
    },
    /// Export schema as JSON
    Export {
        /// Output file path
        output: PathBuf,
    },
}

/// DAX-related commands.
#[derive(Subcommand, Debug, Clone)]
pub enum DaxAction {
    /// Generate DAX measures from a description
    Generate {
        /// Natural language description of the measure(s)
        description: String,

        /// Complexity level: beginner, intermediate, advanced, expert
        #[arg(short, long, default_value = "intermediate")]
        complexity: String,

        /// Target table name
        #[arg(short, long)]
        table: Option<String>,
    },
    /// Validate a DAX expression
    Validate {
        /// DAX expression to validate
        expression: String,
    },
    /// Format a DAX expression (pretty-print)
    Format {
        /// DAX expression to format
        expression: String,
    },
    /// Explain a DAX expression in natural language
    Explain {
        /// DAX expression to explain
        expression: String,
    },
}

/// Dashboard-related commands.
#[derive(Subcommand, Debug, Clone)]
pub enum DashboardAction {
    /// Create a dashboard from a description
    Create {
        /// Natural language description of the desired dashboard
        intent: String,

        /// Theme name to apply
        #[arg(short, long)]
        theme: Option<String>,
    },
    /// Create a dashboard from an image (sketch, infographic, mockup)
    FromImage {
        /// Path to the image file
        image: PathBuf,
    },
    /// List dashboards for the current project
    List,
}

/// MCP-related commands.
#[derive(Subcommand, Debug, Clone)]
pub enum McpAction {
    /// Start the MCP server (stdio transport)
    Serve,
    /// List available MCP tools
    ListTools,
    /// Connect to an external MCP server
    Connect {
        /// MCP server command or URL
        target: String,
    },
}

/// Config-related commands.
#[derive(Subcommand, Debug, Clone)]
pub enum ConfigAction {
    /// Show current resolved configuration
    Show {
        /// Specific config key (omit for all)
        key: Option<String>,
    },
    /// Set a configuration value
    Set {
        /// Config key (e.g., "ai.default_provider")
        key: String,
        /// Config value
        value: String,
        /// Scope: user, project, workspace
        #[arg(short, long, default_value = "user")]
        scope: String,
    },
    /// Reset a configuration value to default
    Reset {
        /// Config key to reset
        key: String,
    },
    /// Export all configuration as JSON
    Export {
        /// Output file path
        output: PathBuf,
    },
    /// Import configuration from JSON
    Import {
        /// Input file path
        input: PathBuf,
    },
}

/// License-related commands.
#[derive(Subcommand, Debug, Clone)]
pub enum LicenseAction {
    /// Activate a license key
    Activate {
        /// License key (JWT)
        key: String,
    },
    /// Show current license status
    Status,
    /// Deactivate the current license
    Deactivate,
}

/// Parse CLI arguments.
pub fn parse_args() -> Args {
    Args::parse()
}

// ── CLI Command Handlers ──────────────────────────────────────────────

/// Handle schema commands.
pub async fn handle_schema(action: &SchemaAction, args: &Args) -> Result<()> {
    let app_state = crate::app::AppState::initialize(args).await?;

    match action {
        SchemaAction::Discover => {
            let port = args.port.unwrap_or(0_u16);
            if port == 0 {
                println!("No SSAS port specified. Use --port or launch from PBI Desktop.");
                return Ok(());
            }
            let client = xmla_client::XmlaClient::connect(port).await?;
            let schema = client.discover_schema().await?;
            println!("{}", serde_json::to_string_pretty(&schema)?);
        }
        SchemaAction::Profile { table } => {
            let table_info = table.as_ref().map(|t| format!(" for table '{}'", t)).unwrap_or_else(|| String::new());
            println!("Data profiling{}...", table_info);
            println!("(Data profiling via Polars — coming in V1)");
        }
        SchemaAction::Export { output } => {
            let port = args.port.unwrap_or(0_u16);
            let client = xmla_client::XmlaClient::connect(port).await?;
            let schema = client.discover_schema().await?;
            let json = serde_json::to_string_pretty(&schema)?;
            tokio::fs::write(output, json).await?;
            println!("Schema exported to {}", output.display());
        }
    }

    let _ = app_state;
    Ok(())
}

/// Handle DAX commands.
pub async fn handle_dax(action: &DaxAction, args: &Args) -> Result<()> {
    let app_state = crate::app::AppState::initialize(args).await?;

    match action {
        DaxAction::Generate { description, complexity, table } => {
            println!("Generating DAX for: \"{}\"", description);
            println!("  Complexity: {}", complexity);
            if let Some(t) = table {
                println!("  Target table: {}", t);
            }
            println!("(DAX generation via AI engine — coming in V1)");
        }
        DaxAction::Validate { expression } => {
            println!("Validating DAX: {}", expression);
            println!("(DAX validation via PEG parser — coming in V1)");
        }
        DaxAction::Format { expression } => {
            println!("Formatting DAX: {}", expression);
            println!("(DAX formatting via pretty-printer — coming in V1)");
        }
        DaxAction::Explain { expression } => {
            println!("Explaining DAX: {}", expression);
            println!("(DAX explanation via AI — coming in V1)");
        }
    }

    let _ = app_state;
    Ok(())
}

/// Handle dashboard commands.
pub async fn handle_dashboard(action: &DashboardAction, args: &Args) -> Result<()> {
    let app_state = crate::app::AppState::initialize(args).await?;

    match action {
        DashboardAction::Create { intent, theme } => {
            println!("Creating dashboard: \"{}\"", intent);
            if let Some(t) = theme { println!("  Theme: {}", t); }
            println!("(Dashboard generation — coming in V1)");
        }
        DashboardAction::FromImage { image } => {
            println!("Creating dashboard from image: {}", image.display());
            println!("(Image-to-dashboard — coming in V1)");
        }
        DashboardAction::List => {
            println!("Listing dashboards...");
            println!("(Dashboard management — coming in V1)");
        }
    }

    let _ = app_state;
    Ok(())
}

/// Handle config commands.
pub async fn handle_config(action: &ConfigAction, args: &Args) -> Result<()> {
    let engine = ConfigEngine::new(HashMap::new())?;

    match action {
        ConfigAction::Show { key } => {
            if let Some(k) = key {
                match engine.get::<serde_json::Value>(k) {
                    Ok(value) => println!("{} = {}", k, serde_json::to_string_pretty(&value)?),
                    Err(e) => println!("Error: {}", e),
                }
            } else {
                let all = engine.export()?;
                println!("{}", serde_json::to_string_pretty(&all)?);
            }
        }
        ConfigAction::Set { key, value, scope } => {
            let scope = config_engine::ConfigScope::parse(scope)
                .ok_or_else(|| shared::AppError::invalid_input(format!("Invalid scope: {}", scope)))?;
            let json_value: serde_json::Value = serde_json::from_str(value)
                .map_err(|e| shared::AppError::invalid_input(format!("Invalid JSON value: {}", e)))?;
            engine.set(key, json_value, scope).await?;
            println!("Set {} = {} (scope: {})", key, value, scope);
        }
        ConfigAction::Reset { key } => {
            let scope = config_engine::ConfigScope::User;
            engine.reset(key, scope).await?;
            println!("Reset {} to default", key);
        }
        ConfigAction::Export { output } => {
            let all = engine.export()?;
            tokio::fs::write(output, serde_json::to_string_pretty(&all)?).await?;
            println!("Configuration exported");
        }
        ConfigAction::Import { input } => {
            let data: serde_json::Value = serde_json::from_str(
                &tokio::fs::read_to_string(input).await?
            )?;
            engine.import(data, config_engine::ConfigScope::User).await?;
            println!("Configuration imported");
        }
    }

    Ok(())
}

/// Handle license commands.
pub async fn handle_license(action: &LicenseAction, _args: &Args) -> Result<()> {
    match action {
        LicenseAction::Activate { key } => {
            println!("Activating license: {}...", &key[..key.len().min(20)]);
            println!("(License activation — coming in V1)");
        }
        LicenseAction::Status => {
            println!("License status: Free tier (no license required)");
        }
        LicenseAction::Deactivate => {
            println!("Deactivating license...");
            println!("(License deactivation — coming in V1)");
        }
    }
    Ok(())
}
