# trrustt — Main Binary + Tauri Shell

The single binary entry point for TRRUSTT. Supports multiple runtime modes.

## Modes

| Mode | Command | UI | Use Case |
|---|---|---|---|
| **GUI** (default) | `TRRUSTT.exe` | Tauri + Svelte 5 | Primary: dashboard wizard, DAX playground, theme designer |
| **CLI** | `TRRUSTT [command]` | Terminal | Automation, scripting, CI/CD |
| **TUI** | `TRRUSTT admin` | ratatui | Quick config, license activation from terminal |
| **MCP** | `TRRUSTT mcp serve` | stdio | Integration with Claude Desktop, Cursor, Continue |

## Usage

```bash
# GUI (default)
TRRUSTT.exe

# CLI: Schema discovery
TRRUSTT schema discover --port 54321

# CLI: Generate DAX
TRRUSTT dax generate "YoY sales growth" --complexity advanced

# CLI: Create dashboard
TRRUSTT dashboard create "Executive KPI dashboard"

# TUI admin
TRRUSTT admin

# MCP server
TRRUSTT mcp serve

# Config management
TRRUSTT config show
TRRUSTT config set ai.default_provider anthropic
```

## Dependencies

- `shared` — types, errors, telemetry
- `config-engine` — configuration
- `data-store` — database
- `xmla-client` — Power BI SSAS bridge
- `clap` — CLI
- `ratatui` + `crossterm` — TUI
- `tauri` — GUI (optional feature)
