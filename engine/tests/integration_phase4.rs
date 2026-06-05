//! Phase 4 integration: AI orchestrator offline, contextual, undoable actions

use cinemastudio_engine::{
    default_project_settings, AiOrchestrator, ProjectStateManager, ACTION_ROUGH_CUT,
};
use std::fs;
use tempfile::TempDir;

#[test]
fn phase4_offline_suggestions_from_state() {
    let tmp = TempDir::new().unwrap();
    let clip = tmp.path().join("scene.mp4");
    fs::write(&clip, b"video").unwrap();

    let mut manager = ProjectStateManager::new();
    manager
        .create_project("AI Film", tmp.path(), default_project_settings())
        .unwrap();

    manager.import_media(&clip).unwrap();

    let suggestions = manager.ai_suggestions();
    assert!(!suggestions.is_empty());
    assert!(suggestions.iter().any(|s| s.action_id == ACTION_ROUGH_CUT));
    assert!(suggestions[0].message.contains("clip"));
}

#[test]
fn phase4_execute_action_undoable() {
    let tmp = TempDir::new().unwrap();
    let clip = tmp.path().join("a.mp4");
    fs::write(&clip, b"v").unwrap();

    let mut manager = ProjectStateManager::new();
    manager.create_project("Undo AI", tmp.path(), default_project_settings()).unwrap();
    manager.import_media(&clip).unwrap();

    let rough = manager
        .ai_suggestions()
        .into_iter()
        .find(|s| s.action_id == ACTION_ROUGH_CUT)
        .unwrap();

    manager.ai_execute(rough.id).unwrap();
    assert!(!manager.state().unwrap().timeline.tracks[0].clips.is_empty());

    manager.undo().unwrap();
    assert!(manager.state().unwrap().timeline.tracks[0].clips.is_empty());
}

#[test]
fn phase4_no_chat_required() {
    let tmp = TempDir::new().unwrap();
    let mut manager = ProjectStateManager::new();
    manager
        .create_project("Offline", tmp.path(), default_project_settings())
        .unwrap();

    let suggestions = manager.ai_analyze().unwrap();
    for s in &suggestions {
        assert!(!s.action_id.is_empty());
        assert!(!s.message.is_empty());
    }
}
