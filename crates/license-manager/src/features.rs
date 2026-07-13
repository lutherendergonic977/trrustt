// TRRUSTT — Feature Manager. Tier-based feature availability.
use shared::PlanTier;

pub struct FeatureManager;

impl FeatureManager {
    pub fn new() -> Self { Self }
    pub fn is_available(&self, tier: PlanTier, feature: &str) -> bool {
        let min = ALL_FEATURES.iter().find(|(n, _)| *n == feature).map(|(_, t)| *t).unwrap_or(PlanTier::Enterprise);
        tier as i32 >= min as i32
    }
    pub fn features_for_tier(&self, tier: PlanTier) -> Vec<&'static str> {
        ALL_FEATURES.iter().filter(|(_, min)| tier as i32 >= *min as i32).map(|(n, _)| *n).collect()
    }
}

const ALL_FEATURES: &[(&str, PlanTier)] = &[
    ("schema_discovery", PlanTier::Free), ("basic_dax", PlanTier::Free), ("manual_measures", PlanTier::Free),
    ("advanced_dax", PlanTier::Pro), ("dashboard_gen", PlanTier::Pro), ("image_to_dashboard", PlanTier::Pro),
    ("self_correction", PlanTier::Pro), ("dax_explanation", PlanTier::Pro), ("cli_mode", PlanTier::Pro),
    ("custom_themes", PlanTier::Pro), ("mcp_server", PlanTier::Team), ("mcp_client", PlanTier::Team),
    ("data_ingestion", PlanTier::Team), ("rag", PlanTier::Team), ("multi_user", PlanTier::Team),
    ("workspaces", PlanTier::Team), ("audit", PlanTier::Team), ("sso", PlanTier::Enterprise),
    ("rbac", PlanTier::Enterprise), ("admin_policies", PlanTier::Enterprise), ("white_label", PlanTier::Enterprise),
    ("rebranding", PlanTier::Oem), ("redistribution", PlanTier::Oem), ("source_access", PlanTier::Oem),
];

impl Default for FeatureManager { fn default() -> Self { Self::new() } }

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_free_features() {
        let m = FeatureManager::new();
        assert!(m.is_available(PlanTier::Free, "schema_discovery"));
        assert!(!m.is_available(PlanTier::Free, "dashboard_gen"));
    }
}
}
