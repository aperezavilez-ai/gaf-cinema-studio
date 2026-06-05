//! Demo: AI orchestrator analyze + execute rough cut.

use cinemastudio_engine::{default_project_settings, ProjectStateManager};
use std::env;
use std::fs;
use std::path::PathBuf;

fn main() -> cinemastudio_engine::Result<()> {
    let output_dir = env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| env::temp_dir().join("cinemastudio-ai-demo"));

    fs::create_dir_all(&output_dir)?;
    let clip = output_dir.join("scene.mp4");
    fs::write(&clip, b"demo")?;

    let mut manager = ProjectStateManager::new();
    manager.create_project("AI Demo", &output_dir, default_project_settings())?;
    manager.import_media(&clip)?;

    println!("── AI Suggestions ──");
    for sug in manager.ai_suggestions() {
        println!("  [{}] {} → {}", format!("{:?}", sug.priority), sug.message, sug.action_id);
    }

    if let Some(rough) = manager
        .ai_suggestions()
        .into_iter()
        .find(|s| s.action_id == "rough_cut")
    {
        let result = manager.ai_execute(rough.id)?;
        println!("\nExecuted: {result}");
        println!("Timeline clips: {}", manager.state().unwrap().timeline.tracks[0].clips.len());
    }

    manager.save()?;
    Ok(())
}
