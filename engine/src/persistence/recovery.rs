use std::fs;
use std::path::Path;

use uuid::Uuid;

use crate::error::{CinemaError, Result};
use crate::persistence::atomic_io::atomic_write;
use crate::persistence::sqlite_store::SqliteStore;
use crate::project_state::types::ProjectState;
use crate::project_state::validation::{ensure_valid, validate_project_state};

pub const PROJECT_FILE: &str = "project.json";
pub const SNAPSHOTS_DIR: &str = "snapshots";

#[derive(Debug, Clone)]
pub enum RecoverySource {
    ProjectJson,
    Sqlite,
    SnapshotFile { id: Uuid },
    SqliteSnapshot { id: Uuid },
}

#[derive(Debug, Clone)]
pub struct RecoveryResult {
    pub state: ProjectState,
    pub source: RecoverySource,
    pub was_recovered: bool,
}

/// Load project with automatic recovery cascade:
/// 1. project.json → 2. SQLite → 3. file snapshots → 4. SQLite snapshots
pub fn load_with_recovery(project_dir: &Path) -> Result<RecoveryResult> {
    let project_file = project_dir.join(PROJECT_FILE);
    let store = SqliteStore::open(project_dir)?;

    // Attempt 1: valid project.json
    if project_file.exists() {
        if let Ok(state) = try_load_json(&project_file) {
            if validate_project_state(&state).is_valid() {
                return Ok(RecoveryResult {
                    state,
                    source: RecoverySource::ProjectJson,
                    was_recovered: false,
                });
            }
        }
    }

    // Attempt 2: SQLite primary state
    if let Ok(project_id) = infer_project_id(project_dir, &store) {
        if let Ok(state) = store.load_state(project_id) {
            if validate_project_state(&state).is_valid() {
                restore_canonical_files(project_dir, &state)?;
                return Ok(RecoveryResult {
                    state,
                    source: RecoverySource::Sqlite,
                    was_recovered: true,
                });
            }
        }

        // Attempt 3: SQLite latest snapshot
        if let Ok(Some((snap_id, state))) = store.latest_snapshot(project_id) {
            if validate_project_state(&state).is_valid() {
                restore_canonical_files(project_dir, &state)?;
                return Ok(RecoveryResult {
                    state,
                    source: RecoverySource::SqliteSnapshot { id: snap_id },
                    was_recovered: true,
                });
            }
        }
    }

    // Attempt 4: filesystem snapshots (newest first)
    if let Some((snap_id, state)) = try_latest_file_snapshot(project_dir)? {
        if validate_project_state(&state).is_valid() {
            restore_canonical_files(project_dir, &state)?;
            store.save_state(&state)?;
            return Ok(RecoveryResult {
                state,
                source: RecoverySource::SnapshotFile { id: snap_id },
                was_recovered: true,
            });
        }
    }

    Err(CinemaError::RecoveryFailed(
        "all recovery sources failed — project may be unrecoverable".into(),
    ))
}

pub fn restore_canonical_files(project_dir: &Path, state: &ProjectState) -> Result<()> {
    ensure_valid(state)?;
    let project_file = project_dir.join(PROJECT_FILE);
    let json = serde_json::to_string_pretty(state)?;
    atomic_write(&project_file, json.as_bytes())?;
    Ok(())
}

fn try_load_json(path: &Path) -> Result<ProjectState> {
    let contents = fs::read_to_string(path)?;
    serde_json::from_str(&contents).map_err(|e| {
        CinemaError::CorruptedState(format!("failed to parse {}: {e}", path.display()))
    })
}

fn infer_project_id(project_dir: &Path, store: &SqliteStore) -> Result<Uuid> {
    if let Ok(Some(id)) = store.any_project_id() {
        return Ok(id);
    }

    let project_file = project_dir.join(PROJECT_FILE);
    if project_file.exists() {
        if let Ok(contents) = fs::read_to_string(&project_file) {
            if let Ok(value) = serde_json::from_str::<serde_json::Value>(&contents) {
                if let Some(id) = value.get("projectId").and_then(|v| v.as_str()) {
                    if let Ok(uuid) = Uuid::parse_str(id) {
                        return Ok(uuid);
                    }
                }
            }
        }
    }

    if let Some((_, state)) = try_latest_file_snapshot(project_dir)? {
        return Ok(state.project_id);
    }

    Err(CinemaError::RecoveryFailed(
        "cannot infer project_id for recovery".into(),
    ))
}

fn try_latest_file_snapshot(project_dir: &Path) -> Result<Option<(Uuid, ProjectState)>> {
    let snapshots_dir = project_dir.join(SNAPSHOTS_DIR);
    if !snapshots_dir.exists() {
        return Ok(None);
    }

    let mut entries: Vec<_> = fs::read_dir(&snapshots_dir)?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "json"))
        .collect();

    entries.sort_by_key(|e| e.metadata().and_then(|m| m.modified()).ok());

    if let Some(entry) = entries.last() {
        let path = entry.path();
        let id_str = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
        if let Ok(id) = Uuid::parse_str(id_str) {
            if let Ok(state) = try_load_json(&path) {
                return Ok(Some((id, state)));
            }
        }
    }

    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::persistence::sqlite_store::SqliteStore;
    use crate::project_state::types::default_project_settings;
    use tempfile::TempDir;

    #[test]
    fn recovers_from_corrupt_json_via_sqlite() {
        let tmp = TempDir::new().unwrap();
        let store = SqliteStore::open(tmp.path()).unwrap();
        let state = ProjectState::new("Recover", tmp.path().to_string_lossy(), default_project_settings());

        store.save_state(&state).unwrap();
        restore_canonical_files(tmp.path(), &state).unwrap();

        // Corrupt project.json
        fs::write(tmp.path().join(PROJECT_FILE), b"{ CORRUPT").unwrap();

        let result = load_with_recovery(tmp.path()).unwrap();
        assert!(result.was_recovered);
        assert_eq!(result.state.metadata.name, "Recover");
    }

    #[test]
    fn recovers_from_snapshot_when_json_and_sqlite_missing() {
        let tmp = TempDir::new().unwrap();
        fs::create_dir_all(tmp.path().join(SNAPSHOTS_DIR)).unwrap();

        let state = ProjectState::new("Snap", tmp.path().to_string_lossy(), default_project_settings());
        let snap_id = Uuid::new_v4();
        let snap_path = tmp
            .path()
            .join(SNAPSHOTS_DIR)
            .join(format!("{snap_id}.json"));
        let json = serde_json::to_string_pretty(&state).unwrap();
        fs::write(&snap_path, json).unwrap();

        let result = load_with_recovery(tmp.path()).unwrap();
        assert!(result.was_recovered);
        assert_eq!(result.state.metadata.name, "Snap");
    }
}
