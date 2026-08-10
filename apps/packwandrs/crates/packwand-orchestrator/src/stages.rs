//! The steps a launch is actually made of.
//!
//! Each one is small enough to read in a sitting and owns both halves of what
//! it does: the work, and the undo. See [`crate::steps`] for the ordering
//! guarantee they rely on.

use std::path::{Path, PathBuf};

use packwand_instance::{FsInstanceRepository, InstancePaths, InstanceRepository};
use packwand_launch::build_launch_plan;
use packwand_minecraft::Host;
use packwand_minecraft::model::VersionDoc;
use packwand_minecraft::plan::{InstallLayout, build_version_plan};
use packwand_runtime::{DiscoveryConfig, ProbeCache, discover, select_compatible};

use crate::boot;
use crate::error::{OrchestratorError, Result};
use crate::launch::LaunchSignal;
use crate::steps::{LaunchStep, Outcome, StepContext};

/// Creates the directories Minecraft assumes exist before it starts.
pub struct CreateGameFolders;

impl LaunchStep for CreateGameFolders {
	fn name(&self) -> &'static str {
		"Creating game folders"
	}

	fn run(
		&mut self,
		ctx: &mut StepContext,
		_report: &(dyn Fn(LaunchSignal) + Sync),
	) -> Result<()> {
		for dir in [
			ctx.game_dir.clone(),
			ctx.game_dir.join("logs"),
			ctx.game_dir.join("mods"),
		] {
			std::fs::create_dir_all(&dir).map_err(|e| {
				OrchestratorError::new("io", format!("failed to create {}: {e}", dir.display()))
			})?;
		}
		sweep_stale_natives(&ctx.game_dir);
		Ok(())
	}
}

/// Removes native directories left by a previous run.
///
/// `ExtractNatives::finalize` cleans up after itself, but only if the process
/// lives to run it: a crashed or killed launcher leaves its directory behind,
/// and the name carries a process id the OS will eventually hand out again.
/// Sweeping at the start of a launch is what stops those accumulating — the
/// exact complaint the per-launch directory was introduced to fix.
///
/// Best-effort throughout: a directory that cannot be removed is not a reason
/// to refuse to start the game.
fn sweep_stale_natives(game_dir: &Path) {
	let keep = launch_natives_dir(game_dir);
	let Ok(entries) = std::fs::read_dir(game_dir.join(".packwand")) else {
		return;
	};
	for entry in entries.flatten() {
		let path = entry.path();
		if path == keep || !path.is_dir() {
			continue;
		}
		if entry
			.file_name()
			.to_str()
			.is_some_and(|name| name.starts_with("natives-"))
		{
			let _ = std::fs::remove_dir_all(&path);
		}
	}
}

/// Reads the pack's `pack.toml` to learn which Minecraft version and loader
/// this launch is for.
pub struct ResolveVersion;

impl LaunchStep for ResolveVersion {
	fn name(&self) -> &'static str {
		"Resolving version"
	}

	fn run(&mut self, ctx: &mut StepContext, report: &(dyn Fn(LaunchSignal) + Sync)) -> Result<()> {
		let target = boot::resolve_pack_target(&ctx.pack_dir.join("pack.toml"))
			.map_err(|e| OrchestratorError::new("pack", e.to_string()))?;
		report(LaunchSignal::Log(format!(
			"Minecraft {}{}",
			target.minecraft,
			match (&target.loader, &target.loader_version) {
				(Some(loader), Some(version)) => format!(" with {loader} {version}"),
				(Some(loader), None) => format!(" with {loader}"),
				(None, _) => String::new(),
			}
		)));
		// The feature release the game needs, so the Java steps that follow
		// have something to check against. Minecraft states it from 1.17 on;
		// older versions say nothing and anything installed will do.
		ctx.required_java_major = required_java_major(&ctx.managed_root, &target);
		if let Some(required) = ctx.required_java_major {
			report(LaunchSignal::Log(format!("Requires Java {required}")));
		}
		// Say so when the loader in use is not one its publisher endorses.
		// Silently running a bleeding-edge build is the difference between a
		// bug report someone can act on and one nobody can explain.
		if let Some(note) = loader_note(&target) {
			report(LaunchSignal::Log(note));
		}
		ctx.target = Some(target);
		Ok(())
	}
}

