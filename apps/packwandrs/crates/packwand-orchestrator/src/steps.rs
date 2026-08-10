//! Launching as an ordered sequence of reversible steps.
//!
//! A launch used to be one function that did everything and returned. That
//! shape has no place to put undo: anything a step created on the way to the
//! game — a directory, an extracted native library, a staged file — stayed
//! there whether the launch succeeded, failed halfway, or was cancelled.
//!
//! The fix is the ordering property rather than any individual step: steps run
//! forward, and every step that *started* is finalized in reverse order
//! afterwards, on success and on failure alike. A step therefore owns its own
//! cleanup next to the code that created the mess, instead of a single error
//! path at the bottom trying to guess how far the launch got.
//!
//! Only steps that ran are finalized. A step that never started has nothing
//! to undo, and calling it would be asking it to clean up state it did not
//! create.

use std::path::PathBuf;

use packwand_instance::InstanceRecord;
use packwand_launch::LaunchPlan;

use crate::error::Result;
use crate::launch::LaunchSignal;

/// How the forward pass ended, handed to every `finalize`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
	/// Every step ran without error.
	Succeeded,
	/// A step returned an error, or the launch was cancelled.
	Failed,
}

impl Outcome {
	/// Whether the forward pass completed.
	pub fn succeeded(self) -> bool {
		matches!(self, Self::Succeeded)
	}
}

/// State threaded through a launch.
///
/// Fields are filled in as steps run, so a later step reads what an earlier
/// one resolved. `Option` here means "not produced yet" rather than
/// "optional": a step that needs a value a predecessor should have set treats
/// its absence as a bug and says so.
pub struct StepContext {
	/// Shared root for libraries, assets, and version metadata.
	pub managed_root: PathBuf,
	/// The pack supplying the version and content.
	pub pack_dir: PathBuf,
	/// Minecraft's writable directory for this instance.
	pub game_dir: PathBuf,
	/// Java executable chosen by settings, if the user pinned one.
	pub java_override: Option<PathBuf>,
	/// Minecraft version and loader, once `pack.toml` has been read.
	pub target: Option<crate::pack_target::PackTarget>,
	/// The feature release the version document asks for, once known.
	pub required_java_major: Option<u32>,
	/// The Java executable the launch will actually use.
	pub java_executable: Option<PathBuf>,
	/// The resolved instance record.
	pub record: Option<InstanceRecord>,
	/// The plan handed to the supervisor.
	pub plan: Option<LaunchPlan>,
	/// Per-launch directory holding extracted native libraries.
	pub natives_dir: Option<PathBuf>,
	/// Exit code of the game, once it has run.
	pub exit_code: Option<i32>,
}

impl StepContext {
	/// A context with nothing resolved yet.
	pub fn new(managed_root: PathBuf, pack_dir: PathBuf, game_dir: PathBuf) -> Self {
		Self {
			managed_root,
			pack_dir,
			game_dir,
			java_override: None,
			target: None,
			required_java_major: None,
			java_executable: None,
			record: None,
			plan: None,
			natives_dir: None,
			exit_code: None,
		}
	}
}

/// One reversible stage of a launch.
pub trait LaunchStep: Send {
	/// Name used in progress reporting and error messages.
	fn name(&self) -> &'static str;

	/// Does the step's work. Returning an error stops the forward pass.
	fn run(&mut self, ctx: &mut StepContext, report: &(dyn Fn(LaunchSignal) + Sync)) -> Result<()>;

	/// Undoes whatever `run` created.
	///
	/// Called for every step that started, in reverse order, whether the
	/// launch succeeded or failed — so it must tolerate a partially completed
	/// `run` and must not fail the launch. Errors are reported, not returned:
	/// a cleanup problem is worth telling the user about and is never a reason
	/// to turn a successful launch into a failed one.
	fn finalize(
		&mut self,
		_ctx: &mut StepContext,
		_outcome: Outcome,
		_report: &(dyn Fn(LaunchSignal) + Sync),
	) {
	}
}

/// How far a forward pass got, and whether it succeeded.
///
/// `started` is what [`unwind`] needs: the steps that ran are exactly the
/// ones with something to undo.
pub struct Forward {
	/// Number of steps whose `run` was entered.
	pub started: usize,
	/// The first error, if the pass did not complete.
	pub result: Result<()>,
}

/// Runs `steps` in order, stopping at the first error or cancellation.
///
/// Deliberately separate from [`unwind`]: a launch has to spawn the game and
/// wait for it to exit *between* the two, because a step's cleanup — deleting
/// this run's native libraries, for one — would pull the ground out from
/// under a game that is still running.
pub fn run_forward(
	steps: &mut [Box<dyn LaunchStep>],
	ctx: &mut StepContext,
	is_cancelled: &(dyn Fn() -> bool + Sync),
	report: &(dyn Fn(LaunchSignal) + Sync),
) -> Forward {
	let mut started = 0usize;
	let mut result = Ok(());
	for step in steps.iter_mut() {
		if is_cancelled() {
			result = Err(crate::error::OrchestratorError::new(
				"cancelled",
				"job was cancelled",
			));
			break;
		}
		started += 1;
		report(LaunchSignal::Status(
			"starting",
			Some(step.name().into()),
			None,
		));
		if let Err(error) = step.run(ctx, report) {
			result = Err(error);
			break;
		}
	}
	Forward { started, result }
}

