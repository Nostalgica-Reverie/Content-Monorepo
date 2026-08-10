use std::path::{Path, PathBuf};
use std::sync::mpsc::RecvTimeoutError;
use std::time::Duration;

use packwand_instance::{FsUserInstanceRepository, InstanceSettings};
use packwand_launch::{LaunchEvent, LaunchPlan};

use crate::error::{OrchestratorError, Result};
use crate::install;
use crate::paths::{backing_pack, now_ms};
use crate::steps::{Outcome, StepContext};

/// What a running launch reports back.
///
/// A plain enum over a callback rather than a channel type, so this crate
/// stays independent of whatever async runtime the host uses.
#[derive(Debug, Clone)]
pub enum LaunchSignal {
	/// One line of game output.
	Log(String),
	/// Fractional progress with an optional label.
	Progress(f64, Option<String>),
	/// A lifecycle transition: phase, message, exit code.
	Status(&'static str, Option<String>, Option<i32>),
}

/// Everything a launch needs that is not on the instance record.
pub struct LaunchRequest<'a> {
	/// Where shared libraries, assets and natives live.
	pub managed_root: &'a Path,
	/// Download width when the instance does not override it.
	pub default_jobs: usize,
	/// The instance's settings already merged over the app's defaults.
	pub settings: &'a InstanceSettings,
	/// Azure app registration this build signs in as. `None` means no
	/// Microsoft sign-in is configured, and launches run offline.
	pub msa_client_id: Option<String>,
}

/// Folds per-instance overrides into a plan.
///
/// Every field is additive: an unset override leaves whatever the instance
/// record was built with. Assigning unconditionally here would mean a `None`
/// silently discarding a baked value, which is very hard to diagnose from
/// outside.
pub fn apply_settings(plan: &mut LaunchPlan, settings: &InstanceSettings) {
	if let Some(java) = &settings.java_path {
		plan.java_executable.clone_from(java);
	}
	if settings.memory_min_mb.is_some() {
		plan.memory.initial_mb = settings.memory_min_mb;
	}
	if settings.memory_max_mb.is_some() {
		plan.memory.max_mb = settings.memory_max_mb;
	}
	if let Some(args) = &settings.extra_jvm_args {
		plan.jvm_args.extend(args.iter().cloned());
	}
	if let Some(args) = &settings.extra_game_args {
		plan.game_args.extend(args.iter().cloned());
	}
	if let Some(env) = &settings.env {
		plan.env.extend(env.clone());
	}
	if let (Some(width), Some(height)) = (settings.window_width, settings.window_height) {
		plan.game_args.extend([
			"--width".to_owned(),
			width.to_string(),
			"--height".to_owned(),
			height.to_string(),
		]);
	}
	if settings.fullscreen == Some(true) {
		plan.game_args.push("--fullscreen".to_owned());
	}
}

/// Installs, boots and supervises one instance, returning when the game exits.
///
/// Blocking for its whole duration — the caller is expected to be on a
/// blocking thread and to bridge `on_signal` to wherever its UI lives.
pub fn run(
	repo: &FsUserInstanceRepository,
	id: &str,
	request: &LaunchRequest<'_>,
	is_cancelled: &(dyn Fn() -> bool + Sync),
	on_signal: &(dyn Fn(LaunchSignal) + Sync),
) -> Result<()> {
	on_signal(LaunchSignal::Status(
		"starting",
		Some("Installing pack contents".into()),
		None,
	));
	let mut instance = install::install(repo, id, request.default_jobs)?;
	if is_cancelled() {
		return Err(OrchestratorError::new("cancelled", "job was cancelled"));
	}
	let pack_dir = backing_pack(repo, &instance)?;
	let game_dir = repo.instance_dir(id)?;
	let session =
		crate::boot::session_for_launch(request.msa_client_id.as_deref(), request.managed_root)
			.map_err(|error| OrchestratorError::new("auth", error))?;
	if let Some(note) = &session.note {
		on_signal(LaunchSignal::Log(note.clone()));
	}

	// Everything up to the spawn runs as reversible steps, so a failure or a
	// cancellation unwinds what it created instead of leaving it behind.
	let mut steps = crate::stages::standard_steps(request.managed_root);
	let mut ctx = StepContext::new(request.managed_root.to_path_buf(), pack_dir, game_dir);
	ctx.java_override = request.settings.java_path.clone();
	let forward = crate::steps::run_forward(&mut steps, &mut ctx, is_cancelled, on_signal);

	// The unwind waits for the game: a step's cleanup removes this launch's
	// native libraries, and doing that while the JVM has them mapped would
	// crash a running game.
	let result = forward.result.and_then(|()| {
		run_prepared(
			repo,
			&mut instance,
			&mut ctx,
			request,
			&session,
			is_cancelled,
			on_signal,
		)
	});
	let outcome = if result.is_ok() {
		Outcome::Succeeded
	} else {
		Outcome::Failed
	};
	crate::steps::unwind(&mut steps, forward.started, &mut ctx, outcome, on_signal);
	result
}

