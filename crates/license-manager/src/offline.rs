// TRRUSTT — Offline Grace. 30-day offline grace period.
use shared::LicenseInfo;
use chrono::{Utc, Duration};

const GRACE_PERIOD_DAYS: i64 = 30;

pub struct OfflineGrant {
    last_online_check: std::sync::Mutex<Option<chrono::DateTime<Utc>>>,
}

impl OfflineGrant {
    pub fn new() -> Self { Self { last_online_check: std::sync::Mutex::new(None) } }

    pub fn is_within_grace_period(&self, license: &LicenseInfo) -> bool {
        let now = Utc::now();
        let grace_end = license.expires_at + Duration::days(GRACE_PERIOD_DAYS);
        now < grace_end
    }

    pub fn days_remaining(&self, license: &LicenseInfo) -> i64 {
        let now = Utc::now();
        let grace_end = license.expires_at + Duration::days(GRACE_PERIOD_DAYS);
        (grace_end - now).num_days().max(0)
    }

    pub fn record_online_check(&self) {
        *self.last_online_check.lock().unwrap() = Some(Utc::now());
    }

    pub fn last_check(&self) -> Option<chrono::DateTime<Utc>> {
        *self.last_online_check.lock().unwrap()
    }
}

impl Default for OfflineGrant { fn default() -> Self { Self::new() } }

#[cfg(test)]
mod tests {
    use super::*;
    use shared::PlanTier;

    #[test]
    fn test_grace_period_active() {
        let grant = OfflineGrant::new();
        let license = LicenseInfo {
            licensee: "Test".into(), email: "test@test.com".into(),
            tier: PlanTier::Pro, seats: None, features: vec![],
            issued_at: Utc::now(),
            expires_at: Utc::now() - Duration::days(10), // Expired 10 days ago
            license_id: "test-1".into(),
        };
        assert!(grant.is_within_grace_period(&license));
        assert_eq!(grant.days_remaining(&license), 20);
    }

    #[test]
    fn test_grace_period_expired() {
        let grant = OfflineGrant::new();
        let license = LicenseInfo {
            licensee: "Test".into(), email: "test@test.com".into(),
            tier: PlanTier::Pro, seats: None, features: vec![],
            issued_at: Utc::now(),
            expires_at: Utc::now() - Duration::days(40), // Expired 40 days ago
            license_id: "test-2".into(),
        };
        assert!(!grant.is_within_grace_period(&license));
    }
}
}
