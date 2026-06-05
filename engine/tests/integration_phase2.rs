//! Phase 2 integration: import → timeline → scrub → playback tick

use cinemastudio_engine::{default_project_settings, AddClipParams, ProjectStateManager};
use std::fs;
use std::thread;
use std::time::Duration;
use tempfile::TempDir;

#[test]
fn phase2_full_playback_pipeline() {
    let tmp = TempDir::new().unwrap();
    let clip_path = tmp.path().join("scene.mp4");
    fs::write(&clip_path, vec![0u8; 4096]).unwrap();

    let mut manager = ProjectStateManager::new();
    manager
        .create_project("Phase2 Film", tmp.path(), default_project_settings())
        .unwrap();

    let asset = manager.import_media(&clip_path).unwrap();
    let clip_id = manager
        .add_clip_to_timeline(AddClipParams {
            media_id: asset.id,
            track_id: None,
            start_ms: None,
        })
        .unwrap();

    assert!(!clip_id.is_nil());
    assert!(manager.state().unwrap().timeline.duration_ms > 0);

    for t in (0..20).map(|i| i * 250) {
        let frame = manager.scrub_to(t).unwrap();
        if t == 0 {
            assert_eq!(frame.video_layers.len(), 1);
        }
    }

    let scrub_ms = manager
        .video_engine()
        .unwrap()
        .playback()
        .metrics()
        .last_scrub_latency_ms();
    assert!(scrub_ms < 100.0, "scrub {scrub_ms}ms exceeds budget");

    manager.playback_play().unwrap();
    thread::sleep(Duration::from_millis(30));
    manager.playback_tick().unwrap();
    manager.save().unwrap();

    let path = manager.project_path().unwrap().to_path_buf();
    let mut reopened = ProjectStateManager::new();
    reopened.open_project(&path).unwrap();
    assert_eq!(reopened.state().unwrap().timeline.tracks[0].clips.len(), 1);
}

#[test]
fn phase2_twenty_clip_sync() {
    let tmp = TempDir::new().unwrap();
    let clip_path = tmp.path().join("clip.mp4");
    fs::write(&clip_path, b"video").unwrap();

    let mut manager = ProjectStateManager::new();
    manager
        .create_project("20 Clips", tmp.path(), default_project_settings())
        .unwrap();

    let asset = manager.import_media(&clip_path).unwrap();
    let media_id = asset.id;

    for i in 0..20 {
        manager
            .add_clip_to_timeline(AddClipParams {
                media_id,
                track_id: None,
                start_ms: Some(i * 1000),
            })
            .unwrap();
    }

    assert_eq!(
        manager.state().unwrap().timeline.tracks[0].clips.len(),
        20
    );

    // With default 5000ms placeholder duration, timeline end = 19*1000 + 5000 = 24000
    assert_eq!(manager.state().unwrap().timeline.duration_ms, 24000);

    let frame = manager.scrub_to(15_500).unwrap();
    assert_eq!(frame.video_layers.len(), 1);
    assert_eq!(frame.video_layers[0].source_time_ms, 500);
}
