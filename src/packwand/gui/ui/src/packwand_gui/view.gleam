import gleam/dynamic/decode
import gleam/int
import gleam/list
import gleam/string
import lustre/attribute
import lustre/element.{type Element}
import lustre/element/html
import lustre/event
import packwand_gui/model.{
  type Feature, type ModEntry, type Project, type Subdir, type Variant, AddMod,
  Build, Doctor, ExportCurseforge, ExportModrinth, Lint, PacksIndex, PinMod,
  RefreshSubdir, Rehash, RemoveMod, UnpinMod, UpdateAll, UpdateMod,
  ValidateProject, WorkspaceRefresh, WorkspaceStatus, WorkspaceSync,
  WorkspaceUpdate, project_summary,
}
import packwand_gui/state.{
  type Model, type Msg, type View, Changelog, CopyChangelog, CreateProject,
  Exports, IconFailed, Logs, Mods, Navigate, Overview, RunAction, RunWebview,
  SaveManifest,
  SelectProject, SelectSubdir, SetManifest, SetModSlug, SetNewPackDescription,
  SetNewPackID, SetNewPackLoader, SetNewPackMinecraft, SetNewPackName,
  SetNewPackType, SetNewPackVersion, SetSearch, Settings, job_running,
  query_matches, selected_project,
}

@external(javascript, "./ffi.mjs", "currentHash")
fn current_hash() -> String

pub fn hash(view: View) -> String {
  case view {
    Overview -> "overview"
    Exports -> "exports"
    Mods -> "mods"
    Changelog -> "changelog"
    Logs -> "logs"
    Settings -> "settings"
  }
}

pub fn from_hash() -> View {
  from_name(current_hash())
}

pub fn from_name(value: String) -> View {
  case value {
    "exports" -> Exports
    "mods" -> Mods
    "changelog" -> Changelog
    "logs" -> Logs
    "settings" -> Settings
    _ -> Overview
  }
}

pub fn render(model: Model) -> Element(Msg) {
  html.div(
    [attribute.class("app"), attribute.data("current-view", hash(model.view))],
    [sidebar(model), main_view(model)],
  )
}

fn sidebar(model: Model) {
  html.aside([attribute.class("sidebar")], [
    html.div([attribute.class("brand")], [
      html.div([attribute.class("mark")], [html.text("P")]),
      html.div([], [
        html.strong([], [html.text("Packwand")]),
        html.span([attribute.title(model.root)], [html.text(model.root)]),
      ]),
    ]),
    html.label(
      [
        attribute.class("field-label"),
        attribute.attribute("for", "projectSelect"),
      ],
      [html.text("Current Project")],
    ),
    html.select(
      [
        attribute.id("projectSelect"),
        attribute.value(model.selected_id),
        event.on_change(SelectProject),
      ],
      project_options(model.projects, model.selected_id),
    ),
    html.nav([], [
      nav_button(model.view, Overview, "Open"),
      nav_button(model.view, Exports, "Exports"),
      nav_button(model.view, Mods, "Mods"),
      nav_button(model.view, Changelog, "Changelog"),
      nav_button(model.view, Logs, "Logs"),
      nav_button(model.view, Settings, "Settings"),
    ]),
    html.div([attribute.class("sidebar-footer")], [
      html.div([attribute.class("language-credit")], [
        html.img([
          attribute.class("gleam-logo"),
          attribute.src("/lucy.svg"),
          attribute.alt("Gleam"),
        ]),
        html.span([], [html.text("Frontend source in Gleam")]),
      ]),
      html.span([], [html.text("packwand " <> model.version)]),
    ]),
  ])
}

