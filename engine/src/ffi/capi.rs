//! C ABI for mobile shells — link without UniFFI bindgen (Swift/Kotlin direct).
//! All string outputs are heap-allocated; caller must call `cs_c_free_string`.

use std::ffi::{CStr, CString};
use std::os::raw::c_char;

use uuid::Uuid;

use crate::{
    default_project_settings, parse_hints_json, AddClipParams, ExportSettings, ThermalLevel,
};
use crate::ffi::engine_state::{init_engine, with_manager};
use crate::ffi::types::{FfiAiSuggestion, FfiExportStatus, FfiFrameComposition, to_json};

fn to_c_string(s: String) -> *mut c_char {
    CString::new(s).unwrap_or_default().into_raw()
}

fn from_c_str(ptr: *const c_char) -> Result<String, String> {
    if ptr.is_null() {
        return Err("null pointer".into());
    }
    unsafe {
        CStr::from_ptr(ptr)
            .to_str()
            .map(|s| s.to_owned())
            .map_err(|e| e.to_string())
    }
}

#[no_mangle]
pub extern "C" fn cs_c_free_string(s: *mut c_char) {
    if !s.is_null() {
        unsafe {
            drop(CString::from_raw(s));
        }
    }
}

#[no_mangle]
pub extern "C" fn cs_c_engine_init(data_root: *const c_char) -> i32 {
    let root = if data_root.is_null() {
        None
    } else {
        match from_c_str(data_root) {
            Ok(s) => Some(s),
            Err(_) => return -1,
        }
    };
    init_engine(root);
    0
}

#[no_mangle]
pub extern "C" fn cs_c_bridge_status() -> *mut c_char {
    to_c_string(crate::native_bridge::bridge_status().to_string())
}

#[no_mangle]
pub extern "C" fn cs_c_create_project(name: *const c_char, parent_dir: *const c_char) -> *mut c_char {
    let result = (|| {
        let name = from_c_str(name)?;
        let parent = from_c_str(parent_dir)?;
        with_manager(|m| {
            m.create_project(name, &parent, default_project_settings())?;
            Ok(m.project_path().unwrap().to_string_lossy().into_owned())
        })
    })();
    match result {
        Ok(path) => to_c_string(path),
        Err(e) => to_c_string(format!("ERROR:{e}")),
    }
}

#[no_mangle]
pub extern "C" fn cs_c_open_project(project_dir: *const c_char) -> *mut c_char {
    let result = (|| {
        let dir = from_c_str(project_dir)?;
        with_manager(|m| {
            m.open_project(&dir)?;
            let _ = m.start_telemetry_session();
            Ok(m.state().unwrap().metadata.name.clone())
        })
    })();
    match result {
        Ok(name) => to_c_string(name),
        Err(e) => to_c_string(format!("ERROR:{e}")),
    }
}

#[no_mangle]
pub extern "C" fn cs_c_save_project() -> i32 {
    match with_manager(|m| m.save()) {
        Ok(()) => 0,
        Err(_) => -1,
    }
}

#[no_mangle]
pub extern "C" fn cs_c_import_media(source_path: *const c_char) -> *mut c_char {
    let result = (|| {
        let path = from_c_str(source_path)?;
        with_manager(|m| Ok(m.import_media(&path)?.id.to_string()))
    })();
    match result {
        Ok(id) => to_c_string(id),
        Err(e) => to_c_string(format!("ERROR:{e}")),
    }
}

#[no_mangle]
pub extern "C" fn cs_c_scrub_to(time_ms: u64) -> *mut c_char {
    let result = with_manager(|m| {
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
    });
    match result {
        Ok(json) => to_c_string(json),
        Err(e) => to_c_string(format!("ERROR:{e}")),
    }
}

#[no_mangle]
pub extern "C" fn cs_c_playback_play() -> i32 {
    match with_manager(|m| m.playback_play()) {
        Ok(()) => 0,
        Err(_) => -1,
    }
}

#[no_mangle]
pub extern "C" fn cs_c_playback_pause() -> i32 {
    match with_manager(|m| m.playback_pause()) {
        Ok(()) => 0,
        Err(_) => -1,
    }
}

#[no_mangle]
pub extern "C" fn cs_c_playback_tick() -> *mut c_char {
    let result = with_manager(|m| {
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
    });
    match result {
        Ok(json) => to_c_string(json),
        Err(e) => to_c_string(format!("ERROR:{e}")),
    }
}

