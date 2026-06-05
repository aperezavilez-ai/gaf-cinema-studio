//! Local cloud dir — simulates remote storage on disk (Phase 6).

use std::path::Path;
use uuid::Uuid;

use super::{CloudBackend, CloudBackendKind, CloudBackupService};
use crate::cloud::auth::login_stub;
use crate::cloud::{AuthSession, BackupRecord};
use crate::error::Result;

pub struct LocalCloudBackend;

impl CloudBackend for LocalCloudBackend {
    fn kind(&self) -> CloudBackendKind {
        CloudBackendKind::Local
    }

    fn login(&self, email: &str, password: &str) -> Result<AuthSession> {
        login_stub(email, password)
    }

    fn backup(
        &self,
        backup_svc: &CloudBackupService,
        project_dir: &Path,
        project_id: Uuid,
        name: &str,
        backup_id: Uuid,
    ) -> Result<BackupRecord> {
        backup_svc.backup_project(project_dir, project_id, name, backup_id)
    }

    fn wired(&self) -> bool {
        true
    }
}
