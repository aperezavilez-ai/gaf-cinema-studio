use std::path::{Path, PathBuf};

use uuid::Uuid;

use crate::project_state::types::{MediaAsset, ProjectState, TrackType};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrackKind {
    Video,
    Audio,
}

#[derive(Debug, Clone)]
pub struct ActiveLayer {
    pub clip_id: Uuid,
    pub media_id: Uuid,
    pub track_id: Uuid,
    pub track_order: u32,
    pub playback_path: PathBuf,
    pub uses_proxy: bool,
    /// Time within the source media file (ms)
    pub source_time_ms: u64,
    pub timeline_start_ms: u64,
    pub duration_ms: u64,
}

#[derive(Debug, Clone, Default)]
pub struct FrameComposition {
    pub time_ms: u64,
    pub video_layers: Vec<ActiveLayer>,
    pub audio_layers: Vec<ActiveLayer>,
}

impl FrameComposition {
    pub fn primary_video(&self) -> Option<&ActiveLayer> {
        self.video_layers.first()
    }
}

/// Resolve which clips are active at `time_ms` and their source paths (proxy preferred).
pub fn resolve_frame(state: &ProjectState, time_ms: u64) -> FrameComposition {
    let mut composition = FrameComposition {
        time_ms,
        ..Default::default()
    };

    for track in &state.timeline.tracks {
        for clip in &track.clips {
            let clip_end = clip.start_ms + clip.duration_ms;
            if time_ms >= clip.start_ms && time_ms < clip_end {
                if let Some(media) = state.media.iter().find(|m| m.id == clip.media_id) {
                    if track.muted {
                        continue;
                    }
                    let offset = time_ms - clip.start_ms;
                    let source_time_ms = clip.source_in_ms + offset;
                    let layer = build_layer(track.id, track.order, track.track_type, clip, media, source_time_ms);

                    match track.track_type {
                        TrackType::Video => composition.video_layers.push(layer),
                        TrackType::Audio => composition.audio_layers.push(layer),
                    }
                }
            }
        }
    }

    composition.video_layers.sort_by_key(|l| l.track_order);
    composition.audio_layers.sort_by_key(|l| l.track_order);
    composition
}

fn build_layer(
    track_id: Uuid,
    track_order: u32,
    _track_type: TrackType,
    clip: &crate::project_state::types::Clip,
    media: &MediaAsset,
    source_time_ms: u64,
) -> ActiveLayer {
    let (playback_path, uses_proxy) = resolve_playback_path(media);

    ActiveLayer {
        clip_id: clip.id,
        media_id: media.id,
        track_id,
        track_order,
        playback_path,
        uses_proxy,
        source_time_ms,
        timeline_start_ms: clip.start_ms,
        duration_ms: clip.duration_ms,
    }
}

/// Prefer proxy for editing playback (Rule #2 — performance).
pub fn resolve_playback_path(media: &MediaAsset) -> (PathBuf, bool) {
    if let Some(proxy) = &media.proxy_path {
        if Path::new(proxy).exists() {
            return (PathBuf::from(proxy), true);
        }
    }
    (PathBuf::from(&media.original_path), false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::project_state::types::{default_project_settings, MediaStatus, ProjectState};
    use crate::timeline_engine::{add_clip, AddClipParams};
    use chrono::Utc;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn resolves_proxy_when_available() {
        let tmp = TempDir::new().unwrap();
        let proxy = tmp.path().join("proxy.mp4");
        fs::write(&proxy, b"proxy").unwrap();

        let mut state = ProjectState::new("Resolve", tmp.path().to_string_lossy(), default_project_settings());
        let media_id = Uuid::new_v4();
        state.media.push(MediaAsset {
            id: media_id,
            original_path: tmp.path().join("orig.mp4").to_string_lossy().into_owned(),
            proxy_path: Some(proxy.to_string_lossy().into_owned()),
            thumbnail_path: None,
            file_name: "clip.mp4".into(),
            mime_type: "video/mp4".into(),
            duration_ms: 5000,
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

        let frame = resolve_frame(&state, 1000);
        assert_eq!(frame.video_layers.len(), 1);
        assert!(frame.video_layers[0].uses_proxy);
        assert_eq!(frame.video_layers[0].source_time_ms, 1000);
    }
}
