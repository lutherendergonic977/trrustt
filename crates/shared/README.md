# shared — Shared Types, Errors, Telemetry

Foundation crate for TRRUSTT. All other crates depend on this one.

## What's Inside

- **`types.rs`** — All domain types: User, Organization, Workspace, Project, Measure, Dashboard, Theme, SchemaMetadata, AI request/response types, and more. All serializable via serde.
- **`errors.rs`** — Centralized `AppError` enum via `thiserror`. Every error in the system flows through these variants. Never `unwrap()` — always return `Result<T, AppError>`.
- **`telemetry.rs`** — Tracing setup with console + file + optional OTLP export. Correlation IDs. `#[instrument]` spans.
- **`constants.rs`** — App-wide constants: product name, env prefix, roles, providers, file paths.

## Dependencies

- No sibling crate dependencies (this is the root of the dependency tree).
- External: serde, thiserror, tracing, opentelemetry, uuid, chrono.

## Usage

```rust
use shared::{AppError, Result, User, UserRole, SchemaMetadata, init_telemetry};

// Initialize telemetry at startup
init_telemetry("info", "pretty", None, None, "TRRUSTT");

// Use domain types
let user = User { /* ... */ };
if user.role.can(UserRole::Designer) {
    // Allowed
}

// Return errors properly
fn do_something() -> Result<()> {
    Err(AppError::not_implemented("offline .pbix mode"))
}
```
