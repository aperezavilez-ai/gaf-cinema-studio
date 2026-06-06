//! C ABI for mobile shells — link without UniFFI bindgen (Swift/Kotlin direct).
//! All string outputs are heap-allocated; caller must call `cs_c_free_string`.

use std::ffi::{CStr, CString};
use std::os::raw::c_char;

use uuid::Uuid;

use crate::{
    default_project_settings, parse_hints_json, AddClipParams, ExportSettings, ThermalLevel,
};
use crate::ffi::engine_state::{init_engine, with_manager};
use crate::ffi::types::{
    FfiAiSuggestion, FfiExportStatus, FfiFrameComposition, FfiTimelineClip, FfiTimelineInfo, to_json,
};
use crate::project_state::types::TrackType;
use crate::timeline_engine::resolve::resolve_frame;

fn to_c_string(s: String) -> *mut c_char {
    CString::new(s).unwrap_or_default().into_raw()
}

fn frame_to_dto(frame: crate::timeline_engine::resolve::FrameComposition) -> FfiFrameComposition {
    let primary = frame.primary_video();
    FfiFrameComposition {
        time_ms: frame.time_ms,
        video_layer_count: frame.video_layers.len() as u32,
        primary_path: primary.map(|l| l.playback_path.display().to_string()),
        uses_proxy: primary.map(|l| l.uses_proxy).unwrap_or(false),
        source_time_ms: primary.map(|l| l.source_time_ms),
    }
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
        to_json(&frame_to_dto(frame))
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
            Some(f) => to_json(&frame_to_dto(f)),
            None => Ok("null".into()),
        }
    });
    match result {
        Ok(json) => to_c_string(json),
        Err(e) => to_c_string(format!("ERROR:{e}")),
    }
}

#[no_mangle]
pub extern "C" fn cs_c_add_clip(media_id: *const c_char, start_ms: i64) -> *mut c_char {
    let result = (|| {
        let media_id = from_c_str(media_id)?;
        let id = Uuid::parse_str(&media_id).map_err(|e| e.to_string())?;
        let start = if start_ms < 0 { None } else { Some(start_ms as u64) };
        with_manager(|m| {
            let clip_id = m.add_clip_to_timeline(AddClipParams {
                media_id: id,
                track_id: None,
                start_ms: start,
            })?;
            Ok(clip_id.to_string())
        })
    })();
    match result {
        Ok(id) => to_c_string(id),
        Err(e) => to_c_string(format!("ERROR:{e}")),
    }
}

#[no_mangle]
pub extern "C" fn cs_c_split_at_playhead() -> i32 {
    match with_manager(|m| Ok(m.split_at_playhead()?.is_some())) {
        Ok(true) => 1,
        Ok(false) => 0,
        Err(_) => -1,
    }
}

#[no_mangle]
pub extern "C" fn cs_c_delete_at_playhead() -> i32 {
    match with_manager(|m| m.delete_at_playhead()) {
        Ok(true) => 1,
        Ok(false) => 0,
        Err(_) => -1,
    }
}

#[no_mangle]
pub extern "C" fn cs_c_timeline_info() -> *mut c_char {
    let result = with_manager(|m| {
        let state = m.state()?;
        let mut clips: Vec<FfiTimelineClip> = Vec::new();
        for track in &state.timeline.tracks {
            if track.track_type != TrackType::Video {
                continue;
            }
            for clip in &track.clips {
                clips.push(FfiTimelineClip {
                    id: clip.id.to_string(),
                    start_ms: clip.start_ms,
                    duration_ms: clip.duration_ms,
                    fade_in_ms: clip.transitions.fade_in_ms,
                    fade_out_ms: clip.transitions.fade_out_ms,
                    label: clip.label.clone(),
                    lens_preset: clip.look.lens_preset.clone(),
                    camera_angle: clip.look.camera_angle,
                    brightness: clip.look.brightness,
                    contrast: clip.look.contrast,
                    saturation: clip.look.saturation,
                });
            }
        }
        clips.sort_by_key(|c| c.start_ms);
        let clip_count = clips.len() as u32;
        let frame = resolve_frame(state, state.timeline.playhead_ms);
        let dto = FfiTimelineInfo {
            duration_ms: state.timeline.duration_ms,
            playhead_ms: state.timeline.playhead_ms,
            clip_count,
            primary_path: frame
                .primary_video()
                .map(|l| l.playback_path.display().to_string()),
            clips,
        };
        to_json(&dto)
    });
    match result {
        Ok(json) => to_c_string(json),
        Err(e) => to_c_string(format!("ERROR:{e}")),
    }
}

