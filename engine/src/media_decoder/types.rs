use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum DecoderBackend {
    Stub,
    AvFoundation,
    MediaCodec,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PixelFormat {
    Rgba8,
    Nv12,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DecodeRequest {
    pub path: PathBuf,
    pub time_ms: u64,
    pub width: u32,
    pub height: u32,
    pub prefer_proxy: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DecodedFrame {
    pub time_ms: u64,
    pub width: u32,
    pub height: u32,
    pub format: PixelFormat,
    /// Empty in stub — native backends fill pixel buffer handle or bytes
    pub pixel_bytes_len: u64,
    pub backend: DecoderBackend,
}

#[derive(Debug, thiserror::Error)]
pub enum DecoderError {
    #[error("decoder not open")]
    NotOpen,
    #[error("seek failed at {0}ms")]
    SeekFailed(u64),
    #[error("native backend not wired")]
    NativeNotWired,
}
