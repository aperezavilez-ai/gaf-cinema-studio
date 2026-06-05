//! Workflow phase inference from project state.

use crate::project_state::types::{ProjectState, WorkflowPhase};

pub fn infer_phase(state: &ProjectState) -> WorkflowPhase {
    if state.media.is_empty() {
        return WorkflowPhase::Import;
    }

    let has_timeline_clips = state.timeline.tracks.iter().any(|t| !t.clips.is_empty());

    if !has_timeline_clips {
        return WorkflowPhase::Organize;
    }

    if state.export_state.active_export_id.is_some() {
        return WorkflowPhase::Export;
    }

    if state
        .export_state
        .history
        .iter()
        .any(|e| e.status == crate::project_state::types::ExportStatus::Completed)
    {
        return WorkflowPhase::Complete;
    }

    if state.timeline.duration_ms > 0 && state.timeline.playhead_ms > 0 {
        return WorkflowPhase::Review;
    }

    WorkflowPhase::Edit
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::project_state::types::default_project_settings;

    #[test]
    fn infers_import_when_empty() {
        let state = crate::project_state::types::ProjectState::new(
            "T",
            "/p",
            default_project_settings(),
        );
        assert_eq!(infer_phase(&state), WorkflowPhase::Import);
    }
}
