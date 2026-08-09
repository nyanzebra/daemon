//! Same "exampled" daemon as `install_daemon.rs`, but described in a TOML
//! file (`examples/exampled.toml`) and loaded via `Deserialize` instead of
//! being built directly in Rust.
//!
//! Requires root and a real systemd — run it via the crate's Docker test
//! harness (`./docker/run-tests.sh`), or directly:
//!
//!     cargo build --example install_daemon_from_toml
//!     sudo ./target/debug/examples/install_daemon_from_toml examples/exampled.toml install
//!     sudo ./target/debug/examples/install_daemon_from_toml examples/exampled.toml start
//!     sudo ./target/debug/examples/install_daemon_from_toml examples/exampled.toml stop
//!     sudo ./target/debug/examples/install_daemon_from_toml examples/exampled.toml uninstall

use clap::{Parser, ValueEnum};
use daemon::Daemon;
use env_logger::{Builder, Target};

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

fn main() -> anyhow::Result<()> {
    Builder::from_default_env().target(Target::Stdout).init();

    let Args { option } = Args::parse();

    let contents = include_str!("exampled.toml");
    let daemon: Daemon = toml::from_str(&contents)?;

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
