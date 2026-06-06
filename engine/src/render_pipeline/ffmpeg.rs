//! FFmpeg CLI backend — real H.264 export when `ffmpeg` binary is available.
//! No libav required; uses subprocess. Set `CINEMASTUDIO_FFMPEG_PATH` to override.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use crate::error::{CinemaError, Result};
use crate::project_state::types::ProjectState;
use crate::render_pipeline::stub::StubRenderBackend;
use crate::render_pipeline::timeline_resolve::{resolve_export_segments, ExportSegment};
use crate::render_pipeline::types::{RenderBackend, RenderJob, RenderResult};
use crate::render_pipeline::RenderBackendImpl;

pub struct FfmpegRenderBackend;

impl FfmpegRenderBackend {
    pub fn new() -> Self {
        Self
    }
}

impl Default for FfmpegRenderBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl RenderBackendImpl for FfmpegRenderBackend {
    fn id(&self) -> RenderBackend {
        RenderBackend::Ffmpeg
    }

    fn render(
        &self,
        job: &RenderJob,
        state: &ProjectState,
        on_progress: &dyn Fn(f64),
    ) -> Result<RenderResult> {
        let Some(ffmpeg) = locate_ffmpeg() else {
            eprintln!("ffmpeg not found in PATH — stub fallback. Install ffmpeg or set CINEMASTUDIO_FFMPEG_PATH.");
            return StubRenderBackend.render(job, state, on_progress);
        };

        render_with_ffmpeg(&ffmpeg, job, state, on_progress)
    }
}

pub fn locate_ffmpeg() -> Option<PathBuf> {
    if let Ok(path) = std::env::var("CINEMASTUDIO_FFMPEG_PATH") {
        let p = PathBuf::from(path);
        if p.exists() {
            return Some(p);
        }
    }

    let cmd = if cfg!(windows) { "where" } else { "which" };
    let output = Command::new(cmd).arg("ffmpeg").output().ok()?;
    if !output.status.success() {
        return None;
    }
    let line = String::from_utf8_lossy(&output.stdout)
        .lines()
        .next()?
        .trim()
        .to_string();
    if line.is_empty() {
        None
    } else {
        Some(PathBuf::from(line))
    }
}

pub fn render_with_ffmpeg(
    ffmpeg: &Path,
    job: &RenderJob,
    state: &ProjectState,
    on_progress: &dyn Fn(f64),
) -> Result<RenderResult> {
    let segments = resolve_export_segments(state)?;
    let exports_dir = job.project_dir.join("exports");
    let temp_dir = job.project_dir.join("cache").join(format!("export_{}", job.export_id));
    fs::create_dir_all(&exports_dir)?;
    fs::create_dir_all(&temp_dir)?;

    on_progress(0.05);

    let segment_files = extract_segments(ffmpeg, &segments, &temp_dir)?;
    on_progress(0.4);

    let output_path = exports_dir.join(format!(
        "{}_{}x{}.mp4",
        job.export_id, job.width, job.height
    ));

    if segment_files.len() == 1 {
        encode_single(
            ffmpeg,
            &segment_files[0],
            &output_path,
            job.width,
            job.height,
            job.frame_rate,
        )?;
    } else {
        let list_file = temp_dir.join("concat.txt");
        write_concat_list(&list_file, &segment_files)?;
        concat_and_encode(
            ffmpeg,
            &list_file,
            &output_path,
            job.width,
            job.height,
            job.frame_rate,
        )?;
    }

    on_progress(0.95);

    let sidecar = write_export_sidecar(&exports_dir, job, state, &output_path, "ffmpeg-cli")?;
    let _ = fs::remove_dir_all(&temp_dir);

    on_progress(1.0);

    Ok(RenderResult {
        export_id: job.export_id,
        output_path,
        backend: RenderBackend::Ffmpeg,
        duration_ms: state.timeline.duration_ms,
        sidecar_path: Some(sidecar),
    })
}

fn build_look_filters(seg: &ExportSegment) -> Vec<String> {
    let mut parts = Vec::new();
    match seg.lens_preset.as_str() {
        "anamorphic" => {
            parts.push("vignette=angle=PI/4:mode=forward".to_string());
            parts.push("scale=1920:816:force_original_aspect_ratio=decrease,pad=1920:816:(ow-iw)/2:(oh-ih)/2".to_string());
        }
        "vintage" => {
            parts.push("vignette=angle=PI/5".to_string());
            parts.push("eq=saturation=0.85:contrast=1.05".to_string());
        }
        "wide" => {
            parts.push("scale=iw*1.08:ih*1.08,crop=iw/1.08:ih/1.08".to_string());
        }
        _ => {}
    }
    let b = seg.brightness.clamp(-1.0, 1.0);
    let c = seg.contrast.clamp(0.0, 2.0);
    let s = seg.saturation.clamp(0.0, 2.0);
    if (b.abs() > 0.01) || ((c - 1.0).abs() > 0.01) || ((s - 1.0).abs() > 0.01) {
        parts.push(format!("eq=brightness={b:.2}:contrast={c:.2}:saturation={s:.2}"));
    }
    parts
}

