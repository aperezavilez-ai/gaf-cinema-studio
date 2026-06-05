//! Session performance metrics and auto-degradation triggers.

use std::time::{Duration, Instant};

use crate::device_profiler::profile::ThermalLevel;
use crate::playback_engine::metrics::PlaybackMetrics;

pub const DROP_RATE_THRESHOLD: f64 = 0.05;
pub const SCRUB_LATENCY_BUDGET_MS: f64 = 100.0;
pub const MAX_RAM_GROWTH_MB: u64 = 128;

#[derive(Debug, Clone)]
pub struct SessionMetrics {
    pub started_at: Instant,
    pub peak_drop_rate: f64,
    pub peak_scrub_latency_ms: f64,
    pub frame_ticks: u64,
    pub quality_downgrades: u32,
    pub thermal_events: u32,
    pub estimated_ram_mb: u64,
    pub leak_suspects: u32,
}

impl Default for SessionMetrics {
    fn default() -> Self {
        Self {
            started_at: Instant::now(),
            peak_drop_rate: 0.0,
            peak_scrub_latency_ms: 0.0,
            frame_ticks: 0,
            quality_downgrades: 0,
            thermal_events: 0,
            estimated_ram_mb: 0,
            leak_suspects: 0,
        }
    }
}

impl SessionMetrics {
    pub fn record_playback(&mut self, metrics: &PlaybackMetrics) {
        self.frame_ticks += 1;
        let drop = metrics.drop_rate();
        if drop > self.peak_drop_rate {
            self.peak_drop_rate = drop;
        }
        let scrub = metrics.last_scrub_latency_ms();
        if scrub > self.peak_scrub_latency_ms {
            self.peak_scrub_latency_ms = scrub;
        }
    }

    pub fn should_degrade_quality(&self) -> bool {
        self.peak_drop_rate > DROP_RATE_THRESHOLD
            || self.peak_scrub_latency_ms > SCRUB_LATENCY_BUDGET_MS
    }

    pub fn session_duration(&self) -> Duration {
        self.started_at.elapsed()
    }

    pub fn record_quality_downgrade(&mut self) {
        self.quality_downgrades += 1;
    }

    pub fn record_thermal_event(&mut self, level: ThermalLevel) {
        if level >= ThermalLevel::Warm {
            self.thermal_events += 1;
        }
    }

    pub fn update_ram_estimate(&mut self, mb: u64) {
        if mb > self.estimated_ram_mb {
            let growth = mb - self.estimated_ram_mb;
            if growth > MAX_RAM_GROWTH_MB && self.estimated_ram_mb > 0 {
                self.leak_suspects += 1;
            }
        }
        self.estimated_ram_mb = mb;
    }

    pub fn is_stable(&self) -> bool {
        self.leak_suspects == 0 && self.peak_drop_rate <= DROP_RATE_THRESHOLD
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::playback_engine::metrics::PlaybackMetrics;

    #[test]
    fn triggers_degrade_on_high_drop_rate() {
        let mut playback = PlaybackMetrics::default();
        for _ in 0..10 {
            playback.record_frame_request(true);
        }
        let mut session = SessionMetrics::default();
        session.record_playback(&playback);
        assert!(session.should_degrade_quality());
    }
}
