// Copyright (C) 2019-2022 Aleo Systems Inc.
// This file is part of the snarkOS library.

// The snarkOS library is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// The snarkOS library is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with the snarkOS library. If not, see <https://www.gnu.org/licenses/>.

use colored::Colorize;
use self_update::{backends::github, version::bump_is_greater, Status};
use std::fmt::Write;

pub struct Updater;

impl Updater {
    const SNARKOS_BIN_NAME: &'static str = "snarkos";
    const SNARKOS_REPO_NAME: &'static str = "snarkOS";
    const SNARKOS_REPO_OWNER: &'static str = "AleoHQ";

    /// Show all available releases for `snarkos`.
    pub fn show_available_releases() -> Result<String, UpdaterError> {
        let releases = github::ReleaseList::configure()
            .repo_owner(Self::SNARKOS_REPO_OWNER)
            .repo_name(Self::SNARKOS_REPO_NAME)
            .build()?
            .fetch()?;

        let mut output = "List of available versions\n".to_string();
        for release in releases {
            let _ = writeln!(output, "  * {}", release.version);
        }
        Ok(output)
    }

    /// Update `snarkOS` to the specified release.
    pub fn update_to_release(show_output: bool, version: Option<String>) -> Result<Status, UpdaterError> {
        let mut update_builder = github::Update::configure();

        update_builder
            .repo_owner(Self::SNARKOS_REPO_OWNER)
            .repo_name(Self::SNARKOS_REPO_NAME)
            .bin_name(Self::SNARKOS_BIN_NAME)
            .current_version(env!("CARGO_PKG_VERSION"))
            .show_download_progress(show_output)
            .no_confirm(true)
            .show_output(show_output);

        let status = match version {
            None => update_builder.build()?.update()?,
            Some(v) => update_builder.target_version_tag(&v).build()?.update()?,
        };

        Ok(status)
    }

    /// Check if there is an available update for `aleo` and return the newest release.
    pub fn update_available() -> Result<String, UpdaterError> {
        let updater = github::Update::configure()
            .repo_owner(Self::SNARKOS_REPO_OWNER)
            .repo_name(Self::SNARKOS_REPO_NAME)
            .bin_name(Self::SNARKOS_BIN_NAME)
            .current_version(env!("CARGO_PKG_VERSION"))
            .build()?;

        let current_version = updater.current_version();
        let latest_release = updater.get_latest_release()?;

        if bump_is_greater(&current_version, &latest_release.version)? {
            Ok(latest_release.version)
        } else {
            Err(UpdaterError::OldReleaseVersion(current_version, latest_release.version))
        }
    }

    /// Display the CLI message.
    pub fn print_cli() -> String {
        if let Ok(latest_version) = Self::update_available() {
            let mut output = "🟢 A new version is available! Run".bold().green().to_string();
            output += &" `aleo update` ".bold().white();
            output += &format!("to update to v{}.", latest_version).bold().green();
            output
        } else {
            String::new()
        }
    }
}

#[derive(Debug, Error)]
pub enum UpdaterError {
    #[error("{}: {}", _0, _1)]
    Crate(&'static str, String),

    #[error("The current version {} is more recent than the release version {}", _0, _1)]
    OldReleaseVersion(String, String),
}

impl From<self_update::errors::Error> for UpdaterError {
    fn from(error: self_update::errors::Error) -> Self {
        UpdaterError::Crate("self_update", error.to_string())
    }
}
