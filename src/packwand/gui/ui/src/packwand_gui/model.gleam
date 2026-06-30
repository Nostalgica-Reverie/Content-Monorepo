import gleam/string

pub type Project {
  Project(
    id: String,
    name: String,
    kind: String,
    dir: String,
    manifest_path: String,
    version: String,
    minecraft: String,
    loader: String,
    release_type: String,
    lifecycle: String,
    role: String,
    auto_update: Bool,
    modrinth_id: String,
    curseforge_id: String,
    github_id: String,
    gitea_id: String,
    gitlab_id: String,
    docs_path: String,
    variants: List(Variant),
    subdirs: List(Subdir),
  )
}

pub type Variant {
  Variant(id: String, minecraft: String, loader: String, version: String)
}

pub type Subdir {
  Subdir(
    key: String,
    path: String,
    platform: String,
    mod_count: Int,
    has_index: Bool,
    has_pack: Bool,
  )
}

pub type Action {
  PacksIndex
  ValidateAll
  WorkspaceStatus
  WorkspaceSync(dry_run: Bool)
  WorkspaceRefresh
  RefreshSubdir(path: String)
  AddMod(path: String, slug: String)
  RemoveMod(path: String, slug: String)
  PinMod(path: String, slug: String)
  UnpinMod(path: String, slug: String)
  UpdateMod(path: String, slug: String)
  UpdateAll(path: String)
  ExportModrinth(path: String)
  ExportCurseforge(path: String)
}

pub fn action_name(action: Action) -> String {
  case action {
    PacksIndex -> "packs-index"
    ValidateAll -> "validate-all"
    WorkspaceStatus -> "workspace-status"
    WorkspaceSync(_) -> "workspace-sync"
    WorkspaceRefresh -> "workspace-refresh"
    RefreshSubdir(_) -> "refresh"
    AddMod(_, _) -> "add-mod"
    RemoveMod(_, _) -> "remove-mod"
    PinMod(_, _) -> "pin-mod"
    UnpinMod(_, _) -> "unpin-mod"
    UpdateMod(_, _) -> "update-mod"
    UpdateAll(_) -> "update-all"
    ExportModrinth(_) -> "export-modrinth"
    ExportCurseforge(_) -> "export-curseforge"
  }
}

pub fn action_subdir(action: Action) -> String {
  case action {
    RefreshSubdir(path)
    | AddMod(path, _)
    | RemoveMod(path, _)
    | PinMod(path, _)
    | UnpinMod(path, _)
    | UpdateMod(path, _)
    | UpdateAll(path)
    | ExportModrinth(path)
    | ExportCurseforge(path) -> path
    _ -> ""
  }
}

pub fn action_slug(action: Action) -> String {
  case action {
    AddMod(_, slug)
    | RemoveMod(_, slug)
    | PinMod(_, slug)
    | UnpinMod(_, slug)
    | UpdateMod(_, slug) -> slug
    _ -> ""
  }
}

pub fn action_dry_run(action: Action) -> Bool {
  case action {
    WorkspaceSync(dry_run) -> dry_run
    _ -> False
  }
}

pub fn project_summary(project: Project) -> String {
  [
    project.id,
    project.kind,
    prefix("v", project.version),
    prefix("mc", project.minecraft),
    project.loader,
  ]
  |> list_filter_non_empty
  |> string.join("  ")
}

fn prefix(prefix: String, value: String) -> String {
  case value {
    "" -> ""
    _ -> prefix <> value
  }
}

fn list_filter_non_empty(values: List(String)) -> List(String) {
  case values {
    [] -> []
    [first, ..rest] -> {
      let filtered = list_filter_non_empty(rest)
      case first {
        "" -> filtered
        _ -> [first, ..filtered]
      }
    }
  }
}
