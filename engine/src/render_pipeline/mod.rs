//! Export / render pipeline — FFmpeg backend wired at integration time.

mod stub;
mod types;

#[cfg(feature = "ffmpeg")]
mod ffmpeg;

pub use stub::StubRenderBackend;
pub use types::{RenderBackend, RenderJob, RenderProgress, RenderResult};

use std::path::Path;

use crate::error::Result;
use crate::project_state::types::ProjectState;

/// Pluggable render backend — stub now, FFmpeg when feature enabled + binary linked.
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
            RenderBackend::Ffmpeg => {
                #[cfg(feature = "ffmpeg")]
                {
                    Box::new(ffmpeg::FfmpegRenderBackend::new())
                }
                #[cfg(not(feature = "ffmpeg"))]
                {
                    Box::new(StubRenderBackend)
                }
            }
        };
        Self { backend: impl_ }
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
        Self::for_backend(RenderBackend::Stub)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::export::ExportSettings;
    use crate::project_state::types::default_project_settings;
    use chrono::Utc;
    use std::fs;
    use tempfile::TempDir;
    use uuid::Uuid;

    fn sample_state(dir: &TempDir) -> ProjectState {
        use crate::project_state::types::{MediaAsset, MediaStatus, ProjectState};
        use crate::timeline_engine::{add_clip, AddClipParams};

        let mut state =
            ProjectState::new("Render", dir.path().to_string_lossy(), default_project_settings());
        let media_id = Uuid::new_v4();
        let clip_file = dir.path().join("media").join("clip.mp4");
        fs::create_dir_all(dir.path().join("media")).unwrap();
        fs::write(&clip_file, b"fake mp4").unwrap();

        state.media.push(MediaAsset {
            id: media_id,
            original_path: clip_file.to_string_lossy().into_owned(),
            proxy_path: None,
            thumbnail_path: None,
            file_name: "clip.mp4".into(),
            mime_type: "video/mp4".into(),
            duration_ms: 3000,
            width: 1920,
            height: 1080,
            file_size_bytes: 100,
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
    fn stub_render_produces_output() {
        let tmp = TempDir::new().unwrap();
        let state = sample_state(&tmp);
        fs::create_dir_all(tmp.path().join("exports")).unwrap();

        let settings = ExportSettings::default();
        let job = RenderJob {
            export_id: Uuid::new_v4(),
            project_id: state.project_id,
            project_dir: tmp.path().to_path_buf(),
            width: settings.width,
            height: settings.height,
            frame_rate: settings.frame_rate,
        };

        let pipeline = RenderPipeline::default();
        let result = pipeline
            .render(&job, &state, &|_| {})
            .unwrap();
        assert!(result.output_path.exists());
    }
}
