//! A system user for a daemon to run as: a locked-down (`useradd -M -r`, no
//! home directory, `/usr/sbin/nologin` shell) service account, created
//! idempotently.

use std::process::Command;

use serde::{Deserialize, Serialize};

use crate::{Error, Group, Result};

/// A system user, created as a system account with no home directory and a
/// `nologin` shell — daemons don't need interactive login.
#[derive(Serialize, Deserialize)]
#[cfg_attr(test, derive(schemars::JsonSchema))]
pub struct User {
    /// The username.
    pub(crate) name: String,
    /// Groups this user is added to as a member during [`Daemon::install`](crate::Daemon::install).
    pub(crate) belongs_to: Vec<Group>,
}

impl User {
    /// Builds a new [`User`].
    pub fn new(name: impl ToString, belongs_to: impl IntoIterator<Item = Group>) -> Self {
        Self {
            name: name.to_string(),
            belongs_to: belongs_to.into_iter().collect(),
        }
    }

    /// Creates the user with `useradd -M -r -s /usr/sbin/nologin`, unless
    /// `getent passwd <name>` shows it already exists — safe to call on
    /// every install, not just the first.
    pub(crate) fn create(&self) -> Result<()> {
        log::debug!("creating user {}", self.name);
        let output = Command::new("getent")
            .args(["passwd", self.name.as_str()])
            .output();

        if output.is_err() || !output.unwrap().status.success() {
            let status = Command::new("useradd")
                .arg("-M")
                .arg("-N")
                .arg("-r")
                .arg("-s")
                .arg("/usr/sbin/nologin")
                .arg(self.name.as_str())
                .status()?;

            if !status.success() {
                return Err(Error::Command {
                    thing: format!("create user '{}'", self.name),
                    reason: status,
                });
            }
        }

        Ok(())
    }

    /// Deletes the user with `userdel`. Consumes `self` since a deleted
    /// user shouldn't be reused.
    pub(crate) fn delete(self) -> Result<()> {
        log::debug!("deleting user {}", self.name);
        let status = Command::new("userdel").arg(self.name.as_str()).status()?;

        if !status.success() {
            return Err(Error::Command {
                thing: format!("delete user '{}'", self.name),
                reason: status,
            });
        }

        Ok(())
    }
}
