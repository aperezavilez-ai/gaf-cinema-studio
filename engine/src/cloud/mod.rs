pub mod auth;
pub mod backup;

pub use auth::{login_stub, AuthSession, AuthStore};
pub use backup::{BackupRecord, CloudBackupService};

use std::path::{Path, PathBuf};

use uuid::Uuid;

use crate::error::Result;

/// Optional cloud layer — never required for core functionality.
pub struct CloudService {
    root: PathBuf,
    pub auth: AuthStore,
    pub backup: CloudBackupService,
}

impl CloudService {
    pub fn new(data_root: impl AsRef<Path>) -> Self {
        let root = data_root.as_ref().join("CinemaStudio").join("cloud");
        Self {
            auth: AuthStore::new(&root),
            backup: CloudBackupService::new(&root),
            root,
        }
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn is_logged_in(&self) -> Result<bool> {
        Ok(self.auth.load()?.logged_in)
    }

    pub fn login(&self, email: &str, password: &str) -> Result<AuthSession> {
        let session = auth::login_stub(email, password)?;
        self.auth.save(&session)?;
        Ok(session)
    }

    pub fn logout(&self) -> Result<()> {
        self.auth.clear()
    }

    pub fn backup(&self, project_dir: &Path, project_id: Uuid, name: &str, backup_id: Uuid) -> Result<BackupRecord> {
        self.backup.backup_project(project_dir, project_id, name, backup_id)
    }

    pub fn restore(&self, backup_path: &Path, dest: &Path) -> Result<PathBuf> {
        self.backup.restore_project(backup_path, dest)
    }
}
