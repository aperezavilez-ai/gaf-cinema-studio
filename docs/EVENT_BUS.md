# CinemaStudio — Internal Event Bus

> Decoupling contract. Modules communicate through typed events, not direct calls.

## Design

- **Synchronous dispatch** within engine thread for MVP
- **Single writer** — only `ProjectStateManager` mutates state
- **Subscribers** react to events (UI refresh, AI analysis, render queue)
- Events are **immutable** records with timestamp + projectId

## Event Envelope

```json
{
  "eventId": "uuid",
  "projectId": "uuid",
  "type": "CLIP_ADDED",
  "timestamp": "ISO-8601",
  "payload": { }
}
```

## Event Catalog

### Project Lifecycle

| Event | Payload | Emitters | Subscribers |
|-------|---------|----------|-------------|
| `PROJECT_CREATED` | `{ name, settings }` | ProjectStateManager | UI, AI |
| `PROJECT_OPENED` | `{ path }` | ProjectStateManager | UI, Timeline, Storage |
| `PROJECT_SAVED` | `{ path, snapshotId? }` | Persistence | UI |
| `PROJECT_CLOSED` | `{}` | ProjectStateManager | All |
| `PROJECT_RECOVERED` | `{ snapshotId, reason }` | Recovery | UI, AI |
| `PROJECT_CORRUPTED` | `{ error, recoveredFrom? }` | Recovery | UI |

### Media

| Event | Payload | Emitters | Subscribers |
|-------|---------|----------|-------------|
| `MEDIA_IMPORT_STARTED` | `{ mediaId, path }` | Storage | UI |
| `MEDIA_INDEXED` | `{ mediaId, metadata }` | MediaIndexer | UI, Timeline, AI |
| `MEDIA_IMPORT_FAILED` | `{ mediaId, error }` | Storage | UI |
| `PROXY_GENERATION_STARTED` | `{ mediaId }` | Storage | UI, Render |
| `PROXY_GENERATION_COMPLETED` | `{ mediaId, proxyPath }` | Storage | Playback, UI |
| `PROXY_GENERATION_FAILED` | `{ mediaId, error }` | Storage | UI |

### Timeline

| Event | Payload | Emitters | Subscribers |
|-------|---------|----------|-------------|
| `CLIP_ADDED` | `{ clipId, trackId, mediaId }` | ProjectStateManager | Timeline, Playback, AI, UI |
| `CLIP_REMOVED` | `{ clipId, trackId }` | ProjectStateManager | Timeline, Playback, AI, UI |
| `CLIP_MOVED` | `{ clipId, newStartMs }` | ProjectStateManager | Timeline, Playback, UI |
| `CLIP_TRIMMED` | `{ clipId, sourceInMs, sourceOutMs }` | ProjectStateManager | Timeline, Playback, UI |
| `PLAYHEAD_CHANGED` | `{ timeMs }` | Playback | UI |
| `TIMELINE_DURATION_CHANGED` | `{ durationMs }` | Timeline | UI, AI |

### Playback & Render

| Event | Payload | Emitters | Subscribers |
|-------|---------|----------|-------------|
| `PLAYBACK_STARTED` | `{ timeMs }` | Playback | UI |
| `PLAYBACK_STOPPED` | `{ timeMs }` | Playback | UI |
| `PLAYBACK_STALLED` | `{ reason }` | Playback | UI, DeviceProfiler |
| `RENDER_JOB_QUEUED` | `{ jobId, type }` | RenderPipeline | UI |
| `RENDER_JOB_PROGRESS` | `{ jobId, progress }` | RenderPipeline | UI |
| `RENDER_JOB_COMPLETED` | `{ jobId, outputPath? }` | RenderPipeline | UI, AI |
| `RENDER_JOB_FAILED` | `{ jobId, error }` | RenderPipeline | UI |

### Export

| Event | Payload | Emitters | Subscribers |
|-------|---------|----------|-------------|
| `EXPORT_STARTED` | `{ exportId, settings }` | Export | UI, AI |
| `EXPORT_PROGRESS` | `{ exportId, progress }` | Export | UI |
| `EXPORT_COMPLETED` | `{ exportId, outputPath }` | Export | UI, AI, Workflow |
| `EXPORT_FAILED` | `{ exportId, error }` | Export | UI |

### AI Orchestrator

| Event | Payload | Emitters | Subscribers |
|-------|---------|----------|-------------|
| `AI_SUGGESTION_CREATED` | `{ suggestion }` | AI Orchestrator | UI |
| `AI_SUGGESTION_DISMISSED` | `{ suggestionId }` | AI Orchestrator | UI |
| `AI_ACTION_EXECUTED` | `{ actionId, result }` | AI Orchestrator | ProjectStateManager, UI |
| `WORKFLOW_PHASE_CHANGED` | `{ from, to }` | Workflow Engine | UI, AI |

### Storage & Recovery

| Event | Payload | Emitters | Subscribers |
|-------|---------|----------|-------------|
| `AUTOSAVE_COMPLETED` | `{ snapshotId }` | Persistence | UI (subtle) |
| `AUTOSAVE_FAILED` | `{ error }` | Persistence | UI |
| `SNAPSHOT_CREATED` | `{ snapshotId }` | Persistence | — |
| `CACHE_CLEANUP_COMPLETED` | `{ freedBytes }` | Storage | UI |
| `STORAGE_LOW` | `{ availableBytes }` | Storage | UI, AI |

### Device (Phase 5)

| Event | Payload | Emitters | Subscribers |
|-------|---------|----------|-------------|
| `DEVICE_TIER_DETECTED` | `{ tier }` | DeviceProfiler | Render, Playback |
| `THERMAL_THROTTLE` | `{ level }` | DeviceProfiler | Render, Playback, UI |
| `QUALITY_DEGRADED` | `{ from, to, reason }` | DeviceProfiler | UI |

## Rules

1. **Never** emit events from UI layer directly — UI sends commands to ProjectStateManager
2. **Never** subscribe with circular dependencies (A → B → A)
3. New event types require entry in this document before implementation
4. Payload schemas must be JSON-serializable and versioned if breaking

## Implementation (Rust)

```rust
pub enum CinemaEvent {
    ProjectCreated { name: String },
    ClipAdded { clip_id: Uuid, track_id: Uuid },
    // ...
}

pub trait EventSubscriber: Send + Sync {
    fn on_event(&self, event: &CinemaEvent);
}

pub struct EventBus {
    subscribers: Vec<Box<dyn EventSubscriber>>,
}
```

Phase 0: types defined.
Phase 1: bus wired to ProjectStateManager mutations.
