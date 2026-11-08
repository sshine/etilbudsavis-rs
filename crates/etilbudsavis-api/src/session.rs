//! Token persistence in the XDG data directory.

use std::fs;
use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};
use crate::types::Secret;

/// The token and the account metadata stored beside it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StoredSession {
    /// The bearer token sent as `x-token`.
    pub token: Secret,
    /// The email the account was created with, when known.
    #[serde(default)]
    pub email: Option<String>,
    /// When the file was written.
    #[serde(default = "now")]
    pub created_at: DateTime<Utc>,
}

fn now() -> DateTime<Utc> {
    Utc::now()
}

/// A token file under `~/.local/share/etilbudsavis/token.json`.
#[derive(Debug, Clone)]
pub struct Session {
    path: PathBuf,
}

impl Session {
    /// The default token path, or `./etilbudsavis-token.json` when the XDG
    /// directories cannot be resolved.
    pub fn default_path() -> PathBuf {
        ProjectDirs::from("dk", "sshine", "etilbudsavis")
            .map(|dirs| dirs.data_dir().join("token.json"))
            .unwrap_or_else(|| PathBuf::from("etilbudsavis-token.json"))
    }

    /// Uses the default path.
    pub fn discover() -> Self {
        Self::at(Self::default_path())
    }

    /// Uses a specific file.
    pub fn at(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    /// The file this session reads and writes.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Loads the token, if the file exists.
    pub fn load(&self) -> Result<Option<StoredSession>> {
        match fs::read_to_string(&self.path) {
            Ok(text) => Ok(Some(serde_json::from_str(&text)?)),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(error) => Err(Error::Io(error)),
        }
    }

    /// Writes the token, creating the parent directory.
    pub fn save(&self, session: &StoredSession) -> Result<()> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)?;
        }
        let text = serde_json::to_string_pretty(session)?;
        fs::write(&self.path, text)?;
        restrict_permissions(&self.path)?;
        Ok(())
    }

    /// Removes the token file, if present.
    pub fn clear(&self) -> Result<()> {
        match fs::remove_file(&self.path) {
            Ok(()) => Ok(()),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(error) => Err(Error::Io(error)),
        }
    }
}

#[cfg(unix)]
fn restrict_permissions(path: &Path) -> Result<()> {
    use std::os::unix::fs::PermissionsExt as _;

    let mut permissions = fs::metadata(path)?.permissions();
    permissions.set_mode(0o600);
    fs::set_permissions(path, permissions)?;
    Ok(())
}

#[cfg(not(unix))]
fn restrict_permissions(_path: &Path) -> Result<()> {
    Ok(())
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used)]

    use super::*;

    #[test]
    fn a_missing_file_is_an_empty_session() {
        let directory = tempfile::tempdir().expect("tempdir");
        let session = Session::at(directory.path().join("token.json"));
        assert!(session.load().expect("load").is_none());
    }

    #[test]
    fn a_saved_session_round_trips() {
        let directory = tempfile::tempdir().expect("tempdir");
        let session = Session::at(directory.path().join("nested/token.json"));
        let stored = StoredSession {
            token: Secret::new("secret"),
            email: Some("person@example.com".to_owned()),
            created_at: Utc::now(),
        };
        session.save(&stored).expect("save");
        assert_eq!(session.load().expect("load"), Some(stored));
        session.clear().expect("clear");
        assert!(session.load().expect("load").is_none());
    }
}
