//! Supabase backend — returns NotWired until SDK + env vars connected.

use std::path::Path;
use uuid::Uuid;

use super::{CloudBackend, CloudBackendKind, CloudBackupService};
use crate::cloud::{AuthSession, BackupRecord};
use crate::error::{CinemaError, Result};

pub struct SupabaseCloudBackend;

impl CloudBackend for SupabaseCloudBackend {
    fn kind(&self) -> CloudBackendKind {
        CloudBackendKind::Supabase
    }

    fn login(&self, _email: &str, _password: &str) -> Result<AuthSession> {
        Err(CinemaError::Validation(
            "not wired: set SUPABASE_URL + SUPABASE_ANON_KEY and implement supabase provider".into(),
        ))
    }

    fn backup(
        &self,
        _backup_svc: &CloudBackupService,
        _project_dir: &Path,
        _project_id: Uuid,
        _name: &str,
        _backup_id: Uuid,
    ) -> Result<BackupRecord> {
        Err(CinemaError::Validation(
            "not wired: Supabase Storage presigned upload".into(),
        ))
    }

    fn wired(&self) -> bool {
        false
    }
}