/// A remark when the pack's loader was chosen without an endorsement.
///
/// Only Fabric publishes a stability flag per build, so only Fabric can
/// answer this; the others report `None` rather than guessing.
fn loader_note(target: &boot::PackTarget) -> Option<String> {
	if target.loader.as_deref() != Some("fabric") || target.loader_version.is_some() {
		return None;
	}
	let http = packwand_minecraft::UreqClient::default();
	let client = packwand_minecraft::MetadataClient::new(&http, Default::default());
	client
		.choose_fabric_loader(&target.minecraft, None)
		.ok()?
		.note("Fabric loader", &target.minecraft)
}

/// The Java feature release a Minecraft version asks for.
///
/// An already-installed target answers from disk. That path goes through the
/// instance record rather than guessing a filename: `bootstrap` writes the
/// *merged* document, so a Fabric pack's file is
/// `versions/fabric-loader-0.16.5-1.21.1/…json`, not `versions/1.21.1/…json`.
/// Deriving the name from `target.minecraft` would have hit for vanilla and
/// silently missed for every modded pack.
///
/// A target that has never been installed falls back to the network, where
/// `packwand-net`'s cache makes the identical fetch during installation cheap
/// rather than duplicated.
///
/// `None` covers both "this version does not say" — Minecraft states it only
/// from 1.17 on — and "we could not find out". Neither is a reason to refuse
/// to launch; the JVM's own error is the fallback.
fn required_java_major(managed_root: &Path, target: &boot::PackTarget) -> Option<u32> {
	let installed = FsInstanceRepository::new(managed_root.to_path_buf())
		.get(&boot::instance_id_for(target))
		.ok();
	if let Some(record) = installed
		&& let Ok(doc) = version_doc_for(managed_root, &record.classpath)
	{
		return doc.java_version.map(|j| j.major_version);
	}
	let http = packwand_minecraft::UreqClient::default();
	let client = packwand_minecraft::MetadataClient::new(&http, Default::default());
	let manifest = client.fetch_manifest().ok()?;
	let entry = manifest
		.versions
		.iter()
		.find(|v| v.id == target.minecraft)?;
	let doc = client.fetch_version(entry).ok()?;
	doc.value.java_version.map(|j| j.major_version)
}

/// Downloads a Java runtime when nothing on the machine will do.
///
/// Only acts when discovery comes up empty for the required feature release —
/// a machine that already has a suitable JDK never reaches the network here.
pub struct AutoInstallJava {
	/// Where downloaded runtimes are kept.
	runtimes_dir: PathBuf,
}

impl AutoInstallJava {
	/// Installs into `<managed_root>/runtimes`.
	pub fn new(managed_root: &Path) -> Self {
		Self {
			runtimes_dir: managed_root.join("runtimes"),
		}
	}
}

impl LaunchStep for AutoInstallJava {
	fn name(&self) -> &'static str {
		"Checking Java"
	}

	fn run(&mut self, ctx: &mut StepContext, report: &(dyn Fn(LaunchSignal) + Sync)) -> Result<()> {
		// A pinned executable is the user's decision and is not second-guessed.
		if let Some(java) = &ctx.java_override {
			ctx.java_executable = Some(java.clone());
			return Ok(());
		}
		let installed = discover(&DiscoveryConfig::from_host());
		// Without a stated requirement, anything already installed will do;
		// the version document supplies one only from Minecraft 1.17 on.
		let Some(required) = ctx.required_java_major else {
			ctx.java_executable = installed.first().map(|i| i.executable.clone());
			return Ok(());
		};
		if let Ok(found) = select_compatible(&installed, required) {
			ctx.java_executable = Some(found.executable.clone());
			return Ok(());
		}

		report(LaunchSignal::Log(format!(
			"No installed Java {required} was found; downloading one"
		)));
		let client = packwand_net::Client::downloads();
		let catalog = packwand_runtime::Catalog::fetch(&client)
			.map_err(|e| OrchestratorError::new("java", e.to_string()))?;
		let runtime_os = packwand_runtime::runtime_os()
			.map_err(|e| OrchestratorError::new("java", e.to_string()))?;
		let selection = catalog
			.select(runtime_os, required)
			.map_err(|e| OrchestratorError::new("java", e.to_string()))?;
		let dest = self.runtimes_dir.join(&selection.component);
		report(LaunchSignal::Log(format!(
			"Installing {} {}",
			selection.component, selection.version
		)));
		let executable = packwand_runtime::install_runtime(
			&client,
			&selection,
			&dest,
			packwand_parallel::configured(),
			&|progress| {
				let fraction = if progress.total == 0 {
					0.0
				} else {
					progress.finished as f64 / progress.total as f64
				};
				report(LaunchSignal::Progress(
					fraction,
					Some(format!("Java {}/{}", progress.finished, progress.total)),
				));
			},
		)
		.map_err(|e| OrchestratorError::new("java", e.to_string()))?;
		ctx.java_executable = Some(executable);
		Ok(())
	}
}