fn nav_button(current: View, target: View, label: String) {
  html.button(
    [
      attribute.classes([#("nav-btn", True), #("active", current == target)]),
      attribute.type_("button"),
      event.on_click(Navigate(target)),
    ],
    [html.text(label)],
  )
}

fn main_view(model: Model) {
  case selected_project(model) {
    Error(_) ->
      html.main([], [
        html.header([attribute.class("topbar")], [
          html.div([], [
            html.h1([], [html.text("Packwand")]),
            html.p([], [html.text("No projects indexed")]),
          ]),
        ]),
        html.section([attribute.class("grid empty-workspace")], [
          panel_with_head(
            "span-12",
            "Projects",
            button_disabled(
              "",
              "Regenerate Index",
              RunAction(PacksIndex),
              job_running(model),
            ),
            [
              html.p([attribute.class("panel-copy")], [
                html.text(
                  "No projects are currently indexed. Regenerate the index or scaffold the first project below.",
                ),
              ]),
              notice(model.notice),
            ],
          ),
          new_pack_panel(model),
          logs_panel(model),
        ]),
      ])
    Ok(project) ->
      html.main([], [
        topbar(model, project),
        toolbar(model, project),
        html.section([attribute.class("grid")], sections(model, project)),
      ])
  }
}

fn topbar(model: Model, project: Project) {
  html.header([attribute.class("topbar")], [
    html.div([], [
      html.h1([], [html.text(project.id)]),
      html.p([attribute.id("projectMeta")], [
        html.text(project_summary(project)),
      ]),
    ]),
    html.div([attribute.class("top-actions")], [
      html.label([attribute.class("search-wrap")], [
        html.span([], [html.text("Search")]),
        html.input([
          attribute.type_("search"),
          attribute.placeholder("current pack..."),
          attribute.value(model.search),
          event.on_input(SetSearch),
        ]),
      ]),
      case string.trim(model.search) {
        "" -> html.text("")
        _ -> html.span([attribute.class("pill")], [html.text("filtering")])
      },
      html.img([
        attribute.class("project-icon"),
        attribute.src(case model.icon_failed {
          True -> "/lucy.svg"
          False -> "/api/project-icon/" <> project.id
        }),
        attribute.alt(""),
        event.on("error", decode.success(IconFailed)),
      ]),
      button_disabled(
        "ghost",
        "Refresh Index",
        RunAction(PacksIndex),
        job_running(model),
      ),
      button_disabled(
        "",
        "Validate Pack",
        RunAction(ValidateProject(project.dir)),
        job_running(model),
      ),
    ]),
  ])
}

fn toolbar(model: Model, project: Project) {
  case model.view == Overview || model.view == Settings || model.view == Logs {
    False -> html.text("")
    True ->
      html.section([attribute.class("toolbar")], [
        button_disabled("", "Status", RunAction(WorkspaceStatus), job_running(model)),
        button_disabled("", "Doctor", RunAction(Doctor), job_running(model)),
        button_disabled("", "Lint", RunAction(Lint), job_running(model)),
        button_disabled(
          "",
          "Check Updates",
          RunAction(WorkspaceUpdate(True)),
          job_running(model),
        ),
        button_disabled(
          "ghost",
          "Dry Sync",
          RunAction(WorkspaceSync(True)),
          job_running(model),
        ),
        case model.view == Settings {
          True -> button_disabled(
            "ghost",
            "Refresh Workspace",
            RunAction(WorkspaceRefresh),
            job_running(model),
          )
          False -> button_disabled(
            "ghost",
            "Validate Pack",
            RunAction(ValidateProject(project.dir)),
            job_running(model),
          )
        },
      ])
  }
}

fn sections(model: Model, project: Project) -> List(Element(Msg)) {
  case string.trim(model.search) {
    "" -> sections_for_view(model, project)
    _ -> [search_results_panel(model, project)]
  }
}

fn sections_for_view(model: Model, project: Project) -> List(Element(Msg)) {
  case model.view {
    Overview -> [
      project_panel(model, project),
      subdir_panel(model, project.subdirs),
      actions_panel(model, project.subdirs),
      variant_panel(model, project.variants),
    ]
    Exports -> [actions_panel(model, project.subdirs)]
    Mods -> [add_mod_panel(model), mods_panel(model)]
    Changelog -> [changelog_panel(model)]
    Logs -> [logs_panel(model)]
    Settings -> [
      project_panel(model, project),
      subdir_panel(model, project.subdirs),
      manifest_panel(model),
      capabilities_panel(model.features),
      new_pack_panel(model),
    ]
  }
}

fn search_results_panel(model: Model, project: Project) {
  let project_text =
    [
      project.id,
      project.name,
      project.kind,
      project.dir,
      project.version,
      project.minecraft,
      project.loader,
    ]
    |> string.join(" ")
  let project_rows = case query_matches(model.search, project_text) {
    True -> [search_row("Project", project.id, project_summary(project))]
    False -> []
  }
  let subdir_rows =
    project.subdirs
    |> list.filter(fn(item) {
      query_matches(
        model.search,
        item.key <> " " <> item.path <> " " <> item.platform,
      )
    })
    |> list.map(fn(item) { search_row("Subdir", item.key, item.path) })
  let mod_rows =
    model.mods
    |> list.filter(fn(item) {
      query_matches(
        model.search,
        item.name
          <> " "
          <> item.slug
          <> " "
          <> item.filename
          <> " "
          <> item.platform,
      )
    })
    |> list.map(fn(item) {
      search_row(
        "Mod",
        fallback(item.name, item.slug),
        item.slug <> " / " <> item.platform,
      )
    })
  let variant_rows =
    project.variants
    |> list.filter(fn(item) {
      query_matches(
        model.search,
        item.id
          <> " "
          <> item.minecraft
          <> " "
          <> item.loader
          <> " "
          <> item.version,
      )
    })
    |> list.map(fn(item) {
      search_row(
        "Variant",
        fallback(item.id, item.minecraft),
        item.loader <> " / " <> item.version,
      )
    })
  let changelog_rows =
    model.changelog
    |> string.split("\n")
    |> list.filter(fn(line) {
      string.trim(line) != "" && query_matches(model.search, line)
    })
    |> list.map(fn(line) { search_row("Changelog", line, "changelog.md") })
  let rows =
    list.flatten([
      project_rows,
      subdir_rows,
      mod_rows,
      variant_rows,
      changelog_rows,
    ])
  panel_with_head(
    "span-12 search-results",
    "Search Results",
    pill(int.to_string(list.length(rows)) <> " matches"),
    [
      html.div([attribute.class("list")], case rows {
        [] -> [empty_row("No matches in this pack.")]
        _ -> rows
      }),
    ],
  )
}

fn search_row(category: String, title: String, detail_text: String) {
  html.div([attribute.class("row search-item")], [
    html.div([], [
      html.strong([], [html.text(title)]),
      html.span([], [html.text(detail_text)]),
    ]),
    html.span([attribute.class("result-kind")], [html.text(category)]),
  ])
}

fn project_panel(model: Model, project: Project) {
  let fields = [
    #("Name", project.name),
    #("Directory", project.dir),
    #("Manifest", project.manifest_path),
    #("Lifecycle", fallback(project.lifecycle, "active")),
    #("Auto Update", case project.auto_update {
      True -> "enabled"
      False -> "disabled"
    }),
    #("Modrinth", fallback(project.modrinth_id, "-")),
    #("CurseForge", fallback(project.curseforge_id, "-")),
    #("GitHub", fallback(project.github_id, "-")),
    #("Gitea", fallback(project.gitea_id, "-")),
    #("GitLab", fallback(project.gitlab_id, "-")),
  ]
  let details =
    fields
    |> list.filter(fn(field) {
      query_matches(model.search, field.0 <> " " <> field.1)
    })
    |> list.map(fn(field) { detail(field.0, field.1) })
  panel_with_head("span-7", "Project", pill(fallback(project.role, "none")), [
    html.div([attribute.class("details")], details),
  ])
}

