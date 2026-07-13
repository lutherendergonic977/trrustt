// TRRUSTT — License Cache. AES-256-GCM encrypted local license storage.
use std::path::PathBuf;
use std::sync::Mutex;
use aes_gcm::{Aes256Gcm, Key, Nonce, aead::{Aead, KeyInit}};
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use ring::rand::{SecureRandom, SystemRandom};
use shared::{LicenseInfo, Result};

pub struct LicenseCache {
    cache_path: PathBuf,
    key: [u8; 32],
    cached: Mutex<Option<LicenseInfo>>,
}

impl LicenseCache {
    pub fn new() -> Self {
        let cache_dir = dirs::config_dir().unwrap_or_else(|| PathBuf::from(".")).join(".trrustt");
        let cache_path = cache_dir.join("license.cache");
        let key = Self::get_or_create_key(&cache_dir);
        Self { cache_path, key, cached: Mutex::new(None) }
    }

    fn get_or_create_key(cache_dir: &std::path::Path) -> [u8; 32] {
        let key_path = cache_dir.join("license-key");
        if key_path.exists() {
            if let Ok(encoded) = std::fs::read_to_string(&key_path) {
                if let Ok(decoded) = BASE64.decode(encoded.trim()) {
                    if decoded.len() == 32 {
                        let mut key = [0u8; 32];
                        key.copy_from_slice(&decoded);
                        return key;
                    }
                }
            }
        }
        let rng = SystemRandom::new();
        let mut key = [0u8; 32];
        rng.fill(&mut key).ok();
        if let Some(parent) = key_path.parent() {
            std::fs::create_dir_all(parent).ok();
        }
        std::fs::write(&key_path, BASE64.encode(key)).ok();
        key
    }

    pub fn store(&self, _token: &str, license: &LicenseInfo) -> Result<()> {
        // Encrypt license with AES-256-GCM and write to cache file
        let serialized = serde_json::to_vec(license)
            .map_err(|e| shared::AppError::internal(format!("Serialize error: {}", e)))?;

        let rng = SystemRandom::new();
        let mut nonce_bytes = [0u8; 12];
        rng.fill(&mut nonce_bytes)
            .map_err(|_| shared::AppError::internal("Nonce generation failed"))?;

        let key = Key::<Aes256Gcm>::from_slice(&self.key);
        let cipher = Aes256Gcm::new(key);
        let nonce = Nonce::from_slice(&nonce_bytes);
        let ciphertext = cipher.encrypt(nonce, serialized.as_ref())
            .map_err(|_| shared::AppError::internal("Encryption failed"))?;

        let mut combined = nonce_bytes.to_vec();
        combined.extend_from_slice(&ciphertext);

        if let Some(parent) = self.cache_path.parent() {
            std::fs::create_dir_all(parent).ok();
        }
        std::fs::write(&self.cache_path, BASE64.encode(&combined))
            .map_err(|e| shared::AppError::internal(format!("Write error: {}", e)))?;

        *self.cached.lock().unwrap() = Some(license.clone());
        Ok(())
    }

    pub fn get(&self, _token: &str) -> Option<LicenseInfo> {
        self.get_cached()
    }

    pub fn get_cached(&self) -> Option<LicenseInfo> {
        // Check in-memory cache first
        if let Some(ref cached) = *self.cached.lock().unwrap() {
            return Some(cached.clone());
        }

        // Try to decrypt from disk
        if self.cache_path.exists() {
            let encoded = std::fs::read_to_string(&self.cache_path).ok()?;
            let combined = BASE64.decode(encoded.trim()).ok()?;
            if combined.len() < 12 { return None; }
            let (nonce_bytes, ciphertext) = combined.split_at(12);
            let key = Key::<Aes256Gcm>::from_slice(&self.key);
            let cipher = Aes256Gcm::new(key);
            let nonce = Nonce::from_slice(nonce_bytes);
            let plaintext = cipher.decrypt(nonce, ciphertext).ok()?;
            let license: LicenseInfo = serde_json::from_slice(&plaintext).ok()?;
            *self.cached.lock().unwrap() = Some(license.clone());
            return Some(license);
        }

        None
    }
}

impl Default for LicenseCache { fn default() -> Self { Self::new() } }

#[cfg(test)]
mod tests {
    use super::*;
    use shared::PlanTier;
    use chrono::Utc;

    #[test]
    fn test_cache_roundtrip() {
        let cache = LicenseCache::new();
        let license = LicenseInfo {
            licensee: "Test Corp".into(), email: "test@corp.com".into(),
            tier: PlanTier::Pro, seats: Some(5), features: vec!["advanced_dax".into()],
            issued_at: Utc::now(), expires_at: Utc::now() + chrono::Duration::days(365),
            license_id: "LIC-001".into(),
        };
        cache.store("test-jwt-token", &license).unwrap();
        let retrieved = cache.get("test-jwt-token").unwrap();
        assert_eq!(retrieved.licensee, "Test Corp");
        assert_eq!(retrieved.tier, PlanTier::Pro);
    }
}
}