#[no_mangle]
pub extern "C" fn cs_c_undo() -> i32 {
    match with_manager(|m| m.undo()) {
        Ok(true) => 1,
        Ok(false) => 0,
        Err(_) => -1,
    }
}

#[no_mangle]
pub extern "C" fn cs_c_redo() -> i32 {
    match with_manager(|m| m.redo()) {
        Ok(true) => 1,
        Ok(false) => 0,
        Err(_) => -1,
    }
}

#[no_mangle]
pub extern "C" fn cs_c_can_undo() -> i32 {
    match with_manager(|m| Ok(m.can_undo())) {
        Ok(v) => i32::from(v),
        Err(_) => -1,
    }
}

#[no_mangle]
pub extern "C" fn cs_c_can_redo() -> i32 {
    match with_manager(|m| Ok(m.can_redo())) {
        Ok(v) => i32::from(v),
        Err(_) => -1,
    }
}

#[no_mangle]
pub extern "C" fn cs_c_ai_suggestions() -> *mut c_char {
    let result = with_manager(|m| {
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
    });
    match result {
        Ok(json) => to_c_string(json),
        Err(e) => to_c_string(format!("ERROR:{e}")),
    }
}

#[no_mangle]
pub extern "C" fn cs_c_set_decoder_backend(name: *const c_char) -> i32 {
    use crate::media_decoder::DecoderBackend;
    let Ok(name) = from_c_str(name) else { return -1 };
    match name.to_lowercase().as_str() {
        "stub" => crate::native_bridge::set_decoder_backend(DecoderBackend::Stub),
        "avfoundation" => crate::native_bridge::set_decoder_backend(DecoderBackend::AvFoundation),
        "mediacodec" => crate::native_bridge::set_decoder_backend(DecoderBackend::MediaCodec),
        _ => return -1,
    }
    0
}

#[no_mangle]
pub extern "C" fn cs_c_set_render_backend(name: *const c_char) -> i32 {
    use crate::render_pipeline::RenderBackend;
    let Ok(name) = from_c_str(name) else { return -1 };
    match name.to_lowercase().as_str() {
        "stub" => crate::native_bridge::set_render_backend(RenderBackend::Stub),
        "ffmpeg" => crate::native_bridge::set_render_backend(RenderBackend::Ffmpeg),
        _ => return -1,
    }
    0
}

#[no_mangle]
pub extern "C" fn cs_c_start_export(width: u32, height: u32, frame_rate: f64) -> *mut c_char {
    let result = with_manager(|m| {
        m.start_export(ExportSettings {
            width,
            height,
            frame_rate,
            ..Default::default()
        })
    });
    match result {
        Ok(id) => to_c_string(id.to_string()),
        Err(e) => to_c_string(format!("ERROR:{e}")),
    }
}

#[no_mangle]
pub extern "C" fn cs_c_export_status() -> *mut c_char {
    let result = with_manager(|m| Ok(m.export_status().to_string()));
    match result {
        Ok(json) => to_c_string(json),
        Err(e) => to_c_string(format!("ERROR:{e}")),
    }
}

#[no_mangle]
pub extern "C" fn cs_c_ffmpeg_available() -> i32 {
    i32::from(crate::render_pipeline::ffmpeg_available())
}

#[no_mangle]
pub extern "C" fn cs_c_mvp_readiness() -> *mut c_char {
    let result = with_manager(|m| {
        let report = m.mvp_readiness_report()?;
        serde_json::to_string(&report).map_err(|e| e.to_string())
    });
    match result {
        Ok(json) => to_c_string(json),
        Err(e) => to_c_string(format!("ERROR:{e}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;

    #[test]
    fn capi_create_project_roundtrip() {
        let tmp = tempfile::TempDir::new().unwrap();
        let parent = CString::new(tmp.path().to_string_lossy().as_bytes()).unwrap();
        let name = CString::new("CAPI").unwrap();
        cs_c_engine_init(std::ptr::null());
        let out = cs_c_create_project(name.as_ptr(), parent.as_ptr());
        assert!(!out.is_null());
        let s = unsafe { CStr::from_ptr(out).to_str().unwrap() };
        assert!(!s.starts_with("ERROR:"));
        cs_c_free_string(out);
    }
}
