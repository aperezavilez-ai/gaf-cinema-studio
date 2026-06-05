use std::time::{Duration, Instant};

use uuid::Uuid;

use crate::playback_engine::metrics::PlaybackMetrics;
use crate::project_state::types::ProjectState;
use crate::timeline_engine::{resolve_frame, FrameComposition};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlaybackMode {
    Stopped,
    Playing,
    Paused,
    Scrubbing,
}

#[derive(Debug, Clone)]
pub struct PlaybackStatus {
    pub mode: PlaybackMode,
    pub playhead_ms: u64,
    pub duration_ms: u64,
    pub frame_rate: f64,
    pub metrics: PlaybackMetrics,
}

pub struct PlaybackEngine {
    mode: PlaybackMode,
    playhead_ms: u64,
    duration_ms: u64,
    frame_rate: f64,
    last_tick: Option<Instant>,
    metrics: PlaybackMetrics,
    project_id: Option<Uuid>,
}

impl PlaybackEngine {
    pub fn new(frame_rate: f64) -> Self {
        Self {
            mode: PlaybackMode::Stopped,
            playhead_ms: 0,
            duration_ms: 0,
            frame_rate,
            last_tick: None,
            metrics: PlaybackMetrics::default(),
            project_id: None,
        }
    }

    pub fn sync_from_state(&mut self, state: &ProjectState) {
        self.playhead_ms = state.timeline.playhead_ms;
        self.duration_ms = state.timeline.duration_ms;
        self.frame_rate = state.metadata.frame_rate;
        self.project_id = Some(state.project_id);
    }

    pub fn playhead_ms(&self) -> u64 {
        self.playhead_ms
    }

    pub fn mode(&self) -> PlaybackMode {
        self.mode
    }

    pub fn metrics(&self) -> &PlaybackMetrics {
        &self.metrics
    }

    pub fn status(&self) -> PlaybackStatus {
        PlaybackStatus {
            mode: self.mode,
            playhead_ms: self.playhead_ms,
            duration_ms: self.duration_ms,
            frame_rate: self.frame_rate,
            metrics: self.metrics.clone(),
        }
    }

    pub fn play(&mut self) {
        if self.duration_ms == 0 {
            return;
        }
        self.mode = PlaybackMode::Playing;
        self.last_tick = Some(Instant::now());
    }

    pub fn pause(&mut self) {
        if self.mode == PlaybackMode::Playing {
            self.mode = PlaybackMode::Paused;
        }
        self.last_tick = None;
    }

    pub fn stop(&mut self) {
        self.mode = PlaybackMode::Stopped;
        self.playhead_ms = 0;
        self.last_tick = None;
    }

    /// Seek playhead. Returns frame composition for immediate preview.
    pub fn scrub(&mut self, state: &ProjectState, time_ms: u64) -> FrameComposition {
        let start = Instant::now();
        self.mode = PlaybackMode::Scrubbing;
        self.playhead_ms = clamp_time(time_ms, self.duration_ms);
        let frame = resolve_frame(state, self.playhead_ms);
        self.metrics.record_scrub(start.elapsed());
        self.metrics.record_frame_request(false);
        frame
    }

    /// Advance playback by real elapsed time. Returns current frame if playing.
    pub fn tick(&mut self, state: &ProjectState) -> Option<FrameComposition> {
        self.metrics.ticks += 1;

        if self.mode != PlaybackMode::Playing {
            return None;
        }

        let now = Instant::now();
        let delta = self
            .last_tick
            .map(|last| now.duration_since(last))
            .unwrap_or(Duration::ZERO);
        self.last_tick = Some(now);

        if delta.is_zero() {
            return None;
        }

        let frame_duration = frame_duration_ms(self.frame_rate);
        let advance_ms = delta.as_millis() as u64;

        // Frame pacing: drop frames if we're behind (> 1 frame late)
        let dropped = advance_ms > frame_duration * 2;
        self.metrics.record_frame_request(dropped);

        self.playhead_ms = self.playhead_ms.saturating_add(advance_ms);
        if self.playhead_ms >= self.duration_ms {
            self.playhead_ms = self.duration_ms;
            self.mode = PlaybackMode::Paused;
            self.last_tick = None;
        }

        Some(resolve_frame(state, self.playhead_ms))
    }

    pub fn request_frame(&mut self, state: &ProjectState, time_ms: u64) -> FrameComposition {
        self.metrics.record_frame_request(false);
        resolve_frame(state, clamp_time(time_ms, self.duration_ms))
    }

    pub fn apply_playhead_to_state(&self, state: &mut ProjectState) {
        state.timeline.playhead_ms = self.playhead_ms;
    }
}

fn clamp_time(time_ms: u64, duration_ms: u64) -> u64 {
    if duration_ms == 0 {
        0
    } else {
        time_ms.min(duration_ms)
    }
}

fn frame_duration_ms(frame_rate: f64) -> u64 {
    if frame_rate <= 0.0 {
        33
    } else {
        (1000.0 / frame_rate).round() as u64
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::project_state::types::{default_project_settings, MediaStatus, ProjectState};
    use crate::timeline_engine::{add_clip, AddClipParams};
    use chrono::Utc;
    use std::thread;

    fn state_with_clip() -> ProjectState {
        let mut state = ProjectState::new("Playback", "/tmp/p.csproj", default_project_settings());
        let media_id = Uuid::new_v4();
        state.media.push(crate::project_state::types::MediaAsset {
            id: media_id,
            original_path: "/v/clip.mp4".into(),
            proxy_path: None,
            thumbnail_path: None,
            file_name: "clip.mp4".into(),
            mime_type: "video/mp4".into(),
            duration_ms: 10_000,
            width: 1920,
            height: 1080,
            file_size_bytes: 1,
            status: MediaStatus::Ready,
            imported_at: Utc::now(),
            checksum: None,
        });
        state.timeline = add_clip(
            &state,
            AddClipParams {
                media_id,
                track_id: None,
                start_ms: None,
            },
        )
        .unwrap();
        state
    }

    #[test]
    fn scrub_under_100ms() {
        let state = state_with_clip();
        let mut engine = PlaybackEngine::new(24.0);
        engine.sync_from_state(&state);

        for t in (0..100).map(|i| i * 100) {
            engine.scrub(&state, t);
        }

        assert!(
            engine.metrics().last_scrub_latency_ms() < 100.0,
            "scrub latency {}ms exceeds 100ms budget",
            engine.metrics().last_scrub_latency_ms()
        );
    }

    #[test]
    fn tick_advances_playhead() {
        let state = state_with_clip();
        let mut engine = PlaybackEngine::new(24.0);
        engine.sync_from_state(&state);
        engine.play();

        thread::sleep(Duration::from_millis(50));
        engine.tick(&state);

        assert!(engine.playhead_ms() > 0);
    }

    #[test]
    fn stops_at_end() {
        let mut state = state_with_clip();
        state.timeline.duration_ms = 100;
        state.timeline.playhead_ms = 0;

        let mut engine = PlaybackEngine::new(24.0);
        engine.sync_from_state(&state);
        engine.play();
        engine.playhead_ms = 100;
        engine.mode = PlaybackMode::Playing;

        engine.tick(&state);
        assert_eq!(engine.mode(), PlaybackMode::Paused);
    }
}
