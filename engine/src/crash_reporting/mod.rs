//! Local crash reporting — opt-in, file-based (Phase 5).

use std::fs;
use std::path::{Path, PathBuf};

use chrono::Utc;
use uuid::Uuid;

use crate::error::{CinemaError, Result};

pub struct CrashReporter {
    reports_dir: PathBuf,
    enabled: bool,
}

impl CrashReporter {
    pub fn new(project_dir: impl AsRef<Path>, enabled: bool) -> Self {
        Self {
            reports_dir: project_dir.as_ref().join("crash_reports"),
            enabled,
        }
    }

    pub fn record(&self, error: &str, context: &str) -> Result<Option<PathBuf>> {
        if !self.enabled {
            return Ok(None);
        }

        fs::create_dir_all(&self.reports_dir)?;
        let id = Uuid::new_v4();
        let path = self.reports_dir.join(format!("{id}.crash.json"));

        let report = serde_json::json!({
            "id": id,
            "timestamp": Utc::now().to_rfc3339(),
            "error": error,
            "context": context,
        });

        fs::write(&path, serde_json::to_string_pretty(&report)?).map_err(CinemaError::Io)?;
        Ok(Some(path))
    }

    pub fn list_reports(&self) -> Result<Vec<PathBuf>> {
        if !self.reports_dir.exists() {
            return Ok(Vec::new());
        }
        Ok(fs::read_dir(&self.reports_dir)?
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| p.extension().is_some_and(|x| x == "json"))
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn writes_crash_report_when_enabled() {
        let tmp = TempDir::new().unwrap();
        let reporter = CrashReporter::new(tmp.path(), true);
        let path = reporter.record("test panic", "unit_test").unwrap();
        assert!(path.is_some());
    }

    #[test]
    fn disabled_writes_nothing() {
        let tmp = TempDir::new().unwrap();
        let reporter = CrashReporter::new(tmp.path(), false);
        assert!(reporter.record("err", "ctx").unwrap().is_none());
    }
}
