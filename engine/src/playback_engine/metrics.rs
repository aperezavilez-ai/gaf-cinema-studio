use std::time::Instant;

#[derive(Debug, Clone, Default)]
pub struct PlaybackMetrics {
    pub frames_requested: u64,
    pub frames_dropped: u64,
    pub scrubs: u64,
    pub last_scrub_latency_us: u64,
    pub max_scrub_latency_us: u64,
    pub ticks: u64,
}

impl PlaybackMetrics {
    pub fn drop_rate(&self) -> f64 {
        if self.frames_requested == 0 {
            0.0
        } else {
            self.frames_dropped as f64 / self.frames_requested as f64
        }
    }

    pub fn record_frame_request(&mut self, dropped: bool) {
        self.frames_requested += 1;
        if dropped {
            self.frames_dropped += 1;
        }
    }

    pub fn record_scrub(&mut self, latency: std::time::Duration) {
        self.scrubs += 1;
        let us = latency.as_micros() as u64;
        self.last_scrub_latency_us = us;
        if us > self.max_scrub_latency_us {
            self.max_scrub_latency_us = us;
        }
    }

    pub fn last_scrub_latency_ms(&self) -> f64 {
        self.last_scrub_latency_us as f64 / 1000.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn drop_rate_calculation() {
        let mut m = PlaybackMetrics::default();
        m.record_frame_request(false);
        m.record_frame_request(true);
        assert!((m.drop_rate() - 0.5).abs() < f64::EPSILON);
    }
}
