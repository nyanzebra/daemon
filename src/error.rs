//! This crate's error type and the [`Result`] alias built on it.

use std::{path::PathBuf, process::ExitStatus};

/// Everything that can go wrong installing or uninstalling a [`Daemon`](crate::Daemon).
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// A mutating operation was attempted without root (effective UID 0).
    #[error("Not root")]
    NotRoot,

    /// A filesystem or process-spawn operation failed.
    #[error("io: {0}")]
    Io(#[from] std::io::Error),

    /// A path must not contain '..' or absolute paths.
    #[error("invalid path '{0}': must not contain '..' or absolute paths")]
    InvalidPath(PathBuf),

    /// A file name must not contain '..' or absolute paths.
    #[error("invalid file name '{0}': must not contain '..' or absolute paths")]
    InvalidFile(String),

    /// An external command (`useradd`, `systemctl`, `chmod`, etc.) exited
    /// with a non-zero status.
    #[error("Command failed for {thing} due to {reason}")]
    Command {
        /// What the command was trying to do (for the error message).
        thing: String,
        /// The command's exit status.
        reason: ExitStatus,
    },
}

/// This crate's `Result` alias, using [`Error`].
pub type Result<T> = std::result::Result<T, Error>;
