//! Beta program tracking — local registry for gate 6.3.

use std::fs;
use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::Result;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BetaCompletion {
    pub project_id: Uuid,
    pub project_name: String,
    pub user_label: String,
    pub completed_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct BetaRegistry {
    pub target: u32,
    pub completions: Vec<BetaCompletion>,
}

impl BetaRegistry {
    pub fn new() -> Self {
        Self {
            target: 10,
            completions: Vec::new(),
        }
    }

    pub fn count(&self) -> usize {
        self.completions.len()
    }

    pub fn gate_met(&self) -> bool {
        self.count() >= self.target as usize
    }
}

pub struct BetaTracker {
    path: PathBuf,
}

impl BetaTracker {
    pub fn new(data_root: impl AsRef<Path>) -> Self {
        Self {
            path: data_root
                .as_ref()
                .join("CinemaStudio")
                .join("beta_registry.json"),
        }
    }

    pub fn load(&self) -> Result<BetaRegistry> {
        if !self.path.exists() {
            return Ok(BetaRegistry::new());
        }
        Ok(serde_json::from_str(&fs::read_to_string(&self.path)?)?)
    }

    pub fn save(&self, registry: &BetaRegistry) -> Result<()> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&self.path, serde_json::to_string_pretty(registry)?)?;
        Ok(())
    }

    pub fn mark_complete(
        &self,
        project_id: Uuid,
        project_name: &str,
        user_label: &str,
    ) -> Result<BetaRegistry> {
        let mut reg = self.load()?;
        if reg.completions.iter().any(|c| c.project_id == project_id) {
            return Ok(reg);
        }
        reg.completions.push(BetaCompletion {
            project_id,
            project_name: project_name.to_string(),
            user_label: user_label.to_string(),
            completed_at: Utc::now(),
        });
        self.save(&reg)?;
        Ok(reg)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn tracks_completions() {
        let tmp = TempDir::new().unwrap();
        let tracker = BetaTracker::new(tmp.path());
        tracker
            .mark_complete(Uuid::new_v4(), "Film A", "beta_user_1")
            .unwrap();
        assert_eq!(tracker.load().unwrap().count(), 1);
    }
}
