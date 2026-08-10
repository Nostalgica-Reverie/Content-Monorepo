use std::collections::BTreeSet;
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use clap::ArgMatches;
use packwand_build::{
	ExportFormat, ExportOptions, archive_content_directory, discover_packeater_markers,
	export_pack, run_packeater,
};
use packwand_workspace::Project;

type Result<T = ()> = std::result::Result<T, Box<dyn Error>>;

pub fn run(args: &ArgMatches) -> Result {
	let root = std::env::current_dir()?;
	let suffix = args
		.get_one::<String>("sha")
		.map(String::as_str)
		.unwrap_or("local");
	validate_segment(suffix, "SHA suffix")?;
	let projects = packwand_workspace::discover(&root)?;
	let selected = if let Some(id) = args.get_one::<String>("pack") {
		let project = projects
			.into_iter()
			.find(|project| {
				project.manifest.id == *id
					|| project
						.root
						.file_name()
						.is_some_and(|name| name == id.as_str())
			})
			.ok_or_else(|| format!("project {id:?} was not found"))?;
		vec![project]
	} else {
		changed_projects(&root, projects)?
	};
	if selected.is_empty() {
		println!("no publishable projects detected in the latest commit");
		return Ok(());
	}
	let artifacts = root.join("artifacts");
	fs::create_dir_all(&artifacts)?;
	let mut built = 0usize;
	for project in selected {
		built += match project.category.as_str() {
			"modpacks" => build_modpack(&project, suffix, &artifacts)?,
			"datapacks" | "resourcepacks" => build_content_pack(&project, suffix, &artifacts)?,
			"mods" => build_mod(&project, suffix, &artifacts)?,
			category => {
				eprintln!("warning: category {category:?} does not have a build workflow");
				0
			}
		};
	}
	println!("all {built} build(s) completed successfully");
	Ok(())
}

fn changed_projects(root: &Path, projects: Vec<Project>) -> Result<Vec<Project>> {
	let output = Command::new("git")
		.args(["diff-tree", "--no-commit-id", "--name-only", "-r", "HEAD"])
		.current_dir(root)
		.output()?;
	if !output.status.success() {
		return Err(format!(
			"git diff-tree failed: {}",
			String::from_utf8_lossy(&output.stderr).trim()
		)
		.into());
	}
	let changed = String::from_utf8(output.stdout)?
		.lines()
		.filter_map(|line| {
			let mut parts = line.split('/');
			Some((parts.next()?.to_owned(), parts.next()?.to_owned()))
		})
		.collect::<BTreeSet<_>>();
	Ok(projects
		.into_iter()
		.filter(|project| {
			let directory = project
				.root
				.file_name()
				.map(|name| name.to_string_lossy().into_owned())
				.unwrap_or_default();
			changed.contains(&(project.category.clone(), directory))
		})
		.collect())
}

fn build_modpack(project: &Project, suffix: &str, artifacts: &Path) -> Result<usize> {
	if project.manifest.version.trim().is_empty() {
		return Err(format!("{} has no manifest version", project.manifest.id).into());
	}
	let name = artifact_segment(project.manifest.effective_name());
	let mut count = 0usize;
	for subdir in &project.subdirs {
		let key = subdir
			.file_name()
			.and_then(|name| name.to_str())
			.ok_or("pack subdir has no valid name")?;
		let (platform, format) = if let Some(key) = key.strip_suffix("-mr") {
			(format!("{key}-mr"), ExportFormat::Modrinth)
		} else if let Some(key) = key.strip_suffix("-cf") {
			(format!("{key}-cf"), ExportFormat::CurseForge)
		} else {
			continue;
		};
		let output = artifacts.join(format!(
			"{name}-{platform}-{}-{suffix}.{}",
			project.manifest.version,
			format.extension()
		));
		let artifact = export_pack(subdir, format, Some(&output), ExportOptions::default())?;
		println!(
			"built {} ({} bytes)",
			artifact.path.display(),
			artifact.bytes
		);
		count += 1;
	}
	if count == 0 {
		Err(format!("{} has no exportable -mr/-cf subdirs", project.manifest.id).into())
	} else {
		Ok(count)
	}
}

fn build_content_pack(project: &Project, suffix: &str, artifacts: &Path) -> Result<usize> {
	let markers = discover_packeater_markers(&project.root)?;
	if !markers.is_empty() {
		let mut built = 0;
		for marker in markers {
			let folder = marker
				.parent()
				.ok_or("Packeater marker has no parent folder")?;
			let relative = folder.strip_prefix(&project.root)?;
			let variant = if relative.as_os_str().is_empty() {
				None
			} else {
				Some(artifact_segment(
					&relative.to_string_lossy().replace(['/', '\\'], "-"),
				))
			};
			let folder_name = folder.file_name().and_then(|name| name.to_str());
			let version = folder_name
				.and_then(|name| {
					project
						.manifest
						.variants
						.iter()
						.find(|variant| variant.id.as_deref() == Some(name))
				})
				.and_then(|variant| variant.version.as_deref())
				.filter(|version| !version.is_empty())
				.unwrap_or(&project.manifest.version);
			let version = if version.is_empty() {
				"unknown"
			} else {
				version
			};
			let output = artifacts.join(format!(
				"{}{}{version}-{suffix}.zip",
				artifact_segment(&project.manifest.id),
				variant.map_or_else(|| "-".into(), |variant| format!("-{variant}-")),
			));
			let bytes = run_packeater(&marker, &output)?;
			println!(
				"ate {} into {} ({bytes} bytes)",
				folder.display(),
				output.display()
			);
			built += 1;
		}
		return Ok(built);
	}
	let content = content_root(project)?;
	let version = if project.manifest.version.is_empty() {
		content
			.file_name()
			.and_then(|name| name.to_str())
			.unwrap_or("unknown")
	} else {
		&project.manifest.version
	};
	let output = artifacts.join(format!(
		"{}-{version}-{suffix}.zip",
		artifact_segment(&project.manifest.id)
	));
	let bytes = archive_content_directory(&content, &output)?;
	println!("built {} ({bytes} bytes)", output.display());
	Ok(1)
}

