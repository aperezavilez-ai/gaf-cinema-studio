//! Phase 11 demo: FFmpeg H.264 export workflow.

use std::path::Path;
use std::process::Command;
use std::thread;
use std::time::Duration;

use cinemastudio_engine::{
    default_project_settings, ffmpeg_available, ProjectStateManager,
};

fn main() -> cinemastudio_engine::Result<()> {
    let tmp = std::env::temp_dir().join("CinemaStudioPhase11Demo");
    std::fs::create_dir_all(&tmp)?;
    let source = tmp.join("source.mp4");

    println!("=== Phase 11: FFmpeg Export ===\n");
    println!("FFmpeg available: {}\n", ffmpeg_available());

    if ffmpeg_available() && !source.exists() {
        Command::new("ffmpeg")
            .args([
                "-y", "-f", "lavfi", "-i", "color=c=gray:s=1280x720:d=3",
                "-c:v", "libx264", "-pix_fmt", "yuv420p",
            ])
            .arg(&source)
            .status()
            .expect("ffmpeg test source");
    }

    if !source.exists() {
        std::fs::write(&source, b"placeholder")?;
        println!("Warning: using placeholder — install ffmpeg for real H.264\n");
    }

    let mut manager = ProjectStateManager::with_data_root(tmp.clone());
    manager.create_project("Export Demo", &tmp, default_project_settings())?;

    let export_id = manager.run_edit_export_workflow(&source)?;
    println!("Export queued: {export_id}");

    for i in 0..30 {
        thread::sleep(Duration::from_millis(300));
        manager.sync_export_status()?;
        let status = manager.export_status();
        println!("  poll {i}: {status}");
        if status["activeExportId"].is_null() {
            break;
        }
    }

    if let Some(path) = manager.export_status()["lastOutputPath"].as_str() {
        println!("\nOutput: {path}");
        println!("Exists: {}", Path::new(path).exists());
    }

    Ok(())
}
