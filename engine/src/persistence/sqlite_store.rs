use std::path::{Path, PathBuf};

use chrono::Utc;
use rusqlite::{params, Connection};
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::error::{CinemaError, Result};
use crate::project_state::types::ProjectState;

pub const DB_FILE: &str = "project.db";

pub struct SqliteStore {
    conn: Connection,
    db_path: PathBuf,
}

impl SqliteStore {
    pub fn open(project_dir: &Path) -> Result<Self> {
        let db_path = project_dir.join(DB_FILE);
        let conn = Connection::open(&db_path)
            .map_err(|e| CinemaError::Database(format!("open failed: {e}")))?;
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")
            .map_err(|e| CinemaError::Database(format!("pragma failed: {e}")))?;

        let store = Self { conn, db_path };
        store.init_schema()?;
        Ok(store)
    }

    fn init_schema(&self) -> Result<()> {
        self.conn
            .execute_batch(
                "
                CREATE TABLE IF NOT EXISTS project_state (
                    project_id TEXT PRIMARY KEY NOT NULL,
                    schema_version INTEGER NOT NULL,
                    state_json TEXT NOT NULL,
                    checksum TEXT NOT NULL,
                    updated_at TEXT NOT NULL
                );

                CREATE TABLE IF NOT EXISTS snapshots (
                    id TEXT PRIMARY KEY NOT NULL,
                    project_id TEXT NOT NULL,
                    sequence_num INTEGER NOT NULL,
                    state_json TEXT NOT NULL,
                    checksum TEXT NOT NULL,
                    created_at TEXT NOT NULL,
                    source TEXT NOT NULL DEFAULT 'autosave'
                );

                CREATE INDEX IF NOT EXISTS idx_snapshots_project_seq
                    ON snapshots(project_id, sequence_num DESC);

                CREATE TABLE IF NOT EXISTS media_index (
                    id TEXT PRIMARY KEY NOT NULL,
                    project_id TEXT NOT NULL,
                    file_name TEXT NOT NULL,
                    original_path TEXT NOT NULL,
                    vault_path TEXT,
                    status TEXT NOT NULL,
                    duration_ms INTEGER NOT NULL DEFAULT 0,
                    file_size_bytes INTEGER NOT NULL DEFAULT 0,
                    imported_at TEXT NOT NULL
                );

                CREATE INDEX IF NOT EXISTS idx_media_project
                    ON media_index(project_id);
                ",
            )
            .map_err(|e| CinemaError::Database(format!("schema init failed: {e}")))?;
        Ok(())
    }

    pub fn save_state(&self, state: &ProjectState) -> Result<()> {
        let json = serde_json::to_string(state)?;
        let checksum = compute_checksum(json.as_bytes());

        self.conn
            .execute(
                "INSERT INTO project_state (project_id, schema_version, state_json, checksum, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5)
                 ON CONFLICT(project_id) DO UPDATE SET
                    schema_version = excluded.schema_version,
                    state_json = excluded.state_json,
                    checksum = excluded.checksum,
                    updated_at = excluded.updated_at",
                params![
                    state.project_id.to_string(),
                    state.schema_version,
                    json,
                    checksum,
                    state.updated_at.to_rfc3339(),
                ],
            )
            .map_err(|e| CinemaError::Database(format!("save state failed: {e}")))?;
        Ok(())
    }

