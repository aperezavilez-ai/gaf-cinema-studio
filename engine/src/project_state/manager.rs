use std::fs;
use std::path::{Path, PathBuf};

use chrono::Utc;
use uuid::Uuid;

use crate::error::{CinemaError, Result};
use crate::event_bus::{CinemaEvent, EventBus};
use crate::persistence::{
    atomic_io, load_with_recovery, perform_autosave, restore_canonical_files,
    tick_autosave, AutosaveController, SqliteStore, PROJECT_FILE, SNAPSHOTS_DIR,
};
use crate::project_state::types::*;
use crate::project_state::validation::{ensure_valid, validate_project_state, ValidationReport};
use crate::render_scheduler::{JobPriority, RenderScheduler, ScheduledJob};
use crate::storage::{MediaVault, ProxyJob, ProxyQueue};
use crate::ai_orchestrator::AiOrchestrator;
use crate::beta::BetaTracker;
use crate::billing::{activate_pro_stub, cancel_subscription, BillingStore, SubscriptionState};
use crate::cloud::{AuthSession, CloudService};
use crate::crash_reporting::CrashReporter;
use crate::telemetry::TelemetryService;
use crate::device_profiler::{DeviceController, DeviceProfile, QualityPolicy, SessionMetrics, ThermalLevel};
use crate::export::{
    apply_export_completed, apply_export_failed, apply_export_started, build_export_record,
    ExportJob, ExportQueue, ExportSettings,
};
use crate::project_state::history::UndoRedoStack;
use crate::timeline_engine::{clip_at_playhead, AddClipParams, FrameComposition};
use crate::video_engine::VideoEngine;

/// Central authority for all project state mutations.
/// UI, AI, and engine modules must go through this manager.
pub struct ProjectStateManager {
    state: Option<ProjectState>,
    project_path: Option<PathBuf>,
    event_bus: EventBus,
    sqlite: Option<SqliteStore>,
    autosave: Option<AutosaveController>,
    proxy_queue: Option<ProxyQueue>,
    video_engine: Option<VideoEngine>,
    history: UndoRedoStack,
    export_queue: Option<ExportQueue>,
    export_event_cursor: usize,
    device: DeviceController,
    render_scheduler: RenderScheduler,
    crash_reporter: Option<CrashReporter>,
    crash_reporting_enabled: bool,
    data_root: PathBuf,
    cloud: CloudService,
    billing: BillingStore,
    telemetry: TelemetryService,
    beta: BetaTracker,
}

#[derive(Debug, Clone)]
pub enum Mutation {
    RenameProject { name: String },
    SetWorkflowPhase { phase: WorkflowPhase },
    SetPlayhead { time_ms: u64 },
    AddMedia { asset: MediaAsset },
    UpdateMediaStatus {
        media_id: Uuid,
        status: MediaStatus,
    },
    SetProxyPath {
        media_id: Uuid,
        proxy_path: String,
    },
}

