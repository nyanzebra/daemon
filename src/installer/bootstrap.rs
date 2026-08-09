//! A seed config file written once on install and never overwritten, mirrored
//! on the way out: [`Bootstrap::uninstall`] only removes a file that this
//! same install actually created.

use std::fs::{create_dir_all, remove_dir_all, write};

use serde::{Deserialize, Serialize};

use crate::{Result, path::DaemonPath};

use super::{valid_filename, valid_filename_de};

#[derive(Serialize, Deserialize)]
#[cfg_attr(test, derive(schemars::JsonSchema))]
pub struct Config {
    #[serde(deserialize_with = "valid_filename_de")]
    filename: String,
    content: String,
}

impl Config {
    pub fn new(filename: impl ToString, content: impl ToString) -> Result<Self> {
        let filename = valid_filename(filename.to_string())?;
        let content = content.to_string();

        Ok(Self { filename, content })
    }
}

/// A single seed file, written to `<kind's root>/<dir_name>/<file_name>` and
/// left alone on every subsequent install (or uninstall) if it already existed
/// — this crate never overwrites or deletes config a daemon or its administrator
/// may have since edited.
#[derive(Serialize, Deserialize)]
#[cfg_attr(test, derive(schemars::JsonSchema))]
pub struct Bootstrap {
    /// Which FHS root the file is created under.
    path: DaemonPath,
    /// Configs to be written the first time the file is created. Ignored if the
    /// file already exists.
    configs: Vec<Config>,
}

impl Bootstrap {
    /// Builds a new [`Bootstrap`].
    pub fn new(path: DaemonPath, configs: impl IntoIterator<Item = Config>) -> Self {
        Self {
            path,
            configs: configs.into_iter().collect(),
        }
    }

    /// Writes [`Bootstrap::configs`] to the resolved path.
    pub(crate) fn install(&self) -> Result<()> {
        let path = self.path.path();

        create_dir_all(&path)?;

        log::debug!("attempting to install bootstrap to {path:?}");

        for config in &self.configs {
            write(path.join(&config.filename), &config.content)?;
        }

        Ok(())
    }

    /// Removes the path of [`Bootstrap::path`].
    pub(crate) fn uninstall(self) -> Result<()> {
        let path = self.path.path();

        log::debug!("attempting to uninstall bootstrap from {path:?}");

        remove_dir_all(&path)?;

        Ok(())
    }
}