    pub fn load_state(&self, project_id: Uuid) -> Result<ProjectState> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT state_json, checksum FROM project_state WHERE project_id = ?1",
            )
            .map_err(|e| CinemaError::Database(format!("prepare failed: {e}")))?;

        let row = stmt
            .query_row(params![project_id.to_string()], |row| {
                let json: String = row.get(0)?;
                let checksum: String = row.get(1)?;
                Ok((json, checksum))
            })
            .map_err(|e| CinemaError::Database(format!("load state failed: {e}")))?;

        verify_checksum(row.0.as_bytes(), &row.1)?;
        parse_state(&row.0)
    }

    pub fn insert_snapshot(
        &self,
        snapshot_id: Uuid,
        state: &ProjectState,
        source: &str,
        sequence_num: i64,
    ) -> Result<()> {
        let json = serde_json::to_string(state)?;
        let checksum = compute_checksum(json.as_bytes());

        self.conn
            .execute(
                "INSERT INTO snapshots (id, project_id, sequence_num, state_json, checksum, created_at, source)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                params![
                    snapshot_id.to_string(),
                    state.project_id.to_string(),
                    sequence_num,
                    json,
                    checksum,
                    Utc::now().to_rfc3339(),
                    source,
                ],
            )
            .map_err(|e| CinemaError::Database(format!("insert snapshot failed: {e}")))?;
        Ok(())
    }

    pub fn load_snapshot(&self, snapshot_id: Uuid) -> Result<ProjectState> {
        let mut stmt = self
            .conn
            .prepare("SELECT state_json, checksum FROM snapshots WHERE id = ?1")
            .map_err(|e| CinemaError::Database(format!("prepare failed: {e}")))?;

        let row = stmt
            .query_row(params![snapshot_id.to_string()], |row| {
                let json: String = row.get(0)?;
                let checksum: String = row.get(1)?;
                Ok((json, checksum))
            })
            .map_err(|e| CinemaError::Database(format!("load snapshot failed: {e}")))?;

        verify_checksum(row.0.as_bytes(), &row.1)?;
        parse_state(&row.0)
    }

    pub fn latest_snapshot(&self, project_id: Uuid) -> Result<Option<(Uuid, ProjectState)>> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, state_json, checksum FROM snapshots
                 WHERE project_id = ?1
                 ORDER BY sequence_num DESC LIMIT 1",
            )
            .map_err(|e| CinemaError::Database(format!("prepare failed: {e}")))?;

        let mut rows = stmt
            .query(params![project_id.to_string()])
            .map_err(|e| CinemaError::Database(format!("query failed: {e}")))?;

        if let Some(row) = rows
            .next()
            .map_err(|e| CinemaError::Database(format!("row failed: {e}")))?
        {
            let id_str: String = row
                .get(0)
                .map_err(|e| CinemaError::Database(format!("get id failed: {e}")))?;
            let json: String = row
                .get(1)
                .map_err(|e| CinemaError::Database(format!("get json failed: {e}")))?;
            let checksum: String = row
                .get(2)
                .map_err(|e| CinemaError::Database(format!("get checksum failed: {e}")))?;

            verify_checksum(json.as_bytes(), &checksum)?;
            let id = Uuid::parse_str(&id_str)
                .map_err(|e| CinemaError::Database(format!("invalid snapshot uuid: {e}")))?;
            return Ok(Some((id, parse_state(&json)?)));
        }

        Ok(None)
    }

    pub fn list_snapshots(&self, project_id: Uuid) -> Result<Vec<SnapshotMeta>> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, sequence_num, created_at, source FROM snapshots
                 WHERE project_id = ?1 ORDER BY sequence_num DESC",
            )
            .map_err(|e| CinemaError::Database(format!("prepare failed: {e}")))?;

        let rows = stmt
            .query_map(params![project_id.to_string()], |row| {
                Ok(SnapshotMeta {
                    id: row.get::<_, String>(0)?,
                    sequence_num: row.get(1)?,
                    created_at: row.get(2)?,
                    source: row.get(3)?,
                })
            })
            .map_err(|e| CinemaError::Database(format!("query failed: {e}")))?;

        rows.collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|e| CinemaError::Database(format!("collect failed: {e}")))
    }

    pub fn prune_snapshots(&self, project_id: Uuid, keep: usize) -> Result<usize> {
        let snapshots = self.list_snapshots(project_id)?;
        if snapshots.len() <= keep {
            return Ok(0);
        }

        let to_delete = &snapshots[keep..];
        let mut deleted = 0;
        for snap in to_delete {
            self.conn
                .execute("DELETE FROM snapshots WHERE id = ?1", params![snap.id])
                .map_err(|e| CinemaError::Database(format!("delete snapshot failed: {e}")))?;
            deleted += 1;
        }
        Ok(deleted)
    }

    pub fn next_sequence(&self, project_id: Uuid) -> Result<i64> {
        let mut stmt = self
            .conn
            .prepare("SELECT COALESCE(MAX(sequence_num), 0) + 1 FROM snapshots WHERE project_id = ?1")
            .map_err(|e| CinemaError::Database(format!("prepare failed: {e}")))?;

        stmt.query_row(params![project_id.to_string()], |row| row.get(0))
            .map_err(|e| CinemaError::Database(format!("sequence query failed: {e}")))
    }

    pub fn db_path(&self) -> &Path {
        &self.db_path
    }

    pub fn any_project_id(&self) -> Result<Option<Uuid>> {
        let mut stmt = self
            .conn
            .prepare("SELECT project_id FROM project_state LIMIT 1")
            .map_err(|e| CinemaError::Database(format!("prepare failed: {e}")))?;

        let mut rows = stmt
            .query([])
            .map_err(|e| CinemaError::Database(format!("query failed: {e}")))?;

        if let Some(row) = rows
            .next()
            .map_err(|e| CinemaError::Database(format!("row failed: {e}")))?
        {
            let id_str: String = row
                .get(0)
                .map_err(|e| CinemaError::Database(format!("get id failed: {e}")))?;
            let id = Uuid::parse_str(&id_str)
                .map_err(|e| CinemaError::Database(format!("invalid uuid: {e}")))?;
            return Ok(Some(id));
        }
        Ok(None)
    }
}

