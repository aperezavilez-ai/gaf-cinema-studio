//! Example: create a new CinemaStudio project on disk.

use cinemastudio_engine::{default_project_settings, ProjectStateManager};
use std::env;
use std::path::PathBuf;

fn main() -> cinemastudio_engine::Result<()> {
    let output_dir = env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| env::temp_dir().join("cinemastudio-demo"));

    std::fs::create_dir_all(&output_dir)?;

    let mut manager = ProjectStateManager::new();
    let state = manager.create_project("Demo Film", &output_dir, default_project_settings())?;

    println!("Created project: {}", state.metadata.name);
    println!("Project ID:      {}", state.project_id);
    println!("Location:        {}", state.storage_state.project_root);
    println!("Schema version:  {}", state.schema_version);
    println!("Tracks:          {}", state.timeline.tracks.len());

    manager.save()?;
    let snapshot = manager.create_snapshot()?;
    println!("Snapshot:        {snapshot}");

    Ok(())
}
