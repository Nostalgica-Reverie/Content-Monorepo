use std::fs;
use std::path::{Path, PathBuf};

use serde::Deserialize;

pub const MARKER_NAME: &str = "packeater.json";

#[derive(Clone, Debug, Deserialize)]
#[serde(default, deny_unknown_fields, rename_all = "camelCase")]
pub struct PackeaterConfig {
	#[serde(rename = "$schema")]
	pub schema: Option<String>,
	pub version: u8,
	pub enabled: bool,
	pub output: Option<PathBuf>,
	pub compression: CompressionOptions,
	pub lossy: LossyOptions
}

impl Default for PackeaterConfig {
	fn default() -> Self {
		Self {
			schema: None,
			version: 1,
			enabled: true,
			output: None,
			compression: CompressionOptions::default(),
			lossy: LossyOptions::default()
		}
	}
}

#[derive(Clone, Debug, Deserialize)]
#[serde(default, deny_unknown_fields, rename_all = "camelCase")]
pub struct CompressionOptions {
	pub recompress_compressed_files: bool,
	pub deduplicate_files: bool,
	pub zip_iterations: u8,
	pub image_iterations: u8,
	pub nbt_iterations: u8
}

impl Default for CompressionOptions {
	fn default() -> Self {
		Self {
			recompress_compressed_files: true,
			deduplicate_files: true,
			zip_iterations: 30,
			image_iterations: 15,
			nbt_iterations: 20
		}
	}
}

#[derive(Clone, Debug, Deserialize)]
#[serde(default, deny_unknown_fields, rename_all = "camelCase")]
pub struct LossyOptions {
	pub png: bool,
	pub png_palette: PngPalette,
	pub png_dithering: f32,
	pub downsize_single_color_images: bool,
	pub audio: bool,
	pub audio_quality: f32,
	pub audio_sample_rate: Option<u32>,
	pub audio_channels: Option<u8>
}

impl Default for LossyOptions {
	fn default() -> Self {
		Self {
			png: true,
			png_palette: PngPalette::EightBit,
			png_dithering: 0.8,
			downsize_single_color_images: true,
			audio: true,
			audio_quality: 0.0,
			audio_sample_rate: Some(32_000),
			audio_channels: None
		}
	}
}

#[derive(Clone, Copy, Debug, Default, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PngPalette {
	Lossless,
	Auto,
	FourBit,
	#[default]
	EightBit
}

impl PngPalette {
	const fn packsquash_value(self) -> &'static str {
		match self {
			Self::Lossless => "none",
			Self::Auto => "auto",
			Self::FourBit => "four_bit_depth",
			Self::EightBit => "eight_bit_depth"
		}
	}
}

impl PackeaterConfig {
	pub fn read(path: &Path) -> Result<Self, String> {
		let bytes =
			fs::read(path).map_err(|error| format!("could not read {}: {error}", path.display()))?;
		let config: Self = serde_json::from_slice(&bytes)
			.map_err(|error| format!("could not parse {}: {error}", path.display()))?;
		config.validate(path)?;
		Ok(config)
	}

	fn validate(&self, path: &Path) -> Result<(), String> {
		if self.version != 1 {
			return Err(format!(
				"{} uses unsupported Packeater config version {}; expected 1",
				path.display(),
				self.version
			));
		}
		if !(0.0..=1.0).contains(&self.lossy.png_dithering) {
			return Err("lossy.pngDithering must be between 0 and 1".into());
		}
		if !(-2.0..=10.0).contains(&self.lossy.audio_quality) {
			return Err("lossy.audioQuality must be between -2 and 10".into());
		}
		if self.lossy.audio_sample_rate == Some(0) {
			return Err("lossy.audioSampleRate must be greater than zero".into());
		}
		if self
			.lossy
			.audio_channels
			.is_some_and(|channels| !(1..=2).contains(&channels))
		{
			return Err("lossy.audioChannels must be 1, 2, or null".into());
		}
		Ok(())
	}

