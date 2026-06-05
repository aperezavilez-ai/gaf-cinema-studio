//! Shared engine singleton for UniFFI + C ABI.

use std::path::PathBuf;
use std::sync::Mutex;

use crate::ProjectStateManager;

pub(crate) static ENGINE: Mutex<Option<ProjectStateManager>> = Mutex::new(None);

pub(crate) fn with_manager<F, T>(f: F) -> Result<T, String>
where
    F: FnOnce(&mut ProjectStateManager) -> crate::Result<T>,
{
    let mut guard = ENGINE.lock().map_err(|e| format!("lock error: {e}"))?;
    if guard.is_none() {
        *guard = Some(ProjectStateManager::new());
    }
    f(guard.as_mut().unwrap()).map_err(|e| e.to_string())
}

pub(crate) fn init_engine(data_root: Option<String>) {
    let mut guard = ENGINE.lock().unwrap();
    *guard = Some(match data_root {
        Some(root) => ProjectStateManager::with_data_root(PathBuf::from(root)),
        None => ProjectStateManager::new(),
    });
}
