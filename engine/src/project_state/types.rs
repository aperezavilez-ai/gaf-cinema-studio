use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub const SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ProjectState {
    pub schema_version: u32,
    pub project_id: Uuid,
    pub metadata: ProjectMetadata,
    pub media: Vec<MediaAsset>,
    pub timeline: Timeline,
    pub workflow_state: WorkflowState,
    pub ai_state: AiState,
    pub render_state: RenderState,
    pub export_state: ExportState,
    pub storage_state: StorageState,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ProjectMetadata {
    pub name: String,
    #[serde(default)]
    pub description: String,
    pub frame_rate: f64,
    pub resolution: Resolution,
    #[serde(default = "default_aspect_ratio")]
    pub aspect_ratio: String,
    #[serde(default = "default_color_space")]
    pub color_space: ColorSpace,
    #[serde(default)]
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Resolution {
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ColorSpace {
    Rec709,
    Rec2020,
    DciP3,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct MediaAsset {
    pub id: Uuid,
    pub original_path: String,
    pub proxy_path: Option<String>,
    pub thumbnail_path: Option<String>,
    pub file_name: String,
    pub mime_type: String,
    pub duration_ms: u64,
    #[serde(default)]
    pub width: u32,
    #[serde(default)]
    pub height: u32,
    #[serde(default)]
    pub file_size_bytes: u64,
    pub status: MediaStatus,
    pub imported_at: DateTime<Utc>,
    #[serde(default)]
    pub checksum: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum MediaStatus {
    Pending,
    Indexing,
    Ready,
    Error,
    Missing,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Timeline {
    pub duration_ms: u64,
    pub playhead_ms: u64,
    pub tracks: Vec<Track>,
    #[serde(default)]
    pub markers: Vec<Marker>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Track {
    pub id: Uuid,
    #[serde(rename = "type")]
    pub track_type: TrackType,
    #[serde(default)]
    pub name: String,
    pub order: u32,
    #[serde(default)]
    pub muted: bool,
    pub clips: Vec<Clip>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum TrackType {
    Video,
    Audio,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Clip {
    pub id: Uuid,
    pub media_id: Uuid,
    pub start_ms: u64,
    pub duration_ms: u64,
    pub source_in_ms: u64,
    pub source_out_ms: u64,
    #[serde(default)]
    pub label: String,
    #[serde(default)]
    pub transitions: ClipTransitions,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ClipTransitions {
    #[serde(default)]
    pub fade_in_ms: u64,
    #[serde(default)]
    pub fade_out_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Marker {
    pub id: Uuid,
    pub time_ms: u64,
    pub label: String,
    #[serde(default)]
    pub color: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum WorkflowPhase {
    Import,
    Organize,
    Edit,
    Review,
    Export,
    Complete,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowState {
    pub phase: WorkflowPhase,
    pub completed_steps: Vec<String>,
    #[serde(default)]
    pub last_action: Option<String>,
    #[serde(default)]
    pub last_action_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AiState {
    pub suggestions: Vec<AiSuggestion>,
    pub dismissed_suggestion_ids: Vec<Uuid>,
    #[serde(default)]
    pub dismissed_action_ids: Vec<String>,
    pub last_analysis_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AiSuggestion {
    pub id: Uuid,
    #[serde(rename = "type")]
    pub suggestion_type: AiSuggestionType,
    pub priority: AiPriority,
    pub message: String,
    pub action_id: String,
    #[serde(default)]
    pub payload: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum AiSuggestionType {
    Workflow,
    Edit,
    Audio,
    Export,
    Organization,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum AiPriority {
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct RenderState {
    pub preview_quality: PreviewQuality,
    pub active_jobs: Vec<RenderJob>,
    pub last_preview_frame_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum PreviewQuality {
    Low,
    Medium,
    High,
    #[default]
    Auto,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct RenderJob {
    pub id: Uuid,
    #[serde(rename = "type")]
    pub job_type: RenderJobType,
    pub status: RenderJobStatus,
    pub progress: f64,
    #[serde(default)]
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum RenderJobType {
    Proxy,
    Preview,
    Export,
    Thumbnail,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum RenderJobStatus {
    Queued,
    Running,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ExportState {
    pub active_export_id: Option<Uuid>,
    pub history: Vec<ExportRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ExportRecord {
    pub id: Uuid,
    pub format: ExportFormat,
    pub resolution: String,
    pub output_path: Option<String>,
    pub status: ExportStatus,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ExportFormat {
    Mp4,
    Mov,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ExportStatus {
    Pending,
    Running,
    Completed,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct StorageState {
    pub project_root: String,
    pub cache_size_bytes: u64,
    pub proxy_size_bytes: u64,
    pub last_cleanup_at: Option<DateTime<Utc>>,
    #[serde(default = "default_true")]
    pub autosave_enabled: bool,
    #[serde(default = "default_autosave_interval")]
    pub autosave_interval_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ProjectSettings {
    pub frame_rate: f64,
    pub resolution: Resolution,
    #[serde(default = "default_aspect_ratio")]
    pub aspect_ratio: String,
    #[serde(default = "default_color_space")]
    pub color_space: ColorSpace,
}

impl ProjectState {
    pub fn new(name: impl Into<String>, project_root: impl Into<String>, settings: ProjectSettings) -> Self {
        let now = Utc::now();
        let project_id = Uuid::new_v4();

        Self {
            schema_version: SCHEMA_VERSION,
            project_id,
            metadata: ProjectMetadata {
                name: name.into(),
                description: String::new(),
                frame_rate: settings.frame_rate,
                resolution: settings.resolution,
                aspect_ratio: settings.aspect_ratio,
                color_space: settings.color_space,
                tags: Vec::new(),
            },
            media: Vec::new(),
            timeline: Timeline {
                duration_ms: 0,
                playhead_ms: 0,
                tracks: default_tracks(),
                markers: Vec::new(),
            },
            workflow_state: WorkflowState {
                phase: WorkflowPhase::Import,
                completed_steps: Vec::new(),
                last_action: Some("project_created".into()),
                last_action_at: Some(now),
            },
            ai_state: AiState {
                suggestions: Vec::new(),
                dismissed_suggestion_ids: Vec::new(),
                dismissed_action_ids: Vec::new(),
                last_analysis_at: None,
            },
            render_state: RenderState {
                preview_quality: PreviewQuality::Auto,
                active_jobs: Vec::new(),
                last_preview_frame_at: None,
            },
            export_state: ExportState {
                active_export_id: None,
                history: Vec::new(),
            },
            storage_state: StorageState {
                project_root: project_root.into(),
                cache_size_bytes: 0,
                proxy_size_bytes: 0,
                last_cleanup_at: None,
                autosave_enabled: true,
                autosave_interval_ms: default_autosave_interval(),
            },
            created_at: now,
            updated_at: now,
        }
    }
}

fn default_tracks() -> Vec<Track> {
    vec![
        Track {
            id: Uuid::new_v4(),
            track_type: TrackType::Video,
            name: "Video 1".into(),
            order: 0,
            muted: false,
            clips: Vec::new(),
        },
        Track {
            id: Uuid::new_v4(),
            track_type: TrackType::Audio,
            name: "Audio 1".into(),
            order: 1,
            muted: false,
            clips: Vec::new(),
        },
    ]
}

fn default_aspect_ratio() -> String {
    "16:9".into()
}

fn default_color_space() -> ColorSpace {
    ColorSpace::Rec709
}

fn default_true() -> bool {
    true
}

fn default_autosave_interval() -> u64 {
    5000
}

pub fn default_project_settings() -> ProjectSettings {
    ProjectSettings {
        frame_rate: 24.0,
        resolution: Resolution {
            width: 1920,
            height: 1080,
        },
        aspect_ratio: default_aspect_ratio(),
        color_space: default_color_space(),
    }
}
