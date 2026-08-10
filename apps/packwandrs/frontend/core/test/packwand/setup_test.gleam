import gleeunit/should
import packwand/setup

pub fn choosing_a_workspace_advances_to_the_repository_step_test() {
  setup.init()
  |> setup.update(setup.WorkspaceChosen("C:/repos/monorepo"))
  |> fn(model) { model.step }
  |> should.equal(setup.Repository)
}

/// The picker returns an empty string when the user cancels the dialog, which
/// must not read as a valid workspace.
pub fn a_cancelled_workspace_pick_changes_nothing_test() {
  let model = setup.init() |> setup.update(setup.WorkspaceChosen(""))
  model.step |> should.equal(setup.Workspace)
  model.workspace |> should.equal("")
}

pub fn only_the_workspace_is_required_to_finish_test() {
  setup.init() |> setup.can_finish |> should.be_false

  setup.init()
  |> setup.update(setup.WorkspaceChosen("C:/repos/monorepo"))
  |> setup.can_finish
  |> should.be_true
}

/// Skipping the repository step is a real answer, not an absence of one, and
/// it must still let the user reach the end.
pub fn skipping_the_repository_still_finishes_test() {
  let model =
    setup.init()
    |> setup.update(setup.WorkspaceChosen("C:/repos/monorepo"))
    |> setup.update(setup.RepositorySkipped)
    |> setup.update(setup.CredentialsFinished)

  model.step |> should.equal(setup.Done)
  model.repository |> should.equal([setup.SkipRepository])
}

/// Finishing without a workspace has to be impossible even if the message is
/// dispatched directly — the router would bounce the user straight back.
pub fn finishing_without_a_workspace_is_refused_test() {
  setup.init()
  |> setup.update(setup.CredentialsFinished)
  |> fn(model) { model.step }
  |> should.equal(setup.Workspace)
}

pub fn later_steps_are_unreachable_until_a_workspace_exists_test() {
  let fresh = setup.init()
  setup.is_reachable(fresh, setup.Workspace) |> should.be_true
  setup.is_reachable(fresh, setup.Repository) |> should.be_false
  setup.is_reachable(fresh, setup.Credentials) |> should.be_false

  let ready = setup.update(fresh, setup.WorkspaceChosen("C:/repos/monorepo"))
  setup.is_reachable(ready, setup.Repository) |> should.be_true
  setup.is_reachable(ready, setup.Credentials) |> should.be_true
}

pub fn jumping_to_an_unreachable_step_is_ignored_test() {
  setup.init()
  |> setup.update(setup.StepRequested(setup.Credentials))
  |> fn(model) { model.step }
  |> should.equal(setup.Workspace)
}

/// Changing your mind must replace the earlier answer, not accumulate. A user
/// recorded as having both skipped and cloned makes the summary line lie.
pub fn a_later_repository_choice_replaces_the_earlier_one_test() {
  let model =
    setup.init()
    |> setup.update(setup.WorkspaceChosen("C:/repos/monorepo"))
    |> setup.update(setup.RepositorySkipped)
    |> setup.update(setup.StepRequested(setup.Repository))
    |> setup.update(setup.RepositoryResolved(setup.CloneRemote))

  model.repository |> should.equal([setup.CloneRemote])
}

pub fn linking_providers_is_recorded_but_never_required_test() {
  let model =
    setup.init()
    |> setup.update(setup.WorkspaceChosen("C:/repos/monorepo"))
    |> setup.update(setup.ModrinthLinked)

  model.modrinth_linked |> should.be_true
  model.curseforge_linked |> should.be_false
  setup.can_finish(model) |> should.be_true
}

/// Guards the minification trap: the Vue layer must branch on these strings,
/// never on `constructor.name`, which esbuild rewrites in a release build.
pub fn every_step_has_a_stable_key_test() {
  setup.step_key(setup.Workspace) |> should.equal("workspace")
  setup.step_key(setup.Repository) |> should.equal("repository")
  setup.step_key(setup.Credentials) |> should.equal("credentials")
  setup.step_key(setup.Done) |> should.equal("done")
}