/// Confirms the chosen JVM exists, runs, and is new enough for this version.
pub struct VerifyJava;

impl LaunchStep for VerifyJava {
	fn name(&self) -> &'static str {
		"Verifying Java"
	}

	fn run(&mut self, ctx: &mut StepContext, report: &(dyn Fn(LaunchSignal) + Sync)) -> Result<()> {
		let Some(executable) = ctx.java_executable.clone() else {
			return Err(OrchestratorError::new(
				"java",
				"no Java installation was found; set one in the instance's settings",
			));
		};
		let probed = ProbeCache::shared()
			.probe(&executable)
			.map_err(|e| OrchestratorError::new("java", e.to_string()))?;
		if let Some(required) = ctx.required_java_major
			&& probed.version.major < required
		{
			// Caught here rather than at spawn: the JVM's own error for this
			// is `UnsupportedClassVersionError` from deep inside the game.
			return Err(OrchestratorError::new(
				"java",
				format!(
					"{} is Java {}, but this version needs Java {required} or newer",
					executable.display(),
					probed.version
				),
			));
		}
		report(LaunchSignal::Log(format!(
			"Java {} at {}",
			probed.version,
			executable.display()
		)));
		Ok(())
	}
}

/// Ensures the shared managed install for this version exists, then resolves
/// the instance record the launch plan is built from.
pub struct InstallContent;

impl LaunchStep for InstallContent {
	fn name(&self) -> &'static str {
		"Installing game files"
	}

	fn run(&mut self, ctx: &mut StepContext, report: &(dyn Fn(LaunchSignal) + Sync)) -> Result<()> {
		let target = ctx
			.target
			.clone()
			.ok_or_else(|| OrchestratorError::new("internal", "version was not resolved"))?;
		let record = boot::ensure_instance(
			&ctx.managed_root,
			&target,
			ctx.java_executable.clone(),
			|update| {
				let fraction = if update.total_downloads == 0 {
					0.0
				} else {
					update.finished_downloads as f64 / update.total_downloads as f64
				};
				report(LaunchSignal::Progress(
					fraction,
					Some(format!(
						"{}/{} downloads",
						update.finished_downloads, update.total_downloads
					)),
				));
			},
		)
		.map_err(|e| OrchestratorError::new("bootstrap", e.to_string()))?;
		ctx.record = Some(record);
		Ok(())
	}
}

/// Unpacks the version's native libraries into a directory owned by this
/// launch, and deletes it again when the launch ends.
///
/// Native libraries used to be unpacked once, at install time, into a
/// directory shared by every instance on that version — where they stayed
/// forever and where two simultaneous launches wrote over each other. Doing
/// it per launch costs a few milliseconds and makes both problems go away.
pub struct ExtractNatives;

