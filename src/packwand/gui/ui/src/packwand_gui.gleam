import gleam/int
import gleam/list
import gleam/option.{None, Some}
import gleam/string
import lustre
import lustre/effect.{type Effect}
import packwand_gui/api
import packwand_gui/manifest_form
import packwand_gui/model.{
  type Project, ContentResponse, CreatedProject, FeatureIndex, ProjectIndex,
  action_name, action_refreshes_mods,
}
import packwand_gui/state.{
  type Model, type Msg, CopyChangelog, CreateProject, GotAction, GotChangelog,
  GotFeatures, GotHealth, GotManifest, GotMods, GotProjects, IconFailed,
  JobFinished, JobLine, ManifestSaved, Model, Navigate, NewPack, ProjectCreated, RunAction,
  RunWebview, SaveManifest, SelectProject, SelectSubdir, SetBumpConfigs,
  SetBumpVersion, SetManifest,
  SetManifestField, SetManifestStructured, SetModSlug,
  SetNewPackDescription, SetNewPackID, SetNewPackLoader, SetNewPackMinecraft,
  SetNewPackName, SetNewPackType, SetNewPackVersion, SetSearch, WebviewStarted,
  append_log, http_error, initial, record_progress_line, reset_progress,
  selected_project,
}
import packwand_gui/view

@external(javascript, "./packwand_gui/ffi.mjs", "watchJob")
fn watch_job(
  id: String,
  line: fn(String) -> Nil,
  done: fn(String, String) -> Nil,
) -> Nil

@external(javascript, "./packwand_gui/ffi.mjs", "copyText")
fn copy_text(value: String) -> Nil

@external(javascript, "./packwand_gui/ffi.mjs", "setViewHash")
fn set_view_hash(value: String) -> Nil

@external(javascript, "./packwand_gui/ffi.mjs", "watchViewHash")
fn watch_view_hash(on_change: fn(String) -> Nil) -> Nil

pub fn main() {
  let app = lustre.application(init, update, view.render)
  let assert Ok(_) = lustre.start(app, "#app", Nil)
  Nil
}

fn init(_) -> #(Model, Effect(Msg)) {
  #(
    initial(),
    effect.batch([
      api.health(GotHealth),
      api.projects(GotProjects),
      api.features(GotFeatures),
      browser_view_effect(),
    ]),
  )
}

