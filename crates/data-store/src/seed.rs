// ═══════════════════════════════════════════════════════════════════════
// TRRUSTT — Database Seeding
//
// Seeds the database with default data:
// - 6 default themes
// - Default roles
// - System settings
// - Optional onboarding demo data
// ═══════════════════════════════════════════════════════════════════════

use tracing::{info, instrument};
use uuid::Uuid;

use shared::Result;
use crate::DataStore;

/// Seed the database with default data.
///
/// Idempotent — safe to call multiple times.
#[instrument(skip(store))]
pub async fn seed_defaults(store: &DataStore) -> Result<()> {
    info!("Seeding default data");

    seed_default_themes(store).await?;
    seed_system_settings(store).await?;

    info!("Default data seeding complete");
    Ok(())
}

/// Seed the 6 default themes.
async fn seed_default_themes(store: &DataStore) -> Result<()> {
    let themes_repo = store.themes();
    let existing = themes_repo.list_all().await?;

    if !existing.is_empty() {
        info!("Themes already seeded, skipping");
        return Ok(());
    }

    let default_themes = vec![
        ("default", true, r#"{"primary":"#2563EB","secondary":"#7C3AED","accent":"#F59E0B","background":"#FFFFFF","foreground":"#1F2937","data_colors":["#2563EB","#7C3AED","#F59E0B","#10B981","#EF4444","#8B5CF6","#EC4899","#06B6D4"],"semantic":{"good":"#10B981","bad":"#EF4444","neutral":"#6B7280","warning":"#F59E0B"}}"#, r#"{"font_family":"Segoe UI","title_size":16,"body_size":12,"label_size":10,"kpi_size":36}"#),
        ("dark", true, r#"{"primary":"#3B82F6","secondary":"#8B5CF6","accent":"#FBBF24","background":"#111827","foreground":"#F9FAFB","data_colors":["#3B82F6","#8B5CF6","#FBBF24","#34D399","#F87171","#A78BFA","#F472B6","#22D3EE"],"semantic":{"good":"#34D399","bad":"#F87171","neutral":"#9CA3AF","warning":"#FBBF24"}}"#, r#"{"font_family":"Segoe UI","title_size":16,"body_size":12,"label_size":10,"kpi_size":36}"#),
        ("corporate", false, r#"{"primary":"#1E40AF","secondary":"#047857","accent":"#B45309","background":"#F8FAFC","foreground":"#0F172A","data_colors":["#1E40AF","#047857","#B45309","#0369A1","#BE123C","#7C3AED","#0D9488","#CA8A04"],"semantic":{"good":"#047857","bad":"#BE123C","neutral":"#64748B","warning":"#CA8A04"}}"#, r#"{"font_family":"Calibri","title_size":14,"body_size":11,"label_size":9,"kpi_size":32}"#),
        ("minimal", false, r#"{"primary":"#374151","secondary":"#6B7280","accent":"#111827","background":"#FFFFFF","foreground":"#111827","data_colors":["#374151","#6B7280","#9CA3AF","#D1D5DB","#4B5563","#78716C","#A8A29E","#D6D3D1"],"semantic":{"good":"#059669","bad":"#DC2626","neutral":"#9CA3AF","warning":"#D97706"}}"#, r#"{"font_family":"Inter","title_size":14,"body_size":11,"label_size":9,"kpi_size":32}"#),
        ("vibrant", false, r#"{"primary":"#D946EF","secondary":"#06B6D4","accent":"#F97316","background":"#FAFAFE","foreground":"#18181B","data_colors":["#D946EF","#06B6D4","#F97316","#22C55E","#EF4444","#8B5CF6","#EC4899","#EAB308"],"semantic":{"good":"#22C55E","bad":"#EF4444","neutral":"#A1A1AA","warning":"#EAB308"}}"#, r#"{"font_family":"Segoe UI","title_size":18,"body_size":13,"label_size":11,"kpi_size":40}"#),
        ("accessible", false, r#"{"primary":"#0066CC","secondary":"#BF40BF","accent":"#CC6600","background":"#FFFFFF","foreground":"#000000","data_colors":["#0066CC","#BF40BF","#CC6600","#008000","#CC0000","#6600CC","#CC0066","#006666"],"semantic":{"good":"#008000","bad":"#CC0000","neutral":"#595959","warning":"#CC6600"}}"#, r#"{"font_family":"Atkinson Hyperlegible","title_size":18,"body_size":14,"label_size":12,"kpi_size":40}"#),
    ];

    for (name, is_default, colors, typography) in default_themes {
        let id = Uuid::now_v7();
        themes_repo.create(id, name, is_default, colors, typography).await?;
        info!(theme = name, "Seeded default theme");
    }

    info!("Seeded 6 default themes");
    Ok(())
}

/// Seed system settings with defaults.
async fn seed_system_settings(_store: &DataStore) -> Result<()> {
    // System settings are populated from config at runtime.
    // This is a placeholder for any hard-coded defaults.
    Ok(())
}
