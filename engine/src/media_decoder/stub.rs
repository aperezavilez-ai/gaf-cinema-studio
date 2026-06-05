use std::path::Path;

use crate::error::Result;
use crate::media_decoder::types::{
    DecodeRequest, DecodedFrame, DecoderBackend, DecoderError, PixelFormat,
};

/// Contract for platform video decoders. Mobile shells implement via FFI callbacks.
pub trait VideoDecoder: Send {
    fn backend(&self) -> DecoderBackend;
    fn open(&mut self, path: &Path) -> Result<()>;
    fn seek(&mut self, time_ms: u64) -> Result<()>;
    fn decode_frame(&mut self, request: &DecodeRequest) -> Result<DecodedFrame>;
    fn close(&mut self);
}

pub struct StubDecoder {
    backend: DecoderBackend,
    open_path: Option<String>,
}

impl StubDecoder {
    pub fn new() -> Self {
        Self::with_backend(DecoderBackend::Stub)
    }

    pub fn with_backend(backend: DecoderBackend) -> Self {
        Self {
            backend,
            open_path: None,
        }
    }
}

impl Default for StubDecoder {
    fn default() -> Self {
        Self::new()
    }
}

impl VideoDecoder for StubDecoder {
    fn backend(&self) -> DecoderBackend {
        self.backend
    }

    fn open(&mut self, path: &Path) -> Result<()> {
        if !path.exists() {
            return Err(crate::error::CinemaError::ProjectNotFound(path.display().to_string()));
        }
        self.open_path = Some(path.display().to_string());
        Ok(())
    }

    fn seek(&mut self, _time_ms: u64) -> Result<()> {
        if self.open_path.is_none() {
            return Err(crate::error::CinemaError::Validation(
                DecoderError::NotOpen.to_string(),
            ));
        }
        Ok(())
    }

    fn decode_frame(&mut self, request: &DecodeRequest) -> Result<DecodedFrame> {
        if self.open_path.is_none() {
            return Err(crate::error::CinemaError::Validation(
                DecoderError::NotOpen.to_string(),
            ));
        }

        Ok(DecodedFrame {
            time_ms: request.time_ms,
            width: request.width,
            height: request.height,
            format: PixelFormat::Rgba8,
            pixel_bytes_len: request.width as u64 * request.height as u64 * 4,
            backend: self.backend,
        })
    }

    fn close(&mut self) {
        self.open_path = None;
    }
}
