import gleam/float
import gleeunit/should
import packwand/theme

/// Compares against a tolerance: contrast ratios are floating point, and an
/// exact-equality assertion would pin the test to one platform's rounding.
fn approximately(actual: Float, expected: Float) -> Nil {
  should.be_true(float.absolute_value(actual -. expected) <. 0.01)
}

/// The two anchors of the WCAG scale. Black on white is exactly 21:1 and a
/// colour against itself is exactly 1:1, so these catch a transposed
/// coefficient or an inverted ratio immediately.
pub fn contrast_extremes_test() {
  let assert Ok(maximum) = theme.contrast_ratio("#000000", "#ffffff")
  approximately(maximum, 21.0)

  let assert Ok(minimum) = theme.contrast_ratio("#7f7f7f", "#7f7f7f")
  approximately(minimum, 1.0)
}

/// Contrast is symmetric — the ratio does not depend on which colour is
/// nominally the foreground.
pub fn contrast_is_symmetric_test() {
  let assert Ok(forwards) = theme.contrast_ratio("#1bd96a", "#26292f")
  let assert Ok(backwards) = theme.contrast_ratio("#26292f", "#1bd96a")
  approximately(forwards, backwards)
}

/// Alpha is accepted and ignored, so an `#RRGGBBAA` token measures the same
/// as its opaque form rather than failing to parse.
pub fn alpha_is_accepted_and_ignored_test() {
  let assert Ok(solid) = theme.contrast_ratio("#8a6df0", "#16161b")
  let assert Ok(translucent) = theme.contrast_ratio("#8a6df026", "#16161b")
  approximately(solid, translucent)
}

pub fn unparseable_colours_fail_rather_than_pass_test() {
  theme.contrast_ratio("not a colour", "#ffffff") |> should.be_error
  theme.contrast_ratio("#fff", "#ffffff") |> should.be_error
  // The safety property: something unmeasurable must never be reported as
  // meeting a contrast requirement.
  theme.meets_contrast("#zzzzzz", "#ffffff", 1.0) |> should.be_false
}

pub fn meets_contrast_applies_the_threshold_test() {
  theme.meets_contrast("#000000", "#ffffff", 4.5) |> should.be_true
  theme.meets_contrast("#777777", "#808080", 4.5) |> should.be_false
}

pub fn token_names_are_recognised_test() {
  theme.is_known_token("accent") |> should.be_true
  theme.is_known_token("surface-3") |> should.be_true
  theme.is_known_token("not-a-token") |> should.be_false
  theme.token_names() |> list_length |> should.equal(30)
}

pub fn appearance_and_font_style_are_constrained_test() {
  theme.is_known_appearance("dark") |> should.be_true
  theme.is_known_appearance("high-contrast") |> should.be_true
  theme.is_known_appearance("sepia") |> should.be_false

  // An empty font style means "inherit" and is legal.
  theme.is_known_font_style("") |> should.be_true
  theme.is_known_font_style("italic") |> should.be_true
  theme.is_known_font_style("blink") |> should.be_false
}

pub fn theme_ids_must_be_namespaced_slugs_test() {
  theme.validate_theme_id("builtin.packwand-dark") |> should.be_true
  theme.validate_theme_id("user.my-theme") |> should.be_true
  theme.validate_theme_id("builtin.") |> should.be_false
  theme.validate_theme_id("packwand-dark") |> should.be_false
  theme.validate_theme_id("builtin.Has-Capitals") |> should.be_false
}

pub fn hex_colours_accept_both_lengths_test() {
  theme.validate_hex_colour("#8a6df0") |> should.be_true
  theme.validate_hex_colour("#8a6df026") |> should.be_true
  theme.validate_hex_colour("#8a6df") |> should.be_false
  theme.validate_hex_colour("8a6df0") |> should.be_false
  theme.validate_hex_colour("#gggggg") |> should.be_false
}

fn list_length(items: List(a)) -> Int {
  case items {
    [] -> 0
    [_, ..rest] -> 1 + list_length(rest)
  }
}