fn subdir_panel(model: Model, subdirs: List(Subdir)) {
  let rows =
    subdirs
    |> list.filter(fn(subdir) {
      query_matches(
        model.search,
        subdir.key <> " " <> subdir.path <> " " <> subdir.platform,
      )
    })
    |> list.map(subdir_row)
  panel_with_head(
    "span-5",
    "Subdirs",
    pill(int.to_string(list.length(subdirs)) <> " subdir(s)"),
    [
      html.div([attribute.class("list")], case rows {
        [] -> [empty_row("No subdirs indexed.")]
        _ -> rows
      }),
    ],
  )
}

fn actions_panel(model: Model, subdirs: List(Subdir)) {
  let platform = selected_platform(subdirs, model.selected_subdir)
  let disabled = model.selected_subdir == "" || job_running(model)
  panel("span-12", "Actions", [
    html.div([attribute.class("action-row")], [
      html.select(
        [attribute.value(model.selected_subdir), event.on_change(SelectSubdir)],
        subdirs
          |> list.map(fn(subdir) {
            html.option([attribute.value(subdir.path)], subdir.key)
          }),
      ),
      button_disabled("", "Refresh", RunAction(RefreshSubdir(model.selected_subdir)), disabled),
      button_disabled("", "Update All", RunAction(UpdateAll(model.selected_subdir)), disabled),
      button_disabled("ghost", "Build", RunAction(Build(model.selected_subdir)), disabled),
      button_disabled("ghost", "Rehash", RunAction(Rehash(model.selected_subdir)), disabled),
      button_disabled(
        "ghost",
        "Modrinth Export",
        RunAction(ExportModrinth(model.selected_subdir)),
        disabled || !platform_matches(platform, "modrinth"),
      ),
      button_disabled(
        "ghost",
        "CF Export",
        RunAction(ExportCurseforge(model.selected_subdir)),
        disabled || !platform_matches(platform, "curseforge"),
      ),
    ]),
  ])
}