/// Spawns the game and pumps its events until it exits.
fn run_prepared(
	repo: &FsUserInstanceRepository,
	instance: &mut packwand_instance::Instance,
	ctx: &mut StepContext,
	request: &LaunchRequest<'_>,
	session: &crate::boot::LaunchSession,
	is_cancelled: &(dyn Fn() -> bool + Sync),
	on_signal: &(dyn Fn(LaunchSignal) + Sync),
) -> Result<()> {
	let mut plan = ctx
		.plan
		.clone()
		.ok_or_else(|| OrchestratorError::new("internal", "no launch plan was built"))?;
	let record = ctx
		.record
		.clone()
		.ok_or_else(|| OrchestratorError::new("internal", "no instance record was resolved"))?;
	apply_settings(&mut plan, request.settings);

	// The record names which placeholders its arguments use; the session
	// supplies the values, and only at spawn time.
	let secrets = if record.session_placeholders.is_empty() {
		std::collections::BTreeMap::new()
	} else {
		session.session.secrets()
	};
	let identity = if record.identity_placeholders.is_empty() {
		std::collections::BTreeMap::new()
	} else {
		session.session.identity()
	};

	let handle = packwand_launch::launch(
		&plan,
		packwand_launch::LaunchOptions {
			secrets,
			identity,
			..Default::default()
		},
	)
	.map_err(|error| OrchestratorError::new("launch", error))?;
	instance.last_played_ms = Some(now_ms());
	repo.write(instance)?;

	// Bounded on purpose: a chatty modpack logs faster than anyone reads, and
	// the only lines worth keeping when a run fails are the recent ones.
	let mut parser = packwand_launch::LogParser::new();
	let mut recent = packwand_launch::LogBuffer::new(LOG_BUFFER_LINES);
	loop {
		match handle.events().recv_timeout(Duration::from_millis(250)) {
			Ok(LaunchEvent::Started { pid, .. }) => on_signal(LaunchSignal::Status(
				"running",
				Some(format!("Running (pid {pid})")),
				None,
			)),
			Ok(LaunchEvent::Stdout { line, .. } | LaunchEvent::Stderr { line, .. }) => {
				// Already censored by the supervisor; parsing only adds
				// structure, so a stack trace keeps one level throughout.
				// The newline is added back because the event carries one
				// line with its terminator already stripped, and the parser
				// needs it to know the line is complete.
				let mut parsed = parser.feed(&line);
				parsed.extend(parser.feed("\n"));
				for entry in parsed {
					recent.push(entry);
				}
				on_signal(LaunchSignal::Log(line));
			}
			Ok(LaunchEvent::Exited { code, .. }) => {
				let okay = code == Some(0);
				if !okay {
					report_failure_context(&plan, &recent, on_signal);
				}
				on_signal(LaunchSignal::Status(
					if okay { "stopped" } else { "error" },
					Some(format!("Exited with code {code:?}")),
					code,
				));
				if !okay {
					return Err(OrchestratorError::new(
						"exit_code",
						format!("Minecraft exited with code {code:?}"),
					));
				}
				break;
			}
			Ok(LaunchEvent::Failed { error, .. }) => {
				return Err(OrchestratorError::new("launch", error));
			}
			Ok(LaunchEvent::Cancelled { .. }) => {
				return Err(OrchestratorError::new("cancelled", "job was cancelled"));
			}
			Ok(LaunchEvent::Starting { .. }) => {}
			// The timeout is the only place cancellation can be noticed while
			// the game is quiet.
			Err(RecvTimeoutError::Timeout) if is_cancelled() => handle.cancel(),
			Err(RecvTimeoutError::Timeout) => {}
			Err(RecvTimeoutError::Disconnected) => break,
		}
	}
	Ok(())
}

/// How many parsed log lines are kept for failure reporting.
const LOG_BUFFER_LINES: usize = 2000;

