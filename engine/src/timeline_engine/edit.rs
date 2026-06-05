//! Find clips at playhead and editing helpers.

use uuid::Uuid;

use crate::project_state::types::{ProjectState, TrackType};

#[derive(Debug, Clone)]
pub struct ClipHit {
    pub clip_id: Uuid,
    pub track_id: Uuid,
    pub track_type: TrackType,
    pub offset_in_clip_ms: u64,
}

/// Find the topmost video clip under the playhead.
pub fn clip_at_playhead(state: &ProjectState, time_ms: u64) -> Option<ClipHit> {
    find_clip_at(state, time_ms, TrackType::Video)
}

pub fn find_clip_at(state: &ProjectState, time_ms: u64, track_type: TrackType) -> Option<ClipHit> {
    let mut hits: Vec<ClipHit> = state
        .timeline
        .tracks
        .iter()
        .filter(|t| t.track_type == track_type)
        .flat_map(|track| {
            track.clips.iter().filter_map(|clip| {
                let end = clip.start_ms + clip.duration_ms;
                if time_ms >= clip.start_ms && time_ms < end {
                    Some(ClipHit {
                        clip_id: clip.id,
                        track_id: track.id,
                        track_type: track.track_type,
                        offset_in_clip_ms: time_ms - clip.start_ms,
                    })
                } else {
                    None
                }
            })
        })
        .collect();

    hits.sort_by_key(|h| h.offset_in_clip_ms);
    hits.into_iter().next()
}
