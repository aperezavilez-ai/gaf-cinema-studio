use std::path::Path;
use std::time::{Duration, Instant};

use uuid::Uuid;

use crate::error::{CinemaError, Result};
use crate::event_bus::{CinemaEvent, EventBus};
use crate::persistence::sqlite_store::SqliteStore;
use crate::project_state::types::ProjectState;

pub const DEFAULT_AUTOSAVE_INTERVAL: Duration = Duration::from_secs(5);
pub const MAX_SNAPSHOTS: usize = 10;

pub struct AutosaveController {
    interval: Duration,
    last_save: Option<Instant>,
    enabled: bool,
}

impl AutosaveController {
    pub fn new(enabled: bool, interval_ms: u64) -> Self {
        Self {
            interval: Duration::from_millis(interval_ms),
            last_save: None,
            enabled,
        }
    }

    pub fn from_state(state: &ProjectState) -> Self {
        Self::new(
            state.storage_state.autosave_enabled,
            state.storage_state.autosave_interval_ms,
        )
    }

    pub fn should_autosave(&self) -> bool {
        if !self.enabled {
            return false;
        }
        match self.last_save {
            None => true,
            Some(last) => last.elapsed() >= self.interval,
        }
    }

    pub fn mark_saved(&mut self) {
        self.last_save = Some(Instant::now());
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }
}

/// Perform autosave: snapshot to SQLite + JSON file, rotate old snapshots.
pub fn perform_autosave(
    store: &SqliteStore,
    state: &ProjectState,
    project_json_path: &Path,
    event_bus: &EventBus,
) -> Result<Uuid> {
    let json = serde_json::to_string_pretty(state)?;
    crate::persistence::atomic_io::atomic_write(project_json_path, json.as_bytes())?;

    store.save_state(state)?;

    let snapshot_id = Uuid::new_v4();
    let seq = store.next_sequence(state.project_id)?;
    store.insert_snapshot(snapshot_id, state, "autosave", seq)?;

    store.prune_snapshots(state.project_id, MAX_SNAPSHOTS)?;

    event_bus.emit(CinemaEvent::AutosaveCompleted {
        project_id: state.project_id,
        snapshot_id,
    });

    Ok(snapshot_id)
}

pub fn tick_autosave(
    controller: &mut AutosaveController,
    store: &SqliteStore,
    state: &ProjectState,
    project_json_path: &Path,
    event_bus: &EventBus,
) -> Result<Option<Uuid>> {
    if !controller.should_autosave() {
        return Ok(None);
    }

    match perform_autosave(store, state, project_json_path, event_bus) {
        Ok(id) => {
            controller.mark_saved();
            Ok(Some(id))
        }
        Err(e) => {
            event_bus.emit(CinemaEvent::AutosaveFailed {
                project_id: state.project_id,
                error: e.to_string(),
            });
            Err(e)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::project_state::types::default_project_settings;
    use std::thread;
    use tempfile::TempDir;

    #[test]
    fn autosave_respects_interval() {
        let mut ctrl = AutosaveController::new(true, 100);
        assert!(ctrl.should_autosave());
        ctrl.mark_saved();
        assert!(!ctrl.should_autosave());
        thread::sleep(Duration::from_millis(150));
        assert!(ctrl.should_autosave());
    }

    #[test]
    fn perform_autosave_writes_files() {
        let tmp = TempDir::new().unwrap();
        let store = SqliteStore::open(tmp.path()).unwrap();
        let bus = EventBus::new();
        let state = ProjectState::new("Auto", tmp.path().to_string_lossy(), default_project_settings());
        let json_path = tmp.path().join("project.json");

        let snap_id = perform_autosave(&store, &state, &json_path, &bus).unwrap();
        assert!(json_path.exists());
        assert!(store.load_snapshot(snap_id).is_ok());
    }
}
