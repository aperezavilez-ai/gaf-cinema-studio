use std::fs;
use std::path::PathBuf;

use chrono::Utc;
use uuid::Uuid;

use crate::error::{CinemaError, Result};
use crate::project_state::types::ProjectState;
use crate::render_pipeline::types::{RenderBackend, RenderJob, RenderResult};
use crate::render_pipeline::RenderBackendImpl;
use crate::timeline_engine::resolve::resolve_playback_path;

pub struct StubRenderBackend;

impl RenderBackendImpl for StubRenderBackend {
    fn id(&self) -> RenderBackend {
        RenderBackend::Stub
    }

    fn render(
        &self,
        job: &RenderJob,
        state: &ProjectState,
        on_progress: &dyn Fn(f64),
    ) -> Result<RenderResult> {
        let exports_dir = job.project_dir.join("exports");
        fs::create_dir_all(&exports_dir)?;

        on_progress(0.1);

        let source = resolve_export_source(state)?;
        let output_path = exports_dir.join(format!(
            "{}_{}x{}.mp4",
            job.export_id, job.width, job.height
        ));

        fs::copy(&source, &output_path).map_err(|e| {
            CinemaError::Storage(format!("stub render copy failed: {e}"))
        })?;

        on_progress(0.9);

        let sidecar_path = write_sidecar(&exports_dir, job, state, &output_path)?;

        on_progress(1.0);

        Ok(RenderResult {
            export_id: job.export_id,
            output_path,
            backend: RenderBackend::Stub,
            duration_ms: state.timeline.duration_ms,
            sidecar_path: Some(sidecar_path),
        })
    }
}

fn resolve_export_source(state: &ProjectState) -> Result<PathBuf> {
    use crate::project_state::types::TrackType;

    let track = state
        .timeline
        .tracks
        .iter()
        .find(|t| t.track_type == TrackType::Video)
        .ok_or_else(|| CinemaError::Validation("no video track".into()))?;

    let clip = track
        .clips
        .first()
        .ok_or_else(|| CinemaError::Validation("timeline empty".into()))?;

    let media = state
        .media
        .iter()
        .find(|m| m.id == clip.media_id)
        .ok_or_else(|| CinemaError::Validation("media missing".into()))?;

    let (path, _) = resolve_playback_path(media);
    if !path.exists() {
        return Err(CinemaError::Storage(format!(
            "source not found: {}",
            path.display()
        )));
    }
    Ok(path)
}

fn write_sidecar(
    exports_dir: &std::path::Path,
    job: &RenderJob,
    state: &ProjectState,
    output_path: &std::path::Path,
) -> Result<PathBuf> {
    let sidecar = exports_dir.join(format!("{}.export.json", job.export_id));
    let manifest = serde_json::json!({
        "exportId": job.export_id,
        "backend": "stub",
        "resolution": format!("{}x{}", job.width, job.height),
        "frameRate": job.frame_rate,
        "outputPath": output_path.display().to_string(),
        "timelineDurationMs": state.timeline.duration_ms,
        "note": "Wire FFmpeg backend with --features ffmpeg when binary available"
    });
    fs::write(&sidecar, serde_json::to_string_pretty(&manifest)?)?;
    Ok(sidecar)
}
