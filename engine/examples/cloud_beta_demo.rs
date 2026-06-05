//! Phase 6 demo: optional cloud, auth, billing, telemetry, beta tracking.

use std::fs;

use cinemastudio_engine::{
    default_project_settings, AddClipParams, ProjectStateManager,
};

fn main() -> cinemastudio_engine::Result<()> {
    let data_root = std::env::temp_dir().join("CinemaStudioPhase6Demo");
    fs::create_dir_all(&data_root)?;

    let mut manager = ProjectStateManager::with_data_root(data_root.clone());
    let project_dir = data_root.join("demo.csproj");
    fs::create_dir_all(&project_dir)?;

    let clip = project_dir.join("media").join("sample.mp4");
    fs::create_dir_all(clip.parent().unwrap())?;
    fs::write(&clip, b"fake mp4")?;

    println!("=== Phase 6: Beta + Optional Cloud ===\n");

    // Gate 6.1 — core without account
    println!("[6.1] Core without account");
    manager.create_project("Demo Film", &project_dir, default_project_settings())?;
    let asset = manager.import_media(&clip)?;
    manager.add_clip_to_timeline(AddClipParams {
        media_id: asset.id,
        track_id: None,
        start_ms: None,
    })?;
    manager.save()?;
    println!("  Project saved locally (no login required)\n");

    // Optional auth
    println!("[Auth] Optional login");
    let session = manager.cloud_login("demo@cinemastudio.dev", "demo")?;
    println!("  Logged in as {:?}\n", session.email);

    // Gate 6.2 — cloud backup/restore
    println!("[6.2] Cloud backup");
    let backup = manager.cloud_backup()?;
    println!("  Backup {} bytes -> {}\n", backup.size_bytes, backup.path.display());

    let restore_dir = data_root.join("restored.csproj");
    manager.cloud_restore(&backup.path, &restore_dir)?;
    println!("  Restored to {}\n", restore_dir.display());

    // Billing stub
    println!("[Billing] Stripe Pro stub");
    manager.activate_pro_subscription()?;
    println!("  Tier: {:?}\n", manager.subscription_state()?.tier);

    // Telemetry
    println!("[6.4] Telemetry (opt-in)");
    manager.set_telemetry(true)?;
    manager.start_telemetry_session()?;
    manager.end_telemetry_session(false)?;
    println!("  Crash rate: {:.2}%\n", manager.telemetry_crash_rate()? * 100.0);

    // Beta gate
    println!("[6.3] Beta completion");
    let reg = manager.beta_mark_complete("demo_user")?;
    println!(
        "  Completions: {}/{} (gate met: {})\n",
        reg.count(),
        reg.target,
        reg.gate_met()
    );

    manager.cloud_logout()?;
    println!("Done. Data root: {}", data_root.display());
    Ok(())
}
