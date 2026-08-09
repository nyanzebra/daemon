//! The individual resources a [`Daemon`] is composed of, and the
//! install/uninstall lifecycle that ties them together.
//!
//! Each submodule owns one kind of system resource and knows how to
//! create/delete just that resource:
//!
//! | Module | Resource |
//! |---|---|
//! | `user` | the system user the daemon runs as |
//! | `group` | the system group the daemon runs as |
//! | `directory` | data/state/log/cache/config directories, anchored under [`DaemonPath`]'s FHS roots |
//! | `bootstrap` | a seed config file, written once and never overwritten |
//! | `binary` | the daemon executable itself |
//! | `service` | the systemd unit |
//!
//! [`Daemon`] composes all of the above and drives them in a safe order —
//! see [`Daemon::install`] and [`Daemon::uninstall`].

mod binary;
pub use binary::Binary;

mod bootstrap;
pub use bootstrap::{Bootstrap, Config};

mod directory;
pub use directory::Directory;

mod group;
pub use group::Group;

mod service;
pub use service::Service;

mod user;
pub use user::User;

use serde::{Deserialize, Serialize};

use crate::{Error, Result};

/// A declarative description of a daemon: the system user/group it runs
/// as, the directories it needs, its binary, its systemd unit, and
/// (optionally) a seed config file — everything [`Daemon::install`] needs
/// to set the daemon up, and [`Daemon::uninstall`] needs to tear it back
/// down.
///
/// `Daemon` implements [`Serialize`]/[`Deserialize`], so it can be loaded
/// from a config file (TOML, JSON, or anything else `serde` supports)
/// instead of being built by hand — see `examples/install_daemon_from_toml.rs`.
#[derive(Serialize, Deserialize)]
#[cfg_attr(test, derive(schemars::JsonSchema))]
pub struct Daemon {
    /// Users to create. Each user's `belongs_to` groups are also added as
    /// members during install.
    pub users: Vec<User>,
    /// Groups to create.
    pub groups: Vec<Group>,

    /// Directories to create, each anchored under a [`DaemonPath`] root.
    pub directories: Vec<Directory>,

    /// A seed config file, written once on first install and never
    /// overwritten or removed if it already existed beforehand.
    pub bootstrap: Option<Bootstrap>,
    /// The systemd unit to install.
    pub service: Service,
    /// The daemon binary to copy into place.
    pub binary: Binary,
    /// Printed after a successful install (e.g. next-step instructions for
    /// the person running it).
    pub help: String,
}

impl Daemon {
    /// Installs the daemon: creates groups, then users (adding them to
    /// their groups), then directories, then copies the binary in, installs
    /// the systemd unit, writes the bootstrap file if one is configured and
    /// doesn't already exist, and finally reloads systemd.
    ///
    /// Requires root — returns [`Error::NotRoot`] otherwise.
    pub fn install(&self) -> Result<()> {
        log::info!("installing daemon...");

        // Check if running as root
        if !is_root() {
            log::warn!("not root, giving up...");
            return Err(Error::NotRoot);
        }

        log::info!("creating groups...");

        // Create groups
        for group in &self.groups {
            group.create()?;
        }

        log::info!("creating users...");

        // Create users
        for user in &self.users {
            user.create()?;
            for group in &user.belongs_to {
                group.add_user(user)?;
            }
        }

        log::info!("creating directories...");

        // Create directories
        for dir in &self.directories {
            dir.create()?;
        }

        log::info!("installing binary...");

        // Install binary
        self.binary.install()?;

        log::info!("installing service...");

        // Install systemd service
        self.service.install()?;

        // Create any necessary bootstrap files
        if let Some(bootstrap) = self.bootstrap.as_ref() {
            log::info!("installing bootstrap configs...");

            bootstrap.install()?
        };

        log::info!("reloading systemd...");

        // Reload service
        self.service.reload()?;

        log::info!("installed - {}", self.help);

        Ok(())
    }

    /// Uninstalls the daemon: stops and disables the service, removes the
    /// unit file, removes the bootstrap file (unless it predated this
    /// daemon, in which case it's left alone), removes the binary, reloads
    /// systemd, then removes directories, users, and groups.
    ///
    /// Requires root — returns [`Error::NotRoot`] otherwise. Consumes
    /// `self` since none of these resources should be reused after removal.
    pub fn uninstall(mut self) -> Result<()> {
        log::info!("uninstalling daemon...");

        if !is_root() {
            log::warn!("not root, giving up...");

            return Err(Error::NotRoot);
        }

        log::info!("uninstalling service...");

        // Remove service file
        self.service.stop()?;
        self.service.disable()?;
        self.service.uninstall()?;

        // Remove bootstrapping files
        if let Some(bootstrap) = self.bootstrap.take() {
            log::info!("uninstalling bootstrap configs...");
            bootstrap.uninstall()?
        };

        log::info!("uninstalling binary...");

        // Remove binary
        self.binary.uninstall()?;

        log::info!("reloading systemd...");

        // Reload systemd
        self.service.reload()?;

        log::info!("deleting directories...");

        // Remove directories
        for directory in self.directories {
            directory.delete()?;
        }

        log::info!("deleting users...");

        // Remove users
        for user in self.users {
            for group in &user.belongs_to {
                group.remove_user(&user)?;
            }
            user.delete()?;
        }

        log::info!("deleting groups...");

        // Remove groups
        for group in self.groups {
            group.delete()?;
        }

        Ok(())
    }

    /// Restarts the daemon's systemd service (`systemctl restart`).
    pub fn restart(&self) -> Result<()> {
        log::info!("restarting service...");

        self.service.restart()
    }

    /// Starts the daemon's systemd service (`systemctl start`).
    pub fn start(&self) -> Result<()> {
        log::info!("starting service...");

        self.service.start()
    }

    /// Stops the daemon's systemd service (`systemctl stop`).
    pub fn stop(&self) -> Result<()> {
        log::info!("stopping service...");

        self.service.stop()
    }
}

/// Returns `true` if the current process is running as root (effective
/// UID 0).
fn is_root() -> bool {
    unsafe { libc::geteuid() == 0 }
}

/// Validates a filename for use in [`Bootstrap`] configs.
fn valid_filename_de<'de, D>(deserializer: D) -> std::result::Result<String, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    valid_filename(s).map_err(serde::de::Error::custom)
}

/// Validates a filename for use in [`Bootstrap`] configs.
fn valid_filename(s: String) -> Result<String> {
    if s.is_empty() || s.contains("/") || s.contains("\\") || s.contains("..") {
        return Err(Error::InvalidFile(s));
    }
    Ok(s)
}
