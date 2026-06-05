//! Phase 3 integration: import → edit → undo → export workflow

use cinemastudio_engine::{
    default_project_settings, AddClipParams, ExportSettings, ProjectStateManager,
};
use std::fs;
use std::thread;
use std::time::{Duration, Instant};
use tempfile::TempDir;

#[test]
fn phase3_edit_undo_export_workflow() {
    let tmp = TempDir::new().unwrap();
    let clip = tmp.path().join("scene.mp4");
    fs::write(&clip, b"video data for export").unwrap();

    let mut manager = ProjectStateManager::new();
    manager
        .create_project("Edit Film", tmp.path(), default_project_settings())
        .unwrap();

    let asset = manager.import_media(&clip).unwrap();
    manager
        .add_clip_to_timeline(AddClipParams {
            media_id: asset.id,
            track_id: None,
            start_ms: None,
        })
        .unwrap();

    manager.add_clip_to_timeline(AddClipParams {
        media_id: asset.id,
        track_id: None,
        start_ms: Some(5000),
    }).unwrap();

    assert!(manager.can_undo());
    manager.undo().unwrap();
    assert_eq!(manager.state().unwrap().timeline.tracks[0].clips.len(), 1);

    manager.redo().unwrap();
    assert_eq!(manager.state().unwrap().timeline.tracks[0].clips.len(), 2);

    // Export non-blocking
    let start = Instant::now();
    let export_id = manager.start_export(ExportSettings::default()).unwrap();
    let elapsed = start.elapsed();
    assert!(elapsed < Duration::from_millis(50), "export blocked UI");

    thread::sleep(Duration::from_millis(400));
    manager.sync_export_status().unwrap();

    let exports = &manager.state().unwrap().export_state.history;
    assert!(exports.iter().any(|e| e.id == export_id));
}

#[test]
fn phase3_split_and_fade() {
    let tmp = TempDir::new().unwrap();
    let clip = tmp.path().join("clip.mp4");
    fs::write(&clip, b"v").unwrap();

    let mut manager = ProjectStateManager::new();
    manager.create_project("Split", tmp.path(), default_project_settings()).unwrap();
    let asset = manager.import_media(&clip).unwrap();
    let clip_id = manager
        .add_clip_to_timeline(AddClipParams {
            media_id: asset.id,
            track_id: None,
            start_ms: None,
        })
        .unwrap();

    manager.scrub_to(2500).unwrap();
    manager.split_at_playhead().unwrap();

    assert_eq!(manager.state().unwrap().timeline.tracks[0].clips.len(), 2);

    manager.set_clip_fade(clip_id, 500, 500).unwrap();
    let clip = &manager.state().unwrap().timeline.tracks[0].clips[0];
    assert_eq!(clip.transitions.fade_in_ms, 500);
}
