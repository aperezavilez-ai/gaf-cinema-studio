//! Video decode layer — native backends attach here (AVFoundation / MediaCodec).

mod stub;
mod types;

pub use stub::{StubDecoder, VideoDecoder};
pub use types::{DecodeRequest, DecodedFrame, DecoderBackend, DecoderError, PixelFormat};

use std::path::Path;

use crate::error::Result;

pub struct DecoderRegistry {
    backend: DecoderBackend,
}

impl DecoderRegistry {
    pub fn new(backend: DecoderBackend) -> Self {
        Self { backend }
    }

    pub fn backend(&self) -> DecoderBackend {
        self.backend
    }

    pub fn decode_at(&self, path: &Path, time_ms: u64, width: u32, height: u32) -> Result<DecodedFrame> {
        let mut decoder = StubDecoder::with_backend(self.backend);
        decoder.open(path)?;
        decoder.seek(time_ms)?;
        decoder.decode_frame(&DecodeRequest {
            path: path.to_path_buf(),
            time_ms,
            width,
            height,
            prefer_proxy: true,
        })
    }
}

impl Default for DecoderRegistry {
    fn default() -> Self {
        Self::new(DecoderBackend::Stub)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn stub_decoder_returns_metadata_frame() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("clip.mp4");
        fs::write(&path, b"fake").unwrap();

        let reg = DecoderRegistry::default();
        let frame = reg.decode_at(&path, 500, 1920, 1080).unwrap();
        assert_eq!(frame.time_ms, 500);
        assert_eq!(frame.width, 1920);
    }
}
