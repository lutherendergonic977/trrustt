// ═══════════════════════════════════════════════════════════════════════
// TRRUSTT — Build Script
//
// Only used when building with the `gui` feature (Tauri).
// ═══════════════════════════════════════════════════════════════════════

fn main() {
    #[cfg(feature = "gui")]
    tauri_build::build();
}
