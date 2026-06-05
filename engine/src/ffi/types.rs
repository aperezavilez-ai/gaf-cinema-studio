//! Serializable DTOs for mobile FFI — JSON over the wire until records are generated.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FfiFrameComposition {
    pub time_ms: u64,
    pub video_layer_count: u32,
    pub primary_path: Option<String>,
    pub uses_proxy: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FfiExportStatus {
    pub active_export_id: Option<String>,
    pub history_count: u32,
    pub last_output_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FfiAiSuggestion {
    pub id: String,
    pub message: String,
    pub priority: String,
    pub action_label: Option<String>,
    pub is_actionable: bool,
}

pub fn to_json<T: Serialize>(value: &T) -> Result<String, String> {
    serde_json::to_string(value).map_err(|e| e.to_string())
}

pub fn from_json<T: for<'de> Deserialize<'de>>(json: &str) -> Result<T, String> {
    serde_json::from_str(json).map_err(|e| e.to_string())
}
