//! Quality policies per device tier (Rule #19).

use crate::device_profiler::profile::{DeviceProfile, DeviceTier, ThermalLevel};
use crate::project_state::types::PreviewQuality;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QualityPolicy {
    pub preview_quality: PreviewQuality,
    pub max_proxy_height: u32,
    pub max_playback_fps: f64,
    pub allow_export: bool,
    pub allow_proxy_generation: bool,
    pub max_concurrent_jobs: usize,
    pub max_session_ram_mb: u64,
}

impl QualityPolicy {
    pub fn for_profile(profile: &DeviceProfile) -> Self {
        let effective = profile.effective_tier();
        let mut policy = match effective {
            DeviceTier::Low => Self {
                preview_quality: PreviewQuality::Low,
                max_proxy_height: 720,
                max_playback_fps: 24.0,
                allow_export: true,
                allow_proxy_generation: true,
                max_concurrent_jobs: 1,
                max_session_ram_mb: 384,
            },
            DeviceTier::Mid => Self {
                preview_quality: PreviewQuality::Medium,
                max_proxy_height: 1080,
                max_playback_fps: 30.0,
                allow_export: true,
                allow_proxy_generation: true,
                max_concurrent_jobs: 2,
                max_session_ram_mb: 512,
            },
            DeviceTier::High => Self {
                preview_quality: PreviewQuality::High,
                max_proxy_height: 1080,
                max_playback_fps: 60.0,
                allow_export: true,
                allow_proxy_generation: true,
                max_concurrent_jobs: 3,
                max_session_ram_mb: 768,
            },
        };

        if profile.thermal >= ThermalLevel::Hot {
            policy.allow_export = false;
            policy.max_concurrent_jobs = 1;
            policy.preview_quality = PreviewQuality::Low;
        }

        if profile.thermal == ThermalLevel::Critical {
            policy.allow_export = false;
            policy.allow_proxy_generation = false;
            policy.max_playback_fps = 24.0;
        }

        if profile.available_ram_mb < 1024 {
            policy.preview_quality = PreviewQuality::Low;
            policy.max_concurrent_jobs = 1;
        }

        policy
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::device_profiler::profile::GpuTier;

    #[test]
    fn low_tier_720p() {
        let p = DeviceProfile::from_hints(3072, 2048, GpuTier::Basic);
        let policy = QualityPolicy::for_profile(&p);
        assert_eq!(policy.max_proxy_height, 720);
    }

    #[test]
    fn critical_disables_export() {
        let p = DeviceProfile::from_hints(8192, 4096, GpuTier::Mid)
            .with_thermal(ThermalLevel::Critical);
        let policy = QualityPolicy::for_profile(&p);
        assert!(!policy.allow_export);
    }
}
