use std::fs;
use std::path::{Path, PathBuf};

use chrono::Utc;
use uuid::Uuid;

use crate::error::{CinemaError, Result};
use crate::event_bus::{CinemaEvent, EventBus};
use crate::project_state::types::{MediaAsset, MediaStatus};

const MEDIA_DIR: &str = "media";

pub struct MediaVault {
    project_dir: PathBuf,
}

#[derive(Debug, Clone)]
pub struct ImportedMedia {
    pub asset: MediaAsset,
    pub vault_path: PathBuf,
}

impl MediaVault {
    pub fn new(project_dir: impl AsRef<Path>) -> Self {
        Self {
            project_dir: project_dir.as_ref().to_path_buf(),
        }
    }

    pub fn media_dir(&self) -> PathBuf {
        self.project_dir.join(MEDIA_DIR)
    }

    /// Import a media file into the project vault. Copies file, indexes metadata.
    pub fn import_file(
        &self,
        source_path: impl AsRef<Path>,
        project_id: Uuid,
        event_bus: &EventBus,
    ) -> Result<ImportedMedia> {
        let source_path = source_path.as_ref();

        if !source_path.exists() {
            return Err(CinemaError::Storage(format!(
                "source file not found: {}",
                source_path.display()
            )));
        }

        let media_id = Uuid::new_v4();
        let file_name = source_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();

        event_bus.emit(CinemaEvent::MediaImportStarted {
            project_id,
            media_id,
            path: source_path.display().to_string(),
        });

        let media_dir = self.media_dir();
        fs::create_dir_all(&media_dir)?;

        let ext = source_path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("bin");
        let vault_path = media_dir.join(format!("{media_id}.{ext}"));

        fs::copy(source_path, &vault_path).map_err(|e| {
            CinemaError::Storage(format!("failed to copy media: {e}"))
        })?;

        let metadata = fs::metadata(&vault_path)?;
        let mime_type = mime_guess::from_path(&vault_path)
            .first_or_octet_stream()
            .essence_str()
            .to_string();

        let asset = MediaAsset {
            id: media_id,
            original_path: source_path.display().to_string(),
            proxy_path: None,
            thumbnail_path: None,
            file_name: file_name.clone(),
            mime_type,
            duration_ms: 0, // Phase 2: extract via FFmpeg
            width: 0,
            height: 0,
            file_size_bytes: metadata.len(),
            status: MediaStatus::Indexing,
            imported_at: Utc::now(),
            checksum: Some(crate::persistence::compute_checksum(
                &fs::read(&vault_path).unwrap_or_default(),
            )),
        };

        // Mark ready after basic indexing (no FFmpeg yet)
        let mut asset = asset;
        asset.status = MediaStatus::Ready;

        event_bus.emit(CinemaEvent::MediaIndexed {
            project_id,
            media_id,
            file_name,
        });

        Ok(ImportedMedia { asset, vault_path })
    }

    pub fn list_vault_files(&self) -> Result<Vec<PathBuf>> {
        let media_dir = self.media_dir();
        if !media_dir.exists() {
            return Ok(Vec::new());
        }

        let files = fs::read_dir(&media_dir)?
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| p.is_file())
            .collect();
        Ok(files)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event_bus::EventBus;
    use tempfile::TempDir;

    #[test]
    fn import_copies_file_to_vault() {
        let tmp = TempDir::new().unwrap();
        let source = tmp.path().join("clip.mp4");
        fs::write(&source, b"fake video content").unwrap();

        let vault = MediaVault::new(tmp.path());
        let bus = EventBus::new();
        let project_id = Uuid::new_v4();

        let imported = vault.import_file(&source, project_id, &bus).unwrap();
        assert!(imported.vault_path.exists());
        assert_eq!(imported.asset.status, MediaStatus::Ready);
        assert!(imported.asset.file_size_bytes > 0);
    }
}