/// Locates the version document behind an instance record.
///
/// `InstallPlan` documents that the client jar is last on the classpath, and
/// it lives at `versions/<id>/<id>.jar` beside the `<id>.json` that produced
/// it. That is the link from a record back to the document it was built from.
fn version_doc_for(managed_root: &Path, classpath: &[PathBuf]) -> Result<VersionDoc> {
	let client_jar = classpath
		.last()
		.ok_or_else(|| OrchestratorError::new("internal", "instance record has no classpath"))?;
	let version_id = client_jar
		.file_stem()
		.and_then(|s| s.to_str())
		.ok_or_else(|| {
			OrchestratorError::new(
				"internal",
				format!("cannot read a version id from {}", client_jar.display()),
			)
		})?;
	let path = managed_root
		.join("versions")
		.join(version_id)
		.join(format!("{version_id}.json"));
	let bytes = std::fs::read(&path).map_err(|e| {
		OrchestratorError::new("metadata", format!("cannot read {}: {e}", path.display()))
	})?;
	serde_json::from_slice(&bytes).map_err(|e| {
		OrchestratorError::new("metadata", format!("cannot parse {}: {e}", path.display()))
	})
}

/// Unpacks the native libraries an instance record needs into `dest`,
/// returning how many archives were expanded.
///
/// The one place natives are unpacked. Installation deliberately does not do
/// it (see `bootstrap`), so anything that builds a launch plan — the step
/// machine, the core probe — has to call this first or hand the JVM an empty
/// `java.library.path`.
pub fn extract_natives_into(
	managed_root: &Path,
	classpath: &[PathBuf],
	dest: &Path,
) -> Result<usize> {
	// A record with no classpath describes no version document and therefore
	// no native libraries — a hand-written fixture, or an instance whose main
	// class comes from somewhere else entirely. Nothing to unpack is a
	// legitimate answer, not a failure to unpack.
	if classpath.is_empty() {
		return Ok(0);
	}
	let doc = version_doc_for(managed_root, classpath)?;
	std::fs::create_dir_all(dest).map_err(|e| {
		OrchestratorError::new("io", format!("failed to create {}: {e}", dest.display()))
	})?;
	// Only `natives_dir` is a real destination here; the other entries name
	// where the archives were already downloaded to.
	let layout = InstallLayout {
		versions_dir: managed_root.join("versions"),
		libraries_dir: managed_root.join("libraries"),
		assets_dir: managed_root.join("assets"),
		natives_dir: dest.to_path_buf(),
		resources_dir: None,
	};
	let plan = build_version_plan(&doc, &Host::current(), &layout)
		.map_err(|e| OrchestratorError::new("metadata", e.to_string()))?;
	let mut extracted = 0usize;
	for extraction in &plan.extractions {
		packwand_minecraft::extract_natives(extraction)
			.map_err(|e| OrchestratorError::new("natives", e.to_string()))?;
		extracted += 1;
	}
	Ok(extracted)
}

/// This launch's private natives directory under a game directory.
pub fn launch_natives_dir(game_dir: &Path) -> PathBuf {
	game_dir
		.join(".packwand")
		.join(format!("natives-{}", std::process::id()))
}

impl LaunchStep for ExtractNatives {
	fn name(&self) -> &'static str {
		"Extracting native libraries"
	}

	fn run(&mut self, ctx: &mut StepContext, report: &(dyn Fn(LaunchSignal) + Sync)) -> Result<()> {
		let record = ctx
			.record
			.clone()
			.ok_or_else(|| OrchestratorError::new("internal", "instance was not resolved"))?;
		let natives_dir = launch_natives_dir(&ctx.game_dir);
		let extracted = extract_natives_into(&ctx.managed_root, &record.classpath, &natives_dir)?;
		report(LaunchSignal::Log(format!(
			"Extracted {extracted} native librar{} for this run",
			if extracted == 1 { "y" } else { "ies" }
		)));
		ctx.natives_dir = Some(natives_dir);
		Ok(())
	}

	fn finalize(
		&mut self,
		ctx: &mut StepContext,
		_outcome: Outcome,
		report: &(dyn Fn(LaunchSignal) + Sync),
	) {
		let Some(dir) = ctx.natives_dir.take() else {
			return;
		};
		if let Err(e) = std::fs::remove_dir_all(&dir)
			&& e.kind() != std::io::ErrorKind::NotFound
		{
			// Worth saying out loud, never worth failing a finished launch.
			report(LaunchSignal::Log(format!(
				"warning: could not remove {}: {e}",
				dir.display()
			)));
		}
	}
}

