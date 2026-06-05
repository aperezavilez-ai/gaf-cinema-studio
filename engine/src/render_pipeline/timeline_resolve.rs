//! Resolve timeline clips into ordered export segments.

use std::path::PathBuf;

use uuid::Uuid;

use crate::error::{CinemaError, Result};
use crate::project_state::types::{ProjectState, TrackType};
use crate::timeline_engine::resolve::resolve_playback_path;

#[derive(Debug, Clone)]
pub struct ExportSegment {
    pub clip_id: Uuid,
    pub media_id: Uuid,
    pub source_path: PathBuf,
    pub timeline_start_ms: u64,
    pub source_in_ms: u64,
    pub duration_ms: u64,
}

/// Video track clips sorted by timeline position.
pub fn resolve_export_segments(state: &ProjectState) -> Result<Vec<ExportSegment>> {
    let track = state
        .timeline
        .tracks
        .iter()
        .find(|t| t.track_type == TrackType::Video)
        .ok_or_else(|| CinemaError::Validation("no video track".into()))?;

    if track.clips.is_empty() {
        return Err(CinemaError::Validation("timeline empty — nothing to export".into()));
    }

    let mut clips = track.clips.clone();
    clips.sort_by_key(|c| c.start_ms);

    let mut segments = Vec::with_capacity(clips.len());
    for clip in clips {
        let media = state
            .media
            .iter()
            .find(|m| m.id == clip.media_id)
            .ok_or_else(|| CinemaError::Validation(format!("media missing for clip {}", clip.id)))?;

        let (path, _) = resolve_playback_path(media);
        if !path.exists() {
            return Err(CinemaError::Storage(format!(
                "export source not found: {}",
                path.display()
            )));
        }

        segments.push(ExportSegment {
            clip_id: clip.id,
            media_id: clip.media_id,
            source_path: path,
            timeline_start_ms: clip.start_ms,
            source_in_ms: clip.source_in_ms,
            duration_ms: clip.duration_ms,
        });
    }

    Ok(segments)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::project_state::types::{default_project_settings, MediaAsset, MediaStatus, ProjectState};
    use crate::timeline_engine::{add_clip, AddClipParams};
    use chrono::Utc;
    use std::fs;
    use tempfile::TempDir;
    use uuid::Uuid;

    #[test]
    fn resolves_sorted_segments() {
        let tmp = TempDir::new().unwrap();
        let mut state =
            ProjectState::new("Export", tmp.path().to_string_lossy(), default_project_settings());
        let media_id = Uuid::new_v4();
        let clip_file = tmp.path().join("media").join("a.mp4");
        fs::create_dir_all(tmp.path().join("media")).unwrap();
        fs::write(&clip_file, b"fake").unwrap();

        state.media.push(MediaAsset {
            id: media_id,
            original_path: clip_file.to_string_lossy().into_owned(),
            proxy_path: None,
            thumbnail_path: None,
            file_name: "a.mp4".into(),
            mime_type: "video/mp4".into(),
            duration_ms: 5000,
            width: 1920,
            height: 1080,
            file_size_bytes: 4,
            status: MediaStatus::Ready,
            imported_at: Utc::now(),
            checksum: None,
        });

        state.timeline = add_clip(
            &state,
            AddClipParams {
                media_id,
                track_id: None,
                start_ms: Some(2000),
            },
        )
        .unwrap();

        let segs = resolve_export_segments(&state).unwrap();
        assert_eq!(segs.len(), 1);
        assert_eq!(segs[0].timeline_start_ms, 2000);
    }
}
