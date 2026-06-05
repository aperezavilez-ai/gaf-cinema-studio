//! Demo: timeline + playback scrub across imported clip.

use cinemastudio_engine::{default_project_settings, AddClipParams, ProjectStateManager};
use std::env;
use std::fs;
use std::path::PathBuf;

fn main() -> cinemastudio_engine::Result<()> {
    let output_dir = env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| env::temp_dir().join("cinemastudio-playback-demo"));

    fs::create_dir_all(&output_dir)?;
    let clip_path = output_dir.join("sample.mp4");
    fs::write(&clip_path, vec![0u8; 8192])?;

    let mut manager = ProjectStateManager::new();
    manager.create_project("Playback Demo", &output_dir, default_project_settings())?;

    let asset = manager.import_media(&clip_path)?;
    manager.add_clip_to_timeline(AddClipParams {
        media_id: asset.id,
        track_id: None,
        start_ms: None,
    })?;

    println!("Timeline duration: {}ms", manager.state().unwrap().timeline.duration_ms);

    for t in [0, 1000, 2000, 3000] {
        let frame = manager.scrub_to(t)?;
        let layer = frame.primary_video();
        println!(
            "  scrub {t}ms → layers={} proxy={}",
            frame.video_layers.len(),
            layer.map(|l| l.uses_proxy).unwrap_or(false)
        );
    }

    manager.playback_play()?;
    if let Some(frame) = manager.playback_tick()? {
        println!(
            "  tick → playhead={}ms layers={}",
            manager.video_engine().unwrap().playback().playhead_ms(),
            frame.video_layers.len()
        );
    }

    manager.save()?;
    println!("Saved: {}", manager.project_path().unwrap().display());

    Ok(())
}