#[no_mangle]
pub extern "C" fn cs_c_trim_clip(
    clip_id: *const c_char,
    new_timeline_start: u64,
    new_timeline_end: u64,
) -> i32 {
    let result = (|| {
        let id_str = from_c_str(clip_id)?;
        let clip_id = Uuid::parse_str(&id_str).map_err(|e| e.to_string())?;
        with_manager(|m| {
            let state = m.state()?;
            let clip = state
                .timeline
                .tracks
                .iter()
                .flat_map(|t| t.clips.iter())
                .find(|c| c.id == clip_id)
                .ok_or_else(|| "clip not found".to_string())?;
            let clip_end = clip.start_ms + clip.duration_ms;
            if new_timeline_start >= new_timeline_end || new_timeline_end > clip_end {
                return Err("invalid trim range".into());
            }
            let delta_start = new_timeline_start.saturating_sub(clip.start_ms);
            let source_in = clip.source_in_ms + delta_start;
            let source_out = source_in + (new_timeline_end - new_timeline_start);
            m.trim_clip(clip_id, source_in, source_out)?;
            if new_timeline_start != clip.start_ms {
                m.move_clip(clip_id, new_timeline_start)?;
            }
            Ok(())
        })
    })();
    match result {
        Ok(()) => 1,
        Err(_) => 0,
    }
}

#[no_mangle]
pub extern "C" fn cs_c_ai_execute(suggestion_id: *const c_char) -> *mut c_char {
    let result = (|| {
        let id_str = from_c_str(suggestion_id)?;
        let id = Uuid::parse_str(&id_str).map_err(|e| e.to_string())?;
        with_manager(|m| m.ai_execute(id))
    })();
    match result {
        Ok(msg) => to_c_string(msg),
        Err(e) => to_c_string(format!("ERROR:{e}")),
    }
}

#[no_mangle]
pub extern "C" fn cs_c_ai_dismiss(suggestion_id: *const c_char) -> i32 {
    let result = (|| {
        let id_str = from_c_str(suggestion_id)?;
        let id = Uuid::parse_str(&id_str).map_err(|e| e.to_string())?;
        with_manager(|m| m.ai_dismiss(id)).map(|_| ())
    })();
    match result {
        Ok(()) => 1,
        Err(_) => 0,
    }
}

#[no_mangle]
pub extern "C" fn cs_c_ai_analyze() -> i32 {
    match with_manager(|m| m.ai_analyze().map(|_| ())) {
        Ok(()) => 1,
        Err(_) => -1,
    }
}

#[no_mangle]
pub extern "C" fn cs_c_duplicate_at_playhead() -> i32 {
    match with_manager(|m| m.duplicate_at_playhead()) {
        Ok(true) => 1,
        Ok(false) => 0,
        Err(_) => -1,
    }
}

#[no_mangle]
pub extern "C" fn cs_c_set_clip_look(json: *const c_char) -> i32 {
    let result = (|| {
        let raw = from_c_str(json)?;
        let v: serde_json::Value = serde_json::from_str(&raw).map_err(|e| e.to_string())?;
        let clip_id = v
            .get("clipId")
            .and_then(|x| x.as_str())
            .ok_or_else(|| "clipId required".to_string())?;
        let id = Uuid::parse_str(clip_id).map_err(|e| e.to_string())?;
        let lens = v
            .get("lensPreset")
            .and_then(|x| x.as_str())
            .unwrap_or("none")
            .to_string();
        let angle = v.get("cameraAngle").and_then(|x| x.as_u64()).unwrap_or(1) as u8;
        let brightness = v.get("brightness").and_then(|x| x.as_f64()).unwrap_or(0.0) as f32;
        let contrast = v.get("contrast").and_then(|x| x.as_f64()).unwrap_or(1.0) as f32;
        let saturation = v.get("saturation").and_then(|x| x.as_f64()).unwrap_or(1.0) as f32;
        with_manager(|m| {
            m.set_clip_look(id, lens, angle, brightness, contrast, saturation)?;
            Ok(())
        })
    })();
    match result {
        Ok(()) => 1,
        Err(_) => 0,
    }
}

#[no_mangle]
pub extern "C" fn cs_c_switch_camera_angle(angle: u8) -> i32 {
    match with_manager(|m| m.switch_camera_angle(angle)) {
        Ok(true) => 1,
        Ok(false) => 0,
        Err(_) => -1,
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
