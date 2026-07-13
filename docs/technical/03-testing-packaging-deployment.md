# Testing, Packaging & Deployment — Rust v2

## 1. Testing Strategy

### 1.1 Rust Testing Pyramid

```
              ┌─────────┐
              │  E2E    │  Custom test harness (PBI Desktop + real SSAS)
              │  ~5%    │
              └─────────┘
           ┌───────────────┐
           │  Integration   │  Cross-crate tests (tests/ directory)
           │    ~15%        │
           └───────────────┘
        ┌──────────────────────┐
        │    Unit Tests         │  Per-crate #[test] functions
        │       ~65%            │
        └──────────────────────┘
     ┌─────────────────────────────┐
     │   Property-Based Tests      │  proptest — generative/fuzz
     │         ~15%                │
     └─────────────────────────────┘
```

### 1.2 Toolchain

| Layer | Tool | Command |
|---|---|---|
| Unit tests | `cargo test` | `cargo test --workspace` |
| Doc tests | `cargo test --doc` | `cargo test --doc` |
| Integration tests | `cargo test --test '*'` | `cargo test --test integration` |
| Property tests | `proptest` | (runs with `cargo test`) |
| Benchmarks | `criterion` | `cargo bench` |
| Linting | `clippy` | `cargo clippy -- -D warnings` |
| Formatting | `rustfmt` | `cargo fmt --all -- --check` |
| Unsafe audit | `cargo-geiger` | `cargo geiger` |
| Security audit | `cargo-audit` | `cargo audit` |
| License audit | `cargo-deny` | `cargo deny check` |
| Coverage | `cargo-llvm-cov` | `cargo llvm-cov --workspace --lcov` |
| UB detection | `miri` | `cargo miri test` |
| DAX correctness | Custom harness | `cargo test --test dax_correctness` |

### 1.3 Property-Based Testing (proptest)

```rust
// crates/dax-engine/tests/proptest_validation.rs

use proptest::prelude::*;
use dax_engine::{parse_dax, format_dax};

/// Strategy: generate random valid DAX expressions
fn valid_dax_strategy() -> impl Strategy<Value = String> {
    // Generate recursive DAX expressions:
    // - Simple: "SUM('Sales'[Amount])"
    // - Nested: "CALCULATE(SUM(...), FILTER(...))"
    // - With vars: "VAR x = ... RETURN ..."
    prop::collection::vec(valid_dax_leaf(), 1..10)
        .prop_map(|parts| parts.join(" "))
}

proptest! {
    /// Any valid DAX must parse successfully
    #[test]
    fn valid_dax_always_parses(expr in valid_dax_strategy()) {
        let result = parse_dax(&expr);
        prop_assert!(result.is_ok(), 
            "Expected valid DAX to parse: '{}'. Error: {:?}", 
            expr, result.err()
        );
    }
    
    /// Format → Parse roundtrip must preserve semantics
    #[test]
    fn roundtrip_preserves_ast(expr in valid_dax_strategy()) {
        let ast1 = parse_dax(&expr).unwrap();
        let formatted = format_dax(&ast1).unwrap();
        let ast2 = parse_dax(&formatted).unwrap();
        prop_assert_eq!(ast1, ast2,
            "Roundtrip changed AST.\nOriginal: {}\nFormatted: {}",
            expr, formatted
        );
    }
    
    /// Generated DAX must validate against any schema
    #[test]
    fn generated_dax_validates(
        (schema, description) in (schema_strategy(), description_strategy())
    ) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let engine = DaxEngine::new(test_config());
        let measures = rt.block_on(
            engine.generate(&description, &schema, ComplexityLevel::Intermediate, &[])
        ).unwrap();
        
        for m in &measures {
            let validation = rt.block_on(
                engine.validate(&m.expression, &schema)
            ).unwrap();
            prop_assert!(validation.is_valid,
                "Generated DAX failed validation.\nMeasure: {}\nExpression: {}\nErrors: {:?}",
                m.name, m.expression, validation.errors
            );
        }
    }
}
```

### 1.4 DAX Correctness Test Harness

