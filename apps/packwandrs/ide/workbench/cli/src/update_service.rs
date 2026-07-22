/*---------------------------------------------------------------------------------------------
 *  Copyright (c) Microsoft Corporation. All rights reserved.
 *  Licensed under the MIT License. See License.txt in the project root for license information.
 *--------------------------------------------------------------------------------------------*/

use std::{fmt, path::Path};

use serde::{Deserialize, Serialize};

use crate::{
	constants::{VSCODE_CLI_APP_NAME, VSCODE_CLI_DOWNLOAD_ENDPOINT, VSCODE_CLI_UPDATE_ENDPOINT},
	log, options,
	util::{
		errors::{wrap, AnyError, CodeError, WrappedError},
		http::{BoxedHttp, SimpleResponse},
		io::ReportCopyProgress,
		tar::{self, has_gzip_header},
		zipper,
	}
};

/// Implementation of the VS Code Update service for use in the CLI.
#[derive(Clone)]
pub struct UpdateService {
	client: BoxedHttp,
	log: log::Logger,
}

/// Describes a specific release, can be created manually or returned from the update service.
#[derive(Clone, Eq, PartialEq)]
pub struct Release {
	pub name: String,
	pub platform: Platform,
	pub target: TargetKind,
	pub quality: options::Quality,
	pub commit: String,
}

impl std::fmt::Display for Release {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{} (commit {})", self.name, self.commit)
	}
}

#[derive(Deserialize)]
struct UpdateServerVersion {
	pub version: String,
	pub name: String,
}

fn quality_download_segment(quality: options::Quality) -> &'static str {
	match quality {
		options::Quality::Stable => "stable",
		options::Quality::Insiders => "insider",
		options::Quality::Exploration => "exploration",
	}
}

fn get_app_name() -> Result<&'static str, CodeError> {
	VSCODE_CLI_APP_NAME.ok_or_else(|| CodeError::UpdatesNotConfigured("no app name"))
}

fn get_download_endpoint() -> Result<&'static str, CodeError> {
	VSCODE_CLI_DOWNLOAD_ENDPOINT.ok_or_else(|| CodeError::UpdatesNotConfigured("no download url"))
}

fn get_update_endpoint() -> Result<String, CodeError> {
	if let Ok(url) = std::env::var("VSCODE_CLI_UPDATE_URL") {
		if !url.is_empty() {
			return Ok(url);
		}
	}
	VSCODE_CLI_UPDATE_ENDPOINT
		.map(|s| s.to_string())
		.ok_or(CodeError::UpdatesNotConfigured("no update url"))
}

impl UpdateService {
	pub fn new(log: log::Logger, http: BoxedHttp) -> Self {
		UpdateService { client: http, log }
	}

	/// Gets the latest commit for the target of the given quality.
	pub async fn get_latest_commit(
		&self,
		platform: Platform,
		target: TargetKind,
		quality: options::Quality,
	) -> Result<Release, AnyError> {
		let update_endpoint = get_update_endpoint()?;
		let download_url = format!(
			"{}/{}/{}/{}/latest.json",
			&update_endpoint,
			quality_download_segment(quality),
			platform.os(),
			platform.arch(),
		);

		let mut response = self.client.make_request("GET", download_url).await?;

		if !response.status_code.is_success() {
			return Err(response.into_err().await.into());
		}

		let res = response.json::<UpdateServerVersion>().await?;
		debug!(self.log, "Resolved quality {} to {}", quality, res.version);

		Ok(Release {
			target,
			platform,
			quality,
			name: res.name,
			commit: res.version,
		})
	}

	pub fn get_download_url(&self, release: &Release) -> Result<String, AnyError> {
		let app_name = get_app_name()?;
		let download_endpoint = get_download_endpoint()?;

		let download_url = format!(
			"{}/download/{}/{}-reh-web-{}-{}-{}.tar.gz",
			download_endpoint,
			release.name,
			app_name,
			release.platform.os(),
			release.platform.arch(),
			release.name,
		);

		Ok(download_url)
	}

	/// Gets the download stream for the release.
	pub async fn get_download_stream(&self, release: &Release) -> Result<SimpleResponse, AnyError> {
		let download_url = self.get_download_url(release)?;

		let response = self.client.make_request("GET", download_url).await?;
		if !response.status_code.is_success() {
			return Err(response.into_err().await.into());
		}

		Ok(response)
	}
}

pub fn unzip_downloaded_release<T>(
	compressed_file: &Path,
	target_dir: &Path,
	reporter: T,
) -> Result<(), WrappedError>
where
	T: ReportCopyProgress,
{
	match has_gzip_header(compressed_file) {
		Ok((f, true)) => tar::decompress_tarball(f, target_dir, reporter),
		Ok((f, false)) => zipper::unzip_file(f, target_dir, reporter),
		Err(e) => Err(wrap(e, "error checking for gzip header")),
	}
}

