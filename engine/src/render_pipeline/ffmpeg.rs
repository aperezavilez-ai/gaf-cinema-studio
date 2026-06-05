//! FFmpeg render backend — scaffold only. Connect libav at integration time.
//!
//! Requires: FFmpeg dev libraries + feature `ffmpeg`.
//! Until linked, falls back to error with clear message.

use crate::error::{CinemaError, Result};
use crate::project_state::types::ProjectState;
use crate::render_pipeline::stub::StubRenderBackend;
use crate::render_pipeline::types::{RenderBackend, RenderJob, RenderResult};
use crate::render_pipeline::RenderBackendImpl;

pub struct FfmpegRenderBackend {
    /// Set true once FFmpeg is linked and probed at startup
    linked: bool,
}

impl FfmpegRenderBackend {
    pub fn new() -> Self {
        Self {
            linked: std::env::var("CINEMASTUDIO_FFMPEG_LINKED").is_ok(),
        }
    }
}

impl Default for FfmpegRenderBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl RenderBackendImpl for FfmpegRenderBackend {
    fn id(&self) -> RenderBackend {
        RenderBackend::Ffmpeg
    }

    fn render(
        &self,
        job: &RenderJob,
        state: &ProjectState,
        on_progress: &dyn Fn(f64),
    ) -> Result<RenderResult> {
        if !self.linked {
            // Scaffold: document integration point; stub fallback for dev without FFmpeg
            eprintln!(
                "FFmpeg not linked — set CINEMASTUDIO_FFMPEG_LINKED=1 after wiring libav. Using stub fallback."
            );
            return StubRenderBackend.render(job, state, on_progress);
        }

        // Integration point: concat timeline clips, encode H.264, mux MP4
        // ffmpeg -f concat -i list.txt -c:v libx264 -preset medium -crf 23 out.mp4
        Err(CinemaError::Storage(
            "FFmpeg backend scaffold — implement libav bindings here".into(),
        ))
    }
}
