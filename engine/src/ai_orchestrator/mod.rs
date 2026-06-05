//! AI Orchestrator — rule-based, offline, context-aware (Rule #5, #23).

mod actions;
mod rules;
mod workflow;

pub use actions::{
    execute as execute_action, ACTION_ADVANCE_TO_EDIT, ACTION_ADVANCE_TO_EXPORT,
    ACTION_APPLY_DEFAULT_FADES, ACTION_ORGANIZE_BY_ORDER, ACTION_ROUGH_CUT, ACTION_START_EXPORT,
};
pub use rules::generate_suggestions;
pub use workflow::infer_phase;

use uuid::Uuid;

use crate::error::{CinemaError, Result};
use crate::event_bus::CinemaEvent;
use crate::project_state::manager::ProjectStateManager;
use crate::project_state::types::AiSuggestion;

pub struct AiOrchestrator;

impl AiOrchestrator {
    /// Analyze project state and refresh ai_state.suggestions.
    pub fn analyze(manager: &mut ProjectStateManager) -> Result<Vec<AiSuggestion>> {
        let state = manager
            .state()
            .ok_or_else(|| CinemaError::Validation("no open project".into()))?
            .clone();

        let dismissed_actions = state.ai_state.dismissed_action_ids.clone();
        let inferred = infer_phase(&state);

        let mut fresh = generate_suggestions(&state);
        fresh.retain(|s| !dismissed_actions.contains(&s.action_id));
        fresh.sort_by(|a, b| priority_rank(b.priority).cmp(&priority_rank(a.priority)));

        let project_id = state.project_id;

        if inferred != state.workflow_state.phase {
            manager.apply(crate::project_state::manager::Mutation::SetWorkflowPhase {
                phase: inferred,
            })?;
        }

        for sug in &fresh {
            manager.event_bus().emit(CinemaEvent::AiSuggestionCreated {
                project_id,
                suggestion: sug.clone(),
            });
        }

        manager.set_ai_suggestions(fresh.clone());
        Ok(fresh)
    }

    pub fn dismiss(manager: &mut ProjectStateManager, suggestion_id: Uuid) -> Result<()> {
        let (action_id, project_id) = {
            let state = manager.state.as_ref().unwrap();
            let sug = state
                .ai_state
                .suggestions
                .iter()
                .find(|s| s.id == suggestion_id)
                .ok_or_else(|| CinemaError::Validation("suggestion not found".into()))?;
            (sug.action_id.clone(), state.project_id)
        };

        manager.dismiss_ai_suggestion(suggestion_id);
        if !action_id.starts_with("hint_") {
            if let Some(state) = manager.state.as_mut() {
                if !state.ai_state.dismissed_action_ids.contains(&action_id) {
                    state.ai_state.dismissed_action_ids.push(action_id);
                }
            }
        }

        manager.event_bus().emit(CinemaEvent::AiSuggestionDismissed {
            project_id,
            suggestion_id,
        });

        Ok(())
    }

    pub fn execute_suggestion(
        manager: &mut ProjectStateManager,
        suggestion_id: Uuid,
    ) -> Result<String> {
        let action_id = {
            let state = manager.state.as_ref().unwrap();
            state
                .ai_state
                .suggestions
                .iter()
                .find(|s| s.id == suggestion_id)
                .map(|s| s.action_id.clone())
                .ok_or_else(|| CinemaError::Validation("suggestion not found".into()))?
        };

        if action_id.starts_with("hint_") {
            return Err(CinemaError::Validation(
                "this suggestion is informational only".into(),
            ));
        }

        let result = execute_action(manager, &action_id)?;

        manager.event_bus().emit(CinemaEvent::AiActionExecuted {
            project_id: manager.state.as_ref().unwrap().project_id,
            action_id: action_id.clone(),
            suggestion_id,
            result: result.clone(),
        });

        manager.remove_ai_suggestion(suggestion_id);
        Self::analyze(manager)?;

        Ok(result)
    }

    pub fn suggestions(manager: &ProjectStateManager) -> Vec<AiSuggestion> {
        manager
            .state()
            .map(|s| s.ai_state.suggestions.clone())
            .unwrap_or_default()
    }
}

fn priority_rank(p: crate::project_state::types::AiPriority) -> u8 {
    use crate::project_state::types::AiPriority;
    match p {
        AiPriority::High => 3,
        AiPriority::Medium => 2,
        AiPriority::Low => 1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::project_state::types::default_project_settings;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn analyze_and_execute_rough_cut_undoable() {
        let tmp = TempDir::new().unwrap();
        let clip = tmp.path().join("a.mp4");
        fs::write(&clip, b"v").unwrap();

        let mut manager = ProjectStateManager::new();
        manager
            .create_project("AI", tmp.path(), default_project_settings())
            .unwrap();
        manager.import_media(&clip).unwrap();

        let suggestions = AiOrchestrator::analyze(&mut manager).unwrap();
        let rough = suggestions
            .iter()
            .find(|s| s.action_id == ACTION_ROUGH_CUT)
            .expect("rough cut suggestion");

        AiOrchestrator::execute_suggestion(&mut manager, rough.id).unwrap();
        assert!(manager.state().unwrap().timeline.tracks[0].clips.len() >= 1);
        assert!(manager.can_undo());

        manager.undo().unwrap();
        assert!(manager.state().unwrap().timeline.tracks[0].clips.is_empty());
    }

    #[test]
    fn dismiss_suppresses_action() {
        let tmp = TempDir::new().unwrap();
        let clip = tmp.path().join("a.mp4");
        fs::write(&clip, b"v").unwrap();

        let mut manager = ProjectStateManager::new();
        manager
            .create_project("AI", tmp.path(), default_project_settings())
            .unwrap();
        manager.import_media(&clip).unwrap();

        let suggestions = AiOrchestrator::analyze(&mut manager).unwrap();
        let rough = suggestions
            .iter()
            .find(|s| s.action_id == ACTION_ROUGH_CUT)
            .unwrap();

        AiOrchestrator::dismiss(&mut manager, rough.id).unwrap();
        let again = AiOrchestrator::analyze(&mut manager).unwrap();
        assert!(!again.iter().any(|s| s.action_id == ACTION_ROUGH_CUT));
    }
}