#[derive(Eq, PartialEq, Copy, Clone)]
pub enum TargetKind {
	Server,
	Archive,
	Web,
	Cli,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum Platform {
	LinuxAlpineX64,
	LinuxAlpineARM64,
	LinuxX64,
	LinuxX64Legacy,
	LinuxARM64,
	LinuxARM64Legacy,
	LinuxARM32,
	LinuxARM32Legacy,
	DarwinX64,
	DarwinARM64,
	WindowsX64,
	WindowsX86,
	WindowsARM64,
}

impl Platform {
	pub fn arch(&self) -> String {
		match self {
			Platform::LinuxAlpineARM64 => "arm64",
			Platform::LinuxAlpineX64 => "x64",
			Platform::LinuxX64 => "x64",
			Platform::LinuxX64Legacy => "x64",
			Platform::LinuxARM64 => "arm64",
			Platform::LinuxARM64Legacy => "arm64",
			Platform::LinuxARM32 => "armhf",
			Platform::LinuxARM32Legacy => "armhf",
			Platform::DarwinX64 => "x64",
			Platform::DarwinARM64 => "arm64",
			Platform::WindowsX64 => "x64",
			Platform::WindowsX86 => "ia42",
			Platform::WindowsARM64 => "arm64",
		}
		.to_owned()
	}

	pub fn os(&self) -> String {
		match self {
			Platform::LinuxAlpineARM64 => "alpine",
			Platform::LinuxAlpineX64 => "alpine",
			Platform::LinuxX64 => "linux",
			Platform::LinuxX64Legacy => "linux",
			Platform::LinuxARM64 => "linux",
			Platform::LinuxARM64Legacy => "linux",
			Platform::LinuxARM32 => "linux",
			Platform::LinuxARM32Legacy => "linux",
			Platform::DarwinX64 => "darwin",
			Platform::DarwinARM64 => "darwin",
			Platform::WindowsX64 => "win32",
			Platform::WindowsX86 => "win32",
			Platform::WindowsARM64 => "win32",
		}
		.to_owned()
	}

	pub fn env_default() -> Option<Platform> {
		if cfg!(all(
			target_os = "linux",
			target_arch = "x86_64",
			target_env = "musl"
		)) {
			Some(Platform::LinuxAlpineX64)
		} else if cfg!(all(
			target_os = "linux",
			target_arch = "aarch64",
			target_env = "musl"
		)) {
			Some(Platform::LinuxAlpineARM64)
		} else if cfg!(all(target_os = "linux", target_arch = "x86_64")) {
			Some(Platform::LinuxX64)
		} else if cfg!(all(target_os = "linux", target_arch = "arm")) {
			Some(Platform::LinuxARM32)
		} else if cfg!(all(target_os = "linux", target_arch = "aarch64")) {
			Some(Platform::LinuxARM64)
		} else if cfg!(all(target_os = "macos", target_arch = "x86_64")) {
			Some(Platform::DarwinX64)
		} else if cfg!(all(target_os = "macos", target_arch = "aarch64")) {
			Some(Platform::DarwinARM64)
		} else if cfg!(all(target_os = "windows", target_arch = "x86_64")) {
			Some(Platform::WindowsX64)
		} else if cfg!(all(target_os = "windows", target_arch = "x86")) {
			Some(Platform::WindowsX86)
		} else if cfg!(all(target_os = "windows", target_arch = "aarch64")) {
			Some(Platform::WindowsARM64)
		} else {
			None
		}
	}
}

impl fmt::Display for Platform {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		f.write_str(match self {
			Platform::LinuxAlpineARM64 => "LinuxAlpineARM64",
			Platform::LinuxAlpineX64 => "LinuxAlpineX64",
			Platform::LinuxX64 => "LinuxX64",
			Platform::LinuxX64Legacy => "LinuxX64Legacy",
			Platform::LinuxARM64 => "LinuxARM64",
			Platform::LinuxARM64Legacy => "LinuxARM64Legacy",
			Platform::LinuxARM32 => "LinuxARM32",
			Platform::LinuxARM32Legacy => "LinuxARM32Legacy",
			Platform::DarwinX64 => "DarwinX64",
			Platform::DarwinARM64 => "DarwinARM64",
			Platform::WindowsX64 => "WindowsX64",
			Platform::WindowsX86 => "WindowsX86",
			Platform::WindowsARM64 => "WindowsARM64",
		})
	}
}
