use std::fs;
use std::io::Write;
use std::path::Path;

use crate::error::{CinemaError, Result};

/// Write atomically: temp file + rename. Prevents half-written project files on crash.
pub fn atomic_write(path: &Path, contents: &[u8]) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let tmp_path = path.with_extension("tmp");
    {
        let mut file = fs::File::create(&tmp_path)?;
        file.write_all(contents)?;
        file.sync_all()?;
    }

    // Windows: remove target first if exists (rename fails otherwise)
    if path.exists() {
        fs::remove_file(path)?;
    }
    fs::rename(&tmp_path, path).map_err(|e| {
        let _ = fs::remove_file(&tmp_path);
        CinemaError::Io(e)
    })?;

    Ok(())
}

/// Simulate a crash mid-write (for recovery tests only).
#[cfg(test)]
pub fn corrupt_write(path: &Path, contents: &[u8]) -> Result<()> {
    fs::write(path, contents)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn atomic_write_creates_valid_file() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("test.json");
        atomic_write(&path, br#"{"ok":true}"#).unwrap();
        let read = fs::read_to_string(&path).unwrap();
        assert_eq!(read, r#"{"ok":true}"#);
    }
}
