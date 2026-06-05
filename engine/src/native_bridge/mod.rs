//! Native platform bridge — callback slots for AVFoundation / MediaCodec / FFmpeg.
//! Wire at integration; stubs return NotWired until connected.

use std::path::Path;
use std::sync::{Arc, Mutex};

use crate::error::{CinemaError, Result};
use crate::media_decoder::{DecodedFrame, DecoderBackend, DecoderRegistry};
use crate::render_pipeline::{RenderBackend, RenderPipeline};

pub type DecodeCallback = Arc<dyn Fn(&Path, u64, u32, u32) -> Result<DecodedFrame> + Send + Sync>;

struct BridgeState {
    decode_callback: Option<DecodeCallback>,
    decoder_backend: DecoderBackend,
    render_backend: RenderBackend,
}

impl Default for BridgeState {
    fn default() -> Self {
        Self {
            decode_callback: None,
            decoder_backend: DecoderBackend::Stub,
            render_backend: RenderBackend::Stub,
        }
    }
}

static BRIDGE: Mutex<BridgeState> = Mutex::new(BridgeState {
    decode_callback: None,
    decoder_backend: DecoderBackend::Stub,
    render_backend: RenderBackend::Stub,
});

/// Register native decode callback from Swift/Kotlin (AVFoundation / MediaCodec).
pub fn register_decode_callback(callback: DecodeCallback) {
    let mut guard = BRIDGE.lock().unwrap();
    guard.decode_callback = Some(callback);
    guard.decoder_backend = DecoderBackend::AvFoundation;
}

pub fn set_decoder_backend(backend: DecoderBackend) {
    BRIDGE.lock().unwrap().decoder_backend = backend;
}

pub fn set_render_backend(backend: RenderBackend) {
    BRIDGE.lock().unwrap().render_backend = backend;
}

pub fn decoder_registry() -> DecoderRegistry {
    DecoderRegistry::new(BRIDGE.lock().unwrap().decoder_backend)
}

pub fn render_pipeline() -> RenderPipeline {
    let backend = BRIDGE.lock().unwrap().render_backend;
    match backend {
        RenderBackend::Ffmpeg => RenderPipeline::for_backend(RenderBackend::Ffmpeg),
        RenderBackend::Stub if crate::render_pipeline::ffmpeg_available() => {
            RenderPipeline::for_backend(RenderBackend::Ffmpeg)
        }
        RenderBackend::Stub => RenderPipeline::for_backend(RenderBackend::Stub),
    }
}

pub fn init_render_backend() {
    if crate::render_pipeline::ffmpeg_available() {
        set_render_backend(RenderBackend::Ffmpeg);
    }
}

/// Decode frame — uses native callback if registered, else stub registry.
pub fn decode_frame_at(path: &Path, time_ms: u64, width: u32, height: u32) -> Result<DecodedFrame> {
    let guard = BRIDGE.lock().unwrap();
    if let Some(cb) = &guard.decode_callback {
        return cb(path, time_ms, width, height);
    }
    drop(guard);
    decoder_registry().decode_at(path, time_ms, width, height)
}

pub fn bridge_status() -> serde_json::Value {
    let guard = BRIDGE.lock().unwrap();
    serde_json::json!({
        "decodeCallbackRegistered": guard.decode_callback.is_some(),
        "decoderBackend": format!("{:?}", guard.decoder_backend),
        "renderBackend": format!("{:?}", guard.render_backend),
        "ffmpegAvailable": crate::render_pipeline::ffmpeg_available(),
    })
}

pub fn require_native_decode() -> Result<()> {
    if BRIDGE.lock().unwrap().decode_callback.is_some() {
        Ok(())
    } else {
        Err(CinemaError::Validation(
            "native decode callback not registered — wire AVFoundation/MediaCodec at integration".into(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn stub_decode_without_callback() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("v.mp4");
        fs::write(&path, b"x").unwrap();
        let frame = decode_frame_at(&path, 0, 1280, 720).unwrap();
        assert_eq!(frame.width, 1280);
    }
}
