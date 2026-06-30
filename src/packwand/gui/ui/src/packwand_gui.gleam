import gleam/int
import gleam/string
import packwand_gui/model
import packwand_gui/model.{
  Action, AddMod, ExportCurseforge, ExportModrinth, PacksIndex, PinMod, Project,
  RefreshSubdir, RemoveMod, Subdir, UnpinMod, UpdateAll, UpdateMod, ValidateAll,
  Variant, WorkspaceRefresh, WorkspaceStatus, WorkspaceSync,
}
import packwand_gui/view

pub opaque type RawProjects
pub opaque type RawProject
pub opaque type RawSubdir
pub opaque type RawVariant
pub opaque type RawMods
pub opaque type RawMod

@external(javascript, "./packwand_gui/ffi.mjs", "fetchHealth")
fn fetch_health(done: fn(String) -> Nil) -> Nil

@external(javascript, "./packwand_gui/ffi.mjs", "fetchProjects")
fn fetch_projects(done: fn(RawProjects) -> Nil) -> Nil

@external(javascript, "./packwand_gui/ffi.mjs", "projectCount")
fn project_count(projects: RawProjects) -> Int

@external(javascript, "./packwand_gui/ffi.mjs", "projectAt")
fn project_at(projects: RawProjects, index: Int) -> RawProject

@external(javascript, "./packwand_gui/ffi.mjs", "projectString")
fn project_string(project: RawProject, field: String) -> String

@external(javascript, "./packwand_gui/ffi.mjs", "projectBool")
fn project_bool(project: RawProject, field: String) -> Bool

@external(javascript, "./packwand_gui/ffi.mjs", "variantCount")
fn variant_count(project: RawProject) -> Int

@external(javascript, "./packwand_gui/ffi.mjs", "variantAt")
fn variant_at(project: RawProject, index: Int) -> RawVariant

@external(javascript, "./packwand_gui/ffi.mjs", "variantString")
fn variant_string(variant: RawVariant, field: String) -> String

@external(javascript, "./packwand_gui/ffi.mjs", "subdirCount")
fn raw_subdir_count(project: RawProject) -> Int

@external(javascript, "./packwand_gui/ffi.mjs", "subdirAt")
fn raw_subdir_at(project: RawProject, index: Int) -> RawSubdir

@external(javascript, "./packwand_gui/ffi.mjs", "subdirString")
fn subdir_string(subdir: RawSubdir, field: String) -> String

@external(javascript, "./packwand_gui/ffi.mjs", "subdirInt")
fn subdir_int(subdir: RawSubdir, field: String) -> Int

@external(javascript, "./packwand_gui/ffi.mjs", "subdirBool")
fn subdir_bool(subdir: RawSubdir, field: String) -> Bool

@external(javascript, "./packwand_gui/ffi.mjs", "fetchMods")
fn fetch_mods(subdir: String, done: fn(RawMods) -> Nil) -> Nil

@external(javascript, "./packwand_gui/ffi.mjs", "modCount")
fn mod_count_raw(mods: RawMods) -> Int

@external(javascript, "./packwand_gui/ffi.mjs", "modAt")
fn mod_at(mods: RawMods, index: Int) -> RawMod

@external(javascript, "./packwand_gui/ffi.mjs", "modString")
fn mod_string(mod: RawMod, field: String) -> String

@external(javascript, "./packwand_gui/ffi.mjs", "modBool")
fn mod_bool(mod: RawMod, field: String) -> Bool

@external(javascript, "./packwand_gui/ffi.mjs", "setText")
fn set_text(id: String, value: String) -> Nil

@external(javascript, "./packwand_gui/ffi.mjs", "setHtml")
fn set_html(id: String, value: String) -> Nil

@external(javascript, "./packwand_gui/ffi.mjs", "setValue")
fn set_value(id: String, value: String) -> Nil

@external(javascript, "./packwand_gui/ffi.mjs", "setProjectIcon")
fn set_project_icon(project_id: String) -> Nil

@external(javascript, "./packwand_gui/ffi.mjs", "selectValue")
fn select_value(id: String) -> String

@external(javascript, "./packwand_gui/ffi.mjs", "onClick")
fn on_click(id: String, handler: fn() -> Nil) -> Nil

@external(javascript, "./packwand_gui/ffi.mjs", "onSelect")
fn on_select(id: String, handler: fn(String) -> Nil) -> Nil

@external(javascript, "./packwand_gui/ffi.mjs", "onActionButtons")
fn on_action_buttons(handler: fn(String, Bool) -> Nil) -> Nil

@external(javascript, "./packwand_gui/ffi.mjs", "onSubdirActionButtons")
fn on_subdir_action_buttons(handler: fn(String) -> Nil) -> Nil

