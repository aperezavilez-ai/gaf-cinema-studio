//! Stripe billing stub — Pro tier optional (Phase 6).

use std::fs;
use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::error::{CinemaError, Result};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum SubscriptionTier {
    Free,
    Pro,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SubscriptionState {
    pub tier: SubscriptionTier,
    pub stripe_customer_id: Option<String>,
    pub stripe_subscription_id: Option<String>,
    pub expires_at: Option<DateTime<Utc>>,
    pub updated_at: DateTime<Utc>,
}

impl Default for SubscriptionState {
    fn default() -> Self {
        Self {
            tier: SubscriptionTier::Free,
            stripe_customer_id: None,
            stripe_subscription_id: None,
            expires_at: None,
            updated_at: Utc::now(),
        }
    }
}

pub struct BillingStore {
    path: PathBuf,
}

impl BillingStore {
    pub fn new(cloud_root: impl AsRef<Path>) -> Self {
        Self {
            path: cloud_root.as_ref().join("subscription.json"),
        }
    }

    pub fn load(&self) -> Result<SubscriptionState> {
        if !self.path.exists() {
            return Ok(SubscriptionState::default());
        }
        let data = fs::read_to_string(&self.path)?;
        Ok(serde_json::from_str(&data)?)
    }

    pub fn save(&self, state: &SubscriptionState) -> Result<()> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&self.path, serde_json::to_string_pretty(state)?)?;
        Ok(())
    }

    pub fn is_pro(&self) -> Result<bool> {
        Ok(self.load()?.tier == SubscriptionTier::Pro)
    }
}

/// Phase 6 stub — simulates Stripe checkout success. Production uses Stripe SDK + webhooks.
pub fn activate_pro_stub(store: &BillingStore) -> Result<SubscriptionState> {
    let state = SubscriptionState {
        tier: SubscriptionTier::Pro,
        stripe_customer_id: Some("cus_stub_phase6".into()),
        stripe_subscription_id: Some("sub_stub_phase6".into()),
        expires_at: None,
        updated_at: Utc::now(),
    };
    store.save(&state)?;
    Ok(state)
}

pub fn cancel_subscription(store: &BillingStore) -> Result<SubscriptionState> {
    let mut state = store.load()?;
    state.tier = SubscriptionTier::Free;
    state.stripe_subscription_id = None;
    state.updated_at = Utc::now();
    store.save(&state)?;
    Ok(state)
}

pub fn pro_features_enabled(state: &SubscriptionState) -> bool {
    state.tier == SubscriptionTier::Pro
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn free_by_default_pro_upgrade() {
        let tmp = TempDir::new().unwrap();
        let store = BillingStore::new(tmp.path());
        assert!(!store.is_pro().unwrap());
        activate_pro_stub(&store).unwrap();
        assert!(store.is_pro().unwrap());
    }
}
