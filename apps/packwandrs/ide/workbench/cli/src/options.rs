/*---------------------------------------------------------------------------------------------
 *  Copyright (c) Microsoft Corporation. All rights reserved.
 *  Licensed under the MIT License. See License.txt in the project root for license information.
 *--------------------------------------------------------------------------------------------*/

use std::fmt;

use serde::{Deserialize, Serialize};

use crate::{constants::VSCODE_CLI_BINARY_NAME, util::errors::CodeError};

#[derive(clap::ValueEnum, Copy, Clone, Debug, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub enum Quality {
	#[serde(rename = "stable")]
	Stable,
	#[serde(rename = "exploration")]
	Exploration,
	#[serde(other)]
	Insiders,
}

fn get_binary_name() -> Result<&'static str, CodeError> {
	VSCODE_CLI_BINARY_NAME.ok_or_else(|| CodeError::UpdatesNotConfigured("no binary name"))
}

impl Quality {
	/// Lowercased quality name in paths and protocol
	pub fn get_machine_name(&self) -> &'static str {
		match self {
			Quality::Insiders => "insiders",
			Quality::Exploration => "exploration",
			Quality::Stable => "stable",
		}
	}

	/// Uppercased quality display name for humans
	pub fn get_capitalized_name(&self) -> &'static str {
		match self {
			Quality::Insiders => "Insiders",
			Quality::Exploration => "Exploration",
			Quality::Stable => "Stable",
		}
	}

	/// Server application name
	pub fn server_entrypoint(&self) -> Result<String, CodeError> {
		let mut server_name = get_binary_name()?.to_string();

		if cfg!(windows) {
			server_name.push_str(".cmd");
		}

		Ok(server_name)
	}
}

impl fmt::Display for Quality {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "{}", self.get_capitalized_name())
	}
}

impl TryFrom<&str> for Quality {
	type Error = String;

	fn try_from(s: &str) -> Result<Self, Self::Error> {
		match s {
			"stable" => Ok(Quality::Stable),
			"insiders" | "insider" => Ok(Quality::Insiders),
			"exploration" => Ok(Quality::Exploration),
			_ => Err(format!(
				"Unknown quality: {s}. Must be one of stable, insiders, or exploration."
			)),
		}
	}
}

#[derive(clap::ValueEnum, Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TelemetryLevel {
	Off,
	Crash,
	Error,
	All,
}

impl fmt::Display for TelemetryLevel {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			TelemetryLevel::Off => write!(f, "off"),
			TelemetryLevel::Crash => write!(f, "crash"),
			TelemetryLevel::Error => write!(f, "error"),
			TelemetryLevel::All => write!(f, "all"),
		}
	}
}
