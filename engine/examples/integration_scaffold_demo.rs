//! Phase 7 integration scaffold demo — decoder, render pipeline, native bridge.

use std::fs;

use cinemastudio_engine::{
    bridge_status, decode_frame_at, default_project_settings, set_decoder_backend,
    set_render_backend, AddClipParams, DecoderBackend, ProjectStateManager, RenderBackend,
};

fn main() -> cinemastudio_engine::Result<()> {
    let tmp = std::env::temp_dir().join("CinemaStudioPhase7Demo");
    std::fs::create_dir_all(&tmp)?;
    let clip = tmp.join("clip.mp4");
    fs::write(&clip, b"fake mp4")?;

    println!("=== Phase 7: Integration Scaffold ===\n");
    println!("Bridge status: {}\n", bridge_status());

    set_decoder_backend(DecoderBackend::Stub);
    set_render_backend(RenderBackend::Stub);

    let mut manager = ProjectStateManager::with_data_root(tmp.clone());
    manager.create_project("Integration", &tmp, default_project_settings())?;
    let asset = manager.import_media(&clip)?;
    manager.add_clip_to_timeline(AddClipParams {
        media_id: asset.id,
        track_id: None,
        start_ms: None,
    })?;

    manager.scrub_to(0)?;
    let decoded = manager.decode_at_playhead()?;
    println!(
        "Decoded frame: {}x{} @ {}ms (backend {:?})",
        decoded.width, decoded.height, decoded.time_ms, decoded.backend
    );

    let export_id = manager.start_export(Default::default())?;
    std::thread::sleep(std::time::Duration::from_millis(400));
    manager.sync_export_status()?;
    println!("Export started: {export_id}");

    println!("\nWire AVFoundation / FFmpeg at integration — see docs/INTEGRATION.md");
    Ok(())
}
