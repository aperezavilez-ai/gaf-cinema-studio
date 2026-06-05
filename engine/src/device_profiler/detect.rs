//! Platform detection — native shells pass hints; desktop uses heuristics + env override.

use std::env;

use crate::device_profiler::profile::{DeviceProfile, GpuTier, ThermalLevel};

/// Detect device profile. Mobile shells override via `CINEMASTUDIO_DEVICE_HINTS` JSON env.
pub fn detect() -> DeviceProfile {
    if let Ok(json) = env::var("CINEMASTUDIO_DEVICE_HINTS") {
        if let Ok(profile) = parse_hints_json(&json) {
            return profile;
        }
    }

    detect_desktop()
}

pub fn parse_hints_json(json: &str) -> Result<DeviceProfile, serde_json::Error> {
    #[derive(serde::Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct Hints {
        total_ram_mb: u64,
        available_ram_mb: u64,
        #[serde(default = "default_gpu")]
        gpu_tier: String,
        #[serde(default)]
        thermal: String,
        battery_percent: Option<u8>,
    }

    fn default_gpu() -> String {
        "mid".into()
    }

    let h: Hints = serde_json::from_str(json)?;
    let gpu = match h.gpu_tier.to_lowercase().as_str() {
        "basic" | "low" => GpuTier::Basic,
        "flagship" | "high" => GpuTier::Flagship,
        _ => GpuTier::Mid,
    };
    let thermal = match h.thermal.to_lowercase().as_str() {
        "warm" => ThermalLevel::Warm,
        "hot" => ThermalLevel::Hot,
        "critical" => ThermalLevel::Critical,
        _ => ThermalLevel::Normal,
    };

    let mut profile = DeviceProfile::from_hints(h.total_ram_mb, h.available_ram_mb, gpu)
        .with_thermal(thermal);
    if let Some(b) = h.battery_percent {
        profile = profile.with_battery(b);
    }
    Ok(profile)
}

fn detect_desktop() -> DeviceProfile {
    // Without sysinfo: conservative mid-tier default for dev machines
    let total = env::var("CINEMASTUDIO_RAM_MB")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(8192);
    let available = (total as f64 * 0.6) as u64;
    DeviceProfile::from_hints(total, available, GpuTier::Mid)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::device_profiler::profile::DeviceTier;

    #[test]
    fn parses_hints_json() {
        let json = r#"{"totalRamMb":12288,"availableRamMb":6000,"gpuTier":"flagship"}"#;
        let p = parse_hints_json(json).unwrap();
        assert_eq!(p.tier, DeviceTier::High);
    }
}
