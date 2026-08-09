//! The daemon's own executable: copied into place on install, removed on
//! uninstall.

use std::{
    fs::{Permissions, copy, remove_file, set_permissions},
    os::unix::fs::PermissionsExt as _,
    path::PathBuf,
};

use serde::{Deserialize, Serialize};

use crate::Result;

/// The daemon's executable, copied from `src` to `dest` and made
/// world-executable (`0o755`).
#[derive(Serialize, Deserialize)]
#[cfg_attr(test, derive(schemars::JsonSchema))]
pub struct Binary {
    /// Identifying name for this binary. Not currently used to resolve any
    /// path — `src`/`dst` are used directly.
    name: String,
    /// Path to copy the binary from (e.g. a local build output).
    src: PathBuf,
    /// Path to copy the binary to (e.g. `/usr/local/bin/mydaemon`).
    dst: PathBuf,
}

impl Binary {
    /// Builds a new [`Binary`].
    pub fn new(name: impl ToString, src: impl Into<PathBuf>, dst: impl Into<PathBuf>) -> Self {
        Self {
            name: name.to_string(),
            src: src.into(),
            dst: dst.into(),
        }
    }

    /// Copies `src` to `dst` and sets its permissions to `0o755`.
    pub(crate) fn install(&self) -> Result<()> {
        log::debug!(
            "installing {} from {:?} to {:?}",
            self.name,
            self.src,
            self.dst
        );
        copy(&self.src, &self.dst)?;
        set_permissions(&self.dst, Permissions::from_mode(0o755))?;

        Ok(())
    }

    /// Removes the binary at `dst`. Consumes `self` since a removed binary
    /// shouldn't be reused.
    pub(crate) fn uninstall(self) -> Result<()> {
        log::debug!("uninstalling {} from {:?}", self.name, self.dst);
        remove_file(&self.dst)?;
        Ok(())
    }
}
