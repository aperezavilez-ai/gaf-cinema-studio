//! VideoEngine — coordinates timeline + playback around ProjectState.
//! Single entry point for UI/native shells (Rule #21).

use uuid::Uuid;

use crate::error::Result;
use crate::event_bus::{CinemaEvent, EventBus};
use crate::playback_engine::{PlaybackEngine, PlaybackMode, PlaybackStatus};
use crate::project_state::types::ProjectState;
use crate::timeline_engine::{
    add_clip, move_clip, remove_clip, split_clip, trim_clip, AddClipParams, FrameComposition,
};

pub struct VideoEngine {
    playback: PlaybackEngine,
}

impl VideoEngine {
    pub fn new(frame_rate: f64) -> Self {
        Self {
            playback: PlaybackEngine::new(frame_rate),
        }
    }

    pub fn from_state(state: &ProjectState) -> Self {
        let mut engine = Self::new(state.metadata.frame_rate);
        engine.playback.sync_from_state(state);
        engine
    }

    pub fn sync_from_state(&mut self, state: &ProjectState) {
        self.playback.sync_from_state(state);
    }

    pub fn playback(&self) -> &PlaybackEngine {
        &self.playback
    }

    pub fn playback_mut(&mut self) -> &mut PlaybackEngine {
        &mut self.playback
    }

    pub fn status(&self) -> PlaybackStatus {
        self.playback.status()
    }

    // ── Timeline ops (return new timeline; manager persists) ──

    pub fn add_clip(state: &ProjectState, params: AddClipParams) -> Result<crate::project_state::types::Timeline> {
        add_clip(state, params)
    }

    pub fn remove_clip(state: &ProjectState, clip_id: Uuid) -> Result<crate::project_state::types::Timeline> {
        remove_clip(state, clip_id)
    }

    pub fn move_clip(state: &ProjectState, clip_id: Uuid, new_start_ms: u64) -> Result<crate::project_state::types::Timeline> {
        move_clip(state, clip_id, new_start_ms)
    }

    pub fn trim_clip(
        state: &ProjectState,
        clip_id: Uuid,
        source_in_ms: u64,
        source_out_ms: u64,
    ) -> Result<crate::project_state::types::Timeline> {
        trim_clip(state, clip_id, source_in_ms, source_out_ms)
    }

    pub fn split_clip(state: &ProjectState, clip_id: Uuid, at_ms: u64) -> Result<crate::project_state::types::Timeline> {
        split_clip(state, clip_id, at_ms)
    }

    // ── Playback ops ──

    pub fn play(&mut self, event_bus: &EventBus, project_id: Uuid) {
        self.playback.play();
        event_bus.emit(CinemaEvent::PlaybackStarted {
            project_id,
            time_ms: self.playback.playhead_ms(),
        });
    }

    pub fn pause(&mut self, event_bus: &EventBus, project_id: Uuid) {
        self.playback.pause();
        event_bus.emit(CinemaEvent::PlaybackStopped {
            project_id,
            time_ms: self.playback.playhead_ms(),
        });
    }

    pub fn scrub(&mut self, state: &ProjectState, time_ms: u64) -> FrameComposition {
        let frame = self.playback.scrub(state, time_ms);
        // playhead sync handled by manager after scrub
        frame
    }

    pub fn tick(&mut self, state: &ProjectState, event_bus: &EventBus) -> Option<FrameComposition> {
        let result = self.playback.tick(state);
        if let Some(ref _frame) = result {
            if self.playback.mode() == PlaybackMode::Paused
                && self.playback.playhead_ms() >= state.timeline.duration_ms
                && state.timeline.duration_ms > 0
            {
                event_bus.emit(CinemaEvent::PlaybackStopped {
                    project_id: state.project_id,
                    time_ms: self.playback.playhead_ms(),
                });
            }
        }
        result
    }

    pub fn request_preview_frame(&mut self, state: &ProjectState, time_ms: u64) -> FrameComposition {
        self.playback.request_frame(state, time_ms)
    }
}
