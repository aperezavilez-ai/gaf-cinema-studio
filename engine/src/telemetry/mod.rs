//! Opt-in telemetry — session tracking + crash rate (Phase 6).

use std::fs;
use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::Result;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TelemetryConfig {
    pub enabled: bool,
    pub upload_enabled: bool,
}

impl Default for TelemetryConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            upload_enabled: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionRecord {
    pub session_id: Uuid,
    pub project_id: Option<Uuid>,
    pub started_at: DateTime<Utc>,
    pub ended_at: Option<DateTime<Utc>>,
    pub crashed: bool,
    pub events_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct TelemetryLog {
    pub config: TelemetryConfig,
    pub sessions: Vec<SessionRecord>,
}

pub struct TelemetryService {
    path: PathBuf,
    current: Option<SessionRecord>,
}

impl TelemetryService {
    pub fn new(data_root: impl AsRef<Path>) -> Self {
        Self {
            path: data_root
                .as_ref()
                .join("CinemaStudio")
                .join("telemetry.json"),
            current: None,
        }
    }

    pub fn load(&self) -> Result<TelemetryLog> {
        if !self.path.exists() {
            return Ok(TelemetryLog::default());
        }
        let data = fs::read_to_string(&self.path)?;
        Ok(serde_json::from_str(&data)?)
    }

    fn save(&self, log: &TelemetryLog) -> Result<()> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&self.path, serde_json::to_string_pretty(log)?)?;
        Ok(())
    }

    pub fn set_enabled(&self, enabled: bool) -> Result<()> {
        let mut log = self.load()?;
        log.config.enabled = enabled;
        self.save(&log)
    }

    pub fn is_enabled(&self) -> Result<bool> {
        Ok(self.load()?.config.enabled)
    }

    pub fn start_session(&mut self, project_id: Option<Uuid>) -> Result<Uuid> {
        let id = Uuid::new_v4();
        self.current = Some(SessionRecord {
            session_id: id,
            project_id,
            started_at: Utc::now(),
            ended_at: None,
            crashed: false,
            events_count: 0,
        });
        Ok(id)
    }

    pub fn end_session(&mut self, crashed: bool) -> Result<()> {
        if self.load()?.config.enabled {
            if let Some(mut session) = self.current.take() {
                session.ended_at = Some(Utc::now());
                session.crashed = crashed;
                let mut log = self.load()?;
                log.sessions.push(session);
                self.save(&log)?;
            }
        } else {
            self.current = None;
        }
        Ok(())
    }

    pub fn crash_rate(&self) -> Result<f64> {
        let log = self.load()?;
        let completed: Vec<_> = log.sessions.iter().filter(|s| s.ended_at.is_some()).collect();
        if completed.is_empty() {
            return Ok(0.0);
        }
        let crashes = completed.iter().filter(|s| s.crashed).count();
        Ok(crashes as f64 / completed.len() as f64)
    }

    pub fn upload_pending(&self) -> Result<Option<PathBuf>> {
        let log = self.load()?;
        if !log.config.upload_enabled || !log.config.enabled {
            return Ok(None);
        }
        let upload_path = self.path.parent().unwrap().join("telemetry_upload.json");
        fs::write(&upload_path, serde_json::to_string_pretty(&log)?)?;
        Ok(Some(upload_path))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn crash_rate_calculation() {
        let tmp = TempDir::new().unwrap();
        let mut svc = TelemetryService::new(tmp.path());
        svc.set_enabled(true).unwrap();

        svc.start_session(None).unwrap();
        svc.end_session(false).unwrap();
        svc.start_session(None).unwrap();
        svc.end_session(true).unwrap();

        let rate = svc.crash_rate().unwrap();
        assert!((rate - 0.5).abs() < f64::EPSILON);
    }
}
