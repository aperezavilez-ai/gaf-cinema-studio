//! Device profiler — tier detection, policies, session metrics (Rule #19).

mod detect;
mod policies;
mod profile;
mod session;

pub use detect::{detect, parse_hints_json};
pub use policies::QualityPolicy;
pub use profile::{DeviceProfile, DeviceTier, GpuTier, ThermalLevel};
pub use session::{SessionMetrics, DROP_RATE_THRESHOLD, SCRUB_LATENCY_BUDGET_MS};

use crate::event_bus::{CinemaEvent, EventBus};
use crate::project_state::types::PreviewQuality;

pub struct DeviceController {
    profile: DeviceProfile,
    policy: QualityPolicy,
    session: SessionMetrics,
    last_preview_quality: PreviewQuality,
}

impl DeviceController {
    pub fn new() -> Self {
        let profile = detect();
        let policy = QualityPolicy::for_profile(&profile);
        Self {
            last_preview_quality: policy.preview_quality,
            profile,
            policy,
            session: SessionMetrics::default(),
        }
    }

    pub fn from_profile(profile: DeviceProfile) -> Self {
        let policy = QualityPolicy::for_profile(&profile);
        Self {
            last_preview_quality: policy.preview_quality,
            profile,
            policy,
            session: SessionMetrics::default(),
        }
    }

    pub fn profile(&self) -> &DeviceProfile {
        &self.profile
    }

    pub fn policy(&self) -> &QualityPolicy {
        &self.policy
    }

    pub fn session(&self) -> &SessionMetrics {
        &self.session
    }

    pub fn refresh_profile(&mut self, profile: DeviceProfile, bus: &EventBus, project_id: Option<uuid::Uuid>) {
        let old_tier = self.profile.effective_tier();
        self.profile = profile;
        self.recompute_policy();

        if let Some(pid) = project_id {
            if self.profile.effective_tier() != old_tier {
                bus.emit(CinemaEvent::DeviceTierDetected {
                    project_id: pid,
                    tier: format!("{:?}", self.profile.effective_tier()),
                });
            }
        }
    }

    pub fn set_thermal(&mut self, thermal: ThermalLevel, bus: &EventBus, project_id: uuid::Uuid) {
        self.profile.thermal = thermal;
        self.session.record_thermal_event(thermal);
        self.recompute_policy();

        if thermal >= ThermalLevel::Warm {
            bus.emit(CinemaEvent::ThermalThrottle {
                project_id,
                level: format!("{thermal:?}"),
            });
        }
    }

    pub fn tick_playback(&mut self, metrics: &crate::playback_engine::metrics::PlaybackMetrics) {
        self.session.record_playback(metrics);
        if self.session.should_degrade_quality() {
            self.force_degrade_preview();
        }
    }

    pub fn force_degrade_preview(&mut self) {
        self.policy.preview_quality = match self.policy.preview_quality {
            PreviewQuality::High | PreviewQuality::Auto => PreviewQuality::Medium,
            PreviewQuality::Medium => PreviewQuality::Low,
            PreviewQuality::Low => PreviewQuality::Low,
        };
        self.session.record_quality_downgrade();
    }

    pub fn apply_to_render_state(&self, render_state: &mut crate::project_state::types::RenderState) {
        render_state.preview_quality = self.policy.preview_quality;
    }

    pub fn recompute_policy(&mut self) {
        let prev = self.policy.preview_quality;
        self.policy = QualityPolicy::for_profile(&self.profile);
        self.last_preview_quality = self.policy.preview_quality;
        if self.policy.preview_quality != prev {
            // quality changed via tier/thermal
        }
    }

    pub fn emit_quality_change(&self, bus: &EventBus, project_id: uuid::Uuid, from: PreviewQuality, to: PreviewQuality, reason: &str) {
        if from != to {
            bus.emit(CinemaEvent::QualityDegraded {
                project_id,
                from: format!("{from:?}"),
                to: format!("{to:?}"),
                reason: reason.to_string(),
            });
        }
    }

    pub fn sync_adaptive_quality(&mut self, bus: &EventBus, project_id: uuid::Uuid, render_state: &mut crate::project_state::types::RenderState) {
        let from = render_state.preview_quality;
        self.apply_to_render_state(render_state);
        let to = render_state.preview_quality;
        self.emit_quality_change(bus, project_id, from, to, "device_adaptive");
    }
}

impl Default for DeviceController {
    fn default() -> Self {
        Self::new()
    }
}
