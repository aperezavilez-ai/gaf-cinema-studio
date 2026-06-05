//! Export / render pipeline — FFmpeg CLI H.264 when binary available.

mod ffmpeg;
mod stub;
mod timeline_resolve;
mod types;

pub use ffmpeg::{ffmpeg_available, locate_ffmpeg, FfmpegRenderBackend};
pub use stub::StubRenderBackend;
pub use timeline_resolve::{resolve_export_segments, ExportSegment};
pub use types::{RenderBackend, RenderJob, RenderProgress, RenderResult};

use std::path::Path;

use crate::error::Result;
use crate::project_state::types::ProjectState;

/// Pluggable render backend — stub fallback when FFmpeg unavailable.
pub trait RenderBackendImpl: Send + Sync {
    fn id(&self) -> RenderBackend;
    fn render(&self, job: &RenderJob, state: &ProjectState, on_progress: &dyn Fn(f64)) -> Result<RenderResult>;
}

pub struct RenderPipeline {
    backend: Box<dyn RenderBackendImpl>,
}

impl RenderPipeline {
    pub fn for_backend(backend: RenderBackend) -> Self {
        let impl_: Box<dyn RenderBackendImpl> = match backend {
            RenderBackend::Stub => Box::new(StubRenderBackend),
            RenderBackend::Ffmpeg => Box::new(FfmpegRenderBackend::new()),
        };
        Self { backend: impl_ }
    }

    /// Prefer FFmpeg when binary detected, else stub.
    pub fn auto() -> Self {
        if ffmpeg_available() {
            Self::for_backend(RenderBackend::Ffmpeg)
        } else {
            Self::for_backend(RenderBackend::Stub)
        }
    }

    pub fn backend_id(&self) -> RenderBackend {
        self.backend.id()
    }

    pub fn render(
        &self,
        job: &RenderJob,
        state: &ProjectState,
        on_progress: &dyn Fn(f64),
    ) -> Result<RenderResult> {
        self.backend.render(job, state, on_progress)
    }

    pub fn default_output_path(project_dir: &Path, job: &RenderJob) -> std::path::PathBuf {
        project_dir
            .join("exports")
            .join(format!("{}_{}x{}.mp4", job.export_id, job.width, job.height))
    }
}

impl Default for RenderPipeline {
    fn default() -> Self {
        Self::auto()
    }
}
