//! Pluggable cloud backends — local-first default, Supabase when wired.

mod local;
mod supabase;

use std::env;

use crate::error::Result;

pub use local::LocalCloudBackend;
pub use supabase::SupabaseCloudBackend;

use super::{AuthSession, BackupRecord, CloudBackupService};
use std::path::{Path, PathBuf};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CloudBackendKind {
    Local,
    Supabase,
}

impl CloudBackendKind {
    pub fn from_env() -> Self {
        match env::var("CINEMASTUDIO_CLOUD_BACKEND").as_deref() {
            Ok("supabase") => Self::Supabase,
            _ => Self::Local,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Local => "local",
            Self::Supabase => "supabase",
        }
    }
}

pub trait CloudBackend: Send + Sync {
    fn kind(&self) -> CloudBackendKind;
    fn login(&self, email: &str, password: &str) -> Result<AuthSession>;
    fn backup(
        &self,
        backup_svc: &CloudBackupService,
        project_dir: &Path,
        project_id: Uuid,
        name: &str,
        backup_id: Uuid,
    ) -> Result<BackupRecord>;
    fn wired(&self) -> bool;
}

pub fn active_backend() -> Box<dyn CloudBackend> {
    match CloudBackendKind::from_env() {
        CloudBackendKind::Local => Box::new(LocalCloudBackend),
        CloudBackendKind::Supabase => Box::new(SupabaseCloudBackend),
    }
}

pub fn backend_status() -> BackendStatus {
    let kind = CloudBackendKind::from_env();
    let backend = active_backend();
    BackendStatus {
        kind,
        wired: backend.wired(),
        data_root_hint: PathBuf::from("CinemaStudio/cloud"),
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BackendStatus {
    pub kind: CloudBackendKind,
    pub wired: bool,
    pub data_root_hint: PathBuf,
}

impl serde::Serialize for CloudBackendKind {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> serde::Deserialize<'de> for CloudBackendKind {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> std::result::Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        Ok(match s.as_str() {
            "supabase" => Self::Supabase,
            _ => Self::Local,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn local_backend_default() {
        std::env::remove_var("CINEMASTUDIO_CLOUD_BACKEND");
        assert_eq!(CloudBackendKind::from_env(), CloudBackendKind::Local);
        assert!(active_backend().wired());
    }
}
