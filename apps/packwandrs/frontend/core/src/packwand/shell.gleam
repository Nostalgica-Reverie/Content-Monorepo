//// Pure shell-chrome rules: tab bookkeeping, panel sizing, output buffering.
////
//// The shell store itself stays in TypeScript because most of what it does is
//// side effects — reading and writing `localStorage`, driving Vue refs. What
//// moves here is the part that is a function of its inputs and nothing else,
//// which is also the part with edge cases worth pinning: which tab you land on
//// after closing one, and what a bad stored size does.

import gleam/int
import gleam/list

/// Tabs are addressed by name here rather than as whole records.
///
/// A tab carries a path, label and icon that only the renderer cares about,
/// and modelling that record in both languages would mean converting it on
/// every call for no benefit. The *ordering* rules are the shared part, so
/// those operate on names and the caller maps them back to its own objects.
///
/// Adds a name unless it is already open — opening an already-open view
/// focuses it rather than duplicating it, so this is idempotent.
pub fn open_tab(names: List(String), name: String) -> List(String) {
  case list.contains(names, name) {
    True -> names
    False -> list.append(names, [name])
  }
}

/// Removes a tab, and reports which one should take focus.
///
/// The neighbour rule is the one thing here worth being careful about: closing
/// a tab should land on the one that slid into its place, and on the one
/// *before* it when the closed tab was last. Landing on the first tab instead
/// — or on nothing when others remain — is the kind of thing nobody reports as
/// a bug but everybody feels.
pub fn close_tab(
  names: List(String),
  name: String,
) -> #(List(String), Result(String, Nil)) {
  case index_of(names, name, 0) {
    Error(_) -> #(names, Error(Nil))
    Ok(index) -> {
      let remaining =
        list.append(list.take(names, index), list.drop(names, index + 1))
      let successor = case at(remaining, index) {
        Ok(tab) -> Ok(tab)
        // The closed tab was last: fall back to the one before it.
        Error(_) -> at(remaining, index - 1)
      }
      #(remaining, successor)
    }
  }
}

/// Keeps a panel size inside its usable range.
///
/// Applied to values that come back from `localStorage` as well as from a
/// drag, because a stored size is just as capable of being nonsense — a pane
/// dragged off-screen in a previous session should not persist as unusable.
pub fn clamp(value: Int, minimum: Int, maximum: Int) -> Int {
  int.max(minimum, int.min(maximum, value))
}

/// Appends to a bounded output log, dropping the oldest lines past `limit`.
///
/// Bounded because the dock renders every line it holds: an unbounded log
/// turns a chatty build into an unresponsive window.
pub fn push_bounded(lines: List(a), line: a, limit: Int) -> List(a) {
  let appended = list.append(lines, [line])
  let excess = list.length(appended) - limit
  case excess > 0 {
    True -> list.drop(appended, excess)
    False -> appended
  }
}

fn index_of(names: List(String), name: String, position: Int) -> Result(Int, Nil) {
  case names {
    [] -> Error(Nil)
    [first, ..rest] ->
      case first == name {
        True -> Ok(position)
        False -> index_of(rest, name, position + 1)
      }
  }
}

fn at(names: List(String), index: Int) -> Result(String, Nil) {
  case index < 0 {
    True -> Error(Nil)
    False ->
      case list.drop(names, index) {
        [name, ..] -> Ok(name)
        [] -> Error(Nil)
      }
  }
}
