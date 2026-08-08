import gleeunit/should
import packwand/extension_host

const known = ["game-studio", "datapack-pw", "resourcepack-pw", "lang-pw"]

/// Installing a facet must bring its host along, or the facet contributes
/// views that nothing renders and presents as an extension that did nothing.
pub fn installing_a_facet_pulls_in_its_host_test() {
  extension_host.reconcile_installed(["datapack-pw"], known)
  |> should.equal(["datapack-pw", "game-studio"])
}

pub fn unknown_extensions_are_dropped_test() {
  extension_host.reconcile_installed(["lang-pw", "not-in-this-build"], known)
  |> should.equal(["lang-pw"])
}

/// Sorted and de-duplicated so the persisted value is stable — an unordered
/// set would rewrite storage on every start.
pub fn the_installed_set_is_stable_test() {
  extension_host.reconcile_installed(
    ["lang-pw", "game-studio", "lang-pw", "game-studio"],
    known,
  )
  |> should.equal(["game-studio", "lang-pw"])

  // Order in gives the same order out.
  extension_host.reconcile_installed(["game-studio", "lang-pw"], known)
  |> should.equal(extension_host.reconcile_installed(["lang-pw", "game-studio"], known))
}

pub fn nothing_installed_stays_nothing_test() {
  extension_host.reconcile_installed([], known) |> should.equal([])
}

/// An extension that declares no restriction must not be filtered out for
/// having declared nothing.
pub fn an_empty_when_clause_always_applies_test() {
  extension_host.applies([], "modpacks") |> should.be_true
  extension_host.applies([], "") |> should.be_true
}

pub fn a_when_clause_restricts_to_its_categories_test() {
  extension_host.applies(["modpacks"], "modpacks") |> should.be_true
  extension_host.applies(["modpacks"], "resourcepacks") |> should.be_false
  // No project open: a restricted contribution stays hidden.
  extension_host.applies(["modpacks"], "") |> should.be_false
}

pub fn a_wildcard_activation_always_fires_test() {
  extension_host.activated_by(["*"], "") |> should.be_true
  extension_host.activated_by(["*"], "modpacks") |> should.be_true
}

pub fn category_activation_matches_the_open_project_test() {
  extension_host.activated_by(["project:modpacks"], "modpacks") |> should.be_true
  extension_host.activated_by(["project:modpacks"], "datapacks") |> should.be_false
  extension_host.activated_by(["project:modpacks"], "") |> should.be_false
  extension_host.activated_by([], "modpacks") |> should.be_false
}

pub fn an_ordinary_relative_path_is_accepted_test() {
  extension_host.safe_relative_path("config/sodium.json")
  |> should.equal(Ok("config/sodium.json"))

  // Windows separators are normalized rather than rejected.
  extension_host.safe_relative_path("config\\sodium.json")
  |> should.equal(Ok("config/sodium.json"))
}

/// The security boundary. Extensions run inside the app's webview with its
/// filesystem commands behind them, so a path that climbs out of the pack
/// reads anywhere the app can.
pub fn traversal_and_absolute_paths_are_refused_test() {
  for_each_rejected([
    "../../../etc/passwd",
    "config/../../secret",
    "..",
    "config/..",
    "/etc/passwd",
    "\\\\server\\share",
    "",
  ])
}

/// `..` is rejected as a whole segment, not as a substring: a file genuinely
/// named `..config.json` is not an escape attempt.
pub fn a_filename_containing_dots_is_not_traversal_test() {
  extension_host.safe_relative_path("config/..config.json")
  |> should.equal(Ok("config/..config.json"))

  extension_host.safe_relative_path("config/a..b/file.json")
  |> should.equal(Ok("config/a..b/file.json"))
}

fn for_each_rejected(paths: List(String)) -> Nil {
  case paths {
    [] -> Nil
    [first, ..rest] -> {
      extension_host.safe_relative_path(first) |> should.be_error
      for_each_rejected(rest)
    }
  }
}
