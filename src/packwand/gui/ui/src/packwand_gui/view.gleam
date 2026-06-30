import gleam/int
import gleam/string
import packwand_gui/model.{Project, Subdir, Variant}

pub type ModEntry {
  ModEntry(
    slug: String,
    name: String,
    filename: String,
    side: String,
    pin: Bool,
    platform: String,
  )
}

pub fn project_options(projects: List(Project), selected_id: String) -> String {
  projects
  |> list_map(fn(project) {
    let selected = case project.id == selected_id {
      True -> " selected"
      False -> ""
    }
    "<option value=\"" <> html(project.id) <> "\"" <> selected <> ">"
    <> html(project.id <> " (" <> project.kind <> ")")
    <> "</option>"
  })
  |> string.join("")
}

pub fn project_details(project: Project) -> String {
  [
    detail("Name", project.name),
    detail("Directory", project.dir),
    detail("Manifest", project.manifest_path),
    detail("Lifecycle", fallback(project.lifecycle, "active")),
    detail("Auto Update", case project.auto_update {
      True -> "enabled"
      False -> "disabled"
    }),
    detail("Modrinth", fallback(project.modrinth_id, "-")),
    detail("CurseForge", fallback(project.curseforge_id, "-")),
    detail("GitHub", fallback(project.github_id, "-")),
    detail("Gitea", fallback(project.gitea_id, "-")),
    detail("GitLab", fallback(project.gitlab_id, "-")),
  ]
  |> string.join("")
}

pub fn subdir_rows(subdirs: List(Subdir)) -> String {
  case subdirs {
    [] -> "<div class=\"row\"><span>No subdirs indexed.</span></div>"
    _ ->
      subdirs
      |> list_map(subdir_row)
      |> string.join("")
  }
}

pub fn subdir_options(subdirs: List(Subdir)) -> String {
  subdirs
  |> list_map(fn(subdir) {
    "<option value=\"" <> html(subdir.path) <> "\">"
    <> html(subdir.key)
    <> "</option>"
  })
  |> string.join("")
}

pub fn mod_rows(mods: List(ModEntry)) -> String {
  case mods {
    [] -> "<div class=\"row\"><span>No mods found.</span></div>"
    _ ->
      mods
      |> list_map(fn(mod) {
        let pin_action = case mod.pin {
          True -> "unpin-mod"
          False -> "pin-mod"
        }
        let pin_label = case mod.pin {
          True -> "Unpin"
          False -> "Pin"
        }
        "<div class=\"row\"><div><strong>"
        <> html(fallback(mod.name, mod.slug))
        <> "</strong><span>"
        <> html([mod.slug, mod.filename, mod.side, mod.platform] |> filter_empty |> string.join(" / "))
        <> "</span></div>"
        <> "<button class=\"icon-btn\" data-mod-action=\"update-mod\" data-slug=\""
        <> html(mod.slug)
        <> "\">Update</button>"
        <> "<button class=\"icon-btn\" data-mod-action=\""
        <> pin_action
        <> "\" data-slug=\""
        <> html(mod.slug)
        <> "\">"
        <> pin_label
        <> "</button>"
        <> "<button class=\"icon-btn danger\" data-mod-action=\"remove-mod\" data-slug=\""
        <> html(mod.slug)
        <> "\">Remove</button></div>"
      })
      |> string.join("")
  }
}

pub fn changelog_preview(project: Project) -> String {
  "<p><strong>"
  <> html(project.id <> " " <> project.version)
  <> "</strong></p>"
  <> "<ul>"
  <> "<li>Fetch mod changes can be wired to a dedicated backend action.</li>"
  <> "<li>Use refresh and export actions for the selected subdir.</li>"
  <> "<li>Generated metadata comes from projects.json.</li>"
  <> "</ul>"
}

pub fn variant_rows(variants: List(Variant)) -> String {
  case variants {
    [] -> "<span class=\"empty-note\">No variants declared.</span>"
    _ ->
      variants
      |> list_map(fn(variant) {
        "<div class=\"mini-row\"><strong>"
        <> html(fallback(variant.id, variant.minecraft))
        <> "</strong><span>"
        <> html([variant.minecraft, variant.loader, variant.version] |> filter_empty |> string.join(" / "))
        <> "</span></div>"
      })
      |> string.join("")
  }
}

fn subdir_row(subdir: Subdir) -> String {
  let count = case subdir.mod_count {
    0 -> ""
    _ -> " - " <> int.to_string(subdir.mod_count) <> " mods"
  }
  "<div class=\"row\"><div><strong>"
  <> html(subdir.key)
  <> "</strong><span>"
  <> html(subdir.path <> count)
  <> "</span></div><span>"
  <> html(fallback(subdir.platform, "content"))
  <> "</span></div>"
}

fn detail(label: String, value: String) -> String {
  "<div class=\"detail\"><span>"
  <> html(label)
  <> "</span><strong title=\""
  <> html(value)
  <> "\">"
  <> html(value)
  <> "</strong></div>"
}

fn fallback(value: String, default: String) -> String {
  case value {
    "" -> default
    _ -> value
  }
}

fn filter_empty(values: List(String)) -> List(String) {
  case values {
    [] -> []
    [first, ..rest] -> {
      let filtered = filter_empty(rest)
      case first {
        "" -> filtered
        _ -> [first, ..filtered]
      }
    }
  }
}

fn list_map(items: List(a), mapper: fn(a) -> b) -> List(b) {
  case items {
    [] -> []
    [first, ..rest] -> [mapper(first), ..list_map(rest, mapper)]
  }
}

fn html(value: String) -> String {
  value
  |> string.replace("&", "&amp;")
  |> string.replace("<", "&lt;")
  |> string.replace(">", "&gt;")
  |> string.replace("\"", "&quot;")
}