```rust
// tests/dax_correctness.rs

#[derive(Deserialize)]
struct DaxTestSuite {
    name: String,
    test_cases: Vec<DaxTestCase>,
}

#[derive(Deserialize)]
struct DaxTestCase {
    id: String,
    description: String,
    complexity: String,
    schema: SchemaSnapshot,
    #[serde(default)]
    expected_functions: Vec<String>,
    #[serde(default)]
    forbidden_functions: Vec<String>,
    #[serde(default)]
    expected_pattern: Option<String>,  // regex
}

#[tokio::test]
async fn dax_generation_correctness() {
    let suites: Vec<DaxTestSuite> = serde_json::from_str(
        include_str!("fixtures/dax-test-suites.json")
    ).unwrap();
    
    let engine = DaxEngine::new(test_config());
    
    for suite in &suites {
        println!("\n=== {} ===", suite.name);
        for case in &suite.test_cases {
            let complexity: ComplexityLevel = case.complexity.parse().unwrap();
            
            let measures = engine.generate(
                &case.description,
                &case.schema.into(),
                complexity,
                &[],
            ).await.expect(&format!("Failed to generate: {}", case.id));
            
            // Check expected functions present
            for measure in &measures {
                for func in &case.expected_functions {
                    assert!(
                        measure.expression.contains(func),
                        "[{}] Expected function '{}' not found in:\n{}",
                        case.id, func, measure.expression
                    );
                }
                
                // Check forbidden functions absent
                for func in &case.forbidden_functions {
                    assert!(
                        !measure.expression.contains(func),
                        "[{}] Forbidden function '{}' found in:\n{}",
                        case.id, func, measure.expression
                    );
                }
                
                // Check regex pattern if specified
                if let Some(pattern) = &case.expected_pattern {
                    let re = regex::Regex::new(pattern).unwrap();
                    assert!(
                        re.is_match(&measure.expression),
                        "[{}] Expression doesn't match expected pattern '{}':\n{}",
                        case.id, pattern, measure.expression
                    );
                }
            }
        }
    }
}
```

### 1.5 Benchmarking (criterion)

```rust
// benches/dax_parser_bench.rs

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use dax_engine::parse_dax;

fn benchmark_parse_simple(c: &mut Criterion) {
    c.bench_function("parse simple DAX", |b| {
        b.iter(|| {
            parse_dax(black_box(
                "SUM('Sales'[SalesAmount])"
            ))
        });
    });
}

fn benchmark_parse_complex(c: &mut Criterion) {
    let complex_dax = r#"
        VAR CurrentSales = 
            CALCULATE(
                SUM('Sales'[SalesAmount]),
                'Calendar'[Year] = MAX('Calendar'[Year])
            )
        VAR PreviousSales = 
            CALCULATE(
                SUM('Sales'[SalesAmount]),
                SAMEPERIODLASTYEAR('Calendar'[Date])
            )
        RETURN
            DIVIDE(
                CurrentSales - PreviousSales,
                PreviousSales,
                BLANK()
            )
    "#;
    
    c.bench_function("parse complex DAX", |b| {
        b.iter(|| {
            parse_dax(black_box(complex_dax))
        });
    });
}

criterion_group!(benches, benchmark_parse_simple, benchmark_parse_complex);
criterion_main!(benches);
```

## 2. Packaging

### 2.1 Build Configuration

```toml
# Cargo.toml (workspace)
[profile.release]
opt-level = 3          # Maximum optimization
lto = "fat"            # Link-time optimization (whole crate)
codegen-units = 1      # Single codegen unit (better optimization)
panic = "abort"        # Smaller binary, no unwind tables
strip = true            # Strip debug symbols
incremental = false     # Clean build for release

[profile.dist]
inherits = "release"
# Additional distribution-specific settings
```

### 2.2 Build Commands

```bash
# Windows (MSVC — required for PBI Desktop interop)
RUSTFLAGS="-C target-cpu=native" cargo build --profile dist --target x86_64-pc-windows-msvc

# Windows (cross-compile from Linux)
cargo build --profile dist --target x86_64-pc-windows-msvc

# macOS (Intel)
cargo build --profile dist --target x86_64-apple-darwin

# macOS (Apple Silicon)
cargo build --profile dist --target aarch64-apple-darwin

# Linux (static MUSL — zero runtime deps)
RUSTFLAGS="-C target-feature=+crt-static" cargo build --profile dist --target x86_64-unknown-linux-musl
```

### 2.3 Binary Size Estimates

