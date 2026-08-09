//! Install and uninstall a Linux daemon: the system user/group it runs as,
//! its data/log/config directories, its binary, and its systemd unit — all
//! from one declarative [`Daemon`] description.
//!
//! # Overview
//!
//! Build a [`Daemon`] from its component parts ([`User`], [`Group`],
//! [`Directory`], [`Bootstrap`], [`Binary`], [`Service`]) and call
//! [`Daemon::install`] (requires root):
//!
//! ```no_run
//! use std::{
//!    fs::Permissions,
//!    os::unix::fs::PermissionsExt as _,
//! };
//! use daemon::{Binary, Daemon, Directory, DaemonPath, Group, RelativePath, Service, User};
//!
//! let daemon = Daemon {
//!     groups: vec![Group::from("myapp")],
//!     users: vec![User::new(
//!         "myapp",
//!         [Group::from("myapp")],
//!     )],
//!     directories: vec![Directory::new(
//!         DaemonPath::State(RelativePath::new("myapp")?),
//!         Permissions::from_mode(0o750),
//!         "myapp".to_string(),
//!         "myapp".to_string(),
//!     )],
//!     bootstrap: None,
//!     binary: Binary::new(
//!         "myapp",
//!         "./target/release/myapp",
//!         "/usr/local/bin/myapp",
//!     ),
//!     service: Service::new(
//!         "myapp",
//!         "[Unit]\nDescription=myapp\n",
//!     )?,
//!     help: "myapp installed.".to_string(),
//! };
//!
//! daemon.install()?;
//! # Ok::<(), daemon::Error>(())
//! ```
//!
//! [`Daemon::uninstall`] reverses all of it in a safe order, and every
//! mutating operation requires root — see [`Daemon`] for the full
//! lifecycle and [`DaemonPath`] for why directories are FHS-anchored
//! rather than taking arbitrary paths.
//!
//! See `examples/install_daemon.rs` and `examples/install_daemon_from_toml.rs`
//! for complete worked examples, and the crate README for prerequisites and
//! how to run the test suite and examples inside Docker (both create real
//! system users/groups and write real files, so they're meant to run in a
//! disposable container, not on your host).

mod error;
pub use error::{Error, Result};

mod installer;
pub use installer::{Binary, Bootstrap, Config, Daemon, Directory, Group, Service, User};

mod path;
pub use path::{DaemonPath, RelativePath};

#[cfg(test)]
mod tests {
    use schemars::schema_for;

    use super::*;

    #[test]
    fn schema() {
        let schema = schema_for!(Daemon);
        println!("{}", serde_json::to_string_pretty(&schema).unwrap());
    }
}