#[derive(Debug, Clone)]
pub struct SnapshotMeta {
    pub id: String,
    pub sequence_num: i64,
    pub created_at: String,
    pub source: String,
}

pub fn compute_checksum(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    format!("{:x}", hasher.finalize())
}

fn verify_checksum(data: &[u8], expected: &str) -> Result<()> {
    let actual = compute_checksum(data);
    if actual != expected {
        return Err(CinemaError::CorruptedState(
            "checksum mismatch — data may be corrupted".into(),
        ));
    }
    Ok(())
}

fn parse_state(json: &str) -> Result<ProjectState> {
    serde_json::from_str(json).map_err(|e| {
        CinemaError::CorruptedState(format!("failed to parse stored state: {e}"))
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::project_state::types::{default_project_settings, ProjectState};
    use tempfile::TempDir;

    #[test]
    fn save_and_load_roundtrip() {
        let tmp = TempDir::new().unwrap();
        let store = SqliteStore::open(tmp.path()).unwrap();
        let state = ProjectState::new("DB Test", tmp.path().to_string_lossy(), default_project_settings());

        store.save_state(&state).unwrap();
        let loaded = store.load_state(state.project_id).unwrap();
        assert_eq!(loaded.project_id, state.project_id);
        assert_eq!(loaded.metadata.name, "DB Test");
    }

    #[test]
    fn snapshot_prune_keeps_latest() {
        let tmp = TempDir::new().unwrap();
        let store = SqliteStore::open(tmp.path()).unwrap();
        let mut state = ProjectState::new("Snap", tmp.path().to_string_lossy(), default_project_settings());

        for i in 0..12 {
            state.metadata.name = format!("Version {i}");
            let seq = store.next_sequence(state.project_id).unwrap();
            store
                .insert_snapshot(Uuid::new_v4(), &state, "autosave", seq)
                .unwrap();
        }

        let deleted = store.prune_snapshots(state.project_id, 10).unwrap();
        assert_eq!(deleted, 2);
        assert_eq!(store.list_snapshots(state.project_id).unwrap().len(), 10);
    }
}
