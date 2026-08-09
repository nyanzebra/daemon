//! The daemon's systemd unit, and its `systemctl` lifecycle
//! (start/stop/restart/disable/reload).

use std::{
    fs::{Permissions, remove_file, set_permissions, write},
    os::unix::fs::PermissionsExt as _,
    process::Command,
};

use serde::{Deserialize, Serialize};

use crate::{Error, Result, path::DaemonPath};

use super::{valid_filename, valid_filename_de};

/// A systemd unit, written to `/etc/systemd/system/<name>.service`.
#[derive(Serialize, Deserialize)]
#[cfg_attr(test, derive(schemars::JsonSchema))]
pub struct Service {
    /// The unit name (without `.service`) — also used directly as the
    /// `systemctl` target for start/stop/restart/disable.
    #[serde(deserialize_with = "valid_filename_de")]
    name: String,
    /// The unit file's contents.
    content: String,
}

impl Service {
    /// Builds a new [`Service`].
    pub fn new(name: impl ToString, content: impl ToString) -> Result<Self> {
        Ok(Self {
            name: valid_filename(name.to_string())?,
            content: content.to_string(),
        })
    }

    /// Writes the unit file to `/etc/systemd/system/<name>.service` with
    /// `0o644` permissions.
    pub(crate) fn install(&self) -> Result<()> {
        let path = DaemonPath::systemd(&self.name)?;
        write(&path, &self.content)?;
        set_permissions(&path, Permissions::from_mode(0o644))?;
        Ok(())
    }

    /// Removes the unit file.
    pub(crate) fn uninstall(&self) -> Result<()> {
        let path = DaemonPath::systemd(&self.name)?;
        remove_file(&path)?;
        Ok(())
    }

    /// Runs `systemctl daemon-reload`.
    pub(crate) fn reload(&self) -> Result<()> {
        let status = Command::new("systemctl").arg("daemon-reload").status()?;

        if !status.success() {
            return Err(Error::Command {
                thing: "reload systemd".to_string(),
                reason: status,
            });
        }

        Ok(())
    }

    /// Runs `systemctl start <name>`.
    pub(crate) fn start(&self) -> Result<()> {
        let status = Command::new("systemctl")
            .arg("start")
            .arg(&self.name)
            .status()?;

        if !status.success() {
            return Err(Error::Command {
                thing: format!("start service '{}'", self.name),
                reason: status,
            });
        }

        Ok(())
    }

    /// Runs `systemctl stop <name>`.
    pub(crate) fn stop(&self) -> Result<()> {
        let status = Command::new("systemctl")
            .arg("stop")
            .arg(&self.name)
            .status()?;

        if !status.success() {
            return Err(Error::Command {
                thing: format!("stop service '{}'", self.name),
                reason: status,
            });
        }

        Ok(())
    }

    /// Runs `systemctl restart <name>`.
    pub(crate) fn restart(&self) -> Result<()> {
        let status = Command::new("systemctl")
            .arg("restart")
            .arg(&self.name)
            .status()?;

        if !status.success() {
            return Err(Error::Command {
                thing: format!("restart service '{}'", self.name),
                reason: status,
            });
        }

        Ok(())
    }

    /// Runs `systemctl disable <name>`.
    pub(crate) fn disable(&self) -> Result<()> {
        let status = Command::new("systemctl")
            .arg("disable")
            .arg(&self.name)
            .status()?;

        if !status.success() {
            return Err(Error::Command {
                thing: format!("disable service '{}'", self.name),
                reason: status,
            });
        }

        Ok(())
    }
}
