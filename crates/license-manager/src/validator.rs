// TRRUSTT — License Validator. Ed25519 JWT validation.
use jsonwebtoken::{decode, DecodingKey, Validation, Algorithm};
use serde::{Deserialize, Serialize};
use shared::{LicenseInfo, PlanTier, Result};

/// JWT claims for a TRRUSTT license.
#[derive(Debug, Serialize, Deserialize)]
pub struct LicenseClaims {
    pub sub: String,         // Licensee email
    pub licensee: String,    // Company/name
    pub tier: String,        // free|pro|team|enterprise|oem
    pub seats: Option<i32>,  // Max seats
    pub features: Vec<String>,
    pub iat: i64,            // Issued at
    pub exp: i64,            // Expiration
    pub jti: String,         // License ID
}

/// The public key for license verification (Ed25519).
const LICENSE_PUBLIC_KEY: &[u8] = include_bytes!("../../../../config/license_public.pem");

pub struct LicenseValidator {
    decoding_key: DecodingKey,
}

impl LicenseValidator {
    pub fn new() -> Self {
        Self {
            decoding_key: DecodingKey::from_ed_pem(LICENSE_PUBLIC_KEY)
                .unwrap_or_else(|_| DecodingKey::from_secret(b"dev-secret-key-change-in-production")),
        }
    }

    /// Validate a JWT license token and return LicenseInfo.
    pub fn validate_token(&self, token: &str) -> Result<LicenseInfo> {
        let mut validation = Validation::new(Algorithm::EdDSA);
        validation.set_required_spec_claims(&["exp", "sub", "jti"]);

        let token_data = decode::<LicenseClaims>(token, &self.decoding_key, &validation)
            .map_err(|e| shared::AppError::License(format!("Invalid license: {}", e)))?;

        let claims = token_data.claims;

        let tier = match claims.tier.as_str() {
            "free" => PlanTier::Free,
            "pro" => PlanTier::Pro,
            "team" => PlanTier::Team,
            "enterprise" => PlanTier::Enterprise,
            "oem" => PlanTier::Oem,
            _ => return Err(shared::AppError::License(format!("Unknown tier: {}", claims.tier))),
        };

        Ok(LicenseInfo {
            licensee: claims.licensee,
            email: claims.sub,
            tier,
            seats: claims.seats,
            features: claims.features,
            issued_at: chrono::DateTime::from_timestamp(claims.iat, 0)
                .unwrap_or_default(),
            expires_at: chrono::DateTime::from_timestamp(claims.exp, 0)
                .unwrap_or_default(),
            license_id: claims.jti,
        })
    }

    /// Check if a license is expired.
    pub fn is_expired(&self, license: &LicenseInfo) -> bool {
        license.expires_at < chrono::Utc::now()
    }
}

impl Default for LicenseValidator {
    fn default() -> Self { Self::new() }
}
