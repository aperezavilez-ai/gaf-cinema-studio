//! Full UniFFI surface — all manager capabilities exposed for iOS/Android.
//! Complex types returned as JSON strings; wire native callbacks at integration.

#[cfg(feature = "ffi")]
mod types;

#[cfg(feature = "ffi")]
use std::path::PathBuf;
#[cfg(feature = "ffi")]
use std::sync::Mutex;

#[cfg(feature = "ffi")]
use uuid::Uuid;

#[cfg(feature = "ffi")]
use crate::{
    default_project_settings, parse_hints_json, AddClipParams, ExportSettings, ProjectStateManager,
    ThermalLevel,
};

#[cfg(feature = "ffi")]
use types::{FfiAiSuggestion, FfiExportStatus, FfiFrameComposition, to_json};

#[cfg(feature = "ffi")]
static ENGINE: Mutex<Option<ProjectStateManager>> = Mutex::new(None);

#[cfg(feature = "ffi")]
fn with_manager<F, T>(f: F) -> Result<T, String>
where
    F: FnOnce(&mut ProjectStateManager) -> crate::Result<T>,
{
    let mut guard = ENGINE.lock().map_err(|e| format!("lock error: {e}"))?;
    if guard.is_none() {
        *guard = Some(ProjectStateManager::new());
    }
    f(guard.as_mut().unwrap()).map_err(|e| e.to_string())
}

// ─── Lifecycle ───────────────────────────────────────────────────────────────

#[cfg(feature = "ffi")]
#[uniffi::export]
pub fn cs_engine_init(data_root: Option<String>) {
    let mut guard = ENGINE.lock().unwrap();
    *guard = Some(match data_root {
        Some(root) => ProjectStateManager::with_data_root(PathBuf::from(root)),
        None => ProjectStateManager::new(),
    });
}

#[cfg(feature = "ffi")]
#[uniffi::export]
pub fn cs_bridge_status() -> Result<String, String> {
    Ok(crate::native_bridge::bridge_status().to_string())
}

// ─── Project ─────────────────────────────────────────────────────────────────

#[cfg(feature = "ffi")]
#[uniffi::export]
pub fn cs_create_project(name: String, parent_dir: String) -> Result<String, String> {
    with_manager(|m| {
        m.create_project(name, &parent_dir, default_project_settings())?;
        Ok(m.project_path()
            .unwrap()
            .to_string_lossy()
            .into_owned())
    })
}

#[cfg(feature = "ffi")]
#[uniffi::export]
pub fn cs_open_project(project_dir: String) -> Result<String, String> {
    with_manager(|m| {
        m.open_project(&project_dir)?;
        let _ = m.start_telemetry_session();
        Ok(m.state().unwrap().metadata.name.clone())
    })
}

#[cfg(feature = "ffi")]
#[uniffi::export]
pub fn cs_save_project() -> Result<(), String> {
    with_manager(|m| m.save()).map(|_| ())
}

#[cfg(feature = "ffi")]
#[uniffi::export]
pub fn cs_project_name() -> Result<String, String> {
    with_manager(|m| {
        Ok(m.state()
            .ok_or_else(|| crate::CinemaError::Validation("no project".into()))?
            .metadata
            .name
            .clone())
    })
}

#[cfg(feature = "ffi")]
#[uniffi::export]
pub fn cs_import_media(source_path: String) -> Result<String, String> {
    with_manager(|m| {
        let asset = m.import_media(&source_path)?;
        Ok(asset.id.to_string())
    })
}

// ─── Timeline + playback ─────────────────────────────────────────────────────

#[cfg(feature = "ffi")]
#[uniffi::export]
pub fn cs_add_clip(media_id: String, start_ms: Option<u64>) -> Result<String, String> {
    with_manager(|m| {
        let id = Uuid::parse_str(&media_id).map_err(|e| crate::CinemaError::Validation(e.to_string()))?;
        let clip_id = m.add_clip_to_timeline(AddClipParams {
            media_id: id,
            track_id: None,
            start_ms,
        })?;
        Ok(clip_id.to_string())
    })
}

#[cfg(feature = "ffi")]
#[uniffi::export]
pub fn cs_scrub_to(time_ms: u64) -> Result<String, String> {
    with_manager(|m| {
        let frame = m.scrub_to(time_ms)?;
        let dto = FfiFrameComposition {
            time_ms: frame.time_ms,
            video_layer_count: frame.video_layers.len() as u32,
            primary_path: frame
                .primary_video()
                .map(|l| l.playback_path.display().to_string()),
            uses_proxy: frame.primary_video().map(|l| l.uses_proxy).unwrap_or(false),
        };
        to_json(&dto)
    })
}

