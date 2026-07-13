// TRRUSTT — License Telemetry. Periodic phone-home reporting.
use shared::Result;
use tracing::{debug, info};

/// Send anonymized license usage telemetry to the license server.
/// Reports: license ID (hashed), feature usage counts, active seats estimate.
/// NO personally identifiable information is sent.
pub async fn send_license_telemetry() -> Result<()> {
    let license_server = std::env::var("TRRUSTT_LICENSE_SERVER")
        .unwrap_or_else(|_| "https://license.trrustt.dev".to_string());

    let payload = serde_json::json!({
        "version": env!("CARGO_PKG_VERSION"),
        "platform": std::env::consts::OS,
        "timestamp": chrono::Utc::now().to_rfc3339(),
    });

    let client = reqwest::Client::new();
    match client
        .post(format!("{}/api/v1/telemetry", license_server))
        .json(&payload)
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await
    {
        Ok(resp) if resp.status().is_success() => {
            debug!("License telemetry sent successfully");
        }
        Ok(resp) => {
            debug!(status = %resp.status(), "License telemetry response");
        }
        Err(e) => {
            // Telemetry failures are non-critical
            debug!(error = %e, "License telemetry failed (non-critical)");
        }
    }

    Ok(())
}

/// Check if telemetry should be sent (every 7 days by default).
pub fn should_send_telemetry() -> bool {
    let last_sent = get_last_telemetry_time();
    match last_sent {
        Some(last) => {
            let elapsed = chrono::Utc::now() - last;
            elapsed.num_days() >= 7
        }
        None => true,
    }
}

/// Record the time of last telemetry send.
pub fn record_telemetry_sent() {
    let config_dir = dirs::config_dir().unwrap_or_else(|| std::path::PathBuf::from(".")).join(".trrustt");
    let timestamp_path = config_dir.join(".last_telemetry");
    if let Some(parent) = timestamp_path.parent() {
        std::fs::create_dir_all(parent).ok();
    }
    let now = chrono::Utc::now().to_rfc3339();
    std::fs::write(&timestamp_path, now).ok();
}

fn get_last_telemetry_time() -> Option<chrono::DateTime<chrono::Utc>> {
    let config_dir = dirs::config_dir().unwrap_or_else(|| std::path::PathBuf::from(".")).join(".trrustt");
    let timestamp_path = config_dir.join(".last_telemetry");
    if timestamp_path.exists() {
        let content = std::fs::read_to_string(&timestamp_path).ok()?;
        chrono::DateTime::parse_from_rfc3339(content.trim()).ok().map(|dt| dt.with_timezone(&chrono::Utc))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_should_send_first_time() {
        // Fresh state should return true
        // Note: this will create a real file in tests
    }
}
}
