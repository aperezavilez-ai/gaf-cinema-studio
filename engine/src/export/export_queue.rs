use std::path::PathBuf;
use std::sync::{Arc, Condvar, Mutex};
use std::thread;
use std::time::Duration;

use chrono::Utc;
use uuid::Uuid;

use crate::error::{CinemaError, Result};
use crate::event_bus::{CinemaEvent, EventBus};
use crate::native_bridge;
use crate::project_state::types::{
    ExportFormat, ExportRecord, ExportState, ExportStatus, ProjectState,
};
use crate::render_pipeline::RenderJob;

#[derive(Debug, Clone)]
pub struct ExportSettings {
    pub format: ExportFormat,
    pub width: u32,
    pub height: u32,
    pub frame_rate: f64,
}

impl Default for ExportSettings {
    fn default() -> Self {
        Self {
            format: ExportFormat::Mp4,
            width: 1920,
            height: 1080,
            frame_rate: 24.0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ExportJob {
    pub id: Uuid,
    pub project_id: Uuid,
    pub project_dir: PathBuf,
    pub settings: ExportSettings,
}

struct QueueInner {
    pending: Vec<ExportJob>,
    running: bool,
}

pub struct ExportQueue {
    inner: Arc<(Mutex<QueueInner>, Condvar)>,
    event_bus: EventBus,
}

impl ExportQueue {
    pub fn new(event_bus: EventBus) -> Self {
        Self {
            inner: Arc::new((Mutex::new(QueueInner {
                pending: Vec::new(),
                running: false,
            }), Condvar::new())),
            event_bus,
        }
    }

    pub fn enqueue(&self, job: ExportJob) -> Result<()> {
        let (lock, cvar) = &*self.inner;
        let mut inner = lock.lock().map_err(|e| {
            CinemaError::Storage(format!("export queue lock poisoned: {e}"))
        })?;

        inner.pending.push(job.clone());
        cvar.notify_one();

        self.event_bus.emit(CinemaEvent::ExportStarted {
            project_id: job.project_id,
            export_id: job.id,
            resolution: format!("{}x{}", job.settings.width, job.settings.height),
        });

        if !inner.running {
            inner.running = true;
            drop(inner);
            self.spawn_worker();
        }

        Ok(())
    }

    pub fn pending_count(&self) -> usize {
        self.inner.0.lock().map(|i| i.pending.len()).unwrap_or(0)
    }

    fn spawn_worker(&self) {
        let inner = Arc::clone(&self.inner);
        let event_bus = self.event_bus.clone();

        thread::spawn(move || {
            loop {
                let job = {
                    let (lock, cvar) = &*inner;
                    let mut state = lock.lock().unwrap();
                    while state.pending.is_empty() {
                        state.running = false;
                        cvar.notify_all();
                        return;
                    }
                    state.pending.remove(0)
                };

                let result = run_export(&job, &event_bus);

                match result {
                    Ok(output_path) => {
                        event_bus.emit(CinemaEvent::ExportCompleted {
                            project_id: job.project_id,
                            export_id: job.id,
                            output_path: output_path.display().to_string(),
                        });
                    }
                    Err(e) => {
                        event_bus.emit(CinemaEvent::ExportFailed {
                            project_id: job.project_id,
                            export_id: job.id,
                            error: e.to_string(),
                        });
                    }
                }

                thread::sleep(Duration::from_millis(10));
            }
        });
    }
}

impl Clone for ExportQueue {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
            event_bus: self.event_bus.clone(),
        }
    }
}

fn run_export(job: &ExportJob, event_bus: &EventBus) -> Result<PathBuf> {
    let state_path = job.project_dir.join("project.json");
    let contents = std::fs::read_to_string(&state_path)?;
    let state: ProjectState = serde_json::from_str(&contents)?;

    let exports_dir = job.project_dir.join("exports");
    std::fs::create_dir_all(&exports_dir)?;

    let render_job = RenderJob {
        export_id: job.id,
        project_id: job.project_id,
        project_dir: job.project_dir.clone(),
        width: job.settings.width,
        height: job.settings.height,
        frame_rate: job.settings.frame_rate,
    };

    let pipeline = native_bridge::render_pipeline();
    let result = pipeline.render(
        &render_job,
        &state,
        &|progress| {
            event_bus.emit(CinemaEvent::ExportProgress {
                project_id: job.project_id,
                export_id: job.id,
                progress,
            });
        },
    )?;

    Ok(result.output_path)
}

pub fn build_export_record(export_id: Uuid, settings: &ExportSettings, output_path: Option<String>) -> ExportRecord {
    ExportRecord {
        id: export_id,
        format: settings.format,
        resolution: format!("{}x{}", settings.width, settings.height),
        output_path,
        status: ExportStatus::Running,
        created_at: Utc::now(),
        completed_at: None,
    }
}

pub fn apply_export_started(state: &mut ExportState, record: ExportRecord) {
    state.active_export_id = Some(record.id);
    state.history.push(record);
}

pub fn apply_export_completed(state: &mut ExportState, export_id: Uuid, output_path: String) {
    if let Some(record) = state.history.iter_mut().find(|r| r.id == export_id) {
        record.status = ExportStatus::Completed;
        record.output_path = Some(output_path);
        record.completed_at = Some(Utc::now());
    }
    state.active_export_id = None;
}

pub fn apply_export_failed(state: &mut ExportState, export_id: Uuid) {
    if let Some(record) = state.history.iter_mut().find(|r| r.id == export_id) {
        record.status = ExportStatus::Failed;
        record.completed_at = Some(Utc::now());
    }
    state.active_export_id = None;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event_bus::EventSubscriber;
    use crate::project_state::types::{default_project_settings, MediaStatus, ProjectState};
    use crate::timeline_engine::{add_clip, AddClipParams};
    use chrono::Utc;
    use std::sync::Mutex as StdMutex;
    use tempfile::TempDir;

    struct Collector(Arc<StdMutex<Vec<CinemaEvent>>>);
    impl EventSubscriber for Collector {
        fn on_event(&self, e: &CinemaEvent) {
            self.0.lock().unwrap().push(e.clone());
        }
    }

    fn sample_project(dir: &TempDir) -> ProjectState {
        let mut state = ProjectState::new("Export", dir.path().to_string_lossy(), default_project_settings());
        let media_id = Uuid::new_v4();
        let clip_file = dir.path().join("media").join("clip.mp4");
        std::fs::create_dir_all(dir.path().join("media")).unwrap();
        std::fs::write(&clip_file, b"fake mp4 content").unwrap();

        state.media.push(crate::project_state::types::MediaAsset {
            id: media_id,
            original_path: clip_file.to_string_lossy().into_owned(),
            proxy_path: None,
            thumbnail_path: None,
            file_name: "clip.mp4".into(),
            mime_type: "video/mp4".into(),
            duration_ms: 3000,
            width: 1920,
            height: 1080,
            file_size_bytes: 100,
            status: MediaStatus::Ready,
            imported_at: Utc::now(),
            checksum: None,
        });

        state.timeline = add_clip(
            &state,
            AddClipParams {
                media_id,
                track_id: None,
                start_ms: None,
            },
        )
        .unwrap();

        std::fs::write(dir.path().join("project.json"), serde_json::to_string(&state).unwrap()).unwrap();
        state
    }

    #[test]
    fn export_queue_runs_in_background() {
        let tmp = TempDir::new().unwrap();
        let state = sample_project(&tmp);
        let bus = EventBus::new();
        let log = Arc::new(StdMutex::new(Vec::new()));
        bus.subscribe(Arc::new(Collector(log.clone())));

        let queue = ExportQueue::new(bus);
        let job_id = Uuid::new_v4();
        queue
            .enqueue(ExportJob {
                id: job_id,
                project_id: state.project_id,
                project_dir: tmp.path().to_path_buf(),
                settings: ExportSettings::default(),
            })
            .unwrap();

        thread::sleep(Duration::from_millis(300));

        let events = log.lock().unwrap();
        assert!(events.iter().any(|e| matches!(e, CinemaEvent::ExportCompleted { .. })));
        assert!(tmp.path().join("exports").exists());
    }
}
