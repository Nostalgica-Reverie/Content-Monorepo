import gleam/list
import gleam/option.{type Option, None}
import gleam/string
import packwand_gui/manifest_form
import packwand_gui/model as domain

pub type View {
  Overview
  Exports
  Mods
  Changelog
  Logs
  Settings
}

pub type NewPack {
  NewPack(
    id: String,
    name: String,
    kind: String,
    loader: String,
    minecraft: String,
    version: String,
    description: String,
  )
}

pub type Model {
  Model(
    root: String,
    version: String,
    projects: List(domain.Project),
    features: List(domain.Feature),
    selected_id: String,
    selected_subdir: String,
    view: View,
    search: String,
    mods: List(domain.ModEntry),
    mod_slug: String,
    changelog: String,
    manifest: String,
    /// Parsed structured editor state; None when the manifest JSON could not
    /// be parsed (raw mode is the fallback).
    manifest_form: Option(manifest_form.ManifestForm),
    /// True renders typed controls, False the raw JSON textarea.
    manifest_structured: Bool,
    /// Stored newest-first so appending a line is O(1); reverse for display.
    logs: List(String),
    job_status: String,
    refresh_mods_after_job: Bool,
    icon_failed: Bool,
    new_pack: NewPack,
    notice: String,
  )
}

pub type Msg {
  GotHealth(Result(domain.Health, domain.ApiError))
  GotProjects(Result(domain.ProjectIndex, domain.ApiError))
  GotFeatures(Result(domain.FeatureIndex, domain.ApiError))
  SelectProject(String)
  SelectSubdir(String)
  Navigate(View)
  SetSearch(String)
  SetModSlug(String)
  GotMods(Result(List(domain.ModEntry), domain.ApiError))
  GotChangelog(Result(domain.ContentResponse, domain.ApiError))
  GotManifest(Result(domain.ContentResponse, domain.ApiError))
  RunAction(domain.Action)
  GotAction(domain.Action, Result(domain.ActionResponse, domain.ApiError))
  RunWebview(provider: String, slug: String, file_id: String)
  WebviewStarted(Result(domain.ActionResponse, domain.ApiError))
  JobLine(String)
  JobFinished(String, String)
  SaveManifest
  SetManifest(String)
  SetManifestField(manifest_form.Field)
  SetManifestStructured(Bool)
  ManifestSaved(Result(Nil, domain.ApiError))
  CreateProject
  ProjectCreated(Result(domain.CreatedProject, domain.ApiError))
  SetNewPackID(String)
  SetNewPackName(String)
  SetNewPackType(String)
  SetNewPackLoader(String)
  SetNewPackMinecraft(String)
  SetNewPackVersion(String)
  SetNewPackDescription(String)
  CopyChangelog
  IconFailed
}

pub fn initial() -> Model {
  Model(
    root: "Loading repo...",
    version: "",
    projects: [],
    features: [],
    selected_id: "",
    selected_subdir: "",
    view: Overview,
    search: "",
    mods: [],
    mod_slug: "",
    changelog: "",
    manifest: "",
    manifest_form: None,
    manifest_structured: False,
    logs: [],
    job_status: "idle",
    refresh_mods_after_job: False,
    icon_failed: False,
    new_pack: NewPack("", "", "modpack", "fabric", "", "0.1.0", ""),
    notice: "",
  )
}

pub fn selected_project(model: Model) -> Result(domain.Project, Nil) {
  list.find(model.projects, fn(project) { project.id == model.selected_id })
}

pub fn query_matches(query: String, text: String) -> Bool {
  let needle = query |> string.trim |> string.lowercase
  needle == "" || string.contains(string.lowercase(text), needle)
}

pub fn append_log(model: Model, line: String) -> Model {
  Model(..model, logs: [line, ..model.logs])
}

pub fn job_running(model: Model) -> Bool {
  model.job_status == "starting" || model.job_status == "running"
}

pub fn http_error(error: domain.ApiError) -> String {
  case error {
    domain.ApiError(message) -> message
    domain.DecodeError(message) ->
      "The Packwand API returned invalid data: " <> message
  }
}