fn extract_segments(
    ffmpeg: &Path,
    segments: &[ExportSegment],
    temp_dir: &Path,
) -> Result<Vec<PathBuf>> {
    let mut paths = Vec::with_capacity(segments.len());

    for (i, seg) in segments.iter().enumerate() {
        let out = temp_dir.join(format!("seg_{i:03}.mp4"));
        let start_sec = format!("{:.3}", seg.source_in_ms as f64 / 1000.0);
        let dur_sec = format!("{:.3}", seg.duration_ms as f64 / 1000.0);
        let dur_f = seg.duration_ms as f64 / 1000.0;

        let mut vf_parts: Vec<String> = build_look_filters(seg);
        if seg.fade_in_ms > 0 {
            let d = seg.fade_in_ms as f64 / 1000.0;
            vf_parts.push(format!("fade=t=in:st=0:d={d:.3}"));
        }
        if seg.fade_out_ms > 0 {
            let d = seg.fade_out_ms as f64 / 1000.0;
            let st = (dur_f - d).max(0.0);
            vf_parts.push(format!("fade=t=out:st={st:.3}:d={d:.3}"));
        }

        let mut args = vec![
            "-y".to_string(),
            "-ss".to_string(),
            start_sec,
            "-i".to_string(),
            seg.source_path.to_string_lossy().into_owned(),
            "-t".to_string(),
            dur_sec,
        ];
        if !vf_parts.is_empty() {
            args.push("-vf".to_string());
            args.push(vf_parts.join(","));
        }
        args.extend([
            "-c:v".to_string(),
            "libx264".to_string(),
            "-preset".to_string(),
            "medium".to_string(),
            "-crf".to_string(),
            "23".to_string(),
            "-pix_fmt".to_string(),
            "yuv420p".to_string(),
            "-an".to_string(),
            out.to_string_lossy().into_owned(),
        ]);

        let status = Command::new(ffmpeg)
            .args(&args)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map_err(|e| CinemaError::Storage(format!("ffmpeg segment extract failed: {e}")))?;

        if !status.success() {
            return Err(CinemaError::Storage(format!(
                "ffmpeg segment {i} failed (exit {:?})",
                status.code()
            )));
        }
        paths.push(out);
    }

    Ok(paths)
}

fn write_concat_list(list_file: &Path, files: &[PathBuf]) -> Result<()> {
    let mut content = String::new();
    for f in files {
        let escaped = f.to_string_lossy().replace('\'', "'\\''");
        content.push_str(&format!("file '{escaped}'\n"));
    }
    fs::write(list_file, content)?;
    Ok(())
}

fn concat_and_encode(
    ffmpeg: &Path,
    list_file: &Path,
    output: &Path,
    width: u32,
    height: u32,
    fps: f64,
) -> Result<()> {
    let scale = format!("scale={width}:{height}:force_original_aspect_ratio=decrease,pad={width}:{height}:(ow-iw)/2:(oh-ih)/2");
    let fps_s = format!("{fps:.3}");

    run_ffmpeg(
        ffmpeg,
        &[
            "-y",
            "-f",
            "concat",
            "-safe",
            "0",
            "-i",
            &list_file.to_string_lossy(),
            "-vf",
            &scale,
            "-r",
            &fps_s,
            "-c:v",
            "libx264",
            "-preset",
            "medium",
            "-crf",
            "23",
            "-pix_fmt",
            "yuv420p",
            "-movflags",
            "+faststart",
            "-an",
            &output.to_string_lossy(),
        ],
    )
}

fn encode_single(
    ffmpeg: &Path,
    input: &Path,
    output: &Path,
    width: u32,
    height: u32,
    fps: f64,
) -> Result<()> {
    let scale = format!("scale={width}:{height}:force_original_aspect_ratio=decrease,pad={width}:{height}:(ow-iw)/2:(oh-ih)/2");
    let fps_s = format!("{fps:.3}");

    run_ffmpeg(
        ffmpeg,
        &[
            "-y",
            "-i",
            &input.to_string_lossy(),
            "-vf",
            &scale,
            "-r",
            &fps_s,
            "-c:v",
            "libx264",
            "-preset",
            "medium",
            "-crf",
            "23",
            "-pix_fmt",
            "yuv420p",
            "-movflags",
            "+faststart",
            "-an",
            &output.to_string_lossy(),
        ],
    )
}

fn run_ffmpeg(ffmpeg: &Path, args: &[&str]) -> Result<()> {
    let status = Command::new(ffmpeg)
        .args(args)
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .status()
        .map_err(|e| CinemaError::Storage(format!("ffmpeg spawn failed: {e}")))?;

    if status.success() {
        Ok(())
    } else {
        Err(CinemaError::Storage(format!(
            "ffmpeg exited with code {:?}",
            status.code()
        )))
    }
}

fn write_export_sidecar(
    exports_dir: &Path,
    job: &RenderJob,
    state: &ProjectState,
    output_path: &Path,
    backend: &str,
) -> Result<PathBuf> {
    let sidecar = exports_dir.join(format!("{}.export.json", job.export_id));
    let manifest = serde_json::json!({
        "exportId": job.export_id,
        "backend": backend,
        "codec": "h264",
        "resolution": format!("{}x{}", job.width, job.height),
        "frameRate": job.frame_rate,
        "outputPath": output_path.display().to_string(),
        "timelineDurationMs": state.timeline.duration_ms,
        "clipCount": state.timeline.tracks.iter().map(|t| t.clips.len()).sum::<usize>(),
    });
    fs::write(&sidecar, serde_json::to_string_pretty(&manifest)?)?;
    Ok(sidecar)
}

/// Probe whether ffmpeg is usable (for manager auto-select).
pub fn ffmpeg_available() -> bool {
    locate_ffmpeg().is_some()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn locate_ffmpeg_does_not_panic() {
        let _ = locate_ffmpeg();
    }
}
