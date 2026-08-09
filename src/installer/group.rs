//! A system group, created idempotently (`create` is a no-op if the group
//! already exists) and used to manage a daemon's users' membership.

use std::process::Command;

use serde::{Deserialize, Serialize};

use crate::{Error, Result, User};

/// A system group, identified by name.
#[derive(Serialize, Deserialize)]
#[cfg_attr(test, derive(schemars::JsonSchema))]
pub struct Group(String);

impl From<String> for Group {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl From<&str> for Group {
    fn from(value: &str) -> Self {
        Self(value.to_owned())
    }
}

impl Group {
    /// Creates the group with `groupadd`, unless `getent group <name>`
    /// shows it already exists — safe to call on every install, not just
    /// the first.
    pub(crate) fn create(&self) -> Result<()> {
        let output = Command::new("getent")
            .args(["group", self.0.as_str()])
            .output();

        if output.is_err() || !output.unwrap().status.success() {
            log::debug!("creating group {}", self.0);
            let status = Command::new("groupadd").arg(self.0.as_str()).status()?;

            if !status.success() {
                return Err(Error::Command {
                    thing: format!("create group '{}'", self.0),
                    reason: status,
                });
            }
        }

        Ok(())
    }

    /// Adds `user` as a member of this group (`usermod -aG`).
    pub(crate) fn add_user(&self, user: &User) -> Result<()> {
        log::debug!("adding {} to group {}", user.name, self.0);
        let status = Command::new("usermod")
            .args(["-aG", self.0.as_str(), user.name.as_str()])
            .status()?;

        if !status.success() {
            return Err(Error::Command {
                thing: format!("add user '{}' to group '{}'", user.name, self.0),
                reason: status,
            });
        }

        Ok(())
    }

    /// Removes `user` from this group (`gpasswd -d`).
    pub(crate) fn remove_user(&self, user: &User) -> Result<()> {
        log::debug!("removing {} from group {}", user.name, self.0);
        let status = Command::new("gpasswd")
            .args(["-d", user.name.as_str(), self.0.as_str()])
            .status()?;

        if !status.success() {
            return Err(Error::Command {
                thing: format!("remove user '{}' from group '{}'", user.name, self.0),
                reason: status,
            });
        }

        Ok(())
    }

    /// Deletes the group with `groupdel`. Consumes `self` since a deleted
    /// group shouldn't be reused.
    pub(crate) fn delete(self) -> Result<()> {
        println!("deleting group {}", self.0);
        let status = Command::new("groupdel").arg(self.0.as_str()).status()?;

        if !status.success() {
            return Err(Error::Command {
                thing: format!("delete group '{}'", self.0),
                reason: status,
            });
        }

        Ok(())
    }
}