fn update(model: Model, msg: Msg) -> #(Model, Effect(Msg)) {
  case msg {
    GotHealth(Ok(health)) -> #(
      Model(..model, root: health.root, version: health.version),
      effect.none(),
    )
    GotHealth(Error(error)) -> with_error(model, error)
    GotProjects(Ok(ProjectIndex(projects))) ->
      select_after_projects(model, projects)
    GotProjects(Error(error)) -> with_error(model, error)
    GotFeatures(Ok(FeatureIndex(version, features))) -> #(
      Model(
        ..model,
        features: features,
        version: case model.version {
          "" -> version
          _ -> model.version
        },
      ),
      effect.none(),
    )
    GotFeatures(Error(error)) -> with_error(model, error)
    SelectProject(id) ->
      select_project(Model(..model, selected_id: id, icon_failed: False))
    SelectSubdir(path) -> #(
      Model(..model, selected_subdir: path, mods: []),
      load_mods(path),
    )
    Navigate(next) -> #(
      Model(..model, view: next),
      set_hash_effect(view.hash(next)),
    )
    SetSearch(value) -> #(Model(..model, search: value), effect.none())
    SetModSlug(value) -> #(Model(..model, mod_slug: value), effect.none())
    GotMods(Ok(mods)) -> #(Model(..model, mods: mods), effect.none())
    GotMods(Error(error)) -> with_error(model, error)
    GotChangelog(Ok(ContentResponse(_, content))) -> #(
      Model(..model, changelog: content),
      effect.none(),
    )
    GotChangelog(Error(error)) -> with_error(model, error)
    GotManifest(Ok(ContentResponse(_, content))) ->
      case manifest_form.parse(content) {
        Ok(form) -> #(
          Model(
            ..model,
            manifest: content,
            manifest_form: Some(form),
            manifest_structured: True,
          ),
          effect.none(),
        )
        Error(_) -> #(
          Model(
            ..model,
            manifest: content,
            manifest_form: None,
            manifest_structured: False,
          ),
          effect.none(),
        )
      }
    GotManifest(Error(error)) -> with_error(model, error)
    RunAction(action) -> {
      let running =
        model
        |> reset_progress
        |> append_log("> packwand " <> action_name(action))
      #(
        Model(..running, job_status: "starting", notice: ""),
        api.action(action, fn(result) { GotAction(action, result) }),
      )
    }
    GotAction(action, Ok(response)) -> #(
      Model(
        ..model,
        job_status: "running",
        refresh_mods_after_job: action_refreshes_mods(action),
      ),
      watch_job_effect(response.job_id),
    )
    GotAction(_, Error(error)) -> with_error(model, error)
    RunWebview(provider, slug, file_id) -> {
      let running =
        model
        |> append_log("> mod_browser_webview --provider " <> provider <> " " <> slug)
      #(
        Model(..running, job_status: "starting", notice: ""),
        api.webview_fetch(provider, slug, file_id, WebviewStarted),
      )
    }
    WebviewStarted(Ok(response)) -> #(
      Model(..model, job_status: "running", refresh_mods_after_job: False),
      watch_job_effect(response.job_id),
    )
    WebviewStarted(Error(error)) -> with_error(model, error)
    JobLine(line) -> #(
      record_progress_line(append_log(model, line), line),
      effect.none(),
    )
    JobFinished(status, error) -> {
      let finished = Model(..model, job_status: status)
      let finished = case error {
        "" -> finished
        _ -> append_log(finished, error)
      }
      #(
        Model(..finished, refresh_mods_after_job: False),
        case model.refresh_mods_after_job {
          True -> load_mods(model.selected_subdir)
          False -> effect.none()
        },
      )
    }
    SetManifest(content) -> #(Model(..model, manifest: content), effect.none())
    SetManifestField(field) ->
      case model.manifest_form {
        Some(form) -> #(
          Model(
            ..model,
            manifest_form: Some(manifest_form.apply(form, field)),
            notice: "",
          ),
          effect.none(),
        )
        None -> #(model, effect.none())
      }
    SetManifestStructured(True) ->
      case manifest_form.parse(model.manifest) {
        Ok(form) -> #(
          Model(
            ..model,
            manifest_form: Some(form),
            manifest_structured: True,
            notice: "",
          ),
          effect.none(),
        )
        Error(message) -> #(
          Model(..model, notice: "Cannot open form editor: " <> message),
          effect.none(),
        )
      }
    SetManifestStructured(False) -> {
      let raw = case model.manifest_form {
        Some(form) -> manifest_form.serialize(form)
        None -> model.manifest
      }
      #(
        Model(..model, manifest: raw, manifest_structured: False, notice: ""),
        effect.none(),
      )
    }
    SaveManifest ->
      case model.selected_id {
        "" -> #(model, effect.none())
        id ->
          case model.manifest_structured, model.manifest_form {
            True, Some(form) -> {
              let issues = manifest_form.validate(form)
              case manifest_form.errors(issues) {
                [] -> {
                  let raw = manifest_form.serialize(form)
                  #(
                    Model(
                      ..model,
                      manifest: raw,
                      notice: "Saving manifest...",
                    ),
                    api.save_manifest(id, raw, ManifestSaved),
                  )
                }
                errors -> #(
                  Model(
                    ..model,
                    notice: "Fix "
                      <> int.to_string(list.length(errors))
                      <> " validation error(s) before saving.",
                  ),
                  effect.none(),
                )
              }
            }
            _, _ -> #(
              Model(..model, notice: "Saving manifest..."),
              api.save_manifest(id, model.manifest, ManifestSaved),
            )
          }
      }
    ManifestSaved(Ok(_)) -> #(
      append_log(Model(..model, notice: "Manifest saved."), "Manifest saved."),
      api.projects(GotProjects),
    )
    ManifestSaved(Error(error)) -> with_error(model, error)
    CreateProject -> create_project(model)
    ProjectCreated(Ok(CreatedProject(id, _))) -> #(
      append_log(
        Model(..model, selected_id: id, notice: "Project created."),
        "Created project " <> id <> ".",
      ),
      api.projects(GotProjects),
    )
    ProjectCreated(Error(error)) -> with_error(model, error)
    SetNewPackID(value) -> #(
      Model(..model, new_pack: NewPack(..model.new_pack, id: value)),
      effect.none(),
    )
    SetNewPackName(value) -> #(
      Model(..model, new_pack: NewPack(..model.new_pack, name: value)),
      effect.none(),
    )
    SetNewPackType(value) -> #(
      Model(..model, new_pack: NewPack(..model.new_pack, kind: value)),
      effect.none(),
    )
    SetNewPackLoader(value) -> #(
      Model(..model, new_pack: NewPack(..model.new_pack, loader: value)),
      effect.none(),
    )
    SetNewPackMinecraft(value) -> #(
      Model(..model, new_pack: NewPack(..model.new_pack, minecraft: value)),
      effect.none(),
    )
    SetNewPackVersion(value) -> #(
      Model(..model, new_pack: NewPack(..model.new_pack, version: value)),
      effect.none(),
    )
    SetNewPackDescription(value) -> #(
      Model(..model, new_pack: NewPack(..model.new_pack, description: value)),
      effect.none(),
    )
    CopyChangelog -> #(
      Model(..model, notice: "Changelog copied."),
      copy_effect(model.changelog),
    )
    IconFailed -> #(Model(..model, icon_failed: True), effect.none())
    SetBumpVersion(value) -> #(
      Model(..model, bump_version: value),
      effect.none(),
    )
    SetBumpConfigs(value) -> #(
      Model(..model, bump_configs: value),
      effect.none(),
    )
  }
}

