pub mod ai_orchestrator;
pub mod beta;
pub mod billing;
pub mod cloud;
pub mod crash_reporting;
pub mod device_profiler;
pub mod error;
pub mod event_bus;
pub mod export;
pub mod persistence;
pub mod media_decoder;
pub mod native_bridge;
pub mod playback_engine;
pub mod render_pipeline;
pub mod project_state;
pub mod render_scheduler;
pub mod storage;
pub mod telemetry;
pub mod timeline_engine;
pub mod video_engine;
#[cfg(feature = "ffi")]
pub mod ffi;

pub use ai_orchestrator::AiOrchestrator;
pub use beta::{BetaCompletion, BetaRegistry, BetaTracker};
pub use billing::{activate_pro_stub, cancel_subscription, pro_features_enabled, BillingStore, SubscriptionState, SubscriptionTier};
pub use cloud::{login_stub, AuthSession, AuthStore, BackupRecord, CloudBackupService, CloudService};
pub use crash_reporting::CrashReporter;
pub use device_profiler::{
    detect as detect_device, parse_hints_json, DeviceController, DeviceProfile, DeviceTier,
    GpuTier, QualityPolicy, SessionMetrics, ThermalLevel,
};
pub use error::{CinemaError, Result};
pub use event_bus::{CinemaEvent, EventBus, EventSubscriber};
pub use export::{ExportJob, ExportQueue, ExportSettings};
pub use persistence::{AutosaveController, RecoveryResult, SqliteStore};
pub use media_decoder::{
    DecodeRequest, DecodedFrame, DecoderBackend, DecoderRegistry, PixelFormat, StubDecoder,
    VideoDecoder,
};
pub use native_bridge::{bridge_status, decode_frame_at, render_pipeline, set_decoder_backend, set_render_backend};
pub use playback_engine::{PlaybackEngine, PlaybackMetrics, PlaybackMode, PlaybackStatus};
pub use render_pipeline::{RenderBackend, RenderJob, RenderPipeline, RenderProgress, RenderResult};
pub use project_state::{
    default_project_settings, validate_project_state, Mutation, ProjectSettings, ProjectState,
    ProjectStateManager, UndoRedoStack, ValidationReport,
};
pub use render_scheduler::{JobPriority, RenderScheduler, ScheduledJob};
pub use storage::{ImportedMedia, MediaVault, ProxyJob, ProxyQueue};
pub use telemetry::{SessionRecord, TelemetryConfig, TelemetryLog, TelemetryService};
pub use timeline_engine::{AddClipParams, ActiveLayer, ClipHit, FrameComposition};
pub use video_engine::VideoEngine;