/// Builds the launch plan the supervisor will run.
pub struct BuildPlan;

impl LaunchStep for BuildPlan {
	fn name(&self) -> &'static str {
		"Preparing launch"
	}

	fn run(
		&mut self,
		ctx: &mut StepContext,
		_report: &(dyn Fn(LaunchSignal) + Sync),
	) -> Result<()> {
		let record = ctx
			.record
			.clone()
			.ok_or_else(|| OrchestratorError::new("internal", "instance was not resolved"))?;
		let target = ctx
			.target
			.clone()
			.ok_or_else(|| OrchestratorError::new("internal", "version was not resolved"))?;
		let managed = FsInstanceRepository::new(ctx.managed_root.clone())
			.instance_paths(&boot::instance_id_for(&target));
		let paths = InstancePaths {
			game_dir: ctx.game_dir.clone(),
			logs_dir: ctx.game_dir.join("logs"),
			// This launch's own directory, not the shared one.
			natives_dir: ctx
				.natives_dir
				.clone()
				.unwrap_or(managed.natives_dir.clone()),
			assets_dir: managed.assets_dir,
			libraries_dir: managed.libraries_dir,
		};
		let mut plan = build_launch_plan(&record, &paths);
		if let Some(java) = &ctx.java_executable {
			plan.java_executable.clone_from(java);
		}
		ctx.plan = Some(plan);
		Ok(())
	}
}

/// Writes a short summary of what is about to run.
///
/// Prism's equivalent exists because almost every support thread starts with
/// someone pasting the top of a log and needing the version, the Java, and
/// the directories to be in it.
pub struct PrintInstanceInfo;

impl LaunchStep for PrintInstanceInfo {
	fn name(&self) -> &'static str {
		"Launching"
	}

	fn run(&mut self, ctx: &mut StepContext, report: &(dyn Fn(LaunchSignal) + Sync)) -> Result<()> {
		let Some(plan) = &ctx.plan else {
			return Err(OrchestratorError::new("internal", "plan was not built"));
		};
		report(LaunchSignal::Log(format!(
			"Instance {} | main class {} | game dir {}",
			plan.instance_id,
			plan.main_class,
			plan.working_dir.display()
		)));
		if let Some(natives) = &ctx.natives_dir {
			report(LaunchSignal::Log(format!("Natives: {}", natives.display())));
		}
		Ok(())
	}
}