@external(javascript, "./packwand_gui/ffi.mjs", "onModButtons")
fn on_mod_buttons(handler: fn(String, String) -> Nil) -> Nil

@external(javascript, "./packwand_gui/ffi.mjs", "modSlugInput")
fn mod_slug_input() -> String

@external(javascript, "./packwand_gui/ffi.mjs", "startAction")
fn start_action(name: String, subdir: String, slug: String, dry_run: Bool, done: fn(String) -> Nil) -> Nil

@external(javascript, "./packwand_gui/ffi.mjs", "watchJob")
fn watch_job(id: String) -> Nil

@external(javascript, "./packwand_gui/ffi.mjs", "appendLog")
fn append_log(line: String) -> Nil

@external(javascript, "./packwand_gui/ffi.mjs", "copyText")
fn copy_text(text: String) -> Nil

@external(javascript, "./packwand_gui/ffi.mjs", "innerText")
fn inner_text(id: String) -> String

@external(javascript, "./packwand_gui/ffi.mjs", "setupViews")
fn setup_views() -> Nil

pub fn main() {
  fetch_health(fn(root) {
    set_text("repoRoot", root)
  })
  fetch_projects(fn(raw) {
    let projects = decode_projects(raw)
    render_projects(projects)
    bind_project_select(projects)
    bind_actions()
    setup_views()
  })
}

fn render_projects(projects: List(Project)) {
  let selected = case projects {
    [] -> ""
    [first, ..] -> first.id
  }
  set_html("projectSelect", view.project_options(projects, selected))
  render_selected_project(projects, selected)
}

fn bind_project_select(projects: List(Project)) {
  on_select("projectSelect", fn(id) {
    render_selected_project(projects, id)
  })
}

fn render_selected_project(projects: List(Project), id: String) {
  case find_project(projects, id) {
    Error(_) -> Nil
    Ok(project) -> {
      set_value("projectSelect", project.id)
      set_text("projectName", project.id)
      set_text("projectMeta", model.project_summary(project))
      set_project_icon(project.id)
      set_text("projectRole", fallback(project.role, "none"))
      set_text("subdirCount", int.to_string(list_length(project.subdirs)) <> " subdir(s)")
      set_html("projectDetails", view.project_details(project))
      set_html("subdirList", view.subdir_rows(project.subdirs))
      set_html("subdirSelect", view.subdir_options(project.subdirs))
      set_html("variantList", view.variant_rows(project.variants))
      set_html("changelogPreview", view.changelog_preview(project))
      load_mods(select_value("subdirSelect"))
    }
  }
}

fn bind_actions() {
  on_click("refreshProjects", fn() { run(PacksIndex) })
  on_click("validateAll", fn() { run(ValidateAll) })
  on_click("copySummary", fn() {
    copy_text(inner_text("changelogPreview"))
    append_log("Copied changelog summary.")
  })
  on_action_buttons(fn(name, dry_run) {
    case name {
      "workspace-status" -> run(WorkspaceStatus)
      "packs-index" -> run(PacksIndex)
      "workspace-sync" -> run(WorkspaceSync(dry_run))
      "workspace-refresh" -> run(WorkspaceRefresh)
      _ -> append_log("Unknown action: " <> name)
    }
  })
  on_subdir_action_buttons(fn(name) {
    let subdir = select_value("subdirSelect")
      case subdir {
        "" -> append_log("No subdir selected.")
        _ ->
          case name {
            "refresh" -> run(RefreshSubdir(subdir))
            "update-all" -> run(UpdateAll(subdir))
            "export-modrinth" -> run(ExportModrinth(subdir))
            "export-curseforge" -> run(ExportCurseforge(subdir))
            _ -> append_log("Unknown subdir action: " <> name)
          }
    }
  })
  on_click("addModButton", fn() {
    let subdir = select_value("subdirSelect")
    let slug = mod_slug_input()
    case subdir == "" || slug == "" {
      True -> append_log("Select a subdir and enter a mod slug.")
      False -> run(AddMod(subdir, slug))
    }
  })
  on_select("subdirSelect", fn(subdir) {
    load_mods(subdir)
  })
  bind_mod_buttons()
}

fn run(action: Action) {
  append_log("> " <> model.action_name(action))
  start_action(
    model.action_name(action),
    model.action_subdir(action),
    model.action_slug(action),
    model.action_dry_run(action),
    fn(job_id) { watch_job(job_id) },
  )
}

fn load_mods(subdir: String) {
  fetch_mods(subdir, fn(raw) {
    let mods = decode_mods(raw)
    set_text("modCount", int.to_string(list_length(mods)) <> " mods")
    set_html("modList", view.mod_rows(mods))
    bind_mod_buttons()
  })
}

