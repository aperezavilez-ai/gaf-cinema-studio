use uuid::Uuid;

use crate::error::{CinemaError, Result};
use crate::project_state::types::{Clip, ClipTransitions, MediaAsset, ProjectState, Timeline, TrackType};

#[derive(Debug, Clone)]
pub struct AddClipParams {
    pub media_id: Uuid,
    pub track_id: Option<Uuid>,
    pub start_ms: Option<u64>,
}

/// Add a clip to the timeline. Defaults to video track, appended at end.
pub fn add_clip(state: &ProjectState, params: AddClipParams) -> Result<Timeline> {
    let media = state
        .media
        .iter()
        .find(|m| m.id == params.media_id)
        .ok_or_else(|| CinemaError::Validation(format!("media not found: {}", params.media_id)))?;

    if media.status != crate::project_state::types::MediaStatus::Ready
        && media.status != crate::project_state::types::MediaStatus::Indexing
    {
        return Err(CinemaError::Validation(format!(
            "media {} is not ready for timeline",
            params.media_id
        )));
    }

    let mut timeline = state.timeline.clone();
    let track_idx = resolve_track_index(&timeline, params.track_id, TrackType::Video)?;

    let start_ms = params.start_ms.unwrap_or_else(|| compute_duration(&timeline));
    let duration_ms = effective_media_duration(media);

    let clip = Clip {
        id: Uuid::new_v4(),
        media_id: params.media_id,
        start_ms,
        duration_ms,
        source_in_ms: 0,
        source_out_ms: duration_ms,
        label: media.file_name.clone(),
        transitions: ClipTransitions::default(),
    };

    timeline.tracks[track_idx].clips.push(clip);
    sort_clips(&mut timeline.tracks[track_idx].clips);
    timeline.duration_ms = compute_duration(&timeline);
    clamp_playhead(&mut timeline);

    Ok(timeline)
}

pub fn remove_clip(state: &ProjectState, clip_id: Uuid) -> Result<Timeline> {
    let mut timeline = state.timeline.clone();
    let mut found = false;

    for track in &mut timeline.tracks {
        let before = track.clips.len();
        track.clips.retain(|c| c.id != clip_id);
        if track.clips.len() < before {
            found = true;
        }
    }

    if !found {
        return Err(CinemaError::Validation(format!("clip not found: {clip_id}")));
    }

    timeline.duration_ms = compute_duration(&timeline);
    clamp_playhead(&mut timeline);
    Ok(timeline)
}

pub fn move_clip(state: &ProjectState, clip_id: Uuid, new_start_ms: u64) -> Result<Timeline> {
    let mut timeline = state.timeline.clone();
    let clip = find_clip_mut(&mut timeline, clip_id)?;
    clip.start_ms = new_start_ms;
    Ok(finish_timeline(timeline))
}

pub fn trim_clip(
    state: &ProjectState,
    clip_id: Uuid,
    source_in_ms: u64,
    source_out_ms: u64,
) -> Result<Timeline> {
    if source_out_ms <= source_in_ms {
        return Err(CinemaError::Validation(
            "source_out_ms must be greater than source_in_ms".into(),
        ));
    }

    let mut timeline = state.timeline.clone();
    let clip = find_clip_mut(&mut timeline, clip_id)?;

    let media = state
        .media
        .iter()
        .find(|m| m.id == clip.media_id)
        .ok_or_else(|| CinemaError::Validation("clip media missing".into()))?;

    let max_duration = effective_media_duration(media);
    if source_out_ms > max_duration {
        return Err(CinemaError::Validation(format!(
            "trim exceeds media duration ({max_duration}ms)"
        )));
    }

    clip.source_in_ms = source_in_ms;
    clip.source_out_ms = source_out_ms;
    clip.duration_ms = source_out_ms - source_in_ms;

    Ok(finish_timeline(timeline))
}

pub fn split_clip(state: &ProjectState, clip_id: Uuid, at_timeline_ms: u64) -> Result<Timeline> {
    let mut timeline = state.timeline.clone();

    let (track_idx, clip_idx) = find_clip_location(&timeline, clip_id)?;
    let original = timeline.tracks[track_idx].clips[clip_idx].clone();

    let clip_start = original.start_ms;
    let clip_end = original.start_ms + original.duration_ms;

    if at_timeline_ms <= clip_start || at_timeline_ms >= clip_end {
        return Err(CinemaError::Validation(
            "split point must be inside clip bounds".into(),
        ));
    }

    let offset_in_clip = at_timeline_ms - clip_start;
    let split_source_ms = original.source_in_ms + offset_in_clip;

    let mut left = original.clone();
    left.duration_ms = offset_in_clip;
    left.source_out_ms = split_source_ms;

    let mut right = original;
    right.id = Uuid::new_v4();
    right.start_ms = at_timeline_ms;
    right.duration_ms = clip_end - at_timeline_ms;
    right.source_in_ms = split_source_ms;

    timeline.tracks[track_idx].clips[clip_idx] = left;
    timeline.tracks[track_idx].clips.insert(clip_idx + 1, right);

    Ok(finish_timeline(timeline))
}

