use std::sync::{Arc, Mutex};

use uuid::Uuid;

#[derive(Debug, Clone)]
pub enum CinemaEvent {
    ProjectCreated {
        project_id: Uuid,
        name: String,
    },
    ProjectOpened {
        project_id: Uuid,
        path: String,
    },
    ProjectSaved {
        project_id: Uuid,
        path: String,
    },
    ProjectRecovered {
        project_id: Uuid,
        snapshot_id: Option<Uuid>,
        reason: String,
    },
    SnapshotCreated {
        project_id: Uuid,
        snapshot_id: Uuid,
    },
    MediaIndexed {
        project_id: Uuid,
        media_id: Uuid,
        file_name: String,
    },
    MediaImportStarted {
        project_id: Uuid,
        media_id: Uuid,
        path: String,
    },
    ProxyGenerationStarted {
        project_id: Uuid,
        media_id: Uuid,
    },
    ProxyGenerationCompleted {
        project_id: Uuid,
        media_id: Uuid,
        proxy_path: String,
    },
    ProxyGenerationFailed {
        project_id: Uuid,
        media_id: Uuid,
        error: String,
    },
    AutosaveCompleted {
        project_id: Uuid,
        snapshot_id: Uuid,
    },
    AutosaveFailed {
        project_id: Uuid,
        error: String,
    },
    ProjectCorrupted {
        project_id: Option<Uuid>,
        error: String,
        recovered: bool,
    },
    PlayheadChanged {
        project_id: Uuid,
        time_ms: u64,
    },
    WorkflowPhaseChanged {
        project_id: Uuid,
        from: String,
        to: String,
    },
    ClipAdded {
        project_id: Uuid,
        clip_id: Uuid,
        track_id: Uuid,
        media_id: Uuid,
    },
    ClipRemoved {
        project_id: Uuid,
        clip_id: Uuid,
    },
    ClipMoved {
        project_id: Uuid,
        clip_id: Uuid,
        new_start_ms: u64,
    },
    ClipTrimmed {
        project_id: Uuid,
        clip_id: Uuid,
    },
    TimelineDurationChanged {
        project_id: Uuid,
        duration_ms: u64,
    },
    PlaybackStarted {
        project_id: Uuid,
        time_ms: u64,
    },
    PlaybackStopped {
        project_id: Uuid,
        time_ms: u64,
    },
    PlaybackStalled {
        project_id: Uuid,
        reason: String,
    },
    UndoApplied {
        project_id: Uuid,
    },
    RedoApplied {
        project_id: Uuid,
    },
    ExportStarted {
        project_id: Uuid,
        export_id: Uuid,
        resolution: String,
    },
    ExportProgress {
        project_id: Uuid,
        export_id: Uuid,
        progress: f64,
    },
    ExportCompleted {
        project_id: Uuid,
        export_id: Uuid,
        output_path: String,
    },
    ExportFailed {
        project_id: Uuid,
        export_id: Uuid,
        error: String,
    },
    AiSuggestionCreated {
        project_id: Uuid,
        suggestion: crate::project_state::types::AiSuggestion,
    },
    AiSuggestionDismissed {
        project_id: Uuid,
        suggestion_id: Uuid,
    },
    AiActionExecuted {
        project_id: Uuid,
        action_id: String,
        suggestion_id: Uuid,
        result: String,
    },
    DeviceTierDetected {
        project_id: Uuid,
        tier: String,
    },
    ThermalThrottle {
        project_id: Uuid,
        level: String,
    },
    QualityDegraded {
        project_id: Uuid,
        from: String,
        to: String,
        reason: String,
    },
    AuthStateChanged {
        logged_in: bool,
        email: Option<String>,
    },
    CloudBackupStarted {
        project_id: Uuid,
        backup_id: Uuid,
    },
    CloudBackupCompleted {
        project_id: Uuid,
        backup_id: Uuid,
        size_bytes: u64,
    },
    CloudRestoreCompleted {
        project_id: Uuid,
        path: String,
    },
    SubscriptionUpdated {
        tier: String,
    },
    BetaProjectCompleted {
        project_id: Uuid,
        total_completions: usize,
        gate_met: bool,
    },
    TelemetryUploaded {
        path: String,
    },
}

pub trait EventSubscriber: Send + Sync {
    fn on_event(&self, event: &CinemaEvent);
}

type SubscriberList = Arc<Mutex<Vec<Arc<dyn EventSubscriber>>>>;

#[derive(Clone)]
pub struct EventBus {
    subscribers: SubscriberList,
    log: Arc<Mutex<Vec<CinemaEvent>>>,
}

impl EventBus {
    pub fn new() -> Self {
        Self {
            subscribers: Arc::new(Mutex::new(Vec::new())),
            log: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn subscribe(&self, subscriber: Arc<dyn EventSubscriber>) {
        self.subscribers.lock().unwrap().push(subscriber);
    }

    pub fn emit(&self, event: CinemaEvent) {
        self.log.lock().unwrap().push(event.clone());
        for sub in self.subscribers.lock().unwrap().iter() {
            sub.on_event(&event);
        }
    }

    pub fn event_log(&self) -> Vec<CinemaEvent> {
        self.log.lock().unwrap().clone()
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestSubscriber {
        events: Arc<Mutex<Vec<CinemaEvent>>>,
    }

    impl EventSubscriber for TestSubscriber {
        fn on_event(&self, event: &CinemaEvent) {
            self.events.lock().unwrap().push(event.clone());
        }
    }

    #[test]
    fn emit_reaches_subscribers() {
        let bus = EventBus::new();
        let received = Arc::new(Mutex::new(Vec::new()));
        bus.subscribe(Arc::new(TestSubscriber {
            events: received.clone(),
        }));

        bus.emit(CinemaEvent::ProjectCreated {
            project_id: Uuid::new_v4(),
            name: "Test".into(),
        });

        assert_eq!(received.lock().unwrap().len(), 1);
    }
}