fn add_mod_panel(model: Model) {
  panel("span-12 compact-panel", "Add Mod", [
    html.div([attribute.class("action-row")], [
      html.input([
        attribute.placeholder("mod slug..."),
        attribute.value(model.mod_slug),
        event.on_input(SetModSlug),
      ]),
      button_disabled(
        "",
        "Add",
        RunAction(AddMod(model.selected_subdir, string.trim(model.mod_slug))),
        job_running(model)
          || string.trim(model.mod_slug) == ""
          || model.selected_subdir == "",
      ),
    ]),
  ])
}

fn mods_panel(model: Model) {
  let rows =
    model.mods
    |> list.filter(fn(mod) {
      query_matches(
        model.search,
        mod.name
          <> " "
          <> mod.slug
          <> " "
          <> mod.filename
          <> " "
          <> mod.platform,
      )
    })
    |> list.map(fn(mod) { mod_row(model, mod) })
  panel_with_head(
    "span-12 mods-panel",
    "Mods",
    pill(int.to_string(list.length(model.mods)) <> " mods"),
    [
      html.div([attribute.class("list mod-list")], case rows {
        [] -> [empty_row("No mods found.")]
        _ -> rows
      }),
    ],
  )
}

fn changelog_panel(model: Model) {
  let lines =
    model.changelog
    |> string.split("\n")
    |> list.filter(fn(line) { query_matches(model.search, line) })
    |> string.join("\n")
  panel_with_head(
    "span-12",
    "Changelog",
    button("ghost", "Copy Summary", CopyChangelog),
    [
      html.pre([attribute.class("changelog-preview")], [
        html.text(case lines {
          "" -> "No changelog.md content found."
          _ -> lines
        }),
      ]),
    ],
  )
}

fn manifest_panel(model: Model) {
  panel_with_head(
    "span-12",
    "Manifest",
    button("ghost", "Save Manifest", SaveManifest),
    [
      html.textarea(
        [
          attribute.spellcheck(False),
          event.on_input(SetManifest),
        ],
        model.manifest,
      ),
      notice(model.notice),
    ],
  )
}

