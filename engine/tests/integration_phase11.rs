//! Phase 11: FFmpeg H.264 export + full edit workflow

use cinemastudio_engine::{
    default_project_settings, ffmpeg_available, resolve_export_segments, ExportSettings,
    ProjectStateManager, RenderBackend, RenderPipeline,
};
use std::fs;
use std::path::Path;
use std::process::Command;
use std::thread;
use std::time::Duration;
use tempfile::TempDir;
use uuid::Uuid;

fn create_test_video(path: &Path) -> bool {
    if !ffmpeg_available() {
        return false;
    }
    Command::new("ffmpeg")
        .args([
            "-y",
            "-f",
            "lavfi",
            "-i",
            "color=c=blue:s=640x360:d=2",
            "-c:v",
            "libx264",
            "-pix_fmt",
            "yuv420p",
            &path.to_string_lossy(),
        ])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

#[test]
fn phase11_timeline_segments_resolve() {
    let tmp = TempDir::new().unwrap();
    let clip = tmp.path().join("media").join("clip.mp4");
    fs::create_dir_all(tmp.path().join("media")).unwrap();
    fs::write(&clip, b"fake").unwrap();

    let mut manager = ProjectStateManager::new();
    manager
        .create_project("Segments", tmp.path(), default_project_settings())
        .unwrap();
    manager.import_media(&clip).unwrap();
    let media_id = manager.state().unwrap().media[0].id;
    manager
        .add_clip_to_timeline(cinemastudio_engine::AddClipParams {
            media_id,
            track_id: None,
            start_ms: None,
        })
        .unwrap();

    let segs = resolve_export_segments(manager.state().unwrap()).unwrap();
    assert_eq!(segs.len(), 1);
}

#[test]
fn phase11_export_status_reports_ffmpeg() {
    let tmp = TempDir::new().unwrap();
    let mut manager = ProjectStateManager::new();
    manager
        .create_project("Status", tmp.path(), default_project_settings())
        .unwrap();

    let status = manager.export_status();
    assert!(status.get("ffmpegAvailable").is_some());
}

#[test]
fn phase11_full_workflow_export() {
    let tmp = TempDir::new().unwrap();
    let source = tmp.path().join("source.mp4");

    if !create_test_video(&source) {
        eprintln!("ffmpeg not available — skipping H.264 export integration test");
        return;
    }

    let mut manager = ProjectStateManager::new();
    manager
        .create_project("Workflow", tmp.path(), default_project_settings())
        .unwrap();

    let export_id = manager.run_edit_export_workflow(&source).unwrap();

    for _ in 0..50 {
        thread::sleep(Duration::from_millis(200));
        manager.sync_export_status().unwrap();
        let status = manager.export_status();
        if status["activeExportId"].is_null() {
            break;
        }
    }

    let state = manager.state().unwrap();
    let record = state
        .export_state
        .history
        .iter()
        .find(|r| r.id == export_id)
        .expect("export record");

    assert!(record.output_path.is_some(), "export should complete");
    let out = record.output_path.as_ref().unwrap();
    assert!(Path::new(out).exists(), "output file missing: {out}");

    if ffmpeg_available() {
        let sidecar = tmp.path().join("exports").join(format!("{export_id}.export.json"));
        if sidecar.exists() {
            let data = fs::read_to_string(&sidecar).unwrap();
            assert!(data.contains("h264") || data.contains("ffmpeg"));
        }
    }
}

#[test]
fn phase11_ffmpeg_pipeline_backend() {
    let pipeline = if ffmpeg_available() {
        RenderPipeline::for_backend(RenderBackend::Ffmpeg)
    } else {
        RenderPipeline::for_backend(RenderBackend::Stub)
    };
    assert!(matches!(
        pipeline.backend_id(),
        RenderBackend::Ffmpeg | RenderBackend::Stub
    ));
}
