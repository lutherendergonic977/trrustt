// ═══════════════════════════════════════════════════════════════════════
// TRRUSTT — Config Validation
//
// Validates config values against their schema definitions.
// Ensures type safety, range constraints, and pattern matching.
// ═══════════════════════════════════════════════════════════════════════

use shared::{AppError, Result};

/// Validator for configuration values.
///
/// Checks values against type constraints, allowed values, ranges,
/// and regex patterns defined in the config registry.
pub struct ConfigValidator;

impl ConfigValidator {
    /// Create a new config validator.
    pub fn new() -> Self {
        Self
    }

    /// Validate that a string matches an optional regex pattern.
    pub fn validate_pattern(value: &str, pattern: Option<&str>) -> Result<()> {
        if let Some(pattern_str) = pattern {
            let re = regex::Regex::new(pattern_str)
                .map_err(|e| AppError::internal(format!("Invalid regex pattern: {}", e)))?;
            if !re.is_match(value) {
                return Err(AppError::ConfigValidation {
                    key: "unknown".into(),
                    reason: format!("Value '{}' does not match pattern '{}'", value, pattern_str),
                });
            }
        }
        Ok(())
    }

    /// Validate that a number is within a given range.
    pub fn validate_range(value: f64, min: Option<f64>, max: Option<f64>) -> Result<()> {
        if let Some(min_val) = min {
            if value < min_val {
                return Err(AppError::ConfigValidation {
                    key: "unknown".into(),
                    reason: format!("Value {} is below minimum {}", value, min_val),
                });
            }
        }
        if let Some(max_val) = max {
            if value > max_val {
                return Err(AppError::ConfigValidation {
                    key: "unknown".into(),
                    reason: format!("Value {} is above maximum {}", value, max_val),
                });
            }
        }
        Ok(())
    }

    /// Validate that a string is one of the allowed values.
    pub fn validate_allowed(value: &str, allowed: &[String]) -> Result<()> {
        if allowed.is_empty() {
            return Ok(());
        }
        if !allowed.iter().any(|a| a == value) {
            return Err(AppError::ConfigValidation {
                key: "unknown".into(),
                reason: format!("Value '{}' is not allowed. Allowed values: {:?}", value, allowed),
            });
        }
        Ok(())
    }

    /// Validate that a port number is valid (1-65535).
    pub fn validate_port(port: u16) -> Result<()> {
        if port == 0 {
            // 0 is allowed (means auto-detect)
            return Ok(());
        }
        if port < 1024 {
            return Err(AppError::ConfigValidation {
                key: "unknown".into(),
                reason: format!("Port {} is in the privileged range (1-1023)", port),
            });
        }
        Ok(())
    }

    /// Validate that a URL is well-formed.
    pub fn validate_url(url: &str) -> Result<()> {
        if url.is_empty() {
            return Ok(()); // Empty is allowed (not configured)
        }
        if !url.starts_with("http://") && !url.starts_with("https://") {
            return Err(AppError::ConfigValidation {
                key: "unknown".into(),
                reason: format!("URL '{}' must start with http:// or https://", url),
            });
        }
        Ok(())
    }

    /// Validate that a path doesn't contain dangerous characters.
    pub fn validate_path(path: &str) -> Result<()> {
        if path.contains("..") {
            return Err(AppError::ConfigValidation {
                key: "unknown".into(),
                reason: "Path must not contain '..' (directory traversal)".into(),
            });
        }
        Ok(())
    }
}

impl Default for ConfigValidator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_port_valid() {
        assert!(ConfigValidator::validate_port(0).is_ok());
        assert!(ConfigValidator::validate_port(54321).is_ok());
        assert!(ConfigValidator::validate_port(8080).is_ok());
    }

    #[test]
    fn test_validate_port_privileged() {
        assert!(ConfigValidator::validate_port(80).is_err());
    }

    #[test]
    fn test_validate_url() {
        assert!(ConfigValidator::validate_url("https://api.openai.com/v1").is_ok());
        assert!(ConfigValidator::validate_url("").is_ok());
        assert!(ConfigValidator::validate_url("not-a-url").is_err());
    }

    #[test]
    fn test_validate_path_no_traversal() {
        assert!(ConfigValidator::validate_path("/home/user/config").is_ok());
        assert!(ConfigValidator::validate_path("/etc/../passwd").is_err());
    }

    #[test]
    fn test_validate_range() {
        assert!(ConfigValidator::validate_range(0.5, Some(0.0), Some(2.0)).is_ok());
        assert!(ConfigValidator::validate_range(5.0, Some(0.0), Some(2.0)).is_err());
        assert!(ConfigValidator::validate_range(-1.0, Some(0.0), None).is_err());
    }

    #[test]
    fn test_validate_allowed() {
        let allowed = vec!["openai".into(), "anthropic".into()];
        assert!(ConfigValidator::validate_allowed("openai", &allowed).is_ok());
        assert!(ConfigValidator::validate_allowed("unknown", &allowed).is_err());
        assert!(ConfigValidator::validate_allowed("anything", &[]).is_ok());
    }
}
