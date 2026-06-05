//! Demo: full Phase 3 workflow — import, edit, undo, export.

use cinemastudio_engine::{
    default_project_settings, AddClipParams, ExportSettings, ProjectStateManager,
};
use std::env;
use std::fs;
use std::path::PathBuf;
use std::thread;
use std::time::Duration;

fn main() -> cinemastudio_engine::Result<()> {
    let output_dir = env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| env::temp_dir().join("cinemastudio-edit-demo"));

    fs::create_dir_all(&output_dir)?;
    let clip_path = output_dir.join("scene.mp4");
    fs::write(&clip_path, b"demo video content")?;

    let mut manager = ProjectStateManager::new();
    manager.create_project("Edit Demo", &output_dir, default_project_settings())?;

    let asset = manager.import_media(&clip_path)?;
    manager.add_clip_to_timeline(AddClipParams {
        media_id: asset.id,
        track_id: None,
        start_ms: None,
    })?;

    manager.scrub_to(2500)?;
    if manager.split_at_playhead()?.is_some() {
        println!("Split clip at playhead");
    }

    println!("Clips: {}", manager.state().unwrap().timeline.tracks[0].clips.len());
    println!("Can undo: {}", manager.can_undo());

    let export_id = manager.start_export(ExportSettings::default())?;
    println!("Export queued: {export_id} (non-blocking)");

    thread::sleep(Duration::from_millis(500));
    manager.sync_export_status()?;

    if let Some(record) = manager
        .state()
        .unwrap()
        .export_state
        .history
        .iter()
        .find(|r| r.id == export_id)
    {
        println!("Export status: {:?}", record.status);
        println!("Output: {:?}", record.output_path);
    }

    manager.save()?;
    println!("Project saved: {}", manager.project_path().unwrap().display());

    Ok(())
}
