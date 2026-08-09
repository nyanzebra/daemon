//! A directory a daemon needs, anchored under one of the standard FHS
//! locations rather than an arbitrary path — see [`DaemonPath`] for why.

use std::{
    fs::{Permissions, create_dir_all, remove_dir_all},
    os::unix::fs::PermissionsExt as _,
    process::Command,
};

use serde::{Deserialize, Serialize};

use crate::{Error, Result, path::DaemonPath};

/// A single directory a daemon needs, resolved to `<kind's root>/<name>`
/// (e.g. `DaemonPath::State` + `"myapp"` → `/var/lib/myapp`).
#[derive(Serialize, Deserialize)]
#[cfg_attr(test, derive(schemars::JsonSchema))]
pub struct Directory {
    /// Which FHS root this directory is created under.
    path: DaemonPath,
    /// Octal permission bits (e.g. `0o750`), applied via `chmod`.
    permissions: u32,
    /// Group to `chown` the directory to.
    group: String,
    /// User to `chown` the directory to.
    user: String,
}

impl Directory {
    /// Builds a new [`Directory`].
    pub fn new(
        path: DaemonPath,
        permissions: Permissions,
        group: impl ToString,
        user: impl ToString,
    ) -> Self {
        Self {
            path,
            permissions: permissions.mode(),
            group: group.to_string(),
            user: user.to_string(),
        }
    }

    /// Creates the directory (recursively) and sets its permissions and
    /// ownership.
    pub(crate) fn create(&self) -> Result<()> {
        let path = self.path.path();

        log::debug!("creating directory {path:?}");

        create_dir_all(&path)?;

        // Set permissions
        let status = Command::new("chmod")
            .arg(format!("{:o}", self.permissions))
            .arg(&path)
            .status()?;

        if !status.success() {
            return Err(Error::Command {
                thing: format!("set permissions on {path:?}"),
                reason: status,
            });
        }

        // Set ownership
        let status = Command::new("chown")
            .arg(format!("{}:{}", self.user, self.group))
            .arg(&path)
            .status()?;

        if !status.success() {
            return Err(Error::Command {
                thing: format!("set ownership on {path:?}"),
                reason: status,
            });
        }

        Ok(())
    }

    /// Recursively removes the directory. Consumes `self` since a deleted
    /// directory shouldn't be reused.
    pub(crate) fn delete(self) -> Result<()> {
        let path = self.path.path();
        log::debug!("removing directory {path:?}");
        remove_dir_all(&path)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::RelativePath;

    use super::*;

    #[test]
    fn toml() {
        let dir = Directory {
            path: DaemonPath::Configuration(RelativePath::new("test").unwrap()),
            permissions: 0o777,
            group: "test".to_string(),
            user: "cat".to_string(),
        };
        println!("{}", toml::to_string(&dir).unwrap());
    }
}
