//! Optional auth — local-first, account never required for core (Rule #9).

use std::fs;
use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::{CinemaError, Result};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AuthSession {
    pub logged_in: bool,
    pub user_id: Option<Uuid>,
    pub email: Option<String>,
    #[serde(default)]
    token_hint: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Default for AuthSession {
    fn default() -> Self {
        let now = Utc::now();
        Self {
            logged_in: false,
            user_id: None,
            email: None,
            token_hint: None,
            created_at: now,
            updated_at: now,
        }
    }
}

impl AuthSession {
    pub fn guest() -> Self {
        Self::default()
    }

    pub fn from_login(email: impl Into<String>, token: impl Into<String>) -> Self {
        let now = Utc::now();
        let token = token.into();
        let hint = token.chars().take(8).collect::<String>() + "…";
        Self {
            logged_in: true,
            user_id: Some(Uuid::new_v4()),
            email: Some(email.into()),
            token_hint: Some(hint),
            created_at: now,
            updated_at: now,
        }
    }
}

pub struct AuthStore {
    path: PathBuf,
}

impl AuthStore {
    pub fn new(cloud_root: impl AsRef<Path>) -> Self {
        Self {
            path: cloud_root.as_ref().join("auth.json"),
        }
    }

    pub fn load(&self) -> Result<AuthSession> {
        if !self.path.exists() {
            return Ok(AuthSession::guest());
        }
        let data = fs::read_to_string(&self.path)?;
        Ok(serde_json::from_str(&data)?)
    }

    pub fn save(&self, session: &AuthSession) -> Result<()> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&self.path, serde_json::to_string_pretty(session)?)?;
        Ok(())
    }

    pub fn clear(&self) -> Result<()> {
        if self.path.exists() {
            fs::remove_file(&self.path)?;
        }
        Ok(())
    }
}

/// Phase 6 stub: validates email format only. Real OAuth/Stripe in production shell.
pub fn login_stub(email: &str, _password: &str) -> Result<AuthSession> {
    if !email.contains('@') {
        return Err(CinemaError::Validation("invalid email".into()));
    }
    Ok(AuthSession::from_login(email, "stub_token_phase6"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn guest_by_default() {
        let tmp = TempDir::new().unwrap();
        let store = AuthStore::new(tmp.path());
        assert!(!store.load().unwrap().logged_in);
    }

    #[test]
    fn login_persists() {
        let tmp = TempDir::new().unwrap();
        let store = AuthStore::new(tmp.path());
        let session = login_stub("user@cinemastudio.dev", "pass").unwrap();
        store.save(&session).unwrap();
        assert!(store.load().unwrap().logged_in);
    }
}
