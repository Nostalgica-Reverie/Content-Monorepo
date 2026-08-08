import gleeunit/should
import packwand/workbench

pub fn paths_normalize_across_platforms_test() {
  workbench.normalize_path("Modpacks\\Vital\\") |> should.equal("modpacks/vital")
  workbench.normalize_path("modpacks/vital") |> should.equal("modpacks/vital")
  workbench.normalize_path("modpacks/vital///") |> should.equal("modpacks/vital")
}

pub fn a_pack_inside_the_project_belongs_to_it_test() {
  workbench.pack_belongs_to("modpacks/vital/26.2-mr", "modpacks/vital")
  |> should.be_true

  // A project directory can itself be a pack.
  workbench.pack_belongs_to("modpacks/vital", "modpacks/vital")
  |> should.be_true
}

/// The bug the separator in the prefix check exists to prevent: without it,
/// one project silently claims another whose name it is a prefix of.
pub fn a_sibling_with_a_shared_prefix_does_not_belong_test() {
  workbench.pack_belongs_to("modpacks/vital-legacy/26.2-mr", "modpacks/vital")
  |> should.be_false

  workbench.pack_belongs_to("modpacks/vitality", "modpacks/vital")
  |> should.be_false
}

pub fn belonging_ignores_slash_direction_and_case_test() {
  workbench.pack_belongs_to("Modpacks\\Vital\\26.2-mr", "modpacks/vital")
  |> should.be_true
}

/// A reindex must not clear a selection that is still valid — otherwise adding
/// an unrelated pack elsewhere resets what the user was looking at.
pub fn a_still_valid_selection_survives_test() {
  workbench.select_or_first(["a", "b", "c"], "b") |> should.equal("b")
}

pub fn a_stale_selection_falls_back_to_the_first_test() {
  workbench.select_or_first(["a", "b"], "gone") |> should.equal("a")
  workbench.select_or_first([], "gone") |> should.equal("")
}

pub fn the_title_takes_the_first_thing_that_is_set_test() {
  workbench.first_present(["", "  ", "Vital", "fallback"], "Packwand")
  |> should.equal("Vital")

  workbench.first_present(["", "  "], "Packwand") |> should.equal("Packwand")
  workbench.first_present([], "Packwand") |> should.equal("Packwand")
}

/// Blank parts are dropped rather than joined, or the subtitle reads as
/// missing data instead of inapplicable data.
pub fn the_summary_drops_unset_parts_test() {
  workbench.summary_line(["modpacks", "", "fabric", "1.0"])
  |> should.equal("modpacks · fabric · 1.0")

  workbench.summary_line(["modpacks"]) |> should.equal("modpacks")
  workbench.summary_line(["", ""]) |> should.equal("")
}