	pub fn output_path(&self, pack_directory: &Path) -> PathBuf {
		self.output.as_ref().map_or_else(
			|| {
				let name = pack_directory
					.file_name()
					.and_then(|name| name.to_str())
					.unwrap_or("pack");
				pack_directory
					.parent()
					.unwrap_or_else(|| Path::new("."))
					.join(format!("{name}.zip"))
			},
			|path| {
				if path.is_absolute() {
					path.clone()
				} else {
					pack_directory.join(path)
				}
			}
		)
	}

	pub fn squash_options(
		&self,
		pack_directory: &Path,
		output_path: &Path
	) -> Result<packsquash::config::SquashOptions, String> {
		let mut root = toml::Table::new();
		root.insert(
			"pack_directory".into(),
			pack_directory.to_string_lossy().into_owned().into()
		);
		root.insert(
			"output_file_path".into(),
			output_path.to_string_lossy().into_owned().into()
		);
		root.insert(
			"recompress_compressed_files".into(),
			self.compression.recompress_compressed_files.into()
		);
		root.insert(
			"zip_compression_iterations".into(),
			i64::from(self.compression.zip_iterations).into()
		);
		root.insert(
			"zip_spec_conformance_level".into(),
			if self.compression.deduplicate_files {
				"balanced"
			} else {
				"high"
			}
			.into()
		);
		root.insert("never_store_squash_times".into(), true.into());

		let mut png = toml::Table::new();
		png.insert(
			"image_data_compression_iterations".into(),
			i64::from(self.compression.image_iterations).into()
		);
		png.insert(
			"color_quantization_target".into(),
			if self.lossy.png {
				self.lossy.png_palette.packsquash_value()
			} else {
				"none"
			}
			.into()
		);
		png.insert(
			"color_quantization_dithering_level".into(),
			f64::from(self.lossy.png_dithering).into()
		);
		png.insert(
			"downsize_if_single_color".into(),
			(self.lossy.png && self.lossy.downsize_single_color_images).into()
		);
		root.insert("**/*?.png".into(), png.into());

		let mut audio = toml::Table::new();
		audio.insert("transcode_ogg".into(), self.lossy.audio.into());
		if self.lossy.audio {
			audio.insert("bitrate_control_mode".into(), "CQF".into());
			audio.insert(
				"target_bitrate_control_metric".into(),
				f64::from(self.lossy.audio_quality).into()
			);
			if let Some(sample_rate) = self.lossy.audio_sample_rate {
				audio.insert("sampling_frequency".into(), i64::from(sample_rate).into());
			}
			if let Some(channels) = self.lossy.audio_channels {
				audio.insert("channels".into(), i64::from(channels).into());
			}
		}
		root.insert("**/*?.{ogg,oga,mp3,m4a,flac,wav}".into(), audio.into());

		let mut nbt = toml::Table::new();
		nbt.insert(
			"nbt_compression_iterations".into(),
			i64::from(self.compression.nbt_iterations).into()
		);
		root.insert("**/*?.nbt".into(), nbt.into());

		let serialized = toml::to_string(&root)
			.map_err(|error| format!("could not generate optimizer settings: {error}"))?;
		toml::from_str(&serialized)
			.map_err(|error| format!("generated optimizer settings were invalid: {error}"))
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn defaults_are_aggressive_and_lossy() {
		let config = PackeaterConfig::default();
		assert!(config.compression.recompress_compressed_files);
		assert!(config.compression.deduplicate_files);
		assert!(config.lossy.png);
		assert!(config.lossy.audio);
		let options = config
			.squash_options(Path::new("pack"), Path::new("pack.zip"))
			.unwrap();
		assert_eq!(options.pack_directory, Path::new("pack"));
		assert_eq!(
			options.global_options.output_file_path,
			Path::new("pack.zip")
		);
	}

	#[test]
	fn relative_output_is_resolved_from_pack_folder() {
		let config = PackeaterConfig {
			output: Some(PathBuf::from("../dist/result.zip")),
			..PackeaterConfig::default()
		};
		assert_eq!(
			config.output_path(Path::new("packs/example")),
			Path::new("packs/example/../dist/result.zip")
		);
	}
}
