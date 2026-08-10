//// First-run setup: which step the user is on, and which they may reach.
////
//// The rules are small but each has a wrong version that looks right. Only
//// the workspace step is mandatory — the router already refuses every other
//// route without one — so the git and credential steps must stay skippable
//// *and* re-enterable from Settings afterwards. A flow that let the user
//// finish without a workspace, or that trapped them on an optional step,
//// would both be plausible readings of "three steps".
////
//// Lives here rather than in the Vue component because that is where it can
//// be tested exhaustively, and because Settings re-enters the same steps: two
//// call sites, one set of rules.

import gleam/list

pub type Step {
  Workspace
  Repository
  Credentials
  Done
}

/// How the user resolved the repository step.
pub type RepositoryChoice {
  /// Use the git repository already present at the workspace path.
  LinkExisting
  /// `git init` in the workspace.
  InitNew
  /// Clone a remote into the workspace.
  CloneRemote
  /// Explicitly declined. Distinct from "not yet answered".
  SkipRepository
}

pub type Model {
  Model(
    step: Step,
    workspace: String,
    repository: List(RepositoryChoice),
    modrinth_linked: Bool,
    curseforge_linked: Bool,
  )
}

pub type Message {
  WorkspaceChosen(String)
  RepositoryResolved(RepositoryChoice)
  RepositorySkipped
  ModrinthLinked
  CurseforgeLinked
  CredentialsFinished
  StepRequested(Step)
}

pub fn init() -> Model {
  Model(
    step: Workspace,
    workspace: "",
    repository: [],
    modrinth_linked: False,
    curseforge_linked: False,
  )
}

/// A stable string name for a step.
///
/// The Vue layer needs to branch on which step is active and to key CSS
/// classes off it. It must **not** do that with `step.constructor.name`:
/// Gleam compiles each variant to an ES class, esbuild minifies class names
/// (Vite leaves `keepNames` off), and in a production build that property
/// returns `"Qs"` rather than `"Workspace"` — so every comparison fails, the
/// whole chain falls through to its last branch, and nothing catches it
/// because type-checking and the build both succeed. Only running the built
/// app reveals it.
///
/// Constructor *names* are mangled; namespace *keys* are not, which is why
/// `setup.WorkspaceChosen(..)` is still safe to call from TypeScript.
pub fn step_key(step: Step) -> String {
  case step {
    Workspace -> "workspace"
    Repository -> "repository"
    Credentials -> "credentials"
    Done -> "done"
  }
}

/// Whether the flow has what it needs to let the user leave.
///
/// Only the workspace matters. Everything after it is optional by design, so
/// gating completion on a provider link would strand anyone who just wants to
/// edit a pack offline.
pub fn can_finish(model: Model) -> Bool {
  model.workspace != ""
}

/// Whether a step can be navigated to directly.
///
/// Steps after the workspace unlock together rather than one at a time: once
/// there is a workspace, the remaining two are independent and a user who
/// skipped the repository step should still be able to jump back to it from
/// the credentials step without redoing anything.
pub fn is_reachable(model: Model, step: Step) -> Bool {
  case step {
    Workspace -> True
    _ -> model.workspace != ""
  }
}

pub fn update(model: Model, message: Message) -> Model {
  case message {
    WorkspaceChosen(path) ->
      case path {
        "" -> model
        _ -> Model(..model, workspace: path, step: Repository)
      }

    RepositoryResolved(choice) ->
      Model(
        ..model,
        repository: remember(model.repository, choice),
        step: Credentials,
      )

    RepositorySkipped ->
      Model(
        ..model,
        repository: remember(model.repository, SkipRepository),
        step: Credentials,
      )

    ModrinthLinked -> Model(..model, modrinth_linked: True)

    CurseforgeLinked -> Model(..model, curseforge_linked: True)

    CredentialsFinished ->
      case can_finish(model) {
        True -> Model(..model, step: Done)
        False -> model
      }

    StepRequested(step) ->
      case is_reachable(model, step) {
        True -> Model(..model, step: step)
        False -> model
      }
  }
}

/// Records a repository choice, replacing any earlier one.
///
/// A user who skips and then comes back to clone must not be left recorded as
/// both, because the summary line reads the list and would claim two
/// contradictory things happened.
fn remember(
  choices: List(RepositoryChoice),
  choice: RepositoryChoice,
) -> List(RepositoryChoice) {
  case list.contains(choices, choice) {
    True -> choices
    False -> [choice]
  }
}