#[cfg(feature = "ffi")]
#[uniffi::export]
pub fn cs_playback_play() -> Result<(), String> {
    with_manager(|m| m.playback_play()).map(|_| ())
}

#[cfg(feature = "ffi")]
#[uniffi::export]
pub fn cs_playback_pause() -> Result<(), String> {
    with_manager(|m| m.playback_pause()).map(|_| ())
}

#[cfg(feature = "ffi")]
#[uniffi::export]
pub fn cs_playback_tick() -> Result<String, String> {
    with_manager(|m| {
        let frame = m.playback_tick()?;
        match frame {
            Some(f) => {
                let dto = FfiFrameComposition {
                    time_ms: f.time_ms,
                    video_layer_count: f.video_layers.len() as u32,
                    primary_path: f
                        .primary_video()
                        .map(|l| l.playback_path.display().to_string()),
                    uses_proxy: f.primary_video().map(|l| l.uses_proxy).unwrap_or(false),
                };
                to_json(&dto)
            }
            None => Ok("null".into()),
        }
    })
}

// ─── Edit + undo ─────────────────────────────────────────────────────────────

#[cfg(feature = "ffi")]
#[uniffi::export]
pub fn cs_split_at_playhead() -> Result<Option<String>, String> {
    with_manager(|m| {
        Ok(m.split_at_playhead()?.map(|id| id.to_string()))
    })
}

#[cfg(feature = "ffi")]
#[uniffi::export]
pub fn cs_delete_at_playhead() -> Result<bool, String> {
    with_manager(|m| Ok(m.delete_at_playhead()?))
}

#[cfg(feature = "ffi")]
#[uniffi::export]
pub fn cs_undo() -> Result<bool, String> {
    with_manager(|m| m.undo())
}

#[cfg(feature = "ffi")]
#[uniffi::export]
pub fn cs_redo() -> Result<bool, String> {
    with_manager(|m| m.redo())
}

#[cfg(feature = "ffi")]
#[uniffi::export]
pub fn cs_can_undo() -> Result<bool, String> {
    with_manager(|m| Ok(m.can_undo()))
}

#[cfg(feature = "ffi")]
#[uniffi::export]
pub fn cs_can_redo() -> Result<bool, String> {
    with_manager(|m| Ok(m.can_redo()))
}

// ─── Export ──────────────────────────────────────────────────────────────────

#[cfg(feature = "ffi")]
#[uniffi::export]
pub fn cs_start_export(width: u32, height: u32, frame_rate: f64) -> Result<String, String> {
    with_manager(|m| {
        let id = m.start_export(ExportSettings {
            width,
            height,
            frame_rate,
            ..Default::default()
        })?;
        Ok(id.to_string())
    })
}

#[cfg(feature = "ffi")]
#[uniffi::export]
pub fn cs_sync_export_status() -> Result<(), String> {
    with_manager(|m| m.sync_export_status()).map(|_| ())
}

#[cfg(feature = "ffi")]
#[uniffi::export]
pub fn cs_export_status() -> Result<String, String> {
    with_manager(|m| {
        let state = m.state().ok_or_else(|| crate::CinemaError::Validation("no project".into()))?;
        let last = state.export_state.history.last();
        let dto = FfiExportStatus {
            active_export_id: state.export_state.active_export_id.map(|id| id.to_string()),
            history_count: state.export_state.history.len() as u32,
            last_output_path: last.and_then(|r| r.output_path.clone()),
        };
        to_json(&dto)
    })
}

// ─── AI ──────────────────────────────────────────────────────────────────────

#[cfg(feature = "ffi")]
#[uniffi::export]
pub fn cs_ai_analyze() -> Result<(), String> {
    with_manager(|m| m.ai_analyze()).map(|_| ())
}

#[cfg(feature = "ffi")]
#[uniffi::export]
pub fn cs_ai_suggestions() -> Result<String, String> {
    with_manager(|m| {
        let items: Vec<FfiAiSuggestion> = m
            .ai_suggestions()
            .into_iter()
            .map(|s| FfiAiSuggestion {
                id: s.id.to_string(),
                message: s.message,
                priority: format!("{:?}", s.priority),
                action_label: Some(s.action_id.clone()),
                is_actionable: !s.action_id.is_empty(),
            })
            .collect();
        to_json(&items)
    })
}

