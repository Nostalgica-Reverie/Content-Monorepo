//// Which extensions are active, and what they are allowed to reach.
////
//// The host around this owns the module registry and `localStorage`; these are
//// the decisions it makes. Two of them are worth having in one tested place:
//// the activation rules, which decide whether a contributed command shows up
//// at all, and the pack-relative path guard, which is a security boundary.

import gleam/list
import gleam/string

/// Extensions that are pulled in automatically by another.
///
/// `datapack-pw` and `resourcepack-pw` are facets of the Game Studio
/// extension; installing either without it leaves their views contributed but
/// unhosted, which presents as an extension that installed and then did
/// nothing.
pub fn implied_by(id: String) -> List(String) {
  case id {
    "datapack-pw" | "resourcepack-pw" -> ["game-studio"]
    _ -> []
  }
}

/// The installed set, after adding implied extensions, dropping ones that are
/// not present in this build, de-duplicating and sorting.
///
/// Sorted so the persisted value is stable: an unordered set rewritten on every
/// load produces a `localStorage` write and a settings diff on every start.
pub fn reconcile_installed(
  requested: List(String),
  known: List(String),
) -> List(String) {
  requested
  |> list.flat_map(fn(id) { [id, ..implied_by(id)] })
  |> list.filter(fn(id) { list.contains(known, id) })
  |> list.unique
  |> list.sort(string.compare)
}

/// Does a `when` clause admit the current project category?
///
/// An absent or empty clause means "always", which is why this cannot simply
/// be a membership test — an extension that declares no restriction must not
/// be filtered out for having declared nothing.
pub fn applies(when: List(String), category: String) -> Bool {
  case when {
    [] -> True
    _ -> category != "" && list.contains(when, category)
  }
}

/// Does an activation event list fire for the current project category?
pub fn activated_by(events: List(String), category: String) -> Bool {
  case list.contains(events, "*") {
    True -> True
    False -> category != "" && list.contains(events, "project:" <> category)
  }
}

/// Normalizes a pack-relative path an extension asked to open, or refuses it.
///
/// A security boundary, not a tidy-up: extensions run in the app's own webview
/// with its filesystem commands behind them, so a path that climbs out of the
/// pack is a way to read anywhere the app can. `..` is rejected as a whole
/// segment rather than as a substring, because a file legitimately called
/// `..config.json` is not an escape attempt. Absolute paths are refused rather
/// than rebased — an extension asking for `/etc/passwd` has made a mistake
/// worth surfacing, not one worth silently reinterpreting.
pub fn safe_relative_path(path: String) -> Result(String, String) {
  let normalized = string.replace(path, "\\", "/")
  case string.starts_with(normalized, "/") {
    True -> Error("Invalid pack-relative editor path")
    False -> {
      let segments = string.split(normalized, "/")
      case normalized == "" || list.contains(segments, "..") {
        True -> Error("Invalid pack-relative editor path")
        False -> Ok(normalized)
      }
    }
  }
}