| Build | Size |
|---|---|
| Debug | ~120 MB |
| Release (default) | ~35 MB |
| Release (LTO + strip) | ~18-25 MB |
| Release + UPX compression | ~8-12 MB |

### 2.4 Installation

```
Method 1: Direct Download
─────────────────────────
  1. Download IntelliDashboard.exe (~20 MB)
  2. Copy to C:\Program Files\IntelliDashboard\
  3. Register as External Tool (one JSON file)
  4. Done.

Method 2: Package Manager
─────────────────────────
  winget install IntelliDashboard.Builder
  scoop install intellidashboard
  choco install intellidashboard

Method 3: Cargo (for Rust devs)
───────────────────────────────
  cargo install intellidashboard

Method 4: Docker (server mode)
──────────────────────────────
  docker pull intellidashboard/builder:latest
  docker run --rm intellidashboard/builder dax generate --description "..."
```

### 2.5 External Tool Registration

```json
// %APPDATA%\Microsoft\Power BI Desktop\External Tools\IntelliDashboard.json
{
  "version": "1.0",
  "name": "🧠 IntelliDashboard",
  "description": "AI-powered dashboard builder — generate DAX, dashboards, and insights",
  "path": "C:\\Program Files\\IntelliDashboard\\IntelliDashboard.exe",
  "arguments": "--port %port% --pbix \"%pbix%\""
}
```

### 2.6 Self-Update

```rust
// crates/intellidashboard/src/update.rs

use self_update::{backends::github, Status};

pub async fn check_for_updates() -> Result<Option<UpdateInfo>, UpdateError> {
    let releases = github::ReleaseList::configure()
        .repo_owner("intellidashboard")
        .repo_name("builder")
        .build()?
        .fetch()?;
    
    let current = env!("CARGO_PKG_VERSION");
    let latest = releases.first()
        .ok_or(UpdateError::NoReleases)?;
    
    if latest.version > current {
        Ok(Some(UpdateInfo {
            version: latest.version.clone(),
            download_url: latest.asset_url("windows", "x86_64")?,
            release_notes: latest.body.clone(),
        }))
    } else {
        Ok(None)
    }
}

pub async fn perform_update(update: UpdateInfo) -> Result<Status, UpdateError> {
    let status = github::Update::configure()
        .repo_owner("intellidashboard")
        .repo_name("builder")
        .bin_name("IntelliDashboard")
        .current_version(env!("CARGO_PKG_VERSION"))
        .target_version(&update.version)
        .build()?
        .update()?;
    
    Ok(status)
}
```

## 3. CI/CD

### 3.1 CI Workflow

```yaml
# .github/workflows/ci.yml
name: CI

on:
  pull_request:
  push:
    branches: [main]

jobs:
  # ── Lint & Format ──────────────────────────
  lint:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with: { components: clippy, rustfmt }
      - run: cargo clippy --workspace --all-features -- -D warnings
      - run: cargo fmt --all -- --check

  # ── Test Matrix ────────────────────────────
  test:
    needs: lint
    strategy:
      matrix:
        os: [ubuntu-latest, windows-latest, macos-latest]
        rust: [stable]
    runs-on: ${{ matrix.os }}
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@master
        with: { toolchain: ${{ matrix.rust }} }
      - run: cargo test --workspace --all-features
      - run: cargo test --workspace --doc
      - run: cargo test --test '*'

  # ── Security Audit ─────────────────────────
  security:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo install cargo-audit cargo-deny
      - run: cargo audit
      - run: cargo deny check licenses
      - run: cargo deny check advisories

  # ── Coverage ───────────────────────────────
  coverage:
    needs: test
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo install cargo-llvm-cov
      - run: cargo llvm-cov --workspace --lcov --output-path lcov.info
      - uses: actions/upload-artifact@v4
        with:
          name: coverage
          path: lcov.info

  # ── Benchmarks (regression check) ──────────
  bench:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo bench -- --output-format bencher | tee output.txt
      - uses: benchmark-action/github-action-benchmark@v1
        with:
          tool: cargo
          output-file-path: output.txt
```

### 3.2 Release Workflow

