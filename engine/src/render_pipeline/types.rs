use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum RenderBackend {
    Stub,
    Ffmpeg,
}

#[derive(Debug, Clone)]
pub struct RenderJob {
    pub export_id: Uuid,
    pub project_id: Uuid,
    pub project_dir: PathBuf,
    pub width: u32,
    pub height: u32,
    pub frame_rate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderProgress {
    pub export_id: Uuid,
    pub progress: f64,
    pub stage: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderResult {
    pub export_id: Uuid,
    pub output_path: PathBuf,
    pub backend: RenderBackend,
    pub duration_ms: u64,
    pub sidecar_path: Option<PathBuf>,
}