/// The launch pipeline, in order.
pub fn standard_steps(managed_root: &Path) -> Vec<Box<dyn LaunchStep>> {
	vec![
		Box::new(CreateGameFolders),
		Box::new(ResolveVersion),
		Box::new(AutoInstallJava::new(managed_root)),
		Box::new(VerifyJava),
		Box::new(InstallContent),
		Box::new(ExtractNatives),
		Box::new(BuildPlan),
		Box::new(PrintInstanceInfo),
	]
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn game_folders_are_created_and_the_step_is_repeatable() {
		let dir = tempfile::tempdir().unwrap();
		let mut ctx = StepContext::new(
			dir.path().join("managed"),
			dir.path().join("pack"),
			dir.path().join("game"),
		);
		let mut step = CreateGameFolders;
		step.run(&mut ctx, &|_| {}).unwrap();
		assert!(ctx.game_dir.join("mods").is_dir());
		// A relaunch runs the same steps over an instance that already exists.
		step.run(&mut ctx, &|_| {}).unwrap();
	}

	#[test]
	fn launch_scratch_never_lands_in_the_backing_pack() {
		// The pack directory is the user's source tree and is usually a git
		// checkout. Anything a launch writes belongs beside it, never in it —
		// this is the easiest property to lose while moving orchestration
		// around, so it is pinned rather than assumed.
		let dir = tempfile::tempdir().unwrap();
		let repo = packwand_instance::FsUserInstanceRepository::new(dir.path().to_path_buf());
		let game_dir = repo.instance_dir("demo").unwrap();
		let owned_pack = repo.owned_pack_dir("demo").unwrap();
		let natives = launch_natives_dir(&game_dir);
		assert!(!natives.starts_with(&owned_pack), "{}", natives.display());
		assert!(natives.starts_with(&game_dir));

		// A linked instance's pack lives entirely outside the launcher root.
		let linked = dir.path().join("elsewhere").join("my-pack");
		assert!(!natives.starts_with(&linked));
	}

	#[test]
	fn stale_natives_from_a_crashed_run_are_swept() {
		// `finalize` only runs if the process survives; a killed launcher
		// leaves its directory behind under a pid that gets reused.
		let dir = tempfile::tempdir().unwrap();
		let game_dir = dir.path().join("game");
		let stale = game_dir.join(".packwand").join("natives-999999");
		std::fs::create_dir_all(&stale).unwrap();
		std::fs::write(stale.join("old.dll"), b"stale").unwrap();
		let mine = launch_natives_dir(&game_dir);
		std::fs::create_dir_all(&mine).unwrap();

		let mut ctx = StepContext::new(
			dir.path().join("managed"),
			dir.path().join("pack"),
			game_dir,
		);
		CreateGameFolders.run(&mut ctx, &|_| {}).unwrap();
		assert!(!stale.exists(), "a previous run's natives survived");
		assert!(mine.exists(), "this run's own directory was swept");
	}

	#[test]
	fn natives_are_removed_when_the_launch_ends() {
		// The property that did not exist before: whatever the run unpacked
		// is gone afterwards, on success and on failure alike.
		let dir = tempfile::tempdir().unwrap();
		let mut ctx = StepContext::new(
			dir.path().join("managed"),
			dir.path().join("pack"),
			dir.path().join("game"),
		);
		let natives = ctx.game_dir.join(".packwand").join("natives-test");
		std::fs::create_dir_all(&natives).unwrap();
		std::fs::write(natives.join("lwjgl.dll"), b"stub").unwrap();
		ctx.natives_dir = Some(natives.clone());

		ExtractNatives.finalize(&mut ctx, Outcome::Failed, &|_| {});
		assert!(!natives.exists(), "extracted natives outlived the launch");
		assert!(ctx.natives_dir.is_none());

		// Finalizing again must not fail: a step may be finalized after a
		// `run` that never got as far as creating the directory.
		ExtractNatives.finalize(&mut ctx, Outcome::Succeeded, &|_| {});
	}

	#[test]
	fn verify_java_rejects_a_jvm_older_than_the_version_needs() {
		let dir = tempfile::tempdir().unwrap();
		let home = dir.path().join("jdk-8");
		std::fs::create_dir_all(home.join("bin")).unwrap();
		let executable = home
			.join("bin")
			.join(if cfg!(windows) { "java.exe" } else { "java" });
		std::fs::write(&executable, b"stub").unwrap();
		std::fs::write(home.join("release"), "JAVA_VERSION=\"1.8.0_392\"\n").unwrap();

		let mut ctx = StepContext::new(
			dir.path().join("managed"),
			dir.path().join("pack"),
			dir.path().join("game"),
		);
		ctx.java_executable = Some(executable);
		ctx.required_java_major = Some(21);
		let error = VerifyJava.run(&mut ctx, &|_| {}).unwrap_err();
		assert_eq!(error.kind, "java");
		assert!(error.message.contains("Java 21"), "{}", error.message);

		// The same JVM is fine where the version does not ask for more.
		ctx.required_java_major = Some(8);
		VerifyJava.run(&mut ctx, &|_| {}).unwrap();
	}

	#[test]
	fn a_missing_java_is_a_clear_error_rather_than_a_spawn_failure() {
		let dir = tempfile::tempdir().unwrap();
		let mut ctx = StepContext::new(
			dir.path().join("managed"),
			dir.path().join("pack"),
			dir.path().join("game"),
		);
		let error = VerifyJava.run(&mut ctx, &|_| {}).unwrap_err();
		assert_eq!(error.kind, "java");
		assert!(error.message.contains("settings"), "{}", error.message);
	}
}
