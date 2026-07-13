// TRRUSTT — License Manager
// Pure Rust JWT validation (Ed25519), AES-256-GCM cache, 30-day offline grace.
#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod validator;
pub mod features;
pub mod offline;
pub mod telemetry;
pub mod cache;

use shared::{LicenseInfo, PlanTier, Result};
use chrono::{DateTime, Utc};

/// The license manager.
pub struct LicenseManager {
    validator: validator::LicenseValidator,
    feature_manager: features::FeatureManager,
    offline_grant: offline::OfflineGrant,
    cache: cache::LicenseCache,
}

impl LicenseManager {
    /// Create a new license manager.
    pub fn new() -> Self {
        Self {
            validator: validator::LicenseValidator::new(),
            feature_manager: features::FeatureManager::new(),
            offline_grant: offline::OfflineGrant::new(),
            cache: cache::LicenseCache::new(),
        }
    }

    /// Validate a license JWT and return license information.
    /// Checks JWT signature, expiration, and feature flags.
    pub fn validate(&self, token: &str) -> Result<LicenseInfo> {
        // Check cache first
        if let Some(cached) = self.cache.get(token) {
            if !self.validator.is_expired(&cached) {
                return Ok(cached);
            }
        }

        // Validate JWT signature (Ed25519)
        let license = self.validator.validate_token(token)?;

        // Cache the validated license
        self.cache.store(token, &license)?;

        Ok(license)
    }

    /// Check if a specific feature is available for the current license tier.
    pub fn has_feature(&self, tier: PlanTier, feature: &str) -> bool {
        self.feature_manager.is_available(tier, feature)
    }

    /// Activate license with offline grace period.
    /// Returns Ok even if license server is unreachable, using cached license
    /// if within the 30-day grace period.
    pub fn activate_offline(&self) -> Result<LicenseInfo> {
        if let Some(cached) = self.cache.get_cached() {
            if self.offline_grant.is_within_grace_period(&cached) {
                return Ok(cached);
            }
        }
        Err(shared::AppError::LicenseExpired)
    }

    /// Check if license needs renewal (within 14 days of expiry).
    pub fn needs_renewal(&self) -> bool {
        if let Some(cached) = self.cache.get_cached() {
            let now = Utc::now();
            let days_remaining = (cached.expires_at - now).num_days();
            days_remaining < 14
        } else {
            true
        }
    }

    /// Get the current license info from cache.
    pub fn current_license(&self) -> Option<LicenseInfo> {
        self.cache.get_cached()
    }

    /// Send usage telemetry to license server (periodic phone-home).
    pub async fn phone_home(&self) -> Result<()> {
        telemetry::send_license_telemetry().await
    }
}

impl Default for LicenseManager {
    fn default() -> Self { Self::new() }
}