/// How many problem lines to surface when a run fails.
const REPORTED_PROBLEM_LINES: usize = 20;

/// Explains a failed exit with what the log actually said.
///
/// A bare exit code is the least useful thing a launcher can show. Three
/// things turn it into something actionable: the errors already seen on the
/// pipe, the crash report the game wrote (which never appears on the pipe at
/// all), and `latest.log` for a run that failed before the supervisor saw
/// anything.
fn report_failure_context(
	plan: &LaunchPlan,
	recent: &packwand_launch::LogBuffer,
	on_signal: &(dyn Fn(LaunchSignal) + Sync),
) {
	let problems: Vec<&packwand_launch::LogLine> = recent
		.lines()
		.filter(|line| line.level.is_problem())
		.collect();
	let tail = problems.len().saturating_sub(REPORTED_PROBLEM_LINES);
	if !problems.is_empty() {
		on_signal(LaunchSignal::Log(
			"--- errors and warnings from this run ---".to_string(),
		));
		for line in &problems[tail..] {
			on_signal(LaunchSignal::Log(format!(
				"[{:?}] {}",
				line.level, line.message
			)));
		}
	} else if let Some(lines) = packwand_launch::read_latest_log(&plan.paths.logs) {
		// Nothing on the pipe: a JVM that dies before log4j is configured
		// still leaves the file behind.
		on_signal(LaunchSignal::Log(
			"--- from logs/latest.log ---".to_string(),
		));
		for line in lines
			.iter()
			.filter(|l| l.level.is_problem())
			.rev()
			.take(REPORTED_PROBLEM_LINES)
			.collect::<Vec<_>>()
			.into_iter()
			.rev()
		{
			on_signal(LaunchSignal::Log(format!(
				"[{:?}] {}",
				line.level, line.message
			)));
		}
	}
	if let Some(report) = packwand_launch::latest_crash_report(&plan.working_dir) {
		on_signal(LaunchSignal::Log(format!(
			"Crash report: {}",
			report.display()
		)));
	}
}

/// The shared managed root for the launcher's install cache.
pub fn managed_root(app_data_dir: &Path) -> PathBuf {
	crate::boot::default_managed_root(app_data_dir)
}

#[cfg(test)]
mod tests {
	use super::*;
	use packwand_instance::MemoryLimits;

	fn plan_with_memory(initial_mb: Option<u32>, max_mb: Option<u32>) -> LaunchPlan {
		let dir = PathBuf::from("instance");
		LaunchPlan {
			schema_version: packwand_launch::PLAN_SCHEMA_VERSION,
			instance_id: "memory".into(),
			working_dir: dir.clone(),
			java_executable: PathBuf::from("java"),
			jvm_args: Vec::new(),
			classpath: Vec::new(),
			classpath_separator: packwand_launch::host_classpath_separator().into(),
			main_class: "net.minecraft.client.main.Main".into(),
			game_args: Vec::new(),
			env: std::collections::BTreeMap::new(),
			memory: MemoryLimits { initial_mb, max_mb },
			session: std::collections::BTreeMap::new(),
			identity: std::collections::BTreeMap::new(),
			paths: packwand_launch::LaunchPaths {
				logs: dir.join("logs"),
				natives: dir.join("natives"),
				assets: dir.join("assets"),
				libraries: dir.join("libraries"),
				game_data: dir,
			},
		}
	}

	#[test]
	fn unset_overrides_keep_the_records_baked_limits() {
		let mut plan = plan_with_memory(Some(1024), Some(4096));
		apply_settings(&mut plan, &InstanceSettings::default());
		assert_eq!(plan.memory.initial_mb, Some(1024));
		assert_eq!(plan.memory.max_mb, Some(4096));

		apply_settings(
			&mut plan,
			&InstanceSettings {
				memory_max_mb: Some(8192),
				..InstanceSettings::default()
			},
		);
		assert_eq!(plan.memory.initial_mb, Some(1024), "still untouched");
		assert_eq!(plan.memory.max_mb, Some(8192));
	}

	#[test]
	fn window_and_fullscreen_overrides_reach_the_game_arguments() {
		let mut plan = plan_with_memory(None, None);
		apply_settings(
			&mut plan,
			&InstanceSettings {
				window_width: Some(1280),
				window_height: Some(720),
				fullscreen: Some(true),
				..InstanceSettings::default()
			},
		);
		assert_eq!(
			plan.game_args,
			["--width", "1280", "--height", "720", "--fullscreen"]
		);
	}
}
