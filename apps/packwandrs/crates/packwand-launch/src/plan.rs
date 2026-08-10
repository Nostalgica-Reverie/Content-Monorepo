//! Deterministic launch plans built from instance records.

use std::collections::BTreeMap;
use std::path::PathBuf;

use packwand_instance::{InstancePaths, InstanceRecord, MemoryLimits};
use serde::Serialize;

/// Schema version of the serialized launch plan.
pub const PLAN_SCHEMA_VERSION: u32 = 1;

/// The classpath separator of the host operating system.
pub fn host_classpath_separator() -> &'static str {
	if cfg!(windows) { ";" } else { ":" }
}

/// Directories a running game needs, resolved from the instance layout.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct LaunchPaths {
	pub logs: PathBuf,
	pub natives: PathBuf,
	pub assets: PathBuf,
	pub libraries: PathBuf,
	pub game_data: PathBuf,
}

/// A fully resolved, inspectable launch plan.
///
/// Serialization is deterministic: field order is fixed by the struct and
/// all maps are ordered. Account/session values appear only as redacted
/// `${secret:<name>}` placeholders; real secret values are never part of a
/// plan and will be resolved at spawn time by a future auth subsystem via
/// opaque handles.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct LaunchPlan {
	pub schema_version: u32,
	pub instance_id: String,
	pub working_dir: PathBuf,
	pub java_executable: PathBuf,
	pub jvm_args: Vec<String>,
	pub classpath: Vec<PathBuf>,
	pub classpath_separator: String,
	pub main_class: String,
	pub game_args: Vec<String>,
	pub env: BTreeMap<String, String>,
	pub memory: MemoryLimits,
	pub session: BTreeMap<String, String>,
	pub paths: LaunchPaths,
}

impl LaunchPlan {
	/// The classpath entries joined with the plan's separator.
	pub fn classpath_string(&self) -> String {
		self.classpath
			.iter()
			.map(|p| p.display().to_string())
			.collect::<Vec<_>>()
			.join(&self.classpath_separator)
	}

	/// The ordered argument vector passed to the Java executable:
	/// JVM args, memory flags, classpath, main class, then game args.
	pub fn command_arguments(&self) -> Vec<String> {
		let mut args = self.jvm_args.clone();
		if let Some(mb) = self.memory.initial_mb {
			args.push(format!("-Xms{mb}m"));
		}
		if let Some(mb) = self.memory.max_mb {
			args.push(format!("-Xmx{mb}m"));
		}
		if !self.classpath.is_empty() {
			args.push("-cp".to_string());
			args.push(self.classpath_string());
		}
		args.push(self.main_class.clone());
		args.extend(self.game_args.iter().cloned());
		args
	}
}

/// Replaces `${name}` occurrences for every known variable. Unknown
/// placeholders (including `${secret:*}` session placeholders) are left
/// untouched.
fn substitute(arg: &str, vars: &[(&str, &str)]) -> String {
	let mut out = arg.to_string();
	for (name, value) in vars {
		out = out.replace(&format!("${{{name}}}"), value);
	}
	out
}

/// Builds the deterministic launch plan for one instance record.
pub fn build_launch_plan(record: &InstanceRecord, paths: &InstancePaths) -> LaunchPlan {
	let classpath_separator = host_classpath_separator().to_string();
	let classpath_string = record
		.classpath
		.iter()
		.map(|p| p.display().to_string())
		.collect::<Vec<_>>()
		.join(&classpath_separator);
	let game_dir = paths.game_dir.display().to_string();
	let logs_dir = paths.logs_dir.display().to_string();
	let natives_dir = paths.natives_dir.display().to_string();
	let assets_dir = paths.assets_dir.display().to_string();
	let libraries_dir = paths.libraries_dir.display().to_string();
	let vars: [(&str, &str); 12] = [
		("instance_id", &record.id),
		("game_dir", &game_dir),
		("logs_dir", &logs_dir),
		("natives_dir", &natives_dir),
		("assets_dir", &assets_dir),
		("libraries_dir", &libraries_dir),
		("classpath", &classpath_string),
		("classpath_separator", &classpath_separator),
		// Mojang's launcher-metadata names for the same locations, so
		// arguments resolved from version documents stay relocatable.
		("game_directory", &game_dir),
		("natives_directory", &natives_dir),
		("assets_root", &assets_dir),
		("library_directory", &libraries_dir),
	];
	LaunchPlan {
		schema_version: PLAN_SCHEMA_VERSION,
		instance_id: record.id.clone(),
		working_dir: paths.game_dir.clone(),
		java_executable: record.java_executable.clone(),
		jvm_args: record
			.jvm_args
			.iter()
			.map(|a| substitute(a, &vars))
			.collect(),
		classpath: record.classpath.clone(),
		classpath_separator: classpath_separator.clone(),
		main_class: record.main_class.clone(),
		game_args: record
			.game_args
			.iter()
			.map(|a| substitute(a, &vars))
			.collect(),
		env: record.env.clone(),
		memory: record.memory.clone(),
		session: record
			.session_placeholders
			.iter()
			.map(|name| (name.clone(), format!("${{secret:{name}}}")))
			.collect(),
		paths: LaunchPaths {
			logs: paths.logs_dir.clone(),
			natives: paths.natives_dir.clone(),
			assets: paths.assets_dir.clone(),
			libraries: paths.libraries_dir.clone(),
			game_data: paths.game_dir.clone(),
		},
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use packwand_instance::MemoryLimits;

	fn record() -> InstanceRecord {
		InstanceRecord {
			schema_version: 1,
			id: "forge".to_string(),
			name: "Forge".to_string(),
			java_executable: PathBuf::from("java"),
			jvm_args: vec![
				"-p".to_string(),
				"${library_directory}/a.jar${classpath_separator}${library_directory}/b.jar"
					.to_string(),
				"-Dlog4j.configurationFile=${assets_root}/log_configs/client-1.xml".to_string(),
			],
			main_class: "cpw.mods.bootstraplauncher.BootstrapLauncher".to_string(),
			classpath: vec![PathBuf::from("libs\\client.jar")],
			game_args: vec!["--gameDir".to_string(), "${game_directory}".to_string()],
			env: BTreeMap::new(),
			memory: MemoryLimits::default(),
			session_placeholders: vec![],
		}
	}

	fn paths() -> InstancePaths {
		InstancePaths {
			game_dir: PathBuf::from("root\\instances\\forge"),
			logs_dir: PathBuf::from("root\\instances\\forge\\logs"),
			natives_dir: PathBuf::from("root\\instances\\forge\\natives"),
			assets_dir: PathBuf::from("root\\assets"),
			libraries_dir: PathBuf::from("root\\libraries"),
		}
	}

	#[test]
	fn launch_plan_substitutes_classpath_separator_and_logging_path() {
		let plan = build_launch_plan(&record(), &paths());
		assert!(plan.jvm_args[1].contains(host_classpath_separator()));
		assert!(plan.jvm_args[2].contains("log_configs"));
		assert!(!plan.jvm_args[2].contains("${assets_root}"));
		assert_eq!(plan.game_args[1], paths().game_dir.display().to_string());
	}
}