/// Timeline duration = end of the last clip on any track.
pub fn compute_duration(timeline: &Timeline) -> u64 {
    timeline
        .tracks
        .iter()
        .flat_map(|t| t.clips.iter())
        .map(|c| c.start_ms + c.duration_ms)
        .max()
        .unwrap_or(0)
}

fn effective_media_duration(media: &MediaAsset) -> u64 {
    if media.duration_ms > 0 {
        media.duration_ms
    } else {
        5000 // placeholder until FFmpeg probe (Phase 2+)
    }
}

fn resolve_track_index(
    timeline: &Timeline,
    track_id: Option<Uuid>,
    fallback_type: TrackType,
) -> Result<usize> {
    if let Some(id) = track_id {
        timeline
            .tracks
            .iter()
            .position(|t| t.id == id)
            .ok_or_else(|| CinemaError::Validation(format!("track not found: {id}")))
    } else {
        timeline
            .tracks
            .iter()
            .position(|t| t.track_type == fallback_type)
            .ok_or_else(|| CinemaError::Validation("no video track".into()))
    }
}

fn find_clip_location(timeline: &Timeline, clip_id: Uuid) -> Result<(usize, usize)> {
    for (ti, track) in timeline.tracks.iter().enumerate() {
        if let Some(ci) = track.clips.iter().position(|c| c.id == clip_id) {
            return Ok((ti, ci));
        }
    }
    Err(CinemaError::Validation(format!("clip not found: {clip_id}")))
}

fn find_clip_mut(timeline: &mut Timeline, clip_id: Uuid) -> Result<&mut Clip> {
    for track in &mut timeline.tracks {
        if let Some(clip) = track.clips.iter_mut().find(|c| c.id == clip_id) {
            return Ok(clip);
        }
    }
    Err(CinemaError::Validation(format!("clip not found: {clip_id}")))
}

fn sort_clips(clips: &mut [Clip]) {
    clips.sort_by_key(|c| c.start_ms);
}

fn clamp_playhead(timeline: &mut Timeline) {
    if timeline.playhead_ms > timeline.duration_ms {
        timeline.playhead_ms = timeline.duration_ms;
    }
}

fn finish_timeline(mut timeline: Timeline) -> Timeline {
    for track in &mut timeline.tracks {
        sort_clips(&mut track.clips);
    }
    timeline.duration_ms = compute_duration(&timeline);
    clamp_playhead(&mut timeline);
    timeline
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::project_state::types::{default_project_settings, MediaStatus, ProjectState};
    use chrono::Utc;

    fn sample_state_with_media(duration_ms: u64) -> ProjectState {
        let mut state = ProjectState::new("Timeline Test", "/tmp/test.csproj", default_project_settings());
        let media_id = Uuid::new_v4();
        state.media.push(MediaAsset {
            id: media_id,
            original_path: "/vault/clip.mp4".into(),
            proxy_path: Some("/vault/clip_proxy.mp4".into()),
            thumbnail_path: None,
            file_name: "clip.mp4".into(),
            mime_type: "video/mp4".into(),
            duration_ms,
            width: 1920,
            height: 1080,
            file_size_bytes: 1024,
            status: MediaStatus::Ready,
            imported_at: Utc::now(),
            checksum: None,
        });
        state
    }

    #[test]
    fn add_clip_extends_duration() {
        let state = sample_state_with_media(3000);
        let media_id = state.media[0].id;
        let timeline = add_clip(
            &state,
            AddClipParams {
                media_id,
                track_id: None,
                start_ms: None,
            },
        )
        .unwrap();
        assert_eq!(timeline.duration_ms, 3000);
        assert_eq!(timeline.tracks[0].clips.len(), 1);
    }

    #[test]
    fn twenty_clips_sync_correctly() {
        let mut state = sample_state_with_media(1000);
        let media_id = state.media[0].id;

        for i in 0..20 {
            state.timeline = add_clip(
                &state,
                AddClipParams {
                    media_id,
                    track_id: None,
                    start_ms: Some(i * 1000),
                },
            )
            .unwrap();
        }

        assert_eq!(state.timeline.tracks[0].clips.len(), 20);
        assert_eq!(state.timeline.duration_ms, 21000);

        for (i, clip) in state.timeline.tracks[0].clips.iter().enumerate() {
            assert_eq!(clip.start_ms, (i as u64) * 1000);
            assert_eq!(clip.duration_ms, 1000);
        }
    }

    #[test]
    fn split_clip_creates_two_segments() {
        let mut state = sample_state_with_media(4000);
        let media_id = state.media[0].id;
        state.timeline = add_clip(
            &state,
            AddClipParams {
                media_id,
                track_id: None,
                start_ms: None,
            },
        )
        .unwrap();
        let clip_id = state.timeline.tracks[0].clips[0].id;
        state.timeline = split_clip(&state, clip_id, 2000).unwrap();
        assert_eq!(state.timeline.tracks[0].clips.len(), 2);
        assert_eq!(state.timeline.tracks[0].clips[0].duration_ms, 2000);
        assert_eq!(state.timeline.tracks[0].clips[1].duration_ms, 2000);
    }
}