/// Finalizes the first `started` steps in reverse order.
pub fn unwind(
	steps: &mut [Box<dyn LaunchStep>],
	started: usize,
	ctx: &mut StepContext,
	outcome: Outcome,
	report: &(dyn Fn(LaunchSignal) + Sync),
) {
	let started = started.min(steps.len());
	for step in steps[..started].iter_mut().rev() {
		step.finalize(ctx, outcome, report);
	}
}

/// Forward pass immediately followed by the unwind.
///
/// The forward error, if any, is what the caller sees; a `finalize` cannot
/// replace it, because the first failure is the one that explains the launch.
pub fn run_steps(
	steps: &mut [Box<dyn LaunchStep>],
	ctx: &mut StepContext,
	is_cancelled: &(dyn Fn() -> bool + Sync),
	report: &(dyn Fn(LaunchSignal) + Sync),
) -> Result<()> {
	let forward = run_forward(steps, ctx, is_cancelled, report);
	let outcome = if forward.result.is_ok() {
		Outcome::Succeeded
	} else {
		Outcome::Failed
	};
	unwind(steps, forward.started, ctx, outcome, report);
	forward.result
}

#[cfg(test)]
mod tests {
	use super::*;
	use std::sync::{Arc, Mutex};

	/// Records the order in which its hooks fire into a shared log.
	struct Recorder {
		name: &'static str,
		fail: bool,
		log: Arc<Mutex<Vec<String>>>,
	}

	impl LaunchStep for Recorder {
		fn name(&self) -> &'static str {
			self.name
		}

		fn run(
			&mut self,
			_ctx: &mut StepContext,
			_report: &(dyn Fn(LaunchSignal) + Sync),
		) -> Result<()> {
			self.log.lock().unwrap().push(format!("run {}", self.name));
			if self.fail {
				return Err(crate::error::OrchestratorError::new("test", "boom"));
			}
			Ok(())
		}

		fn finalize(
			&mut self,
			_ctx: &mut StepContext,
			outcome: Outcome,
			_report: &(dyn Fn(LaunchSignal) + Sync),
		) {
			self.log
				.lock()
				.unwrap()
				.push(format!("finalize {} ({outcome:?})", self.name));
		}
	}

	fn context() -> StepContext {
		StepContext::new("managed".into(), "pack".into(), "game".into())
	}

	fn steps(
		log: &Arc<Mutex<Vec<String>>>,
		failing: Option<&'static str>,
	) -> Vec<Box<dyn LaunchStep>> {
		["one", "two", "three"]
			.into_iter()
			.map(|name| {
				Box::new(Recorder {
					name,
					fail: failing == Some(name),
					log: Arc::clone(log),
				}) as Box<dyn LaunchStep>
			})
			.collect()
	}

	#[test]
	fn success_finalizes_every_step_in_reverse() {
		let log = Arc::new(Mutex::new(Vec::new()));
		let mut steps = steps(&log, None);
		run_steps(&mut steps, &mut context(), &|| false, &|_| {}).unwrap();
		assert_eq!(
			*log.lock().unwrap(),
			[
				"run one",
				"run two",
				"run three",
				"finalize three (Succeeded)",
				"finalize two (Succeeded)",
				"finalize one (Succeeded)",
			]
		);
	}

	#[test]
	fn a_failure_still_unwinds_and_skips_steps_that_never_ran() {
		// The property the old single-function launch could not have: the
		// steps that got far enough to create something are the exact set
		// that gets to clean it up, newest first.
		let log = Arc::new(Mutex::new(Vec::new()));
		let mut steps = steps(&log, Some("two"));
		let error = run_steps(&mut steps, &mut context(), &|| false, &|_| {}).unwrap_err();
		assert_eq!(error.kind, "test");
		assert_eq!(
			*log.lock().unwrap(),
			[
				"run one",
				"run two",
				"finalize two (Failed)",
				"finalize one (Failed)",
			],
			"a step that never ran must not be finalized"
		);
	}

	#[test]
	fn cancellation_before_a_step_unwinds_what_already_ran() {
		let log = Arc::new(Mutex::new(Vec::new()));
		let mut steps = steps(&log, None);
		// Cancelled from the outset: nothing runs, so nothing is finalized.
		let error = run_steps(&mut steps, &mut context(), &|| true, &|_| {}).unwrap_err();
		assert_eq!(error.kind, "cancelled");
		assert!(log.lock().unwrap().is_empty());
	}
}
