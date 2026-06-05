//! Phase 7 integration: decoder, render pipeline, native bridge

use cinemastudio_engine::{
    bridge_status, decode_frame_at, default_project_settings, render_pipeline,
    set_decoder_backend, set_render_backend, AddClipParams, DecoderBackend, ProjectStateManager,
    RenderBackend, RenderJob,
};
use std::fs;
use tempfile::TempDir;
use uuid::Uuid;

#[test]
fn phase7_decoder_stub() {
    let tmp = TempDir::new().unwrap();
    let path = tmp.path().join("v.mp4");
    fs::write(&path, b"data").unwrap();

    set_decoder_backend(DecoderBackend::Stub);
    let frame = decode_frame_at(&path, 1000, 1920, 1080).unwrap();
    assert_eq!(frame.time_ms, 1000);
}

#[test]
fn phase7_render_pipeline_via_export() {
    let tmp = TempDir::new().unwrap();
    let clip = tmp.path().join("clip.mp4");
    fs::write(&clip, b"video").unwrap();

    let mut manager = ProjectStateManager::new();
    manager
        .create_project("Render", tmp.path(), default_project_settings())
        .unwrap();
    let asset = manager.import_media(&clip).unwrap();
    manager
        .add_clip_to_timeline(AddClipParams {
            media_id: asset.id,
            track_id: None,
            start_ms: None,
        })
        .unwrap();

    set_render_backend(RenderBackend::Stub);
    let export_id = manager.start_export(Default::default()).unwrap();
    std::thread::sleep(std::time::Duration::from_millis(400));
    manager.sync_export_status().unwrap();

    let state = manager.state().unwrap();
    let record = state
        .export_state
        .history
        .iter()
        .find(|r| r.id == export_id)
        .expect("export record");
    assert!(record.output_path.is_some());
}

#[test]
fn phase7_decode_at_playhead() {
    let tmp = TempDir::new().unwrap();
    let clip = tmp.path().join("c.mp4");
    fs::write(&clip, b"v").unwrap();

    let mut manager = ProjectStateManager::new();
    manager
        .create_project("Decode", tmp.path(), default_project_settings())
        .unwrap();
    let asset = manager.import_media(&clip).unwrap();
    manager
        .add_clip_to_timeline(AddClipParams {
            media_id: asset.id,
            track_id: None,
            start_ms: None,
        })
        .unwrap();

    let frame = manager.decode_at_playhead().unwrap();
    assert!(frame.width > 0);
}

#[test]
fn phase7_bridge_status_json() {
    set_decoder_backend(DecoderBackend::AvFoundation);
    set_render_backend(RenderBackend::Ffmpeg);
    let status = bridge_status();
    assert!(status.get("decoderBackend").is_some());
}

#[test]
fn phase7_render_pipeline_direct() {
    use cinemastudio_engine::project_state::types::{
        default_project_settings, MediaAsset, MediaStatus, ProjectState,
    };
    use chrono::Utc;

    let tmp = TempDir::new().unwrap();
    let clip_file = tmp.path().join("media").join("clip.mp4");
    fs::create_dir_all(tmp.path().join("media")).unwrap();
    fs::write(&clip_file, b"mp4").unwrap();

    let media_id = Uuid::new_v4();
    let mut state =
        ProjectState::new("Direct", tmp.path().to_string_lossy(), default_project_settings());
    state.media.push(MediaAsset {
        id: media_id,
        original_path: clip_file.to_string_lossy().into_owned(),
        proxy_path: None,
        thumbnail_path: None,
        file_name: "clip.mp4".into(),
        mime_type: "video/mp4".into(),
        duration_ms: 5000,
        width: 1280,
        height: 720,
        file_size_bytes: 4,
        status: MediaStatus::Ready,
        imported_at: Utc::now(),
        checksum: None,
    });
    state.timeline = cinemastudio_engine::timeline_engine::add_clip(
        &state,
        AddClipParams {
            media_id,
            track_id: None,
            start_ms: None,
        },
    )
    .unwrap();

    let pipeline = render_pipeline();
    let job = RenderJob {
        export_id: Uuid::new_v4(),
        project_id: state.project_id,
        project_dir: tmp.path().to_path_buf(),
        width: 1280,
        height: 720,
        frame_rate: 24.0,
    };
    let result = pipeline.render(&job, &state, &|_| {}).unwrap();
    assert!(result.output_path.exists());
}