fn new_pack_panel(model: Model) {
  let draft = model.new_pack
  panel_with_head("span-12", "New Pack", button("", "Create", CreateProject), [
    html.div([attribute.class("form-grid")], [
      form_input("ID", draft.id, "my-new-pack", SetNewPackID, ""),
      form_input("Name", draft.name, "My New Pack", SetNewPackName, ""),
      html.label([], [
        html.span([], [html.text("Type")]),
        html.select(
          [attribute.value(draft.kind), event.on_change(SetNewPackType)],
          [
            html.option([attribute.value("modpack")], "modpack"),
            html.option([attribute.value("resourcepack")], "resourcepack"),
            html.option([attribute.value("datapack")], "datapack"),
          ],
        ),
      ]),
      form_input("Loader", draft.loader, "fabric", SetNewPackLoader, ""),
      form_input(
        "Minecraft",
        draft.minecraft,
        "1.21.1",
        SetNewPackMinecraft,
        "",
      ),
      form_input("Version", draft.version, "0.1.0", SetNewPackVersion, ""),
      form_input(
        "Description",
        draft.description,
        "Optional summary",
        SetNewPackDescription,
        "span-form",
      ),
    ]),
    notice(model.notice),
  ])
}

fn logs_panel(model: Model) {
  panel_with_head("span-12", "Command Logs", pill(model.job_status), [
    html.pre([attribute.id("logPane")], [
      html.text(string.join(list.reverse(model.logs), "\n")),
    ]),
  ])
}

fn capabilities_panel(features: List(Feature)) {
  let runnable = list.filter(features, fn(feature) { feature.runnable })
  let integrated = list.filter(runnable, fn(feature) {
    feature.gui_status == "integrated"
  })
  panel_with_head(
    "span-12 capabilities-panel",
    "Packwand Feature Coverage",
    pill(
      int.to_string(list.length(integrated))
        <> " / "
        <> int.to_string(list.length(runnable))
        <> " commands integrated",
    ),
    [
      html.p([attribute.class("panel-copy")], [
        html.text(
          "This matrix is generated from Packwand's live command tree. CLI-only commands remain available in the terminal but are not exposed as unrestricted web actions.",
        ),
      ]),
      html.div(
        [attribute.class("feature-list")],
        runnable |> list.map(feature_row),
      ),
    ],
  )
}

fn feature_row(feature: Feature) {
  html.div([attribute.class("feature-row")], [
    html.code([], [html.text("packwand " <> feature.command)]),
    html.span([attribute.class("feature-summary")], [
      html.text(fallback(feature.summary, feature.usage)),
    ]),
    html.span(
      [
        attribute.classes([
          #("status-badge", True),
          #("integrated", feature.gui_status == "integrated"),
        ]),
      ],
      [html.text(case feature.gui_status {
        "integrated" -> "GUI"
        _ -> "CLI"
      })],
    ),
  ])
}

fn variant_panel(model: Model, variants: List(Variant)) {
  let rows =
    variants
    |> list.filter(fn(variant) {
      query_matches(
        model.search,
        variant.id
          <> " "
          <> variant.minecraft
          <> " "
          <> variant.loader
          <> " "
          <> variant.version,
      )
    })
    |> list.map(variant_row)
  panel("span-12", "Variants", [
    html.div([attribute.class("variant-list")], case rows {
      [] -> [
        html.span([attribute.class("empty-note")], [
          html.text("No variants declared."),
        ]),
      ]
      _ -> rows
    }),
  ])
}

fn project_options(projects: List(Project), selected_id: String) {
  projects
  |> list.map(fn(project) {
    html.option(
      [
        attribute.value(project.id),
        attribute.selected(project.id == selected_id),
      ],
      project.id <> " (" <> project.kind <> ")",
    )
  })
}

fn detail(label: String, value: String) {
  html.div([attribute.class("detail")], [
    html.span([], [html.text(label)]),
    html.strong([attribute.title(value)], [html.text(value)]),
  ])
}

fn subdir_row(subdir: Subdir) {
  let count = case subdir.mod_count {
    0 -> ""
    _ -> " - " <> int.to_string(subdir.mod_count) <> " mods"
  }
  html.div([attribute.class("row search-item")], [
    html.div([], [
      html.strong([], [html.text(subdir.key)]),
      html.span([], [html.text(subdir.path <> count)]),
    ]),
    html.span([], [html.text(fallback(subdir.platform, "content"))]),
  ])
}