fn bind_mod_buttons() {
  on_mod_buttons(fn(name, slug) {
    let subdir = select_value("subdirSelect")
    case subdir == "" || slug == "" {
      True -> append_log("Select a subdir and mod first.")
      False ->
        case name {
          "update-mod" -> run(UpdateMod(subdir, slug))
          "pin-mod" -> run(PinMod(subdir, slug))
          "unpin-mod" -> run(UnpinMod(subdir, slug))
          "remove-mod" -> run(RemoveMod(subdir, slug))
          _ -> append_log("Unknown mod action: " <> name)
        }
    }
  })
}

fn decode_projects(raw: RawProjects) -> List(Project) {
  decode_project_loop(raw, 0, project_count(raw))
}

fn decode_project_loop(raw: RawProjects, index: Int, count: Int) -> List(Project) {
  case index >= count {
    True -> []
    False -> {
      let project = project_at(raw, index) |> decode_project
      [project, ..decode_project_loop(raw, index + 1, count)]
    }
  }
}

fn decode_project(raw: RawProject) -> Project {
  Project(
    id: project_string(raw, "id"),
    name: project_string(raw, "name"),
    kind: project_string(raw, "type"),
    dir: project_string(raw, "dir"),
    manifest_path: project_string(raw, "manifest_path"),
    version: project_string(raw, "version"),
    minecraft: project_string(raw, "mc_version"),
    loader: project_string(raw, "loader"),
    release_type: project_string(raw, "release_type"),
    lifecycle: project_string(raw, "lifecycle"),
    role: project_string(raw, "role"),
    auto_update: project_bool(raw, "auto_update"),
    modrinth_id: project_string(raw, "modrinth_id"),
    curseforge_id: project_string(raw, "curseforge_id"),
    github_id: project_string(raw, "github_id"),
    gitea_id: project_string(raw, "gitea_id"),
    gitlab_id: project_string(raw, "gitlab_id"),
    docs_path: project_string(raw, "docs_path"),
    variants: decode_variants(raw),
    subdirs: decode_subdirs(raw),
  )
}

fn decode_variants(project: RawProject) -> List(Variant) {
  decode_variant_loop(project, 0, variant_count(project))
}

fn decode_variant_loop(project: RawProject, index: Int, count: Int) -> List(Variant) {
  case index >= count {
    True -> []
    False -> {
      let raw = variant_at(project, index)
      [
        Variant(
          id: variant_string(raw, "id"),
          minecraft: variant_string(raw, "mc_version"),
          loader: variant_string(raw, "loader"),
          version: variant_string(raw, "version"),
        ),
        ..decode_variant_loop(project, index + 1, count)
      ]
    }
  }
}

fn decode_subdirs(project: RawProject) -> List(Subdir) {
  decode_subdir_loop(project, 0, raw_subdir_count(project))
}

fn decode_subdir_loop(project: RawProject, index: Int, count: Int) -> List(Subdir) {
  case index >= count {
    True -> []
    False -> {
      let raw = raw_subdir_at(project, index)
      [
        Subdir(
          key: subdir_string(raw, "key"),
          path: subdir_string(raw, "path"),
          platform: subdir_string(raw, "platform"),
          mod_count: subdir_int(raw, "mod_count"),
          has_index: subdir_bool(raw, "has_index"),
          has_pack: subdir_bool(raw, "has_pack"),
        ),
        ..decode_subdir_loop(project, index + 1, count)
      ]
    }
  }
}

fn decode_mods(raw: RawMods) -> List(view.ModEntry) {
  decode_mod_loop(raw, 0, mod_count_raw(raw))
}

fn decode_mod_loop(raw: RawMods, index: Int, count: Int) -> List(view.ModEntry) {
  case index >= count {
    True -> []
    False -> {
      let mod = mod_at(raw, index)
      [
        view.ModEntry(
          slug: mod_string(mod, "slug"),
          name: mod_string(mod, "name"),
          filename: mod_string(mod, "filename"),
          side: mod_string(mod, "side"),
          pin: mod_bool(mod, "pin"),
          platform: mod_string(mod, "platform"),
        ),
        ..decode_mod_loop(raw, index + 1, count)
      ]
    }
  }
}

fn find_project(projects: List(Project), id: String) -> Result(Project, Nil) {
  case projects {
    [] -> Error(Nil)
    [first, ..rest] ->
      case first.id == id {
        True -> Ok(first)
        False -> find_project(rest, id)
      }
  }
}

fn list_length(items: List(a)) -> Int {
  case items {
    [] -> 0
    [_, ..rest] -> 1 + list_length(rest)
  }
}

fn fallback(value: String, default: String) -> String {
  case string.trim(value) {
    "" -> default
    _ -> value
  }
}