fn content_root(project: &Project) -> Result<PathBuf> {
	if project.root.join("pack.mcmeta").is_file() {
		return Ok(project.root.clone());
	}
	if let Some(version) = &project.manifest.mc_version {
		let candidate = project.root.join(version);
		if candidate.is_dir() {
			return Ok(candidate);
		}
	}
	let mut candidates = fs::read_dir(&project.root)?
		.collect::<std::result::Result<Vec<_>, _>>()?
		.into_iter()
		.filter(|entry| entry.file_type().is_ok_and(|kind| kind.is_dir()))
		.map(|entry| entry.path())
		.filter(|path| {
			path.join("pack.mcmeta").is_file()
				|| path.join("data").is_dir()
				|| path.join("assets").is_dir()
		})
		.collect::<Vec<_>>();
	candidates.sort();
	match candidates.as_slice() {
		[only] => Ok(only.clone()),
		[] => Err(format!("{} has no content root", project.manifest.id).into()),
		_ => Err(format!(
			"{} has multiple content roots; publish a specific variant",
			project.manifest.id
		)
		.into()),
	}
}

fn build_mod(project: &Project, suffix: &str, artifacts: &Path) -> Result<usize> {
	if project.manifest.variants.is_empty() {
		return Err(format!("{} has no Gradle variants", project.manifest.id).into());
	}
	let wrapper = if cfg!(windows) {
		project.root.join("gradlew.bat")
	} else {
		project.root.join("gradlew")
	};
	if !wrapper.is_file() {
		return Err(format!("Gradle wrapper is missing at {}", wrapper.display()).into());
	}
	let mut built = 0usize;
	for variant in &project.manifest.variants {
		let gradle_project = variant
			.gradle_project
			.as_deref()
			.ok_or("mod variant is missing gradle_project")?;
		validate_segment(gradle_project, "Gradle project")?;
		let key = variant
			.id
			.as_deref()
			.or(variant.mc_version.as_deref())
			.ok_or("mod variant has neither id nor mc_version")?;
		let task = format!(":{gradle_project}:build");
		let status = if cfg!(windows) {
			Command::new("cmd")
				.args([
					"/C",
					wrapper.to_string_lossy().as_ref(),
					"--no-daemon",
					&task,
				])
				.current_dir(&project.root)
				.status()?
		} else {
			Command::new(&wrapper)
				.args(["--no-daemon", &task])
				.current_dir(&project.root)
				.status()?
		};
		if !status.success() {
			return Err(format!("Gradle task {task} failed with {status}").into());
		}
		let jar = distributable_jar(
			&project
				.root
				.join("versions")
				.join(gradle_project)
				.join("build/libs"),
		)?;
		let version = variant
			.version
			.as_deref()
			.filter(|value| !value.is_empty())
			.unwrap_or(&project.manifest.version);
		let output = artifacts.join(format!(
			"{}-{version}-{}-{suffix}.jar",
			artifact_segment(project.manifest.effective_name()),
			artifact_segment(key)
		));
		fs::copy(&jar, &output)?;
		println!("built {}", output.display());
		built += 1;
	}
	Ok(built)
}

fn distributable_jar(directory: &Path) -> Result<PathBuf> {
	let mut jars = fs::read_dir(directory)?
		.collect::<std::result::Result<Vec<_>, _>>()?
		.into_iter()
		.filter(|entry| entry.file_type().is_ok_and(|kind| kind.is_file()))
		.map(|entry| entry.path())
		.filter(|path| {
			let name = path
				.file_name()
				.and_then(|name| name.to_str())
				.unwrap_or_default()
				.to_ascii_lowercase();
			name.ends_with(".jar")
				&& ![
					"-sources.jar",
					"-javadoc.jar",
					"-dev.jar",
					"-dev-shadow.jar",
				]
				.iter()
				.any(|suffix| name.ends_with(suffix))
		})
		.collect::<Vec<_>>();
	jars.sort();
	match jars.as_slice() {
		[jar] => Ok(jar.clone()),
		_ => Err(format!(
			"expected exactly one distributable jar in {}, found {}",
			directory.display(),
			jars.len()
		)
		.into()),
	}
}

fn validate_segment(value: &str, label: &str) -> Result {
	if value.is_empty()
		|| !value
			.bytes()
			.all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-'))
	{
		Err(format!("invalid {label} {value:?}").into())
	} else {
		Ok(())
	}
}

fn artifact_segment(value: &str) -> String {
	value
		.chars()
		.map(|character| {
			if character.is_ascii_alphanumeric() || matches!(character, '.' | '_' | '-') {
				character
			} else {
				'-'
			}
		})
		.collect()
}