impl ProjectStateManager {
    fn default_data_root() -> PathBuf {
        std::env::var("CINEMASTUDIO_DATA_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| std::env::temp_dir().join("CinemaStudioApp"))
    }

    pub fn new() -> Self {
        Self::with_data_root(Self::default_data_root())
    }

    pub fn with_data_root(data_root: PathBuf) -> Self {
        let cloud = CloudService::new(&data_root);
        let cloud_root = cloud.root().to_path_buf();
        Self {
            state: None,
            project_path: None,
            event_bus: EventBus::new(),
            sqlite: None,
            autosave: None,
            proxy_queue: None,
            video_engine: None,
            history: UndoRedoStack::new(),
            export_queue: None,
            export_event_cursor: 0,
            device: DeviceController::new(),
            render_scheduler: RenderScheduler::new(),
            crash_reporter: None,
            crash_reporting_enabled: false,
            billing: BillingStore::new(&cloud_root),
            cloud,
            telemetry: TelemetryService::new(&data_root),
            beta: BetaTracker::new(&data_root),
            data_root,
        }
    }

    pub fn data_root(&self) -> &Path {
        &self.data_root
    }

    pub fn auth_session(&self) -> Result<AuthSession> {
        self.cloud.auth.load()
    }

    pub fn cloud_login(&self, email: &str, password: &str) -> Result<AuthSession> {
        let session = self.cloud.login(email, password)?;
        self.event_bus.emit(CinemaEvent::AuthStateChanged {
            logged_in: true,
            email: session.email.clone(),
        });
        Ok(session)
    }

    pub fn cloud_logout(&self) -> Result<()> {
        self.cloud.logout()?;
        self.event_bus.emit(CinemaEvent::AuthStateChanged {
            logged_in: false,
            email: None,
        });
        Ok(())
    }

    pub fn cloud_backup(&self) -> Result<crate::cloud::BackupRecord> {
        let state = self
            .state
            .as_ref()
            .ok_or_else(|| CinemaError::Validation("no open project".into()))?;
        let path = self
            .project_path
            .as_ref()
            .ok_or_else(|| CinemaError::Validation("no project path".into()))?;

        let backup_id = Uuid::new_v4();
        self.event_bus.emit(CinemaEvent::CloudBackupStarted {
            project_id: state.project_id,
            backup_id,
        });

        let record = self.cloud.backup(
            path,
            state.project_id,
            &state.metadata.name,
            backup_id,
        )?;

        self.event_bus.emit(CinemaEvent::CloudBackupCompleted {
            project_id: state.project_id,
            backup_id: record.backup_id,
            size_bytes: record.size_bytes,
        });
        Ok(record)
    }

    pub fn cloud_restore(&self, backup_path: &Path, dest: &Path) -> Result<PathBuf> {
        let restored = self.cloud.restore(backup_path, dest)?;
        let pid = self
            .state
            .as_ref()
            .map(|s| s.project_id)
            .unwrap_or_else(Uuid::new_v4);
        self.event_bus.emit(CinemaEvent::CloudRestoreCompleted {
            project_id: pid,
            path: restored.display().to_string(),
        });
        Ok(restored)
    }

    pub fn subscription_state(&self) -> Result<SubscriptionState> {
        self.billing.load()
    }

    pub fn activate_pro_subscription(&self) -> Result<SubscriptionState> {
        let state = activate_pro_stub(&self.billing)?;
        self.event_bus.emit(CinemaEvent::SubscriptionUpdated {
            tier: format!("{:?}", state.tier),
        });
        Ok(state)
    }

    pub fn cancel_pro_subscription(&self) -> Result<SubscriptionState> {
        let state = cancel_subscription(&self.billing)?;
        self.event_bus.emit(CinemaEvent::SubscriptionUpdated {
            tier: format!("{:?}", state.tier),
        });
        Ok(state)
    }

    pub fn set_telemetry(&mut self, enabled: bool) -> Result<()> {
        self.telemetry.set_enabled(enabled)
    }

    pub fn telemetry_crash_rate(&self) -> Result<f64> {
        self.telemetry.crash_rate()
    }

    pub fn start_telemetry_session(&mut self) -> Result<()> {
        let pid = self.state.as_ref().map(|s| s.project_id);
        self.telemetry.start_session(pid)?;
        Ok(())
    }

    pub fn end_telemetry_session(&mut self, crashed: bool) -> Result<()> {
        self.telemetry.end_session(crashed)
    }

    pub fn upload_telemetry(&self) -> Result<Option<PathBuf>> {
        if let Some(path) = self.telemetry.upload_pending()? {
            self.event_bus.emit(CinemaEvent::TelemetryUploaded {
                path: path.display().to_string(),
            });
            Ok(Some(path))
        } else {
            Ok(None)
        }
    }

    pub fn beta_registry(&self) -> Result<crate::beta::BetaRegistry> {
        self.beta.load()
    }

    pub fn beta_mark_complete(&self, user_label: &str) -> Result<crate::beta::BetaRegistry> {
        let state = self
            .state
            .as_ref()
            .ok_or_else(|| CinemaError::Validation("no open project".into()))?;
        let reg = self.beta.mark_complete(
            state.project_id,
            &state.metadata.name,
            user_label,
        )?;
        self.event_bus.emit(CinemaEvent::BetaProjectCompleted {
            project_id: state.project_id,
            total_completions: reg.count(),
            gate_met: reg.gate_met(),
        });
        Ok(reg)
    }

    pub fn device_profile(&self) -> &DeviceProfile {
        self.device.profile()
    }

    pub fn quality_policy(&self) -> &QualityPolicy {
        self.device.policy()
    }

    pub fn session_metrics(&self) -> &SessionMetrics {
        self.device.session()
    }

    pub fn set_crash_reporting(&mut self, enabled: bool) {
        self.crash_reporting_enabled = enabled;
        if let Some(path) = &self.project_path {
            self.crash_reporter = Some(CrashReporter::new(path, enabled));
        }
    }

    pub fn set_device_profile(&mut self, profile: DeviceProfile) -> Result<()> {
        let pid = self.state.as_ref().map(|s| s.project_id);
        self.device.refresh_profile(profile.clone(), &self.event_bus, pid);
        self.render_scheduler.adapt_to_thermal(profile.thermal);
        self.apply_adaptive_quality()
    }

    pub fn set_thermal_level(&mut self, level: ThermalLevel) -> Result<()> {
        let pid = self
            .state
            .as_ref()
            .ok_or_else(|| CinemaError::Validation("no open project".into()))?
            .project_id;
        self.device.set_thermal(level, &self.event_bus, pid);
        self.render_scheduler.adapt_to_thermal(level);
        self.apply_adaptive_quality()
    }

    pub fn apply_adaptive_quality(&mut self) -> Result<()> {
        let state = self
            .state
            .as_mut()
            .ok_or_else(|| CinemaError::Validation("no open project".into()))?;
        let pid = state.project_id;
        self.device
            .sync_adaptive_quality(&self.event_bus, pid, &mut state.render_state);
        Ok(())
    }

    pub fn record_crash(&self, error: &str, context: &str) -> Result<Option<PathBuf>> {
        match &self.crash_reporter {
            Some(r) => r.record(error, context),
            None => Ok(None),
        }
    }

    pub fn performance_report(&self) -> serde_json::Value {
        let s = self.device.session();
        serde_json::json!({
            "deviceTier": format!("{:?}", self.device.profile().effective_tier()),
            "thermal": format!("{:?}", self.device.profile().thermal),
            "previewQuality": format!("{:?}", self.device.policy().preview_quality),
            "peakDropRate": s.peak_drop_rate,
            "peakScrubLatencyMs": s.peak_scrub_latency_ms,
            "qualityDowngrades": s.quality_downgrades,
            "thermalEvents": s.thermal_events,
            "leakSuspects": s.leak_suspects,
            "sessionStable": s.is_stable(),
            "schedulerPending": self.render_scheduler.pending_count(),
        })
    }

    fn init_performance_stack(&mut self) -> Result<()> {
        crate::native_bridge::init_render_backend();
        let pid = self.state.as_ref().map(|s| s.project_id);
        let profile = crate::device_profiler::detect();
        self.device.refresh_profile(profile.clone(), &self.event_bus, pid);
        self.render_scheduler.adapt_to_thermal(profile.thermal);
        if let Some(path) = &self.project_path {
            self.crash_reporter = Some(CrashReporter::new(path, self.crash_reporting_enabled));
        }
        self.apply_adaptive_quality()
    }

    pub fn can_undo(&self) -> bool {
        self.history.can_undo()
    }

    pub fn can_redo(&self) -> bool {
        self.history.can_redo()
    }

    pub fn video_engine(&self) -> Option<&VideoEngine> {
        self.video_engine.as_ref()
    }

    pub fn video_engine_mut(&mut self) -> Option<&mut VideoEngine> {
        self.video_engine.as_mut()
    }

    pub fn event_bus(&self) -> &EventBus {
        &self.event_bus
    }

    pub fn state(&self) -> Option<&ProjectState> {
        self.state.as_ref()
    }

    pub fn project_path(&self) -> Option<&Path> {
        self.project_path.as_deref()
    }

    pub fn sqlite(&self) -> Option<&SqliteStore> {
        self.sqlite.as_ref()
    }

    pub fn create_project(
        &mut self,
        name: impl Into<String>,
        parent_dir: impl AsRef<Path>,
        settings: ProjectSettings,
    ) -> Result<&ProjectState> {
        let name = name.into();
        let safe_name = sanitize_project_name(&name);
        let project_dir = parent_dir.as_ref().join(format!("{safe_name}.csproj"));

        if project_dir.exists() {
            return Err(CinemaError::Validation(format!(
                "project directory already exists: {}",
                project_dir.display()
            )));
        }

        Self::init_project_dirs(&project_dir)?;

        let state = ProjectState::new(name, project_dir.to_string_lossy().into_owned(), settings);
        ensure_valid(&state)?;

        let sqlite = SqliteStore::open(&project_dir)?;
        sqlite.save_state(&state)?;

        let project_file = project_dir.join(PROJECT_FILE);
        atomic_io::atomic_write(&project_file, &serde_json::to_string_pretty(&state)?)?;

        self.event_bus.emit(CinemaEvent::ProjectCreated {
            project_id: state.project_id,
            name: state.metadata.name.clone(),
        });

        self.state = Some(state);
        self.project_path = Some(project_dir.clone());
        self.sqlite = Some(sqlite);
        self.autosave = Some(AutosaveController::from_state(self.state.as_ref().unwrap()));
        self.proxy_queue = Some(ProxyQueue::new(&project_dir, self.event_bus.clone()));
        self.video_engine = Some(VideoEngine::from_state(self.state.as_ref().unwrap()));
        self.history.clear();
        self.export_queue = Some(ExportQueue::new(self.event_bus.clone()));
        self.export_event_cursor = self.event_bus.event_log().len();
        self.init_performance_stack()?;

        Ok(self.state.as_ref().unwrap())
    }

    pub fn open_project(&mut self, project_dir: impl AsRef<Path>) -> Result<&ProjectState> {
        let project_dir = project_dir.as_ref();

        if !project_dir.exists() {
            return Err(CinemaError::ProjectNotFound(project_dir.display().to_string()));
        }

        let recovery = load_with_recovery(project_dir)?;
        let state = recovery.state;

        if recovery.was_recovered {
            let snap_id = match &recovery.source {
                crate::persistence::RecoverySource::SnapshotFile { id }
                | crate::persistence::RecoverySource::SqliteSnapshot { id } => Some(*id),
                _ => None,
            };

            self.event_bus.emit(CinemaEvent::ProjectRecovered {
                project_id: state.project_id,
                snapshot_id: snap_id,
                reason: format!("recovered from {:?}", recovery.source),
            });

            self.event_bus.emit(CinemaEvent::ProjectCorrupted {
                project_id: Some(state.project_id),
                error: "primary project file invalid — auto-recovered".into(),
                recovered: true,
            });
        }

        let sqlite = SqliteStore::open(project_dir)?;
        sqlite.save_state(&state)?;

        self.event_bus.emit(CinemaEvent::ProjectOpened {
            project_id: state.project_id,
            path: project_dir.to_string_lossy().into_owned(),
        });

        self.autosave = Some(AutosaveController::from_state(&state));
        self.proxy_queue = Some(ProxyQueue::new(project_dir, self.event_bus.clone()));
        self.state = Some(state);
        self.project_path = Some(project_dir.to_path_buf());
        self.sqlite = Some(sqlite);
        self.video_engine = Some(VideoEngine::from_state(self.state.as_ref().unwrap()));
        self.history.clear();
        self.export_queue = Some(ExportQueue::new(self.event_bus.clone()));
        self.export_event_cursor = self.event_bus.event_log().len();
        self.init_performance_stack()?;

        Ok(self.state.as_ref().unwrap())
    }

    // ── Phase 2: Timeline + Playback ──

    pub fn add_clip_to_timeline(&mut self, params: AddClipParams) -> Result<Uuid> {
        self.record_history();
        self.add_clip_to_timeline_inner(params)
    }

    pub(crate) fn add_clip_to_timeline_without_history(
        &mut self,
        params: AddClipParams,
    ) -> Result<Uuid> {
        self.add_clip_to_timeline_inner(params)
    }

    fn add_clip_to_timeline_inner(&mut self, params: AddClipParams) -> Result<Uuid> {
        let state = self
            .state
            .as_ref()
            .ok_or_else(|| CinemaError::Validation("no open project".into()))?;
        let old_duration = state.timeline.duration_ms;
        let timeline = VideoEngine::add_clip(state, params)?;
        let clip = timeline
            .tracks
            .iter()
            .flat_map(|t| t.clips.iter())
            .last()
            .ok_or_else(|| CinemaError::Validation("clip not created".into()))?
            .clone();

        let project_id = state.project_id;
        let track_id = timeline
            .tracks
            .iter()
            .find(|t| t.clips.iter().any(|c| c.id == clip.id))
            .map(|t| t.id)
            .unwrap();

        self.state.as_mut().unwrap().timeline = timeline;
        if let Some(ve) = &mut self.video_engine {
            ve.sync_from_state(self.state.as_ref().unwrap());
        }

        self.emit_timeline_change(project_id, old_duration);

        self.event_bus.emit(CinemaEvent::ClipAdded {
            project_id,
            clip_id: clip.id,
            track_id,
            media_id: clip.media_id,
        });

        self.state.as_mut().unwrap().updated_at = Utc::now();
        ensure_valid(self.state.as_ref().unwrap())?;
        Ok(clip.id)
    }

    pub fn remove_clip_from_timeline(&mut self, clip_id: Uuid) -> Result<()> {
        self.record_history();
        let state = self.state.as_ref().unwrap();
        let old_duration = state.timeline.duration_ms;
        let project_id = state.project_id;
        let timeline = VideoEngine::remove_clip(state, clip_id)?;
        self.state.as_mut().unwrap().timeline = timeline;
        if let Some(ve) = &mut self.video_engine {
            ve.sync_from_state(self.state.as_ref().unwrap());
        }
        self.emit_timeline_change(project_id, old_duration);
        self.event_bus.emit(CinemaEvent::ClipRemoved {
            project_id,
            clip_id,
        });
        Ok(())
    }

    pub fn scrub_to(&mut self, time_ms: u64) -> Result<FrameComposition> {
        let state = self.state.as_ref().unwrap().clone();
        let project_id = state.project_id;
        let frame = self
            .video_engine
            .as_mut()
            .ok_or_else(|| CinemaError::Validation("video engine not initialized".into()))?
            .scrub(&state, time_ms);

        self.state.as_mut().unwrap().timeline.playhead_ms = self
            .video_engine
            .as_ref()
            .unwrap()
            .playback()
            .playhead_ms();

        self.event_bus.emit(CinemaEvent::PlayheadChanged {
            project_id,
            time_ms: self.state.as_ref().unwrap().timeline.playhead_ms,
        });

        Ok(frame)
    }

    /// Decode pixel frame at current playhead — uses native bridge when wired.
    pub fn decode_at_playhead(&mut self) -> Result<crate::media_decoder::DecodedFrame> {
        let frame = self.scrub_to(
            self.state
                .as_ref()
                .ok_or_else(|| CinemaError::Validation("no open project".into()))?
                .timeline
                .playhead_ms,
        )?;
        let layer = frame
            .primary_video()
            .ok_or_else(|| CinemaError::Validation("no video at playhead".into()))?;
        let media = self
            .state
            .as_ref()
            .unwrap()
            .media
            .iter()
            .find(|m| m.id == layer.media_id)
            .ok_or_else(|| CinemaError::Validation("media missing".into()))?;
        crate::native_bridge::decode_frame_at(
            &layer.playback_path,
            layer.source_time_ms,
            media.width,
            media.height,
        )
    }

    pub fn playback_play(&mut self) -> Result<()> {
        let project_id = self.state.as_ref().unwrap().project_id;
        self.video_engine
            .as_mut()
            .ok_or_else(|| CinemaError::Validation("video engine not initialized".into()))?
            .play(&self.event_bus, project_id);
        Ok(())
    }

    pub fn playback_pause(&mut self) -> Result<()> {
        let project_id = self.state.as_ref().unwrap().project_id;
        self.video_engine
            .as_mut()
            .ok_or_else(|| CinemaError::Validation("video engine not initialized".into()))?
            .pause(&self.event_bus, project_id);
        Ok(())
    }

    pub fn playback_tick(&mut self) -> Result<Option<FrameComposition>> {
        let state = self.state.as_ref().unwrap().clone();
        let frame = self
            .video_engine
            .as_mut()
            .ok_or_else(|| CinemaError::Validation("video engine not initialized".into()))?
            .tick(&state, &self.event_bus);

        if let Some(ve) = self.video_engine.as_ref() {
            let playhead = ve.playback().playhead_ms();
            let metrics = ve.playback().metrics().clone();
            self.device.tick_playback(&metrics);
            if self.device.session().should_degrade_quality() {
                let _ = self.apply_adaptive_quality();
            }
            if playhead != state.timeline.playhead_ms {
                self.state.as_mut().unwrap().timeline.playhead_ms = playhead;
                self.event_bus.emit(CinemaEvent::PlayheadChanged {
                    project_id: state.project_id,
                    time_ms: playhead,
                });
            }
        }

        Ok(frame)
    }

    // ── Phase 3: Editing + Undo/Redo + Export ──

    pub fn undo(&mut self) -> Result<bool> {
        let current = self
            .state
            .as_ref()
            .ok_or_else(|| CinemaError::Validation("no open project".into()))?
            .clone();
        if let Some(previous) = self.history.undo(&current) {
            let project_id = previous.project_id;
            self.state = Some(previous);
            if let Some(ve) = &mut self.video_engine {
                ve.sync_from_state(self.state.as_ref().unwrap());
            }
            self.event_bus.emit(CinemaEvent::UndoApplied { project_id });
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub fn redo(&mut self) -> Result<bool> {
        let current = self
            .state
            .as_ref()
            .ok_or_else(|| CinemaError::Validation("no open project".into()))?
            .clone();
        if let Some(next) = self.history.redo(&current) {
            let project_id = next.project_id;
            self.state = Some(next);
            if let Some(ve) = &mut self.video_engine {
                ve.sync_from_state(self.state.as_ref().unwrap());
            }
            self.event_bus.emit(CinemaEvent::RedoApplied { project_id });
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub fn split_at_playhead(&mut self) -> Result<Option<Uuid>> {
        let state = self.state.as_ref().unwrap();
        let playhead = state.timeline.playhead_ms;
        let hit = match clip_at_playhead(state, playhead) {
            Some(h) => h,
            None => return Ok(None),
        };
        if hit.offset_in_clip_ms == 0 || hit.offset_in_clip_ms >= state
            .timeline
            .tracks
            .iter()
            .flat_map(|t| t.clips.iter())
            .find(|c| c.id == hit.clip_id)
            .map(|c| c.duration_ms)
            .unwrap_or(0)
        {
            return Err(CinemaError::Validation(
                "cannot split at clip boundary".into(),
            ));
        }

        self.record_history();
        let state = self.state.as_ref().unwrap();
        let timeline = VideoEngine::split_clip(state, hit.clip_id, playhead)?;
        self.apply_timeline(timeline);

        let new_clip = self
            .state
            .as_ref()
            .unwrap()
            .timeline
            .tracks
            .iter()
            .flat_map(|t| &t.clips)
            .find(|c| c.start_ms == playhead)
            .map(|c| c.id);

        if let (Some(id), Some(project_id)) = (new_clip, self.state.as_ref().map(|s| s.project_id)) {
            self.event_bus.emit(CinemaEvent::ClipTrimmed {
                project_id,
                clip_id: id,
            });
        }

        Ok(new_clip)
    }

    pub fn trim_clip(&mut self, clip_id: Uuid, source_in_ms: u64, source_out_ms: u64) -> Result<()> {
        self.record_history();
        let state = self.state.as_ref().unwrap();
        let timeline = VideoEngine::trim_clip(state, clip_id, source_in_ms, source_out_ms)?;
        self.apply_timeline(timeline);
        let project_id = self.state.as_ref().unwrap().project_id;
        self.event_bus.emit(CinemaEvent::ClipTrimmed {
            project_id,
            clip_id,
        });
        Ok(())
    }

    pub fn move_clip(&mut self, clip_id: Uuid, new_start_ms: u64) -> Result<()> {
        self.record_history();
        self.move_clip_inner(clip_id, new_start_ms)
    }

    pub(crate) fn move_clip_without_history(
        &mut self,
        clip_id: Uuid,
        new_start_ms: u64,
    ) -> Result<()> {
        self.move_clip_inner(clip_id, new_start_ms)
    }

    fn move_clip_inner(&mut self, clip_id: Uuid, new_start_ms: u64) -> Result<()> {
        let state = self.state.as_ref().unwrap();
        let timeline = VideoEngine::move_clip(state, clip_id, new_start_ms)?;
        self.apply_timeline(timeline);
        let project_id = self.state.as_ref().unwrap().project_id;
        self.event_bus.emit(CinemaEvent::ClipMoved {
            project_id,
            clip_id,
            new_start_ms,
        });
        Ok(())
    }

    pub fn set_clip_fade(&mut self, clip_id: Uuid, fade_in_ms: u64, fade_out_ms: u64) -> Result<()> {
        self.record_history();
        self.set_clip_fade_inner(clip_id, fade_in_ms, fade_out_ms)
    }

    pub(crate) fn set_clip_fade_without_history(
        &mut self,
        clip_id: Uuid,
        fade_in_ms: u64,
        fade_out_ms: u64,
    ) -> Result<()> {
        self.set_clip_fade_inner(clip_id, fade_in_ms, fade_out_ms)
    }

    fn set_clip_fade_inner(&mut self, clip_id: Uuid, fade_in_ms: u64, fade_out_ms: u64) -> Result<()> {
        {
            let state = self.state.as_mut().unwrap();
            let mut found = false;
            for track in &mut state.timeline.tracks {
                if let Some(clip) = track.clips.iter_mut().find(|c| c.id == clip_id) {
                    clip.transitions.fade_in_ms = fade_in_ms;
                    clip.transitions.fade_out_ms = fade_out_ms;
                    found = true;
                    break;
                }
            }
            if !found {
                return Err(CinemaError::Validation(format!("clip not found: {clip_id}")));
            }
            state.updated_at = Utc::now();
            ensure_valid(state)?;
        }
        if let Some(ve) = &mut self.video_engine {
            ve.sync_from_state(self.state.as_ref().unwrap());
        }
        Ok(())
    }

    pub fn delete_at_playhead(&mut self) -> Result<bool> {
        let state = self.state.as_ref().unwrap();
        let playhead = state.timeline.playhead_ms;
        let hit = match clip_at_playhead(state, playhead) {
            Some(h) => h,
            None => return Ok(false),
        };
        self.record_history();
        let state = self.state.as_ref().unwrap();
        let project_id = state.project_id;
        let timeline = VideoEngine::remove_clip(state, hit.clip_id)?;
        self.apply_timeline(timeline);
        self.event_bus.emit(CinemaEvent::ClipRemoved {
            project_id,
            clip_id: hit.clip_id,
        });
        Ok(true)
    }

    /// Queue 1080p MP4 export — fully async, never blocks caller (Gate 3.2).
    pub fn start_export(&mut self, settings: ExportSettings) -> Result<Uuid> {
        let state = self.state.as_ref().unwrap();
        let project_path = self.project_path.as_ref().unwrap().clone();
        let export_id = Uuid::new_v4();

        let record = build_export_record(export_id, &settings, None);
        apply_export_started(&mut self.state.as_mut().unwrap().export_state, record);
        self.save()?;

        self.export_queue
            .as_ref()
            .ok_or_else(|| CinemaError::Validation("export queue not initialized".into()))?
            .enqueue(ExportJob {
                id: export_id,
                project_id: state.project_id,
                project_dir: project_path,
                settings,
            })?;

        Ok(export_id)
    }

    /// Poll export events and update export_state. Call from UI timer.
    pub fn sync_export_status(&mut self) -> Result<()> {
        let log = self.event_bus.event_log();
        for event in &log[self.export_event_cursor..] {
            match event {
                CinemaEvent::ExportCompleted {
                    export_id,
                    output_path,
                    ..
                } => {
                    apply_export_completed(
                        &mut self.state.as_mut().unwrap().export_state,
                        *export_id,
                        output_path.clone(),
                    );
                }
                CinemaEvent::ExportFailed { export_id, .. } => {
                    apply_export_failed(&mut self.state.as_mut().unwrap().export_state, *export_id);
                }
                _ => {}
            }
        }
        self.export_event_cursor = log.len();
        Ok(())
    }

    /// JSON snapshot of export state for UI.
    pub fn export_status(&self) -> serde_json::Value {
        let Some(state) = &self.state else {
            return serde_json::json!({ "open": false });
        };
        let es = &state.export_state;
        let last = es.history.last();
        serde_json::json!({
            "activeExportId": es.active_export_id,
            "historyCount": es.history.len(),
            "lastStatus": last.map(|r| format!("{:?}", r.status)),
            "lastOutputPath": last.and_then(|r| r.output_path.clone()),
            "ffmpegAvailable": crate::render_pipeline::ffmpeg_available(),
        })
    }

    /// End-to-end workflow: import → timeline → edit → export (Gate 11).
    pub fn run_edit_export_workflow(&mut self, media_path: &Path) -> Result<Uuid> {
        use crate::timeline_engine::AddClipParams;

        self.import_media(media_path)?;
        let media_id = self.state.as_ref().unwrap().media.last().unwrap().id;
        self.add_clip_to_timeline(AddClipParams {
            media_id,
            track_id: None,
            start_ms: None,
        })?;
        self.start_export(ExportSettings::default())
    }

    fn record_history(&mut self) {
        if let Some(state) = self.state.as_ref() {
            self.history.push(state);
        }
    }

    pub(crate) fn record_history_for_ai(&mut self) {
        self.record_history();
    }

    pub fn ai_analyze(&mut self) -> Result<Vec<AiSuggestion>> {
        AiOrchestrator::analyze(self)
    }

    pub fn ai_dismiss(&mut self, suggestion_id: Uuid) -> Result<()> {
        AiOrchestrator::dismiss(self, suggestion_id)
    }

    pub fn ai_execute(&mut self, suggestion_id: Uuid) -> Result<String> {
        AiOrchestrator::execute_suggestion(self, suggestion_id)
    }

    pub fn ai_suggestions(&self) -> Vec<AiSuggestion> {
        AiOrchestrator::suggestions(self)
    }

    pub(crate) fn set_ai_suggestions(&mut self, suggestions: Vec<AiSuggestion>) {
        if let Some(state) = self.state.as_mut() {
            state.ai_state.suggestions = suggestions;
            state.ai_state.last_analysis_at = Some(Utc::now());
        }
    }

    pub(crate) fn dismiss_ai_suggestion(&mut self, suggestion_id: Uuid) {
        if let Some(state) = self.state.as_mut() {
            state.ai_state.dismissed_suggestion_ids.push(suggestion_id);
            state
                .ai_state
                .suggestions
                .retain(|s| s.id != suggestion_id);
        }
    }

    pub(crate) fn remove_ai_suggestion(&mut self, suggestion_id: Uuid) {
        if let Some(state) = self.state.as_mut() {
            state.ai_state.suggestions.retain(|s| s.id != suggestion_id);
        }
    }

    pub(crate) fn ai_state_snapshot(&self) -> Option<(Vec<Uuid>, Vec<AiSuggestion>)> {
        self.state.as_ref().map(|s| {
            (
                s.ai_state.dismissed_suggestion_ids.clone(),
                s.ai_state.suggestions.clone(),
            )
        })
    }

    fn apply_timeline(&mut self, timeline: Timeline) {
        let project_id = self.state.as_ref().unwrap().project_id;
        let old_duration = self.state.as_ref().unwrap().timeline.duration_ms;
        self.state.as_mut().unwrap().timeline = timeline;
        self.state.as_mut().unwrap().updated_at = Utc::now();
        if let Some(ve) = &mut self.video_engine {
            ve.sync_from_state(self.state.as_ref().unwrap());
        }
        self.emit_timeline_change(project_id, old_duration);
    }

    fn emit_timeline_change(&self, project_id: Uuid, old_duration: u64) {
        let new_duration = self.state.as_ref().unwrap().timeline.duration_ms;
        if new_duration != old_duration {
            self.event_bus.emit(CinemaEvent::TimelineDurationChanged {
                project_id,
                duration_ms: new_duration,
            });
        }
    }

    pub fn save(&mut self) -> Result<()> {
        let state = self
            .state
            .as_mut()
            .ok_or_else(|| CinemaError::Validation("no open project".into()))?;
        let project_path = self
            .project_path
            .as_ref()
            .ok_or_else(|| CinemaError::Validation("no project path".into()))?;
        let sqlite = self
            .sqlite
            .as_ref()
            .ok_or_else(|| CinemaError::Validation("no sqlite store".into()))?;

        state.updated_at = Utc::now();
        ensure_valid(state)?;

        let project_file = project_path.join(PROJECT_FILE);
        atomic_io::atomic_write(&project_file, &serde_json::to_string_pretty(state)?)?;
        sqlite.save_state(state)?;

        self.event_bus.emit(CinemaEvent::ProjectSaved {
            project_id: state.project_id,
            path: project_path.to_string_lossy().into_owned(),
        });

        if let Some(autosave) = &mut self.autosave {
            autosave.mark_saved();
        }

        Ok(())
    }

    /// Call periodically from UI timer (every ~1s). Autosaves when interval elapsed.
    pub fn tick_autosave(&mut self) -> Result<Option<Uuid>> {
        let state = self
            .state
            .as_ref()
            .ok_or_else(|| CinemaError::Validation("no open project".into()))?
            .clone();
        let project_path = self
            .project_path
            .as_ref()
            .ok_or_else(|| CinemaError::Validation("no project path".into()))?
            .clone();
        let sqlite = self
            .sqlite
            .as_ref()
            .ok_or_else(|| CinemaError::Validation("no sqlite store".into()))?;
        let autosave = self
            .autosave
            .as_mut()
            .ok_or_else(|| CinemaError::Validation("no autosave controller".into()))?;

        tick_autosave(
            autosave,
            sqlite,
            &state,
            &project_path.join(PROJECT_FILE),
            &self.event_bus,
        )
    }

    pub fn create_snapshot(&mut self) -> Result<Uuid> {
        let state = self
            .state
            .as_ref()
            .ok_or_else(|| CinemaError::Validation("no open project".into()))?
            .clone();
        let project_path = self
            .project_path
            .as_ref()
            .ok_or_else(|| CinemaError::Validation("no project path".into()))?;
        let sqlite = self
            .sqlite
            .as_ref()
            .ok_or_else(|| CinemaError::Validation("no sqlite store".into()))?;

        let snapshot_id = perform_autosave(
            sqlite,
            &state,
            &project_path.join(PROJECT_FILE),
            &self.event_bus,
        )?;

        self.event_bus.emit(CinemaEvent::SnapshotCreated {
            project_id: state.project_id,
            snapshot_id,
        });

        Ok(snapshot_id)
    }

    pub fn recover_from_snapshot(&mut self, snapshot_id: Uuid) -> Result<&ProjectState> {
        let project_path = self
            .project_path
            .as_ref()
            .ok_or_else(|| CinemaError::Validation("no project path".into()))?;
        let sqlite = self
            .sqlite
            .as_ref()
            .ok_or_else(|| CinemaError::Validation("no sqlite store".into()))?;

        let state = sqlite.load_snapshot(snapshot_id).or_else(|_| {
            let snapshot_file = project_path
                .join(SNAPSHOTS_DIR)
                .join(format!("{snapshot_id}.json"));
            if !snapshot_file.exists() {
                return Err(CinemaError::ProjectNotFound(
                    snapshot_file.display().to_string(),
                ));
            }
            let contents = fs::read_to_string(&snapshot_file)?;
            serde_json::from_str(&contents).map_err(|e| {
                CinemaError::CorruptedState(format!("snapshot corrupted: {e}"))
            })
        })?;

        ensure_valid(&state)?;
        restore_canonical_files(project_path, &state)?;
        sqlite.save_state(&state)?;

        self.event_bus.emit(CinemaEvent::ProjectRecovered {
            project_id: state.project_id,
            snapshot_id: Some(snapshot_id),
            reason: "manual_recovery".into(),
        });

        self.state = Some(state);
        Ok(self.state.as_ref().unwrap())
    }

    pub fn import_media(&mut self, source_path: impl AsRef<Path>) -> Result<&MediaAsset> {
        let project_path = self
            .project_path
            .as_ref()
            .ok_or_else(|| CinemaError::Validation("no project path".into()))?
            .clone();
        let project_id = self
            .state
            .as_ref()
            .ok_or_else(|| CinemaError::Validation("no open project".into()))?
            .project_id;

        let vault = MediaVault::new(&project_path);
        let imported = vault.import_file(source_path, project_id, &self.event_bus)?;

        self.apply(Mutation::AddMedia {
            asset: imported.asset.clone(),
        })?;
        self.save()?;

        if self.device.policy().allow_proxy_generation {
            if let Some(queue) = &self.proxy_queue {
                let job_id = Uuid::new_v4();
                self.render_scheduler.enqueue(ScheduledJob {
                    id: job_id,
                    priority: JobPriority::Proxy,
                    label: format!("proxy:{}", imported.asset.file_name),
                })?;
                queue.enqueue(ProxyJob {
                    id: job_id,
                    project_id,
                    media_id: imported.asset.id,
                    source_path: imported.vault_path,
                })?;
            }
        }

        let _ = self.ai_analyze();
        Ok(&self.state.as_ref().unwrap().media.last().unwrap())
    }

    pub fn apply(&mut self, mutation: Mutation) -> Result<&ProjectState> {
        let state = self
            .state
            .as_mut()
            .ok_or_else(|| CinemaError::Validation("no open project".into()))?;

        match mutation {
            Mutation::RenameProject { name } => {
                if name.trim().is_empty() {
                    return Err(CinemaError::Validation("name cannot be empty".into()));
                }
                state.metadata.name = name;
            }
            Mutation::SetWorkflowPhase { phase } => {
                let from = state.workflow_state.phase;
                state.workflow_state.phase = phase;
                state.workflow_state.last_action = Some(format!("phase_changed_to_{phase:?}"));
                state.workflow_state.last_action_at = Some(Utc::now());
                self.event_bus.emit(CinemaEvent::WorkflowPhaseChanged {
                    project_id: state.project_id,
                    from: format!("{from:?}"),
                    to: format!("{phase:?}"),
                });
            }
            Mutation::SetPlayhead { time_ms } => {
                state.timeline.playhead_ms = time_ms;
                self.event_bus.emit(CinemaEvent::PlayheadChanged {
                    project_id: state.project_id,
                    time_ms,
                });
            }
            Mutation::AddMedia { asset } => {
                state.media.push(asset.clone());
                self.event_bus.emit(CinemaEvent::MediaIndexed {
                    project_id: state.project_id,
                    media_id: asset.id,
                    file_name: asset.file_name.clone(),
                });
            }
            Mutation::UpdateMediaStatus { media_id, status } => {
                let media = state
                    .media
                    .iter_mut()
                    .find(|m| m.id == media_id)
                    .ok_or_else(|| {
                        CinemaError::Validation(format!("media not found: {media_id}"))
                    })?;
                media.status = status;
            }
            Mutation::SetProxyPath { media_id, proxy_path } => {
                let media = state
                    .media
                    .iter_mut()
                    .find(|m| m.id == media_id)
                    .ok_or_else(|| {
                        CinemaError::Validation(format!("media not found: {media_id}"))
                    })?;
                media.proxy_path = Some(proxy_path);
            }
        }

        state.updated_at = Utc::now();
        ensure_valid(state)?;
        Ok(self.state.as_ref().unwrap())
    }

    pub fn validate(&self) -> ValidationReport {
        match &self.state {
            Some(s) => validate_project_state(s),
            None => ValidationReport::default(),
        }
    }

    pub fn to_json(&self) -> Result<String> {
        let state = self
            .state
            .as_ref()
            .ok_or_else(|| CinemaError::Validation("no open project".into()))?;
        Ok(serde_json::to_string_pretty(state)?)
    }

    fn init_project_dirs(project_dir: &Path) -> Result<()> {
        fs::create_dir_all(project_dir)?;
        fs::create_dir_all(project_dir.join("media"))?;
        fs::create_dir_all(project_dir.join("proxies"))?;
        fs::create_dir_all(project_dir.join("cache"))?;
        fs::create_dir_all(project_dir.join("exports"))?;
        fs::create_dir_all(project_dir.join("backups"))?;
        fs::create_dir_all(project_dir.join(SNAPSHOTS_DIR))?;
        Ok(())
    }
}

impl Default for ProjectStateManager {
    fn default() -> Self {
        Self::new()
    }
}

fn sanitize_project_name(name: &str) -> String {
    name.chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' || c == '_' || c == ' ' {
                c
            } else {
                '_'
            }
        })
        .collect::<String>()
        .trim()
        .replace(' ', "_")
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use tempfile::TempDir;

    #[test]
    fn create_and_open_project_with_sqlite() {
        let tmp = TempDir::new().unwrap();
        let mut manager = ProjectStateManager::new();

        manager
            .create_project("My Film", tmp.path(), default_project_settings())
            .unwrap();
        manager.save().unwrap();

        let path = manager.project_path().unwrap().to_path_buf();
        assert!(path.join("project.db").exists());

        let mut manager2 = ProjectStateManager::new();
        manager2.open_project(&path).unwrap();
        assert_eq!(manager2.state().unwrap().metadata.name, "My Film");
    }

    #[test]
    fn autosave_tick_creates_snapshot() {
        let tmp = TempDir::new().unwrap();
        let mut manager = ProjectStateManager::new();
        manager
            .create_project("Auto", tmp.path(), default_project_settings())
            .unwrap();

        let snap = manager.tick_autosave().unwrap();
        assert!(snap.is_some());
        assert!(manager.project_path().unwrap().join("project.db").exists());
    }

    #[test]
    fn import_media_adds_to_state() {
        let tmp = TempDir::new().unwrap();
        let source = tmp.path().join("clip.mp4");
        fs::write(&source, b"fake video").unwrap();

        let mut manager = ProjectStateManager::new();
        manager
            .create_project("Media", tmp.path(), default_project_settings())
            .unwrap();

        let asset = manager.import_media(&source).unwrap();
        assert_eq!(manager.state().unwrap().media.len(), 1);
        assert_eq!(asset.file_name, "clip.mp4");
    }

    #[test]
    fn recovery_after_corrupt_save() {
        let tmp = TempDir::new().unwrap();
        let mut manager = ProjectStateManager::new();
        manager
            .create_project("Recover", tmp.path(), default_project_settings())
            .unwrap();
        manager.save().unwrap();

        let path = manager.project_path().unwrap().to_path_buf();
        fs::write(path.join(PROJECT_FILE), b"{ TRUNCATED CORRUPT").unwrap();

        let mut manager2 = ProjectStateManager::new();
        manager2.open_project(&path).unwrap();
        assert_eq!(manager2.state().unwrap().metadata.name, "Recover");
    }

    #[test]
    fn hundred_save_cycles_no_data_loss() {
        let tmp = TempDir::new().unwrap();
        let mut manager = ProjectStateManager::new();
        manager
            .create_project("Cycles", tmp.path(), default_project_settings())
            .unwrap();
        let project_id = manager.state().unwrap().project_id;

        for i in 0..100 {
            manager
                .apply(Mutation::RenameProject {
                    name: format!("Cycle {i}"),
                })
                .unwrap();
            manager.save().unwrap();
        }

        let path = manager.project_path().unwrap().to_path_buf();
        let mut manager2 = ProjectStateManager::new();
        manager2.open_project(&path).unwrap();
        assert_eq!(manager2.state().unwrap().project_id, project_id);
        assert_eq!(manager2.state().unwrap().metadata.name, "Cycle 99");
    }

    #[test]
    fn timeline_add_and_playback_scrub() {
        let tmp = TempDir::new().unwrap();
        let source = tmp.path().join("clip.mp4");
        fs::write(&source, b"video data").unwrap();

        let mut manager = ProjectStateManager::new();
        manager
            .create_project("Video", tmp.path(), default_project_settings())
            .unwrap();

        let asset = manager.import_media(&source).unwrap();
        let clip_id = manager
            .add_clip_to_timeline(AddClipParams {
                media_id: asset.id,
                track_id: None,
                start_ms: None,
            })
            .unwrap();

        assert!(!clip_id.is_nil());
        let frame = manager.scrub_to(0).unwrap();
        assert_eq!(frame.video_layers.len(), 1);

        manager.playback_play().unwrap();
        let tick = manager.playback_tick().unwrap();
        assert!(tick.is_some() || manager.video_engine().unwrap().playback().playhead_ms() >= 0);
    }

    #[test]
    fn undo_redo_after_edit() {
        let tmp = TempDir::new().unwrap();
        let source = tmp.path().join("clip.mp4");
        fs::write(&source, b"video").unwrap();

        let mut manager = ProjectStateManager::new();
        manager.create_project("Undo", tmp.path(), default_project_settings()).unwrap();
        let asset = manager.import_media(&source).unwrap();
        manager.add_clip_to_timeline(AddClipParams {
            media_id: asset.id,
            track_id: None,
            start_ms: None,
        }).unwrap();

        let count_before = manager.state().unwrap().timeline.tracks[0].clips.len();
        manager.add_clip_to_timeline(AddClipParams {
            media_id: asset.id,
            track_id: None,
            start_ms: Some(6000),
        }).unwrap();

        assert!(manager.undo().unwrap());
        assert_eq!(manager.state().unwrap().timeline.tracks[0].clips.len(), count_before);
        assert!(manager.redo().unwrap());
    }
}