```yaml
# .github/workflows/release.yml
name: Release

on:
  push:
    tags: ['v*']

jobs:
  build:
    strategy:
      matrix:
        include:
          - target: x86_64-pc-windows-msvc
            os: windows-latest
            binary: IntelliDashboard.exe
          - target: x86_64-apple-darwin
            os: macos-latest
            binary: IntelliDashboard
          - target: aarch64-apple-darwin
            os: macos-latest
            binary: IntelliDashboard
          - target: x86_64-unknown-linux-musl
            os: ubuntu-latest
            binary: IntelliDashboard
    
    runs-on: ${{ matrix.os }}
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with: { targets: ${{ matrix.target }} }
      
      - name: Build Release
        run: |
          cargo build --profile dist --target ${{ matrix.target }}
      
      - name: Strip + Compress (optional)
        if: matrix.os == 'windows-latest'
        run: |
          strip target/${{ matrix.target }}/dist/${{ matrix.binary }}
      
      - uses: actions/upload-artifact@v4
        with:
          name: ${{ matrix.target }}
          path: target/${{ matrix.target }}/dist/${{ matrix.binary }}

  sign-windows:
    needs: build
    runs-on: windows-latest
    steps:
      - uses: actions/download-artifact@v4
        with: { name: x86_64-pc-windows-msvc }
      - name: Code Sign
        run: |
          signtool sign /fd SHA256 /f ${{ secrets.CERT_PATH }} `
            /p ${{ secrets.CERT_PASSWORD }} `
            /tr http://timestamp.digicert.com /td SHA256 `
            IntelliDashboard.exe
      - uses: actions/upload-artifact@v4
        with:
          name: x86_64-pc-windows-msvc-signed
          path: IntelliDashboard.exe

  github-release:
    needs: [build, sign-windows]
    runs-on: ubuntu-latest
    steps:
      - uses: actions/download-artifact@v4
      - name: Generate SBOM
        run: cargo cyclonedx --all --output-format json
      - uses: softprops/action-gh-release@v2
        with:
          files: |
            **/IntelliDashboard.exe
            **/IntelliDashboard
          body_path: CHANGELOG.md
          generate_release_notes: true
          
  publish-winget:
    needs: github-release
    runs-on: windows-latest
    steps:
      - name: Update WinGet Manifest
        uses: vedantmgoyal2009/winget-releaser@v2
        with:
          identifier: IntelliDashboard.Builder
          token: ${{ secrets.WINGET_TOKEN }}
```

## 4. Deployment Architecture

### 4.1 Deployment Modes Summary

| Mode | Binary | Use Case |
|---|---|---|
| **External Tool** | `IntelliDashboard.exe` in ExtTools folder | PBI Desktop plugin |
| **CLI** | `IntelliDashboard.exe` on PATH | Terminal usage |
| **MCP Server** | Same binary, `mcp serve` subcommand | AI assistant integration |
| **Docker** | `FROM scratch` container | CI/CD, server deployments |
| **Winget/Scoop/Choco** | Package manager distribution | Enterprise deployment |

### 4.2 Docker Image (Minimal)

```dockerfile
# Dockerfile
FROM scratch
COPY target/x86_64-unknown-linux-musl/dist/IntelliDashboard /IntelliDashboard
COPY prompts/ /prompts/
COPY themes/ /themes/
COPY config/defaults.toml /config/defaults.toml
ENTRYPOINT ["/IntelliDashboard"]
```

```bash
# Build: ~8 MB final image
docker build -t intellidashboard/builder:latest .

# Usage:
docker run --rm \
  -v /path/to/pbix:/data \
  intellidashboard/builder \
  dax generate --pbix /data/report.pbix --description "YoY growth"
```

### 4.3 Enterprise Deployment

For organizations with managed Power BI environments:

```
┌─────────────────────────────────────────────────────────────┐
│                 ENTERPRISE DEPLOYMENT                        │
│                                                              │
│  Admin pushes IntelliDashboard.exe via SCCM / Intune         │
│  Admin deploys External Tool JSON via GPO / script           │
│  Admin configures admin-policies.toml (centrally managed)    │
│  Admin configures whitelist (DAX functions, data sources)    │
│                                                              │
│  Users click 🧠 IntelliDashboard in PBI Desktop ribbon       │
│  Tool automatically uses org's Azure OpenAI / LLM config     │
│  All settings locked by admin policies                       │
│  Audit logs shipped to org's SIEM                            │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

---

> **Document Version:** 2.0  
> **Part of:** IntelliDashboard Builder Technical Docs (Rust-Native)
