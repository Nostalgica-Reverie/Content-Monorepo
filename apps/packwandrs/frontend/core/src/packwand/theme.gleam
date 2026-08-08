//// Theme colour maths and field validation.
////
//// The rules a theme has to satisfy are pure functions of a few strings, so
//// they live here rather than in the Vue layer: they are the part worth
//// testing exhaustively, and the part that would otherwise be re-implemented
//// slightly differently in the theme editor, the importer, and the settings
//// gallery.
////
//// What stays in TypeScript is the object walking — pulling `colors` and
//// `editor.rules` out of untyped JSON — because that is where Vue and the
//// file picker hand us `unknown` and a `dynamic` decoder would only move the
//// awkwardness rather than remove it.

import gleam/float
import gleam/result
import gleam/int
import gleam/list
import gleam/string

/// Every application colour token a theme may set.
///
/// The canonical list. The TypeScript side derives its `ThemeTokenName` union
/// from this so the two cannot drift.
pub fn token_names() -> List(String) {
  [
    "rail", "side", "bg", "bg-2", "surface", "surface-2", "surface-3",
    "surface-soft", "elevated", "hover", "active", "selected", "line",
    "line-soft", "line-strong", "text", "text-strong", "muted", "faint",
    "accent", "accent-2", "accent-dim", "accent-soft", "accent-line", "danger",
    "danger-bg", "danger-line", "warning", "success", "success-bg",
  ]
}

pub fn is_known_token(name: String) -> Bool {
  list.contains(token_names(), name)
}

pub fn is_known_appearance(value: String) -> Bool {
  list.contains(["light", "dark", "high-contrast"], value)
}

pub fn is_known_font_style(value: String) -> Bool {
  list.contains(["", "bold", "italic", "underline", "strikethrough"], value)
}

/// Relative luminance per WCAG 2.x, from a `#RRGGBB` or `#RRGGBBAA` colour.
///
/// Alpha is ignored: a contrast ratio is only meaningful against an opaque
/// backdrop, and treating a translucent token as if it were opaque is the
/// conservative reading — it never reports a passing ratio that the composited
/// result would fail.
pub fn relative_luminance(colour: String) -> Result(Float, Nil) {
  case channels(colour) {
    Ok([red, green, blue]) ->
      Ok(0.2126 *. linearise(red) +. 0.7152 *. linearise(green) +. 0.0722 *. linearise(blue))
    _ -> Error(Nil)
  }
}

/// WCAG contrast ratio between two colours, always >= 1.0.
pub fn contrast_ratio(left: String, right: String) -> Result(Float, Nil) {
  case relative_luminance(left), relative_luminance(right) {
    Ok(first), Ok(second) -> {
      let lighter = float.max(first, second)
      let darker = float.min(first, second)
      Ok({ lighter +. 0.05 } /. { darker +. 0.05 })
    }
    _, _ -> Error(Nil)
  }
}

/// Does this pair clear `minimum`? An unparseable colour fails rather than
/// passes — a theme that cannot be measured must not be reported as accessible.
pub fn meets_contrast(foreground: String, background: String, minimum: Float) -> Bool {
  case contrast_ratio(foreground, background) {
    Ok(ratio) -> ratio >=. minimum
    Error(_) -> False
  }
}

/// A `builtin.*` or `user.*` slug.
pub fn validate_theme_id(value: String) -> Bool {
  let valid_prefix =
    string.starts_with(value, "user.") || string.starts_with(value, "builtin.")
  let tail = value |> string.split(".") |> list.drop(1) |> string.join(".")
  valid_prefix
  && tail != ""
  && string.to_graphemes(tail) |> list.all(valid_slug_character)
}

/// `#RRGGBB` or `#RRGGBBAA`.
pub fn validate_hex_colour(value: String) -> Bool {
  let graphemes = string.to_graphemes(value)
  let length = list.length(graphemes)
  let body = list.drop(graphemes, 1)
  list.first(graphemes) == Ok("#")
  && { length == 7 || length == 9 }
  && list.all(body, valid_hex_character)
}

/// The three pairs every bundled theme is held to, as
/// `#(foreground_token, background_token, label, minimum_ratio)`.
///
/// 4.5:1 is WCAG AA for body text; 3:1 is AA for large text and UI components,
/// which is what an accent used on controls has to clear.
pub fn contrast_requirements() -> List(#(String, String, String, Float)) {
  [
    #("text", "bg", "body text", 4.5),
    #("text-strong", "surface", "strong text", 4.5),
    #("accent", "bg", "accent controls", 3.0),
  ]
}

fn channels(colour: String) -> Result(List(Float), Nil) {
  case validate_hex_colour(colour) {
    False -> Error(Nil)
    True -> {
      let digits = string.drop_start(colour, 1)
      [0, 2, 4]
      |> list.try_map(fn(offset) {
        digits
        |> string.slice(offset, 2)
        |> int.base_parse(16)
        |> result.map(fn(value) { int.to_float(value) /. 255.0 })
      })
    }
  }
}

/// sRGB -> linear, the transfer function WCAG specifies.
fn linearise(channel: Float) -> Float {
  case channel <=. 0.04045 {
    True -> channel /. 12.92
    False ->
      case float.power({ channel +. 0.055 } /. 1.055, 2.4) {
        Ok(value) -> value
        // `power` only fails for a negative base with a fractional exponent,
        // which cannot happen here: the base is at least 0.0499.
        Error(_) -> 0.0
      }
  }
}

fn valid_slug_character(value: String) -> Bool {
  string.contains("abcdefghijklmnopqrstuvwxyz0123456789.-", value)
}

fn valid_hex_character(value: String) -> Bool {
  string.contains("0123456789abcdefABCDEF", value)
}
