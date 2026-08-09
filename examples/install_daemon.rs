//! Describes a small example daemon ("exampled") and installs/uninstalls it.
//!
//! This actually modifies the running system — it creates a user and group,
//! writes files under `/etc`, `/var/lib`, `/var/log`, and
//! `/etc/systemd/system`, and registers a systemd service — so it must be
//! run as root against a real systemd. The easiest way is the crate's
//! Docker test harness (`./docker/run-tests.sh`, or see the README's
//! "Running the tests and examples" section), which runs this in a
//! disposable container instead of on your host. To run it directly:
//!
//!     cargo build --example install_daemon
//!     sudo ./target/debug/examples/install_daemon install
//!     sudo ./target/debug/examples/install_daemon start
//!     sudo ./target/debug/examples/install_daemon stop
//!     sudo ./target/debug/examples/install_daemon uninstall

use std::{fs::Permissions, os::unix::fs::PermissionsExt as _};

use clap::{Parser, ValueEnum};
use daemon::{
    Binary, Bootstrap, Config, Daemon, DaemonPath, Directory, Group, RelativePath, Service, User,
};
use env_logger::{Builder, Target};

const SERVICE_UNIT: &str = "\
[Unit]
Description=Example daemon

[Service]
Type=simple
User=exampled
Group=exampled
ExecStart=/usr/local/bin/exampled
Restart=on-failure

[Install]
WantedBy=multi-user.target
";

const BOOTSTRAP_CONFIG: &str = "\
# exampled default configuration
port = 9000
";

#[derive(Parser)]
struct Args {
    #[arg(short, long, value_enum)]
    option: DaemonOption,
}

#[derive(Clone, Debug, ValueEnum)]
enum DaemonOption {
    Install,
    Uninstall,
    Start,
    Stop,
    Restart,
}

fn build_daemon() -> daemon::Result<Daemon> {
    Ok(Daemon {
        groups: vec![Group::from("exampled")],
        users: vec![User::new("exampled", [Group::from("exampled")])],
        directories: vec![
            // -> /var/lib/exampled
            Directory::new(
                DaemonPath::State(RelativePath::new("exampled")?),
                Permissions::from_mode(0o750),
                "exampled",
                "exampled",
            ),
            // -> /var/log/exampled
            Directory::new(
                DaemonPath::Logs(RelativePath::new("exampled")?),
                Permissions::from_mode(0o750),
                "exampled",
                "exampled",
            ),
        ],
        // -> /etc/exampled/config.toml (written once, never overwritten)
        bootstrap: Some(Bootstrap::new(
            DaemonPath::Configuration(RelativePath::new("exampled")?),
            [Config::new("config.toml", BOOTSTRAP_CONFIG)?],
        )),
        binary: Binary::new(
            "exampled",
            "./target/release/examples/exampled",
            "/usr/local/bin/exampled",
        ),
        service: Service::new("exampled", SERVICE_UNIT)?,
        help: "exampled installed. Manage it with: systemctl {start,stop,status} exampled"
            .to_string(),
    })
}

fn main() -> anyhow::Result<()> {
    Builder::from_default_env().target(Target::Stdout).init();

    let daemon = build_daemon()?;

    let Args { option } = Args::parse();

    match option {
        DaemonOption::Install => daemon.install(),
        DaemonOption::Uninstall => daemon.uninstall(),
        DaemonOption::Start => daemon.start(),
        DaemonOption::Stop => daemon.stop(),
        DaemonOption::Restart => daemon.restart(),
    }
    .inspect_err(|err| log::error!("got err: '{err}'"))?;

    Ok(())
}
