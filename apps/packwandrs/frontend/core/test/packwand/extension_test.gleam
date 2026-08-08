import gleeunit/should
import packwand/extension

/// The bug this rule exists to avoid: manifests and entry points sit at
/// different depths, so deriving the directory from the end of the path gives
/// `src` for an entry point and unpairs it from its manifest.
pub fn directory_comes_from_after_the_extensions_anchor_test() {
  extension.directory_of("../../../extensions/game-studio/extension.pw.json")
  |> should.equal("game-studio")

  extension.directory_of("../../../extensions/game-studio/src/index.ts")
  |> should.equal("game-studio")
}

/// A path with no `extensions/` anchor still has to yield something usable
/// rather than the whole path.
pub fn a_path_without_the_anchor_falls_back_to_the_parent_test() {
  extension.directory_of("lang-pw/extension.pw.json") |> should.equal("lang-pw")
  extension.directory_of("solo") |> should.equal("solo")
}

/// A directory literally called `extensions` inside the tree must not confuse
/// the anchor search — the first match wins, which is the outer one.
pub fn a_nested_extensions_directory_takes_the_first_anchor_test() {
  extension.directory_of("../extensions/outer/src/extensions/inner.ts")
  |> should.equal("outer")
}

pub fn a_complete_manifest_has_no_problem_test() {
  extension.manifest_problem("game-studio", "game-studio", [], 1)
  |> should.be_ok
}

pub fn missing_fields_are_listed_together_test() {
  let assert Error(message) =
    extension.manifest_problem("x", "x", ["name", "entry"], 1)
  message |> should.equal("extension.pw.json is missing name, entry")
}

pub fn an_unsupported_api_version_is_rejected_test() {
  let assert Error(message) = extension.manifest_problem("x", "x", [], 2)
  message |> should.equal("unsupported extension apiVersion 2")
}

/// The id and the directory must agree, or the host pairs a manifest with the
/// wrong module.
pub fn a_mismatched_id_is_rejected_test() {
  let assert Error(message) =
    extension.manifest_problem("game-studio", "gamestudio", [], 1)
  message
  |> should.equal(
    "extension.pw.json id \"gamestudio\" does not match its directory \"game-studio\"",
  )
}

/// Missing fields are reported before the version and id checks: a manifest
/// that is missing `id` entirely should say so rather than complain that an
/// empty id does not match its directory.
pub fn missing_fields_are_reported_before_other_problems_test() {
  let assert Error(message) = extension.manifest_problem("x", "", ["id"], 99)
  message |> should.equal("extension.pw.json is missing id")
}

pub fn the_required_field_lists_are_the_canonical_ones_test() {
  extension.required_string_fields()
  |> should.equal(["id", "name", "version", "entry"])
  extension.required_array_fields()
  |> should.equal([
    "activation", "commands", "views", "validators", "capabilities",
  ])
  extension.api_version_supported(1) |> should.be_true
  extension.api_version_supported(0) |> should.be_false
}