fn select_after_projects(model: Model, projects: List(Project)) {
  let selected = case
    list.find(projects, fn(project) { project.id == model.selected_id })
  {
    Ok(project) -> project.id
    Error(_) ->
      case projects {
        [first, ..] -> first.id
        [] -> ""
      }
  }
  select_project(Model(..model, projects: projects, selected_id: selected))
}

fn select_project(model: Model) -> #(Model, Effect(Msg)) {
  case selected_project(model) {
    Error(_) -> #(model, effect.none())
    Ok(project) -> {
      let subdir = case project.subdirs {
        [first, ..] -> first.path
        [] -> ""
      }
      #(
        Model(
          ..model,
          selected_subdir: subdir,
          mods: [],
          changelog: "",
          manifest: "",
          manifest_form: None,
          manifest_structured: False,
          search: "",
          icon_failed: False,
          bump_version: "",
        ),
        effect.batch([
          load_mods(subdir),
          api.changelog(project.id, GotChangelog),
          api.manifest(project.id, GotManifest),
        ]),
      )
    }
  }
}

fn load_mods(path: String) -> Effect(Msg) {
  case path {
    "" -> effect.none()
    _ -> api.mods(path, GotMods)
  }
}

fn create_project(model: Model) {
  let draft = model.new_pack
  case string.trim(draft.id) == "" || string.trim(draft.name) == "" {
    True -> #(
      Model(..model, notice: "A project ID and name are required."),
      effect.none(),
    )
    False -> #(
      Model(..model, notice: "Creating project..."),
      api.create_project(
        draft.id,
        draft.name,
        draft.kind,
        draft.loader,
        draft.minecraft,
        draft.version,
        draft.description,
        ProjectCreated,
      ),
    )
  }
}

fn with_error(model: Model, error) {
  let message = http_error(error)
  #(
    append_log(Model(..model, notice: message, job_status: "failed"), message),
    effect.none(),
  )
}

fn watch_job_effect(id: String) -> Effect(Msg) {
  effect.from(fn(dispatch) {
    watch_job(id, fn(line) { dispatch(JobLine(line)) }, fn(status, error) {
      dispatch(JobFinished(status, error))
    })
  })
}

fn copy_effect(value: String) -> Effect(Msg) {
  effect.from(fn(_) { copy_text(value) })
}

fn set_hash_effect(value: String) -> Effect(Msg) {
  effect.from(fn(_) { set_view_hash(value) })
}

fn browser_view_effect() -> Effect(Msg) {
  effect.from(fn(dispatch) {
    watch_view_hash(fn(value) { dispatch(Navigate(view.from_name(value))) })
    dispatch(Navigate(view.from_hash()))
  })
}
