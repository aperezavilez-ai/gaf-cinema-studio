//! Cloud backup/restore — optional, async-friendly stub (local cloud dir simulates S3).

use std::fs;
use std::path::{Path, PathBuf};

use chrono::Utc;
use uuid::Uuid;

use crate::error::{CinemaError, Result};

#[derive(Debug, Clone)]
pub struct BackupRecord {
    pub backup_id: Uuid,
    pub project_id: Uuid,
    pub project_name: String,
    pub path: PathBuf,
    pub size_bytes: u64,
    pub created_at: String,
}

pub struct CloudBackupService {
    backups_root: PathBuf,
}

impl CloudBackupService {
    pub fn new(cloud_root: impl AsRef<Path>) -> Self {
        Self {
            backups_root: cloud_root.as_ref().join("backups"),
        }
    }

    /// Copy entire .csproj folder to cloud backups. Non-blocking caller responsibility.
    pub fn backup_project(
        &self,
        project_dir: &Path,
        project_id: Uuid,
        project_name: &str,
        backup_id: Uuid,
    ) -> Result<BackupRecord> {
        if !project_dir.exists() {
            return Err(CinemaError::ProjectNotFound(project_dir.display().to_string()));
        }

        let dest = self
            .backups_root
            .join(project_id.to_string())
            .join(format!("{backup_id}.bundle"));

        copy_dir_recursive(project_dir, &dest)?;

        let size = dir_size(&dest)?;

        Ok(BackupRecord {
            backup_id,
            project_id,
            project_name: project_name.to_string(),
            path: dest,
            size_bytes: size,
            created_at: Utc::now().to_rfc3339(),
        })
    }

    pub fn restore_project(&self, backup_path: &Path, dest_dir: &Path) -> Result<PathBuf> {
        if !backup_path.exists() {
            return Err(CinemaError::ProjectNotFound(
                backup_path.display().to_string(),
            ));
        }

        if dest_dir.exists() {
            return Err(CinemaError::Validation(format!(
                "destination already exists: {}",
                dest_dir.display()
            )));
        }

        copy_dir_recursive(backup_path, dest_dir)?;
        Ok(dest_dir.to_path_buf())
    }

    pub fn list_backups(&self, project_id: Uuid) -> Result<Vec<BackupRecord>> {
        let project_backups = self.backups_root.join(project_id.to_string());
        if !project_backups.exists() {
            return Ok(Vec::new());
        }

        let mut records = Vec::new();
        for entry in fs::read_dir(&project_backups)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                records.push(BackupRecord {
                    backup_id: Uuid::new_v4(),
                    project_id,
                    project_name: path
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .into_owned(),
                    path: path.clone(),
                    size_bytes: dir_size(&path)?,
                    created_at: Utc::now().to_rfc3339(),
                });
            }
        }
        Ok(records)
    }
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());
        if ty.is_dir() {
            copy_dir_recursive(&src_path, &dst_path)?;
        } else {
            fs::copy(&src_path, &dst_path)?;
        }
    }
    Ok(())
}

fn dir_size(path: &Path) -> Result<u64> {
    let mut size = 0u64;
    if path.is_dir() {
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let p = entry.path();
            if p.is_dir() {
                size += dir_size(&p)?;
            } else {
                size += entry.metadata()?.len();
            }
        }
    } else {
        size = fs::metadata(path)?.len();
    }
    Ok(size)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn backup_and_restore_roundtrip() {
        let cloud = TempDir::new().unwrap();
        let project = TempDir::new().unwrap();
        fs::write(project.path().join("project.json"), b"{}").unwrap();

        let svc = CloudBackupService::new(cloud.path());
        let pid = Uuid::new_v4();
        let record = svc
            .backup_project(project.path(), pid, "Test", Uuid::new_v4())
            .unwrap();
        assert!(record.path.exists());

        let restore_dest = project.path().parent().unwrap().join("restored.csproj");
        svc.restore_project(&record.path, &restore_dest).unwrap();
        assert!(restore_dest.join("project.json").exists());
    }
}
