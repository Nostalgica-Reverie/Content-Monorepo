import gleeunit/should
import packwand/shell

pub fn opening_appends_in_order_test() {
  []
  |> shell.open_tab("a")
  |> shell.open_tab("b")
  |> should.equal(["a", "b"])
}

/// Opening a view that is already open focuses it rather than duplicating it,
/// so the strip cannot accumulate copies of the same tab.
pub fn opening_an_open_tab_changes_nothing_test() {
  ["a", "b"]
  |> shell.open_tab("a")
  |> should.equal(["a", "b"])
}

/// Closing lands on the tab that slid into the closed one's position.
pub fn closing_focuses_the_tab_that_takes_its_place_test() {
  let #(remaining, successor) =
    shell.close_tab(["a", "b", "c"], "b")
  remaining |> should.equal(["a", "c"])
  successor |> should.equal(Ok("c"))
}

/// Closing the last tab falls back to the one before it, rather than to the
/// first or to nothing.
pub fn closing_the_last_tab_focuses_its_predecessor_test() {
  let #(remaining, successor) =
    shell.close_tab(["a", "b", "c"], "c")
  remaining |> should.equal(["a", "b"])
  successor |> should.equal(Ok("b"))
}

pub fn closing_the_only_tab_focuses_nothing_test() {
  let #(remaining, successor) = shell.close_tab(["a"], "a")
  remaining |> should.equal([])
  successor |> should.be_error
}

pub fn closing_a_tab_that_is_not_open_changes_nothing_test() {
  let #(remaining, successor) = shell.close_tab(["a"], "ghost")
  remaining |> should.equal(["a"])
  successor |> should.be_error
}

pub fn sizes_are_clamped_into_range_test() {
  shell.clamp(300, 190, 460) |> should.equal(300)
  shell.clamp(10, 190, 460) |> should.equal(190)
  shell.clamp(9999, 190, 460) |> should.equal(460)
  // A nonsensical stored size cannot leave a pane unusable.
  shell.clamp(-5, 190, 460) |> should.equal(190)
}

pub fn the_output_log_stays_bounded_test() {
  let filled =
    list_range(1, 6)
    |> fold_push(3)
  filled |> should.equal([4, 5, 6])
}

pub fn a_short_log_keeps_everything_test() {
  [1, 2] |> shell.push_bounded(3, 10) |> should.equal([1, 2, 3])
}

fn fold_push(values: List(Int), limit: Int) -> List(Int) {
  case values {
    [] -> []
    _ -> push_all([], values, limit)
  }
}

fn push_all(acc: List(Int), values: List(Int), limit: Int) -> List(Int) {
  case values {
    [] -> acc
    [first, ..rest] -> push_all(shell.push_bounded(acc, first, limit), rest, limit)
  }
}

fn list_range(from: Int, to: Int) -> List(Int) {
  case from > to {
    True -> []
    False -> [from, ..list_range(from + 1, to)]
  }
}
