pub mod atomic_io;
pub mod autosave;
pub mod recovery;
pub mod sqlite_store;

pub use autosave::{perform_autosave, tick_autosave, AutosaveController, DEFAULT_AUTOSAVE_INTERVAL, MAX_SNAPSHOTS};
pub use recovery::{load_with_recovery, restore_canonical_files, RecoveryResult, RecoverySource, PROJECT_FILE, SNAPSHOTS_DIR};
pub use sqlite_store::{compute_checksum, SqliteStore, SnapshotMeta, DB_FILE};