#[cfg(feature = "ffi")]
#[uniffi::export]
pub fn cs_ai_execute(suggestion_id: String) -> Result<String, String> {
    with_manager(|m| {
        let id = Uuid::parse_str(&suggestion_id)
            .map_err(|e| crate::CinemaError::Validation(e.to_string()))?;
        m.ai_execute(id)
    })
}

#[cfg(feature = "ffi")]
#[uniffi::export]
pub fn cs_ai_dismiss(suggestion_id: String) -> Result<(), String> {
    with_manager(|m| {
        let id = Uuid::parse_str(&suggestion_id)
            .map_err(|e| crate::CinemaError::Validation(e.to_string()))?;
        m.ai_dismiss(id)
    })
}

// ─── Device + performance ────────────────────────────────────────────────────

#[cfg(feature = "ffi")]
#[uniffi::export]
pub fn cs_set_device_hints(json: String) -> Result<(), String> {
    with_manager(|m| {
        let profile = parse_hints_json(&json)?;
        m.set_device_profile(profile)
    })
    .map(|_| ())
}

#[cfg(feature = "ffi")]
#[uniffi::export]
pub fn cs_set_thermal(level: String) -> Result<(), String> {
    with_manager(|m| {
        let thermal = match level.to_lowercase().as_str() {
            "normal" | "nominal" => ThermalLevel::Normal,
            "warm" | "fair" => ThermalLevel::Warm,
            "hot" | "serious" => ThermalLevel::Hot,
            "critical" => ThermalLevel::Critical,
            other => {
                return Err(crate::CinemaError::Validation(format!(
                    "unknown thermal level: {other}"
                )))
            }
        };
        m.set_thermal_level(thermal)
    })
    .map(|_| ())
}

#[cfg(feature = "ffi")]
#[uniffi::export]
pub fn cs_performance_report() -> Result<String, String> {
    with_manager(|m| Ok(m.performance_report().to_string()))
}

// ─── Cloud + billing (optional) ──────────────────────────────────────────────

#[cfg(feature = "ffi")]
#[uniffi::export]
pub fn cs_cloud_login(email: String, password: String) -> Result<(), String> {
    with_manager(|m| m.cloud_login(&email, &password)).map(|_| ())
}

#[cfg(feature = "ffi")]
#[uniffi::export]
pub fn cs_cloud_logout() -> Result<(), String> {
    with_manager(|m| m.cloud_logout()).map(|_| ())
}

#[cfg(feature = "ffi")]
#[uniffi::export]
pub fn cs_cloud_backup() -> Result<String, String> {
    with_manager(|m| {
        let record = m.cloud_backup()?;
        to_json(&serde_json::json!({
            "backupId": record.backup_id.to_string(),
            "sizeBytes": record.size_bytes,
            "path": record.path.display().to_string(),
        }))
    })
}

#[cfg(feature = "ffi")]
#[uniffi::export]
pub fn cs_activate_pro() -> Result<(), String> {
    with_manager(|m| m.activate_pro_subscription()).map(|_| ())
}

// ─── Integration hooks ───────────────────────────────────────────────────────

#[cfg(feature = "ffi")]
#[uniffi::export]
pub fn cs_set_render_backend(name: String) -> Result<(), String> {
    match name.to_lowercase().as_str() {
        "stub" => crate::native_bridge::set_render_backend(crate::render_pipeline::RenderBackend::Stub),
        "ffmpeg" => crate::native_bridge::set_render_backend(crate::render_pipeline::RenderBackend::Ffmpeg),
        other => return Err(format!("unknown render backend: {other}")),
    }
    Ok(())
}

#[cfg(feature = "ffi")]
#[uniffi::export]
pub fn cs_set_decoder_backend(name: String) -> Result<(), String> {
    use crate::media_decoder::DecoderBackend;
    match name.to_lowercase().as_str() {
        "stub" => crate::native_bridge::set_decoder_backend(DecoderBackend::Stub),
        "avfoundation" => crate::native_bridge::set_decoder_backend(DecoderBackend::AvFoundation),
        "mediacodec" => crate::native_bridge::set_decoder_backend(DecoderBackend::MediaCodec),
        other => return Err(format!("unknown decoder backend: {other}")),
    }
    Ok(())
}

#[cfg(feature = "ffi")]
uniffi::setup_scaffolding!();
