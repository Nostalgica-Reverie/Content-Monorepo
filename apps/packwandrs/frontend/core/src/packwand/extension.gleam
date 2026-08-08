//// Rules for loading a build-time extension.
////
//// The Vite glob machinery that finds extensions has to stay in TypeScript —
//// `import.meta.glob` is a build-time construct with no equivalent here. What
//// moves is everything that is a pure function of the paths and manifests it
//// returns: which directory a glob result belongs to, and whether a manifest
//// is usable.
////
//// Both are the kind of rule that is easy to get subtly wrong and hard to
//// notice: a mis-derived directory silently unpairs a manifest from its entry
//// point, and a missing validation lets a broken extension half-load.

import gleam/int
import gleam/list
import gleam/string

/// Manifest fields that must be present, non-empty strings.
pub fn required_string_fields() -> List(String) {
  ["id", "name", "version", "entry"]
}

/// Manifest fields that must be present arrays. They may be empty — an
/// extension that contributes no commands is legal, one that omits the key is
/// not, because that is usually a typo rather than a decision.
pub fn required_array_fields() -> List(String) {
  ["activation", "commands", "views", "validators", "capabilities"]
}

/// The extension API generation this host understands.
pub const api_version: Int = 1

pub fn api_version_supported(value: Int) -> Bool {
  value == api_version
}

/// The extension directory a glob result belongs to: the segment straight
/// after `extensions/`.
///
/// Taken from the *front* rather than the back. Manifests and entry points sit
/// at different depths — `<id>/extension.pw.json` against `<id>/src/index.ts`
/// — so counting from the end yields `src` for the latter, which would pair
/// every extension's manifest with nothing and report them all as broken.
pub fn directory_of(path: String) -> String {
  let segments = string.split(path, "/")
  case after_extensions(segments) {
    Ok(directory) -> directory
    // No `extensions/` anchor: fall back to the second-to-last segment, which
    // is right for `<id>/extension.pw.json`.
    Error(_) ->
      case list.reverse(segments) {
        [_, directory, ..] -> directory
        _ -> path
      }
  }
}

/// Why this manifest cannot be loaded, or `Ok` when it can.
///
/// Takes already-extracted facts rather than a decoded document: pulling
/// fields out of untyped JSON is where TypeScript is already sitting, and a
/// `dynamic` decoder here would move that awkwardness rather than remove it.
pub fn manifest_problem(
  directory: String,
  id: String,
  missing_fields: List(String),
  api_version_value: Int,
) -> Result(Nil, String) {
  case missing_fields {
    [_, ..] ->
      Error("extension.pw.json is missing " <> string.join(missing_fields, ", "))
    [] ->
      case api_version_supported(api_version_value) {
        False ->
          Error(
            "unsupported extension apiVersion "
            <> int.to_string(api_version_value),
          )
        True ->
          case id == directory {
            False ->
              Error(
                "extension.pw.json id \""
                <> id
                <> "\" does not match its directory \""
                <> directory
                <> "\"",
              )
            True -> Ok(Nil)
          }
      }
  }
}

fn after_extensions(segments: List(String)) -> Result(String, Nil) {
  case segments {
    ["extensions", directory, ..] -> Ok(directory)
    [_, ..rest] -> after_extensions(rest)
    [] -> Error(Nil)
  }
}
