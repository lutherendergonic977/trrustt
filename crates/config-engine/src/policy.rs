// ═══════════════════════════════════════════════════════════════════════
// TRRUSTT — Policy Engine
//
// Policy-driven architecture. Policies define the rules that govern
// behavior: backups, retention, security, AI usage, DAX governance,
// access control, notifications, and compliance.
//
// Policies are distinct from config. Config defines *what values exist*.
// Policies define *what rules constrain behavior*.
// ═══════════════════════════════════════════════════════════════════════

use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use shared::{User, UserRole};

/// A policy definition with its value and metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyDefinition<T> {
    /// Unique policy identifier.
    pub id: String,

    /// Human-readable name.
    pub name: String,

    /// Policy category.
    pub category: PolicyCategory,

    /// The enforced value.
    pub value: T,

    /// Default value (if not explicitly set).
    pub default: T,

    /// Who can override this policy.
    pub overridable_by: Vec<UserRole>,

    /// Whether this policy is currently enabled.
    pub enabled: bool,

    /// Rationale / documentation for this policy.
    pub rationale: String,

    /// Who set this policy (audit).
    pub set_by: Option<String>,

    /// When this policy was set (audit).
    pub set_at: Option<DateTime<Utc>>,
}

/// Policy categories.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PolicyCategory {
    /// Backup & recovery policies.
    Backup,
    /// Data retention & lifecycle.
    Retention,
    /// Security & access control.
    Security,
    /// AI usage & cost management.
    AiUsage,
    /// DAX generation rules.
    DaxGovernance,
    /// API & MCP rate limiting.
    RateLimit,
    /// Notification & alerting.
    Notification,
    /// Compliance (GDPR, SOC2, etc.).
    Compliance,
    /// Feature access (license-tier enforcement).
    FeatureAccess,
}

/// An admin-enforced policy entry (non-overridable).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminPolicy {
    /// Whether this policy is actively enforced.
    pub enforce: bool,

    /// The enforced value (JSON).
    pub value: serde_json::Value,

    /// Whether users can override with a stricter value.
    pub allow_user_override: bool,

    /// Who set this policy.
    pub set_by: String,

    /// When this policy was set.
    pub set_at: DateTime<Utc>,
}

/// The policy evaluation engine.
///
/// Evaluates policies against user requests and returns violations.
/// Used to enforce AI cost limits, DAX complexity caps, backup schedules, etc.
pub struct PolicyEngine {
    /// All loaded policies indexed by ID.
    policies: HashMap<String, serde_json::Value>,

    /// Policy overrides (admin-set, higher priority).
    overrides: HashMap<String, serde_json::Value>,
}

impl PolicyEngine {
    /// Create a new, empty policy engine.
    pub fn new() -> Self {
        Self {
            policies: HashMap::new(),
            overrides: HashMap::new(),
        }
    }

    /// Load a policy into the engine.
    pub fn load_policy<T: Serialize>(&mut self, policy: PolicyDefinition<T>) -> Result<(), serde_json::Error> {
        let value = serde_json::to_value(&policy.value)?;
        self.policies.insert(policy.id, value);
        Ok(())
    }

    /// Get a policy value by ID.
    pub fn get_policy<T: for<'de> Deserialize<'de>>(&self, policy_id: &str) -> Option<T> {
        // Check overrides first
        if let Some(override_value) = self.overrides.get(policy_id) {
            return serde_json::from_value(override_value.clone()).ok();
        }
        // Fall back to policy
        self.policies.get(policy_id)
            .and_then(|v| serde_json::from_value(v.clone()).ok())
    }

    /// Check if a user's action is allowed under current policies.
    pub fn is_allowed(&self, action: &str, _user: &User) -> bool {
        // Default to allowed if no policy blocks it
        // Specific policy checks would be implemented per category
        let _ = action;
        true
    }

    /// Set an admin override for a policy.
    pub fn set_override(&mut self, policy_id: &str, value: serde_json::Value) {
        self.overrides.insert(policy_id.to_string(), value);
    }

    /// Remove an admin override.
    pub fn remove_override(&mut self, policy_id: &str) {
        self.overrides.remove(policy_id);
    }

    /// Check all policies for a given category.
    pub fn policies_in_category(&self, category: PolicyCategory) -> Vec<&String> {
        // In a full implementation, policies would be indexed by category
        let _ = category;
        vec![]
    }

    /// Clear all policies and overrides.
    pub fn reset(&mut self) {
        self.policies.clear();
        self.overrides.clear();
    }
}

impl Default for PolicyEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_policy_engine_new() {
        let engine = PolicyEngine::new();
        assert!(engine.policies.is_empty());
    }

    #[test]
    fn test_load_and_get_policy() {
        let mut engine = PolicyEngine::new();
        let policy = PolicyDefinition {
            id: "backup.retention_count".into(),
            name: "Backup Retention Count".into(),
            category: PolicyCategory::Backup,
            value: 30i32,
            default: 30i32,
            overridable_by: vec![UserRole::Admin],
            enabled: true,
            rationale: "Keep 30 days of backups".into(),
            set_by: None,
            set_at: None,
        };

        engine.load_policy(policy).unwrap();
        let value: i32 = engine.get_policy("backup.retention_count").unwrap();
        assert_eq!(value, 30);
    }

    #[test]
    fn test_override_takes_priority() {
        let mut engine = PolicyEngine::new();
        let policy = PolicyDefinition {
            id: "backup.retention_count".into(),
            name: "Backup Retention Count".into(),
            category: PolicyCategory::Backup,
            value: 30i32,
            default: 30i32,
            overridable_by: vec![UserRole::Admin],
            enabled: true,
            rationale: "Keep 30 days".into(),
            set_by: None,
            set_at: None,
        };

        engine.load_policy(policy).unwrap();
        engine.set_override("backup.retention_count", serde_json::json!(90));

        let value: i32 = engine.get_policy("backup.retention_count").unwrap();
        assert_eq!(value, 90);
    }
}
