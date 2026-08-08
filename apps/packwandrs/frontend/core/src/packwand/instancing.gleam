//// Presentation rules shared by instance list and detail views.

import gleam/string
import gleam/result

/// Human-friendly loader/version subtitle with a stable vanilla fallback.
pub fn version_label(loader: String, game_version: String) -> String {
  let loader = case string.trim(loader) {
    "" -> "Vanilla"
    "vanilla" -> "Vanilla"
    other -> string.uppercase(string.first(other) |> result.unwrap("")) <> string.drop_start(other, 1)
  }
  loader <> " " <> game_version
}

/// Empty form values are inherited, so show the current application default.
pub fn inherited_placeholder(value: String, fallback: String) -> String {
  case string.trim(value) {
    "" -> fallback <> " (inherited)"
    explicit -> explicit
  }
}
