//! Executable AI actions — all reversible via ProjectStateManager undo stack.

use uuid::Uuid;

use crate::error::{CinemaError, Result};
use crate::project_state::manager::ProjectStateManager;
use crate::project_state::types::WorkflowPhase;
use crate::timeline_engine::AddClipParams;

pub const ACTION_ROUGH_CUT: &str = "rough_cut";
pub const ACTION_ORGANIZE_BY_ORDER: &str = "organize_by_import_order";
pub const ACTION_APPLY_DEFAULT_FADES: &str = "apply_default_fades";
pub const ACTION_ADVANCE_TO_EDIT: &str = "advance_to_edit";
pub const ACTION_ADVANCE_TO_EXPORT: &str = "advance_to_export";
pub const ACTION_START_EXPORT: &str = "start_export";

const DEFAULT_FADE_MS: u64 = 300;

pub fn execute(manager: &mut ProjectStateManager, action_id: &str) -> Result<String> {
    match action_id {
        ACTION_ROUGH_CUT => rough_cut(manager),
        ACTION_ORGANIZE_BY_ORDER => organize_by_import_order(manager),
        ACTION_APPLY_DEFAULT_FADES => apply_default_fades(manager),
        ACTION_ADVANCE_TO_EDIT => advance_phase(manager, WorkflowPhase::Edit),
        ACTION_ADVANCE_TO_EXPORT => advance_phase(manager, WorkflowPhase::Export),
        ACTION_START_EXPORT => start_export_action(manager),
        _ => Err(CinemaError::Validation(format!("unknown action: {action_id}"))),
    }
}

/// Add all ready media not yet on timeline, sequentially.
fn rough_cut(manager: &mut ProjectStateManager) -> Result<String> {
    let state = manager
        .state()
        .ok_or_else(|| CinemaError::Validation("no project".into()))?;

    let on_timeline: std::collections::HashSet<Uuid> = state
        .timeline
        .tracks
        .iter()
        .flat_map(|t| t.clips.iter().map(|c| c.media_id))
        .collect();

    let to_add: Vec<Uuid> = state
        .media
        .iter()
        .filter(|m| {
            use crate::project_state::types::MediaStatus;
            m.status == MediaStatus::Ready && !on_timeline.contains(&m.id)
        })
        .map(|m| m.id)
        .collect();

    if to_add.is_empty() {
        return Ok("all media already on timeline".into());
    }

    manager.record_history_for_ai();

    let mut added = 0usize;
    for media_id in to_add {
        manager.add_clip_to_timeline_without_history(AddClipParams {
            media_id,
            track_id: None,
            start_ms: None,
        })?;
        added += 1;
    }

    Ok(format!("added {added} clip(s) to timeline"))
}

/// Reorder clips on video track by media import timestamp.
fn organize_by_import_order(manager: &mut ProjectStateManager) -> Result<String> {
    let state = manager.state().unwrap();
    let media_order: std::collections::HashMap<Uuid, chrono::DateTime<chrono::Utc>> = state
        .media
        .iter()
        .map(|m| (m.id, m.imported_at))
        .collect();

    let video_track = state
        .timeline
        .tracks
        .iter()
        .find(|t| t.track_type == crate::project_state::types::TrackType::Video);

    let track = video_track.ok_or_else(|| CinemaError::Validation("no video track".into()))?;

    if track.clips.is_empty() {
        return Ok("timeline empty".into());
    }

    let mut sorted: Vec<_> = track.clips.iter().collect();
    sorted.sort_by_key(|c| media_order.get(&c.media_id).copied());

    manager.record_history_for_ai();

    let count = sorted.len();
    let mut cursor = 0u64;
    for clip in sorted {
        manager.move_clip_without_history(clip.id, cursor)?;
        cursor += clip.duration_ms;
    }

    Ok(format!("organized {count} clip(s) by import order"))
}

fn apply_default_fades(manager: &mut ProjectStateManager) -> Result<String> {
    let state = manager.state().unwrap();
    let clip_ids: Vec<Uuid> = state
        .timeline
        .tracks
        .iter()
        .flat_map(|t| t.clips.iter().map(|c| c.id))
        .collect();

    if clip_ids.is_empty() {
        return Ok("no clips to fade".into());
    }

    manager.record_history_for_ai();

    for id in &clip_ids {
        manager.set_clip_fade_without_history(*id, DEFAULT_FADE_MS, DEFAULT_FADE_MS)?;
    }

    Ok(format!("applied {DEFAULT_FADE_MS}ms fades to {} clip(s)", clip_ids.len()))
}

fn advance_phase(manager: &mut ProjectStateManager, phase: WorkflowPhase) -> Result<String> {
    manager.record_history_for_ai();
    manager.apply(crate::project_state::manager::Mutation::SetWorkflowPhase { phase })?;
    Ok(format!("workflow phase → {phase:?}"))
}

fn start_export_action(manager: &mut ProjectStateManager) -> Result<String> {
    use crate::export::ExportSettings;
    let id = manager.start_export(ExportSettings::default())?;
    Ok(format!("export started: {id}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::project_state::types::default_project_settings;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn rough_cut_adds_media() {
        let tmp = TempDir::new().unwrap();
        let clip = tmp.path().join("a.mp4");
        fs::write(&clip, b"v").unwrap();

        let mut manager = ProjectStateManager::new();
        manager.create_project("AI", tmp.path(), default_project_settings()).unwrap();
        manager.import_media(&clip).unwrap();

        let result = rough_cut(&mut manager).unwrap();
        assert!(result.contains("added"));
        assert_eq!(manager.state().unwrap().timeline.tracks[0].clips.len(), 1);
    }
}