fn mod_row(model: Model, mod: ModEntry) {
  let subdir = model.selected_subdir
  let #(pin_label, pin_action) = case mod.pin {
    True -> #("Unpin", UnpinMod(subdir, mod.slug))
    False -> #("Pin", PinMod(subdir, mod.slug))
  }
  let webview_button = case mod.platform, mod.version_id {
    _, "" -> html.text("")
    "curseforge", file_id ->
      button_disabled(
        "icon-btn",
        "CF Fetch",
        RunWebview("curseforge", mod.slug, file_id),
        job_running(model),
      )
    "modrinth", file_id ->
      button_disabled(
        "icon-btn",
        "MR Fetch",
        RunWebview("modrinth", mod.slug, file_id),
        job_running(model),
      )
    _, _ -> html.text("")
  }
  html.div([attribute.class("row search-item")], [
    html.div([], [
      html.strong([], [html.text(fallback(mod.name, mod.slug))]),
      html.span([], [
        html.text(
          [mod.slug, mod.filename, mod.side, mod.platform]
          |> non_empty
          |> string.join(" / "),
        ),
      ]),
    ]),
    webview_button,
    button_disabled("icon-btn", "Update", RunAction(UpdateMod(subdir, mod.slug)), job_running(model)),
    button_disabled("icon-btn", pin_label, RunAction(pin_action), job_running(model)),
    button_disabled("icon-btn danger", "Remove", RunAction(RemoveMod(subdir, mod.slug)), job_running(model)),
  ])
}

fn variant_row(variant: Variant) {
  html.div([attribute.class("mini-row search-item")], [
    html.strong([], [html.text(fallback(variant.id, variant.minecraft))]),
    html.span([], [
      html.text(
        [variant.minecraft, variant.loader, variant.version]
        |> non_empty
        |> string.join(" / "),
      ),
    ]),
  ])
}

fn form_input(
  label: String,
  value: String,
  placeholder: String,
  message: fn(String) -> Msg,
  class: String,
) {
  html.label([attribute.class(class)], [
    html.span([], [html.text(label)]),
    html.input([
      attribute.value(value),
      attribute.placeholder(placeholder),
      event.on_input(message),
    ]),
  ])
}

fn panel(class: String, title: String, children: List(Element(Msg))) {
  panel_with_head(class, title, html.text(""), children)
}

fn panel_with_head(
  class: String,
  title: String,
  action: Element(Msg),
  children: List(Element(Msg)),
) {
  html.section([attribute.class("panel " <> class)], [
    html.div([attribute.class("panel-head")], [
      html.h2([], [html.text(title)]),
      action,
    ]),
    ..children
  ])
}

fn button(class: String, label: String, message: Msg) {
  html.button(
    [attribute.class(class), attribute.type_("button"), event.on_click(message)],
    [html.text(label)],
  )
}

fn button_disabled(
  class: String,
  label: String,
  message: Msg,
  is_disabled: Bool,
) {
  html.button(
    [
      attribute.class(class),
      attribute.type_("button"),
      attribute.disabled(is_disabled),
      attribute.aria_disabled(is_disabled),
      event.on_click(message),
    ],
    [html.text(label)],
  )
}

fn selected_platform(subdirs: List(Subdir), path: String) -> String {
  case list.find(subdirs, fn(subdir) { subdir.path == path }) {
    Ok(subdir) -> subdir.platform
    Error(_) -> ""
  }
}

fn platform_matches(platform: String, expected: String) -> Bool {
  platform == expected
    || { platform == "mr" && expected == "modrinth" }
    || { platform == "cf" && expected == "curseforge" }
}

fn pill(value: String) {
  html.span([attribute.class("pill")], [html.text(value)])
}

fn notice(value: String) {
  case value {
    "" -> html.text("")
    _ -> html.p([attribute.class("notice")], [html.text(value)])
  }
}

fn empty_row(value: String) {
  html.div([attribute.class("row")], [html.span([], [html.text(value)])])
}

fn fallback(value: String, default: String) -> String {
  case string.trim(value) {
    "" -> default
    _ -> value
  }
}

fn non_empty(values: List(String)) -> List(String) {
  list.filter(values, fn(value) { string.trim(value) != "" })
}
