//// Which project and pack are in scope, and how they relate.
////
//// The store around this does the loading and the `localStorage` writes; what
//// lives here is the set of rules that decide what the rest of the app is
//// looking at. They are small, and every one of them has a wrong version that
//// looks right until a specific pack name comes along.

import gleam/list
import gleam/string

/// Compares paths the way the workspace does: forward slashes, no trailing
/// separator, case-insensitive.
///
/// Case-insensitive because the packs come from Windows paths, where
/// `Modpacks\Vital` and `modpacks/vital` are the same directory and would
/// otherwise fail to match.
pub fn normalize_path(path: String) -> String {
  path
  |> string.replace("\\", "/")
  |> drop_trailing_slash
  |> string.lowercase
}

/// Does this pack live inside this project?
///
/// The separator in the prefix check is load-bearing: without it,
/// `modpacks/vital` also claims `modpacks/vital-legacy`, and two unrelated
/// projects silently share a pack list. Matching the root exactly is allowed
/// because a project directory can itself be a pack.
pub fn pack_belongs_to(pack_path: String, project_root: String) -> Bool {
  let pack = normalize_path(pack_path)
  let root = normalize_path(project_root)
  pack == root || string.starts_with(pack, root <> "/")
}

/// Keeps the current selection when it still exists, otherwise takes the first
/// candidate, otherwise nothing.
///
/// This is what stops a reindex from clearing the user's selection whenever a
/// pack is added or removed elsewhere in the workspace.
pub fn select_or_first(candidates: List(String), selected: String) -> String {
  case list.contains(candidates, selected) {
    True -> selected
    False ->
      case candidates {
        [first, ..] -> first
        [] -> ""
      }
  }
}

/// The first non-empty candidate, or `fallback` when there is none.
pub fn first_present(candidates: List(String), fallback: String) -> String {
  case list.filter(candidates, fn(value) { string.trim(value) != "" }) {
    [first, ..] -> first
    [] -> fallback
  }
}

/// Joins the parts of a subtitle, dropping the ones a pack did not set.
///
/// Filtering first matters: joining blanks produces `modpacks ·  · 1.0`, which
/// reads as missing data rather than as data that was never applicable.
pub fn summary_line(parts: List(String)) -> String {
  parts
  |> list.filter(fn(part) { string.trim(part) != "" })
  |> string.join(" · ")
}

fn drop_trailing_slash(path: String) -> String {
  case string.ends_with(path, "/") {
    True -> drop_trailing_slash(string.drop_end(path, 1))
    False -> path
  }
}
