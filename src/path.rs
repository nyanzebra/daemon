//! `DaemonPath` describes a directory under one of the standard FHS
//! locations, rather than an arbitrary path. There is no variant or
//! constructor that can resolve outside of `/etc`, `/var/lib`,
//! `/var/cache`, `/var/log`, or `/run`.
//!
//! The variants are public (so `DaemonPath` can still be matched on
//! directly), but their payload is [`RelativePath`], whose field is
//! private — the only way to obtain one at all is [`RelativePath::new`] or
//! `Deserialize`, both of which validate. There is no way to construct a
//! `DaemonPath` — through `serde` or by hand in Rust — that hasn't been
//! checked.

use std::{
    fmt::{self, Display},
    path::{Component, Path, PathBuf},
};

use serde::{Deserialize, Deserializer, Serialize};

use crate::{Error, Result};

/// A path that's been checked to be relative and free of `..` traversal.
/// The only way to get one is [`RelativePath::new`] (or `Deserialize`,
/// which calls it) — the inner `PathBuf` is private, so there's no way to
/// construct a `RelativePath` that skipped the check. Having one at all
/// *is* the proof it's valid.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, schemars::JsonSchema)]
pub struct RelativePath(PathBuf);

impl RelativePath {
    /// Validates `path`: rejects absolute paths and any `..` component.
    /// `PathBuf::join` won't resolve `..` on its own — an unvalidated
    /// `../../etc/passwd` would join onto any prefix and still escape it —
    /// so this is the one gate everything else in this module relies on.
    pub fn new(path: impl Into<PathBuf>) -> Result<Self> {
        let path = path.into();
        if path.is_absolute() || path.components().any(|c| matches!(c, Component::ParentDir)) {
            return Err(Error::InvalidPath(path));
        }
        Ok(Self(path))
    }

    pub fn as_path(&self) -> &Path {
        &self.0
    }
}

impl<'de> Deserialize<'de> for RelativePath {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let path = PathBuf::deserialize(deserializer)?;
        Self::new(path).map_err(serde::de::Error::custom)
    }
}

/// A directory anchored under one of the standard FHS roots.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, schemars::JsonSchema)]
pub enum DaemonPath {
    /// `/etc/<rest of path>` — configuration
    Configuration(RelativePath),
    /// `/var/lib/<rest of path>` — persistent state/data
    State(RelativePath),
    /// `/var/cache/<rest of path>` — cache
    Cache(RelativePath),
    /// `/var/log/<rest of path>` — logs
    Logs(RelativePath),
    /// `/run/<rest of path>` — runtime state (pid files, sockets)
    Runtime(RelativePath),
}

impl DaemonPath {
    /// Resolves to the real filesystem path this points at. Infallible —
    /// every `DaemonPath` that exists at all is already known-valid, since
    /// [`RelativePath`] can't be constructed without passing
    /// [`RelativePath::new`].
    pub(crate) fn path(&self) -> PathBuf {
        match self {
            DaemonPath::Configuration(path) => PathBuf::from("/etc").join(path.as_path()),
            DaemonPath::State(path) => PathBuf::from("/var/lib").join(path.as_path()),
            DaemonPath::Cache(path) => PathBuf::from("/var/cache").join(path.as_path()),
            DaemonPath::Logs(path) => PathBuf::from("/var/log").join(path.as_path()),
            DaemonPath::Runtime(path) => PathBuf::from("/run").join(path.as_path()),
        }
    }

    pub(crate) fn systemd(name: impl AsRef<Path>) -> Result<PathBuf> {
        let mut path = PathBuf::from("systemd/system/").join(name);
        path.add_extension("service");
        let path = Self::Configuration(RelativePath::new(path)?).path();
        Ok(path)
    }
}

impl Display for DaemonPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DaemonPath::Configuration(path) => write!(f, "/etc/{}", path.as_path().display()),
            DaemonPath::State(path) => write!(f, "/var/lib/{}", path.as_path().display()),
            DaemonPath::Cache(path) => write!(f, "/var/cache/{}", path.as_path().display()),
            DaemonPath::Logs(path) => write!(f, "/var/log/{}", path.as_path().display()),
            DaemonPath::Runtime(path) => write!(f, "/run/{}", path.as_path().display()),
        }
    }
}
