// ═══════════════════════════════════════════════════════════════════════
// TRRUSTT — Encryption Engine
//
// AES-256-GCM encryption for sensitive config values.
// Uses the `ring` crate for cryptographic operations.
// Each installation generates a unique key stored in ~/.trrustt/.
// ═══════════════════════════════════════════════════════════════════════

use std::path::{Path, PathBuf};

use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use ring::rand::{SecureRandom, SystemRandom};
use shared::{AppError, Result};
use tracing::{debug, error, info, instrument};

/// Encryption engine for sensitive configuration values.
///
/// Uses AES-256-GCM (authenticated encryption with associated data).
/// The encryption key is stored in `~/.trrustt/encryption-key` and is
/// automatically generated on first use.
pub struct EncryptionEngine {
    /// The AES-256-GCM cipher instance.
    cipher: Aes256Gcm,

    /// Path to the encryption key file.
    key_path: PathBuf,
}

impl EncryptionEngine {
    /// Create a new encryption engine.
    ///
    /// Loads or generates the encryption key from the config directory.
    ///
    /// # Arguments
    /// * `config_dir` - The base config directory (~/.trrustt/).
    ///
    /// # Errors
    /// Returns an error if key generation or loading fails.
    #[instrument(skip(config_dir))]
    pub fn new(config_dir: &Path) -> Result<Self> {
        let key_path = config_dir.join("encryption-key");

        let key_bytes = if key_path.exists() {
            debug!("Loading existing encryption key");
            Self::load_key(&key_path)?
        } else {
            info!("Generating new AES-256-GCM encryption key");
            let key = Self::generate_key()?;
            Self::save_key(&key_path, &key)?;
            key
        };

        let key = Key::<Aes256Gcm>::from_slice(&key_bytes);
        let cipher = Aes256Gcm::new(key);

        Ok(Self { cipher, key_path })
    }

    /// Encrypt a plaintext string.
    ///
    /// Returns a base64-encoded string containing the nonce + ciphertext.
    /// Format: `base64(nonce (12 bytes) || ciphertext)`.
    ///
    /// # Example
    /// ```rust,no_run
    /// use config_engine::EncryptionEngine;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let engine = EncryptionEngine::new(std::path::Path::new("/tmp"))?;
    /// let encrypted = engine.encrypt("my-secret-api-key")?;
    /// let decrypted = engine.decrypt(&encrypted)?;
    /// assert_eq!(decrypted, "my-secret-api-key");
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(skip(self, plaintext))]
    pub fn encrypt(&self, plaintext: &str) -> Result<String> {
        let rng = SystemRandom::new();
        let mut nonce_bytes = [0u8; 12];
        rng.fill(&mut nonce_bytes)
            .map_err(|_| AppError::internal("Failed to generate nonce"))?;

        let nonce = Nonce::from_slice(&nonce_bytes);
        let ciphertext = self.cipher.encrypt(nonce, plaintext.as_bytes())
            .map_err(|e| AppError::internal(format!("Encryption failed: {}", e)))?;

        // Prepend nonce to ciphertext
        let mut combined = nonce_bytes.to_vec();
        combined.extend_from_slice(&ciphertext);

        Ok(BASE64.encode(&combined))
    }

    /// Decrypt a base64-encoded encrypted string.
    ///
    /// Expects the format produced by `encrypt()`:
    /// `base64(nonce (12 bytes) || ciphertext)`.
    #[instrument(skip(self, encrypted))]
    pub fn decrypt(&self, encrypted: &str) -> Result<String> {
        let combined = BASE64.decode(encrypted)
            .map_err(|e| AppError::internal(format!("Base64 decode failed: {}", e)))?;

        if combined.len() < 12 {
            return Err(AppError::internal("Invalid encrypted data: too short"));
        }

        let (nonce_bytes, ciphertext) = combined.split_at(12);
        let nonce = Nonce::from_slice(nonce_bytes);

        let plaintext = self.cipher.decrypt(nonce, ciphertext)
            .map_err(|e| AppError::internal(format!("Decryption failed: {}", e)))?;

        String::from_utf8(plaintext)
            .map_err(|e| AppError::internal(format!("UTF-8 decode failed: {}", e)))
    }

    /// Check if a string appears to be encrypted (base64 + length check).
    pub fn is_encrypted(value: &str) -> bool {
        // Heuristic: encrypted values are base64-encoded and longer than 16 bytes raw
        value.len() > 24 && BASE64.decode(value).is_ok()
    }

    /// Get the key file path.
    pub fn key_path(&self) -> &Path {
        &self.key_path
    }

    // ── Private key management ──────────────────────────────────────

    /// Generate a new random 256-bit key.
    fn generate_key() -> Result<Vec<u8>> {
        let rng = SystemRandom::new();
        let mut key = [0u8; 32]; // 256 bits
        rng.fill(&mut key)
            .map_err(|_| AppError::internal("Failed to generate encryption key"))?;
        Ok(key.to_vec())
    }

    /// Load the key from disk.
    fn load_key(path: &Path) -> Result<Vec<u8>> {
        let encoded = std::fs::read_to_string(path)
            .map_err(|e| AppError::io(path.to_path_buf(), e))?;
        BASE64.decode(encoded.trim())
            .map_err(|e| AppError::internal(format!("Failed to decode key: {}", e)))
    }

    /// Save the key to disk with restricted permissions.
    fn save_key(path: &Path, key: &[u8]) -> Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| AppError::io(parent.to_path_buf(), e))?;
        }

        let encoded = BASE64.encode(key);

        // On Unix, set restrictive permissions (owner read only)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::write(path, &encoded)
                .map_err(|e| AppError::io(path.to_path_buf(), e))?;
            std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600))
                .map_err(|e| AppError::io(path.to_path_buf(), e))?;
        }

        #[cfg(not(unix))]
        {
            std::fs::write(path, &encoded)
                .map_err(|e| AppError::io(path.to_path_buf(), e))?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let dir = TempDir::new().unwrap();
        let engine = EncryptionEngine::new(dir.path()).unwrap();

        let plaintext = "sk-this-is-a-secret-api-key-12345";
        let encrypted = engine.encrypt(plaintext).unwrap();
        let decrypted = engine.decrypt(&encrypted).unwrap();

        assert_eq!(decrypted, plaintext);
        assert_ne!(encrypted, plaintext);
    }

    #[test]
    fn test_encrypt_produces_different_outputs() {
        let dir = TempDir::new().unwrap();
        let engine = EncryptionEngine::new(dir.path()).unwrap();

        let encrypted1 = engine.encrypt("same-text").unwrap();
        let encrypted2 = engine.encrypt("same-text").unwrap();

        // Same plaintext should produce different ciphertexts due to random nonce
        assert_ne!(encrypted1, encrypted2);
    }

    #[test]
    fn test_is_encrypted_heuristic() {
        assert!(EncryptionEngine::is_encrypted("dGhpc2lzYWJhc2U2NGVuY29kZWRzdHJpbmc="));
        assert!(!EncryptionEngine::is_encrypted("not-encrypted"));
    }

    #[test]
    fn test_key_persistence() {
        let dir = TempDir::new().unwrap();
        let engine = EncryptionEngine::new(dir.path()).unwrap();
        assert!(engine.key_path().exists());

        // Re-load should use the same key
        let engine2 = EncryptionEngine::new(dir.path()).unwrap();
        let encrypted = engine.encrypt("test").unwrap();
        let decrypted = engine2.decrypt(&encrypted).unwrap();
        assert_eq!(decrypted, "test");
    }
}
